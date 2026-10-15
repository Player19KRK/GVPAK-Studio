import subprocess,os,time,json,urllib.request,urllib.error,base64,hashlib,io,uuid
from pathlib import Path
from PIL import Image,ImageDraw
root=Path(__file__).resolve().parent.parent
case=root/'backend'/('verification-'+str(uuid.uuid4())[:8]);case.mkdir()
env=os.environ.copy();env.update(EDGE_PORT='8790',GVPAK_PROFILE='test',GVPAK_DATA_ROOT=str(case/'data'))
server=subprocess.Popen([str(root/'bin/edge-studio.exe')],cwd=root,env=env,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
base='http://127.0.0.1:8790';checks=[]
def req(name,args={},raw=False):
 request=urllib.request.Request(base+'/api/invoke/'+name,data=json.dumps(args).encode(),headers={'Content-Type':'application/json','X-Edge-IPC':'1'})
 with urllib.request.urlopen(request,timeout=90) as r:
  data=r.read();return data if raw else json.loads(data)
def ok(name,value=True):
 assert value,name
 checks.append(name);print('PASS',name,flush=True)
try:
 for n in range(50):
  try:
   with urllib.request.urlopen(base+'/api/health',timeout=1) as r:health=json.load(r)
   break
  except Exception:time.sleep(.2)
 ok('Rust v2 health',health['version']=='2.0.0')
 b=req('bootstrap');ok('isolated data root',str(case/'data').replace('\\','/').lower()==b['dataRoot'].replace('\\','/').lower())
 project=req('project_create',{'name':'Migration verification'});pid=project['id']
 image=Image.new('RGBA',(320,240),(255,255,255,255));d=ImageDraw.Draw(image);d.rectangle((70,40,250,200),fill=(225,30,40,255));d.text((95,110),'EDGE TEST',fill='white');src=case/'input.png';image.save(src);original=hashlib.sha256(src.read_bytes()).hexdigest()
 imported=req('import_paths',{'projectId':pid,'paths':[str(src)]});a=imported['project']['assets'][0];aid=a['id'];ok('real local import',imported['imported']==1)
 duplicate=req('import_paths',{'projectId':pid,'paths':[str(src)]});ok('duplicate skipped',duplicate['imported']==0 and len(duplicate['skipped'])==1)
 thumb=Image.open(io.BytesIO(req('thumbnail',{'projectId':pid,'assetId':aid},True)));ok('thumbnail decodes',thumb.width>0)
 edit=a['edit'];edit.update(exposure=.3,contrast=1.1,saturation=.8,temperature=.1,rotation=90,crop=[.1,.1,.8,.8])
 preview=Image.open(io.BytesIO(req('preview',{'projectId':pid,'assetId':aid,'edit':edit},True)));ok('crop and rotation pixel geometry',preview.size==(192,256))
 p=req('edit_commit',{'projectId':pid,'assetId':aid,'edit':edit});version=p['assets'][0]['versions'][-1]['id'];ok('edit persists as version',len(p['assets'][0]['versions'])==1)
 req('restore_version',{'projectId':pid,'assetId':aid,'versionId':None});p=req('restore_version',{'projectId':pid,'assetId':aid,'versionId':version});ok('version restore',p['assets'][0]['edit']['rotation']==90)
 req('asset_trash',{'projectId':pid,'ids':[aid],'restore':False});p=req('asset_trash',{'projectId':pid,'ids':[aid],'restore':True});ok('trash restore',not p['assets'][0]['trashed'])
 out=case/'exports';out.mkdir()
 for fmt in ['webp','jpeg','png']:
  report=req('export_run',{'projectId':pid,'ids':[aid],'preset':{'format':fmt,'width':180,'height':180,'fit':'pad','quality':90,'maxBytes':100000,'outputDir':str(out),'background':[255,255,255]}})
  f=Path(report[0]['path']);decoded=Image.open(f);ok(fmt+' dimensions and byte limit',decoded.size==(180,180) and f.stat().st_size<=100000)
 plan={'summary':'Local sample','steps':[{'op':'adjust','exposure':.1,'contrast':1.,'saturation':1.,'temperature':0.},{'op':'resize','width':200,'height':200,'fit':'pad','background':[255,255,255]}],'warnings':[],'clarification':None,'output':None}
 sample=req('assistant_preview',{'projectId':pid,'ids':[aid],'plan':plan});ok('sample images decode',Image.open(io.BytesIO(base64.b64decode(sample['after']))).width>0)
 applied=req('assistant_apply',{'sampleId':sample['sampleId'],'outputDir':None});p=applied['project'];v=p['assets'][0]['versions'][-1];ok('local processing saves reviewable version',v['kind']=='local')
 rp=req('generated_preview',{'projectId':pid,'assetId':aid,'versionId':v['id']},True);ok('version review image',Image.open(io.BytesIO(rp)).width>0)
 p=req('adopt_generated',{'projectId':pid,'assetId':aid,'versionId':v['id']});ok('adoption switches pixel base',p['assets'][0]['generatedBase']==v['id'])
 req('assistant_save_preset',{'name':'Verification preset','plan':plan});presets=req('assistant_presets');ok('local preset persists','Verification preset' in presets)
 ok('original source unchanged',hashlib.sha256(src.read_bytes()).hexdigest()==original)
 ok('catalog available',len(req('model_catalog')['catalog']['models'])>0)
 ok('jobs endpoint',isinstance(req('hybrid_jobs')['jobs'],list))
 try:req('generate',{'projectId':pid,'assetId':aid,'edit':edit,'prompt':'test','model':'invalid','resolution':'1K','ratio':'1:1','approved':False,'estimatedInputTokens':None,'estimatedOutputTokens':None})
 except urllib.error.HTTPError as e:ok('paid generation approval gate', 'COST_APPROVAL_REQUIRED' in e.read().decode())
 else:raise AssertionError('Missing approval gate')
 try:
  request=urllib.request.Request(base+'/api/invoke/bootstrap',data=b'{}',headers={'Content-Type':'application/json'})
  urllib.request.urlopen(request)
 except urllib.error.HTTPError as e:ok('HTTP bridge rejects missing IPC header',e.code==403)
 else:raise AssertionError('Missing bridge boundary')
 ok('OpenCV status is explicit','available' in req('assistant_tools'))
 result={'passed':len(checks),'checks':checks,'workspace':str(case),'paidRequests':0}
 (root/'docs/verification/api-report.json').write_text(json.dumps(result,indent=2),encoding='utf8')
 print(json.dumps(result),flush=True)
finally:
 server.terminate();server.wait(timeout=10)

