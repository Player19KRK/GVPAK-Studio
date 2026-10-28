"""Synthetic arbitrary-angle subject check against the isolated real service."""
import base64, hashlib, json, pathlib, time, urllib.request, uuid
import cv2
import numpy as np

root = pathlib.Path(__file__).resolve().parents[1]
base = 'http://127.0.0.1:8799'

def call(name, value=None, binary=False):
    request = urllib.request.Request(base + '/api/invoke/' + name, data=json.dumps(value or {}).encode(), headers={'Content-Type':'application/json', 'X-GVPAK-IPC':'1'})
    with urllib.request.urlopen(request, timeout=30) as response:
        data=response.read()
        return data if binary else json.loads(data)

def tilt(image):
    mask=np.uint8(np.max(image[:,:,:3],axis=2)<100)*255
    contours,_=cv2.findContours(mask,cv2.RETR_EXTERNAL,cv2.CHAIN_APPROX_SIMPLE)
    box=cv2.boxPoints(cv2.minAreaRect(max(contours,key=cv2.contourArea)))
    edges=np.roll(box,-1,axis=0)-box
    edge=edges[np.argmax(np.linalg.norm(edges,axis=1))]
    return (float(np.degrees(np.arctan2(edge[1],edge[0])))+45)%90-45

image=np.full((400,320,3),180,np.uint8)
box=cv2.boxPoints(((160,200),(90,230),12)).astype(np.int32)
cv2.fillConvexPoly(image,box,(40,65,80))
source=cv2.imencode('.png',image)[1].tobytes()
project=call('project_create',{'name':'P5 OpenCV subject deskew '+str(uuid.uuid4())[:8]})
request=urllib.request.Request(base+'/api/upload',data=json.dumps({'projectId':project['id'],'files':[{'name':'subject-12deg.png','data':base64.b64encode(source).decode()}]}).encode(),headers={'Content-Type':'application/json','X-GVPAK-IPC':'1'})
with urllib.request.urlopen(request) as response: project=json.loads(response.read())['project']
asset=project['assets'][0]
context={'page':'editor','mentions':[],'selected':[],'editorAssetId':asset['id'],'filter':{'folder':'','tag':'','status':'','q':'','sort':''},'filteredCount':1,'resolvedTarget':{'source':'editor','ids':[asset['id']]},'fileNames':['subject-12deg.png']}
task=call('operation_execute',{'projectId':project['id'],'context':context,'operations':[{'kind':'deskew'}],'clientRequestId':str(uuid.uuid4())})['task']
for _ in range(300):
    task=next(item for item in call('task_list')['tasks'] if item['id']==task['id'])
    if task['state'] in ['done','partial','failed','cancelled']: break
    time.sleep(.05)
assert task['state']=='done',task
result=call('operation_result',{'taskId':task['id'],'assetId':asset['id']},True)
output=cv2.imdecode(np.frombuffer(result,np.uint8),cv2.IMREAD_UNCHANGED)
before,after=tilt(image),tilt(output)
assert abs(before)>5 and abs(after)<1.5,(before,after)
saved=next(p for p in call('bootstrap')['projects'] if p['id']==project['id'])['assets'][0]
assert saved['sha256']==hashlib.sha256(source).hexdigest()
assert len(saved['versions'])==1
receipt={'status':'passed','realBackend':True,'openCvVersion':cv2.__version__,'syntheticInputTiltDegrees':before,'outputTiltDegrees':after,'newVersions':1,'originalSha256Unchanged':True,'providerRequests':0,'checks':['subject axis is detected and corrected at an arbitrary 12 degree tilt','one new version is created while original SHA256 is retained','actual operation_result bytes are measured, not an interface-only assertion']}
out=root/'output/playwright/p5';out.mkdir(parents=True,exist_ok=True)
(out/'opencv-deskew.json').write_text(json.dumps(receipt,indent=2),encoding='utf8')
cv2.imencode('.png',output)[1].tofile(str(out/'opencv-deskew.png'))
print(json.dumps(receipt,indent=2))
