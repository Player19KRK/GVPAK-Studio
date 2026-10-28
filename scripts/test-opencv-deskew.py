"""Local-only feedback loop for arbitrary subject tilt; no AI/network requests."""
import cv2, numpy as np, subprocess, sys, pathlib, json
root=pathlib.Path(__file__).resolve().parents[1]
out=root/'output'/'playwright'/'opencv-deskew';out.mkdir(parents=True,exist_ok=True)
report={'python':sys.executable,'opencv':cv2.__version__,'cases':[]}
def tilt(image):
    rgb=image[:,:,:3];mask=(np.min(rgb,axis=2)>210).astype(np.uint8)
    contours,_=cv2.findContours(mask,cv2.RETR_EXTERNAL,cv2.CHAIN_APPROX_SIMPLE)
    box=cv2.boxPoints(cv2.minAreaRect(max(contours,key=cv2.contourArea)))
    edges=np.roll(box,-1,axis=0)-box;edge=edges[np.argmax(np.linalg.norm(edges,axis=1))]
    return (np.degrees(np.arctan2(edge[1],edge[0]))+45)%90-45
try:
    for background in ('plain','two-tone'):
        for degrees in (-19,7,31):
            image=np.full((500,500,3),(120,140,155),np.uint8)
            if background=='two-tone':image[:220]=(65,95,135);image[220:]=(130,165,190)
            box=cv2.boxPoints(((250.,250.),(110.,240.),float(degrees))).astype(np.int32)
            cv2.fillConvexPoly(image,box,(240,242,239))
            source=out/f'{background}-{degrees}.png';destination=out/f'{background}-{degrees}-upright.png'
            cv2.imencode('.png',image)[1].tofile(str(source))
            result=subprocess.run([sys.executable,'-I',str(root/'backend/src/opencv_worker.py'),str(source),str(destination),'deskew','45'],capture_output=True,text=True)
            assert result.returncode==0,(background,degrees,result.stderr[-500:])
            corrected=cv2.imdecode(np.fromfile(str(destination),np.uint8),cv2.IMREAD_UNCHANGED)
            error=abs(float(tilt(corrected)));assert error<2,(background,degrees,error)
            report['cases'].append({'background':background,'inputTilt':degrees,'remainingTilt':error})
    report['status']='passed'
    print(json.dumps(report,ensure_ascii=False,indent=2))
finally:
    (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf8')
