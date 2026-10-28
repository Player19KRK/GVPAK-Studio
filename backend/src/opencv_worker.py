"""Trusted built-in worker. Receives only app-owned temporary PNG paths and a number."""
import sys
import cv2
import numpy as np

if sys.argv[1:] == ["status"]:
    print(cv2.__version__)
    raise SystemExit(0)
source, destination, mode, value = sys.argv[1:]
value = float(value)
image = cv2.imdecode(np.fromfile(source, dtype=np.uint8), cv2.IMREAD_UNCHANGED)
if image is None or image.shape[0] * image.shape[1] > 16000000:
    raise ValueError("image size")
alpha = image[:, :, 3].copy() if image.ndim == 3 and image.shape[2] == 4 else None
rgb = cv2.cvtColor(image, cv2.COLOR_GRAY2BGR) if image.ndim == 2 else image[:, :, :3]
if mode == "denoise":
    if not 1 <= value <= 20:
        raise ValueError("strength")
    result = cv2.fastNlMeansDenoisingColored(rgb, None, value, value, 7, 21)
    if alpha is not None:
        result = np.dstack((result, alpha))
elif mode in ("center", "deskew", "cutout"):
    import math
    h, w = image.shape[:2]
    border = np.concatenate((rgb[0], rgb[-1], rgb[:, 0], rgb[:, -1]))
    bg = np.median(border, axis=0)
    transparent = alpha is not None and np.any(alpha < 128)
    if transparent:
        mask = (alpha > 32).astype(np.uint8) * 255
    else:
        distance = np.linalg.norm(rgb.astype(np.float32) - bg, axis=2)
        mask = (distance > 28).astype(np.uint8) * 255
    count, labels, stats, _ = cv2.connectedComponentsWithStats(mask)
    if count < 2:
        raise ValueError("subject not found")
    largest = 1 + np.argmax(stats[1:, cv2.CC_STAT_AREA])
    x, y, bw, bh, area = stats[largest]
    uncertain = not 0.01 < area / (w * h) < 0.90 or (not transparent and np.mean(np.linalg.norm(border - bg, axis=1) > 28) > 0.15)
    if uncertain and mode in ("deskew", "cutout") and not transparent:
        # Multiple background tones (e.g. cardboard + tabletop) are not one color.
        # Segment the subject against a border-color model before measuring its axis.
        scale = min(1.0, 1000 / max(w, h))
        work = cv2.resize(rgb, (max(1, round(w*scale)), max(1, round(h*scale))))
        lab = cv2.cvtColor(work, cv2.COLOR_BGR2LAB).astype(np.float32)
        sample = np.concatenate((lab[0], lab[-1], lab[:, 0], lab[:, -1]))[::2].copy()
        cv2.setRNGSeed(0)
        _, _, centers = cv2.kmeans(sample, 3, None, (cv2.TERM_CRITERIA_EPS+cv2.TERM_CRITERIA_MAX_ITER, 30, .2), 3, cv2.KMEANS_PP_CENTERS)
        distance = np.min(np.linalg.norm(lab[:, :, None, :]-centers[None, None, :, :], axis=3), axis=2)
        foreground = (distance > 22).astype(np.uint8)*255
        foreground = cv2.morphologyEx(foreground, cv2.MORPH_CLOSE, np.ones((5,5),np.uint8))
        foreground = cv2.morphologyEx(foreground, cv2.MORPH_OPEN, np.ones((3,3),np.uint8))
        n, labelling, regions, _ = cv2.connectedComponentsWithStats(foreground)
        candidates = [i for i in range(1,n) if .01 < regions[i,cv2.CC_STAT_AREA]/foreground.size < .85 and regions[i,0]>1 and regions[i,1]>1 and regions[i,0]+regions[i,2]<work.shape[1]-1 and regions[i,1]+regions[i,3]<work.shape[0]-1]
        if not candidates:
            raise ValueError("subject uncertain; use a clear single-subject image")
        chosen = max(candidates, key=lambda i:regions[i,cv2.CC_STAT_AREA])
        seed = (labelling == chosen).astype(np.uint8)
        grab = np.full(seed.shape, cv2.GC_PR_BGD, np.uint8)
        grab[seed>0] = cv2.GC_PR_FGD
        grab[cv2.erode(seed,np.ones((5,5),np.uint8))>0] = cv2.GC_FGD
        grab[:2] = grab[-2:] = cv2.GC_BGD
        grab[:,:2] = grab[:,-2:] = cv2.GC_BGD
        cv2.grabCut(work,grab,None,np.zeros((1,65),np.float64),np.zeros((1,65),np.float64),2,cv2.GC_INIT_WITH_MASK)
        mask = cv2.resize(np.where((grab==cv2.GC_FGD)|(grab==cv2.GC_PR_FGD),255,0).astype(np.uint8),(w,h),interpolation=cv2.INTER_NEAREST)
        count, labels, stats, _ = cv2.connectedComponentsWithStats(mask)
        largest = 1 + np.argmax(stats[1:,cv2.CC_STAT_AREA])
        x,y,bw,bh,area = stats[largest]
        if not .01 < area/(w*h) < .85:
            raise ValueError("subject uncertain")
    elif uncertain:
        raise ValueError("subject uncertain on complex background")
    if mode == "cutout":
        # Refine the detected foreground into a matte; reject uncertain scenes above.
        if not transparent:
            seed = (labels == largest).astype(np.uint8)
            grab = np.full(seed.shape, cv2.GC_PR_BGD, np.uint8)
            grab[seed > 0] = cv2.GC_PR_FGD
            grab[cv2.erode(seed, np.ones((3,3),np.uint8)) > 0] = cv2.GC_FGD
            grab[:1] = grab[-1:] = cv2.GC_BGD
            grab[:,:1] = grab[:,-1:] = cv2.GC_BGD
            cv2.grabCut(rgb, grab, None, np.zeros((1,65),np.float64), np.zeros((1,65),np.float64), 2, cv2.GC_INIT_WITH_MASK)
            matte = np.where((grab==cv2.GC_FGD)|(grab==cv2.GC_PR_FGD),255,0).astype(np.uint8)
            if not .005 < np.count_nonzero(matte)/matte.size < .95:
                raise ValueError("subject uncertain")
        else:
            matte = alpha
        result = np.dstack((rgb, matte))
        ok, encoded = cv2.imencode(".png", result)
        if not ok:
            raise ValueError("encode")
        encoded.tofile(destination)
        raise SystemExit(0)
    points = np.column_stack(np.where(labels == largest)[::-1]).astype(np.float32)
    if mode == "center":
        if not 0 <= value <= 0.35:
            raise ValueError("padding")
        scale = min(w * (1 - 2 * value) / bw, h * (1 - 2 * value) / bh, 1.0)
        matrix = np.array([[scale, 0, w/2-scale*(x+bw/2)], [0, scale, h/2-scale*(y+bh/2)]], np.float32)
        shape = (w, h)
    else:
        if not 1 <= value <= 45:
            raise ValueError("max degrees")
        box = cv2.boxPoints(cv2.minAreaRect(points))
        edges = np.roll(box, -1, axis=0) - box
        edge = edges[np.argmax(np.linalg.norm(edges, axis=1))]
        angle = (math.degrees(math.atan2(edge[1], edge[0])) + 45) % 90 - 45
        if abs(angle) > value:
            raise ValueError("subject tilt beyond allowed range")
        matrix = cv2.getRotationMatrix2D((w/2, h/2), angle, 1.0)
        nw = int(math.ceil(h * abs(matrix[0, 1]) + w * abs(matrix[0, 0])))
        nh = int(math.ceil(h * abs(matrix[0, 0]) + w * abs(matrix[0, 1])))
        matrix[0, 2] += (nw-w)/2
        matrix[1, 2] += (nh-h)/2
        shape = (nw, nh)
    if transparent:
        premul = image.astype(np.float32)
        premul[:, :, :3] *= premul[:, :, 3:4] / 255
        warped = cv2.warpAffine(premul, matrix, shape, flags=cv2.INTER_LINEAR, borderValue=(0,0,0,0))
        a = warped[:, :, 3:4]
        warped[:, :, :3] = np.divide(warped[:, :, :3]*255, a, out=np.zeros_like(warped[:, :, :3]), where=a>0)
        result = np.clip(warped, 0, 255).astype(np.uint8)
    else:
        border_value = tuple(float(v) for v in bg) + ((255.0,) if alpha is not None else ())
        result = cv2.warpAffine(image, matrix, shape, flags=cv2.INTER_LINEAR, borderValue=border_value)
else:
    raise ValueError("unsupported local operation")
ok, encoded = cv2.imencode(".png", result)
if not ok:
    raise ValueError("encode")
encoded.tofile(destination)
