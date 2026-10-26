import base64,concurrent.futures,hashlib,http.cookiejar,io,json,os,pathlib,secrets,ssl,subprocess,time,urllib.request,urllib.error
from PIL import Image
root=pathlib.Path(__file__).resolve().parents[1]
lab=root/'backend'/('verification-team-'+secrets.token_hex(4));lab.mkdir()
team=lab/'team';owner=lab/'owner'
env={**os.environ,'GVPAK_PORT':'8793','GVPAK_TEAM_PORT':'8794','GVPAK_TEAM_HOSTS':'localhost,127.0.0.1','GVPAK_DATA_ROOT':str(owner),'GVPAK_TEAM_ROOT':str(team),'GVPAK_PROFILE':'test','GVPAK_FEEDBACK_ROOT':str(lab/'feedback')}
exe=root/'backend/target/release/gvpak-studio.exe'
p=subprocess.Popen([str(exe)],cwd=root,env=env,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
checks=[]
def check(name,value):
 assert value,name
 checks.append(name)
def call(base,path,data=None,opener=None,headers=None):
 headers={'X-GVPAK-IPC':'1',**(headers or {})}
 if data is not None:headers['Content-Type']='application/json'
 req=urllib.request.Request(base+path,data=None if data is None else json.dumps(data).encode(),headers=headers)
 try:
  with (opener.open(req,timeout=60) if opener else urllib.request.urlopen(req,timeout=60)) as r:
   raw=r.read();return r.status,json.loads(raw) if 'json' in r.headers.get('Content-Type','') else raw,r.headers
 except urllib.error.HTTPError as e:return e.code,e.read(),e.headers
try:
 for _ in range(160):
  if (team/'server.crt').exists():break
  if p.poll() is not None:raise RuntimeError('Server exited')
  time.sleep(.25)
 ctx=ssl.create_default_context(cafile=str(team/'server.crt'))
 def client():return urllib.request.build_opener(urllib.request.ProxyHandler({}),urllib.request.HTTPSHandler(context=ctx),urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
 host='http://127.0.0.1:8793';lan='https://127.0.0.1:8794'
 a,b,anon=client(),client(),client();password=secrets.token_urlsafe(20)
 import httpx
 with httpx.Client(http2=True,verify=ctx,trust_env=False) as browser:
  r=browser.get(lan+'/api/team/session');check('HTTP2 browser session endpoint works',r.http_version=='HTTP/2' and r.status_code==200 and r.json()['user'] is None)
  r=browser.get(lan+'/api/team/session',headers={'Origin':'https://evil.invalid'});check('HTTP2 still rejects foreign origins',r.status_code==403)

 for name in ['qa_alice','qa_bob']:
  status,_,_=call(host,'/api/invoke/team_accounts',{'action':'create','username':name,'password':password});check('host creates '+name,status==200)
 check('anonymous API rejected',call(lan,'/api/invoke/bootstrap',{},anon)[0]==401)
 check('cross-origin login rejected',call(lan,'/api/team/login',{'username':'qa_alice','password':password},anon,{'Origin':'https://evil.invalid'})[0]==403)
 for c,name in [(a,'qa_alice'),(b,'qa_bob')]:
  status,_,headers=call(lan,'/api/team/login',{'username':name,'password':password},c);check('login '+name,status==200);check('secure HttpOnly cookie '+name,'Secure' in headers.get('Set-Cookie','') and 'HttpOnly' in headers.get('Set-Cookie',''))
 
 with httpx.Client(http2=True,verify=ctx,trust_env=False,headers={'X-GVPAK-IPC':'1'}) as browser:
  r=browser.post(lan+'/api/team/login',json={'username':'qa_alice','password':password});check('HTTP2 browser login works',r.http_version=='HTTP/2' and r.status_code==200)
  r=browser.post(lan+'/api/invoke/bootstrap',json={});check('HTTP2 authenticated workspace works',r.status_code==200 and r.json()['projects']==[])
  r=browser.post(lan+'/api/invoke/import_paths',json={});check('HTTP2 keeps host file APIs blocked',r.status_code==403)
 check('host sees logged-in account lights',all(x['loggedIn'] for x in call(host,'/api/invoke/team_accounts',{})[1]['accounts']))
 status,feedback,_=call(host,'/api/invoke/submit_feedback',{'description':'Host feedback test'})
 check('host feedback saved in shared feedback root',status==200 and len(list((lab/'feedback'/'主机').glob('*/问题说明.txt')))==1)
 check('team cannot impersonate host feedback',call(lan,'/api/invoke/submit_feedback',{'description':'blocked'},a)[0]==403)
 print('PASS: login indicators, host feedback save, team isolation')
 check('host projects not inherited',call(lan,'/api/invoke/bootstrap',{},a)[1]['projects']==[])
 _,project,_=call(lan,'/api/invoke/project_create',{'name':'Private Alice'},a);pid=project['id']
 check('Bob cannot see Alice project',call(lan,'/api/invoke/bootstrap',{},b)[1]['projects']==[])
 check('Bob cannot rename Alice project',call(lan,'/api/invoke/project_rename',{'projectId':pid,'name':'intrusion'},b)[0]!=200)
 for command,args in [('pick_import',{}),('import_paths',{'projectId':pid,'paths':['C:/Windows/win.ini']}),('pick_output',{}),('data_root_switch',{'path':'C:/Windows'}),('credential_set',{'key':'not-a-key'}),('assistant_key_delete',{}),('team_accounts',{}),('hybrid_open_output',{})]:
  check('blocked '+command,call(lan,'/api/invoke/'+command,args,a)[0]==403)
 im=Image.new('RGB',(160,120),(80,140,190));buf=io.BytesIO();im.save(buf,format='PNG');data=base64.b64encode(buf.getvalue()).decode()
 check('upload path traversal rejected',call(lan,'/api/upload',{'projectId':pid,'files':[{'name':'../escape.png','data':data}]},a)[0]==400)
 status,report,_=call(lan,'/api/upload',{'projectId':pid,'files':[{'name':'sample.png','data':data}]},a);check('browser upload succeeds',status==200)
 aid=report['project']['assets'][0]['id'];check('host paths removed',not any(k in json.dumps(report) for k in ['sourcePath',str(lab),'source_path']))
 check('Bob cannot preview Alice asset',call(lan,'/api/invoke/preview_source',{'projectId':pid,'assetId':aid},b)[0]!=200)
 preset={'format':'webp','width':80,'height':80,'fit':'pad','quality':90,'maxBytes':100000,'background':[255,255,255],'outputDir':'C:/Windows'}
 status,report,_=call(lan,'/api/invoke/export_run',{'projectId':pid,'ids':[aid],'preset':preset},a);check('export succeeds with forced private destination',status==200 and report[0]['status']=='ok')
 url=report[0]['path'];check('download uses opaque scoped route',url.startswith('/api/download/'))
 status,raw,headers=call(lan,url,None,a);check('owner downloads decoded output',status==200 and Image.open(io.BytesIO(raw)).size==(80,80));check('download keeps output extension',report[0]['downloadName'].endswith('.webp') and '.webp' in headers.get('Content-Disposition',''))
 check('Bob cannot download Alice output',call(lan,url,None,b)[0]==404)
 check('anonymous download rejected',call(lan,url,None,anon)[0]==401)
 bad='/api/download/'+base64.urlsafe_b64encode(b'../accounts.json').decode().rstrip('=');check('download traversal rejected',call(lan,bad,None,a)[0] in (400,404))
 check('downloads are private',len(call(lan,'/api/files',None,b)[1]['files'])==0)
 check('credentials status usable without key disclosure',isinstance(call(lan,'/api/invoke/credential_status',{},a)[1],bool))

 # Real Process output lives in a per-task subdirectory, unlike direct Export.
 import uuid
 recipe={'summary':'Nested export test','before':[{'op':'adjust','exposure':0,'contrast':1,'saturation':1,'temperature':0}],'generation':None,'after':[],'output':{'format':'webp','width':80,'height':80,'fit':'pad','maxBytes':100000,'background':[255,255,255]},'clarification':None,'warnings':[]}
 jid=str(uuid.uuid4());status,job,_=call(lan,'/api/invoke/hybrid_start',{'projectId':pid,'ids':[aid],'instruction':'local nested test','recipe':recipe,'outputDir':'C:/Windows','approved':False,'clientRequestId':jid},a);check('Process local task starts',status==200)
 for _ in range(120):
  status,listing,_=call(lan,'/api/invoke/hybrid_jobs',{},a);job=next(j for j in listing['jobs'] if j['id']==jid)
  if job['state'] in ['completed','failed','partial','cancelled']:break
  time.sleep(.25)
 check('Process task completes',job['state']=='completed');nested=job['items'][0]['exported'];check('nested task output has download URL',isinstance(nested,str) and nested.startswith('/api/download/'))
 check('nested file appears in Downloads',any(f['url']==nested for f in call(lan,'/api/files',None,a)[1]['files']))
 check('nested task file downloads',call(lan,nested,None,a)[0]==200);check('other account cannot download nested file',call(lan,nested,None,b)[0]==404)
 call(lan,'/api/invoke/hybrid_ack',{'jobId':jid},a);check('dismiss completion alert has observable effect',not any(j['id']==jid for j in call(lan,'/api/invoke/hybrid_jobs',{},a)[1]['notices']))
 status,feedback,_=call(lan,'/api/feedback',{'description':'测试多张截图与中文说明','images':[{'data':data},{'data':data}]},a);check('feedback is saved',status==200);folders=list((lab/'feedback'/'qa_alice').iterdir());check('feedback contains two images and description',len(list(folders[0].glob('*.png')))==2 and '中文说明' in (folders[0]/'问题说明.txt').read_text(encoding='utf-8'))
 check('anonymous feedback rejected',call(lan,'/api/feedback',{'description':'test','images':[]},anon)[0]==401)
 check('Poe key writes remain host-only',call(lan,'/api/invoke/poe_key_set',{'key':'not-a-real-key'},a)[0]==403)
 status,_,_=call(lan,'/api/invoke/project_delete',{'projectId':pid},b);check('other account cannot delete project',status!=200)
 status,_,_=call(lan,'/api/invoke/project_delete',{'projectId':pid},a);check('project deletion is recoverable',status==200 and call(lan,'/api/invoke/bootstrap',{},a)[1]['projects']==[])
 check('project can be restored',call(lan,'/api/invoke/project_restore',{'projectId':pid},a)[0]==200)
 # No project or task image-count cap; per-request byte limits remain.
 many=[]
 for i in range(501):
  buf=io.BytesIO();Image.new('RGB',(4,4),(i%256,i//256,83)).save(buf,format='PNG');many.append({'name':str(i)+'.png','data':base64.b64encode(buf.getvalue()).decode()})
 status,report,_=call(lan,'/api/upload',{'projectId':pid,'files':many},a);check('upload more than 100 images and project more than 500 images',status==200 and len(report['project']['assets'])>500)
 ids=[x['id'] for x in report['project']['assets']];large_id=str(uuid.uuid4());status,_,_=call(lan,'/api/invoke/hybrid_start',{'projectId':pid,'ids':ids,'instruction':'scope over 500','recipe':recipe,'outputDir':'Downloads','approved':False,'clientRequestId':large_id},a);check('task accepts more than 500 images',status==200);call(lan,'/api/invoke/hybrid_cancel',{'jobId':large_id},a)

 # Confirmed local save clears only its own temporary inputs and output.
 owner_before={str(p.relative_to(owner)):hashlib.sha256(p.read_bytes()).hexdigest() for p in owner.rglob('*') if p.is_file() and p.name!='.session.lock'}
 _,private_project,_=call(lan,'/api/invoke/project_create',{'name':'Retention Alice'},a);clean_pid=private_project['id']
 _,uploaded,_=call(lan,'/api/upload',{'projectId':clean_pid,'files':[{'name':'retention.png','data':data}]},a);clean_aid=uploaded['project']['assets'][0]['id']
 _,b_project,_=call(lan,'/api/invoke/project_create',{'name':'Retention Bob'},b)
 _,b_upload,_=call(lan,'/api/upload',{'projectId':b_project['id'],'files':[{'name':'keep.png','data':data}]},b);b_aid=b_upload['project']['assets'][0]['id']
 _,exported,_=call(lan,'/api/invoke/export_run',{'projectId':clean_pid,'ids':[clean_aid],'preset':preset},a);saved_url=exported[0]['path']
 _,output,_=call(lan,saved_url,None,a);local_copy=lab/'colleague-saved.webp';local_copy.write_bytes(output);proof={'id':saved_url.split('/')[-1],'sha256':hashlib.sha256(local_copy.read_bytes()).hexdigest(),'bytes':local_copy.stat().st_size}
 check('ordinary GET does not delete originals',call(lan,'/api/invoke/preview_source',{'projectId':clean_pid,'assetId':clean_aid},a)[0]==200)
 pending_files=call(lan,'/api/files',None,a)[1]['files'];lease=next(f for f in pending_files if f['url']==saved_url);check('pending output has 24h expiry',23*3600<int(lease['expiresAt'])-time.time()<=24*3600+3)
 check('wrong save hash cannot clean originals',call(lan,'/api/files/saved',{**proof,'sha256':'0'*64},a)[0]==400)
 check('foreign account cannot confirm another output',call(lan,'/api/files/saved',proof,b)[0]!=200)
 check('anonymous save confirmation rejected',call(lan,'/api/files/saved',proof,anon)[0]==401)
 status,cleaned,_=call(lan,'/api/files/saved',proof,a);check('verified save cleans original and output',status==200 and cleaned['summary']['removedAssets']==1 and call(lan,saved_url,None,a)[0]==404)
 check('saved input is removed from temporary project',not next(p for p in call(lan,'/api/invoke/bootstrap',{},a)[1]['projects'] if p['id']==clean_pid)['assets'])
 check('repeated save confirmation is safe',call(lan,'/api/files/saved',proof,a)[0]==200)
 check('other colleague original stays intact',call(lan,'/api/invoke/preview_source',{'projectId':b_project['id'],'assetId':b_aid},b)[0]==200)
 check('host owner project files stay identical',owner_before=={str(p.relative_to(owner)):hashlib.sha256(p.read_bytes()).hexdigest() for p in owner.rglob('*') if p.is_file() and p.name!='.session.lock'})
 status,_,_=call(lan,'/api/team/logout',{},a);check('logout succeeds',status==200);check('logout invalidates session',call(lan,'/api/invoke/bootstrap',{},a)[0]==401)
 call(host,'/api/invoke/team_accounts',{'action':'disable','username':'qa_bob'});check('disable invalidates session',call(lan,'/api/invoke/bootstrap',{},b)[0]==401)
 account_data=json.loads((team/'accounts.json').read_text());check('no plaintext passwords persisted',password not in json.dumps(account_data));check('passwords independently salted',account_data[0]['salt']!=account_data[1]['salt'])
 report={'passed':len(checks),'checks':checks,'paidCalls':0,'tlsCertificateVerified':True,'testWorkspace':str(lab)}
 out=root/'docs/verification/team-api.json';out.parent.mkdir(exist_ok=True,parents=True);out.write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps({'passed':len(checks),'paidCalls':0,'report':str(out)}))
finally:
 p.terminate();p.wait(timeout=10)
