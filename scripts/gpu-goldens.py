from pathlib import Path
from PIL import Image
import subprocess,os,time,urllib.request,json,uuid
root=Path.cwd();fixtures=root/'docs/verification/gpu-qa/fixtures';env=os.environ.copy();testroot=root/'backend'/('gpu-qa-'+str(uuid.uuid4())[:8]);env.update(EDGE_PORT='8790',GVPAK_PROFILE='test',GVPAK_DATA_ROOT=str(testroot))
p=subprocess.Popen([str(root/'bin/edge-studio.exe')],cwd=root,env=env,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
def api(name,args={},binary=False):
 req=urllib.request.Request('http://127.0.0.1:8790/api/invoke/'+name,data=json.dumps(args).encode(),headers={'Content-Type':'application/json','X-Edge-IPC':'1'})
 with urllib.request.urlopen(req,timeout=30) as r:
  data=r.read();return data if binary else json.loads(data)
try:
 for _ in range(60):
  try:api('bootstrap');break
  except Exception:time.sleep(.15)
 im=Image.new('RGBA',(32,24));pix=im.load()
 for y in range(24):
  for x in range(32):pix[x,y]=((x*17+y*7)%256,(x*5+y*23)%256,(x*31+y*11)%256,255 if x%8 else 0)
 src=fixtures/'source.png';im.save(src)
 project=api('project_create',{'name':'GPU pixel check'});pid=project['id'];project=api('import_paths',{'projectId':pid,'paths':[str(src)]})['project'];aid=project['assets'][0]['id'];base=project['assets'][0]['edit']
 variations=[('identity',{}),('color',{'exposure':.7,'contrast':1.2,'saturation':.4,'temperature':.2}),('opacity',{'exposure':-1.2,'contrast':1.4,'saturation':1.5,'temperature':-.6,'adjustmentOpacity':.35}),('bypass',{'exposure':2,'adjustmentEnabled':False}),('rotate90',{'rotation':90}),('rotate180',{'rotation':180,'flipH':True}),('rotate270',{'rotation':270,'flipV':True}),('crop',{'crop':[.125,.25,.5,.5],'rotation':90,'flipH':True,'contrast':1.2})]
 cases=[]
 for name,patch in variations:
  edit={**base,**patch};data=api('preview',{'projectId':pid,'assetId':aid,'edit':edit},True);(fixtures/(name+'.png')).write_bytes(data);cases.append({'name':name,'edit':edit})
 (root/'docs/verification/gpu-qa/cases.json').write_text(json.dumps(cases),encoding='utf-8');print('Generated',len(cases),'Rust pixel references; no cloud requests')
finally:p.terminate();p.wait(timeout=10)
