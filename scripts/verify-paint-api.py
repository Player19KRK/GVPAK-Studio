from pathlib import Path
from PIL import Image
import base64,hashlib,io,json,urllib.request
root=Path(__file__).resolve().parents[1]
state=json.loads((root/'backend/target/paint-test-state.json').read_text(encoding='utf-8-sig'))
checks=[]
def check(name,condition):
    assert condition,name
    checks.append(name)
def request(path,args):
    req=urllib.request.Request('http://127.0.0.1:8798'+path,json.dumps(args).encode(),{'Content-Type':'application/json','X-Edge-IPC':'1'})
    with urllib.request.urlopen(req,timeout=60) as response:
        body=response.read()
        return json.loads(body) if 'json' in response.headers.get('Content-Type','') else body
def api(name,args={}):return request('/api/invoke/'+name,args)
p=api('project_create',{'name':'Radial paint verification'})
im=Image.new('RGBA',(400,300),(224,231,237,255))
for x in range(40,120):
    for y in range(40,120):im.putpixel((x,y),(35,110,195,255))
buf=io.BytesIO();im.save(buf,format='PNG')
p=request('/api/upload',{'projectId':p['id'],'files':[{'name':'Paint verification.png','data':base64.b64encode(buf.getvalue()).decode()}]})['project']
a=p['assets'][0];base=a['edit'];stroke={'points':[[.25,.5],[.75,.5]],'color':[227,42,50],'size':30,'hardness':1,'opacity':1,'erase':False}
edit={**base,'exposure':1,'strokes':[stroke]}
args={'projectId':p['id'],'assetId':a['id']}
p=api('edit_commit',{**args,'edit':edit});v=p['assets'][0]['versions'][-1]
check('saved version retains editable strokes',p['assets'][0]['edit']['strokes']==[stroke])
preview=api('preview',{**args,'edit':edit});pixels=Image.open(io.BytesIO(preview)).convert('RGBA')
check('paint keeps selected color with base exposure adjustment',pixels.getpixel((200,150))==(227,42,50,255))
for fmt in ['png','webp','jpeg']:
    out=Path(state['testRoot'])/'exports';out.mkdir(exist_ok=True)
    preset={'format':fmt,'width':400,'height':300,'fit':'pad','quality':95,'maxBytes':0,'outputDir':str(out),'background':[255,255,255]}
    ep=api('export_preview',{**args,'edit':edit,'preset':preset})
    exports=api('export_run',{'projectId':p['id'],'ids':[a['id']],'preset':preset})
    exported=Path(exports[0]['path']).read_bytes()
    check(fmt+' preview matches saved export',exported==base64.b64decode(ep['data']))
    pixel=Image.open(io.BytesIO(exported)).convert('RGB').getpixel((200,150))
    check(fmt+' contains real painted pixels',max(abs(x-y) for x,y in zip(pixel,(227,42,50)))<=8)
eraser={**stroke,'points':[[.5,.5]],'size':45,'erase':True}
e={**edit,'strokes':[stroke,eraser]}
cleared=Image.open(io.BytesIO(api('preview',{**args,'edit':e}))).convert('RGBA')
original=Image.open(io.BytesIO(api('preview',{**args,'edit':{**edit,'strokes':[]}}))).convert('RGBA')
check('eraser reveals the adjusted original',cleared.getpixel((200,150))==original.getpixel((200,150)))
api('edit_commit',{**args,'edit':e})
restored=api('restore_version',{**args,'versionId':v['id']})
check('restoring a saved version restores editable strokes',restored['assets'][0]['edit']['strokes']==[stroke])
# Leave the test image clean for browser painting tests.
api('edit_commit',{**args,'edit':base})
state.update(projectId=p['id'],assetId=a['id'],sourceHash=hashlib.sha256(buf.getvalue()).hexdigest())
(root/'backend/target/paint-test-state.json').write_text(json.dumps(state,indent=2),encoding='utf-8')
report={'checks':checks,'testRoot':state['testRoot'],'paidRequests':0}
(root/'docs/verification/paint-api-report.json').parent.mkdir(parents=True,exist_ok=True)
(root/'docs/verification/paint-api-report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps(report))
