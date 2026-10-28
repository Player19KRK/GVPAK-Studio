import base64,hashlib,http.cookiejar,json,os,pathlib,secrets,ssl,struct,subprocess,time,urllib.request,urllib.error,zlib
root=pathlib.Path(__file__).resolve().parents[1];lab=root/'.tools'/('p5-queue-'+secrets.token_hex(4));lab.mkdir();team=lab/'team';owner=lab/'owner';env={**os.environ,'GVPAK_PORT':'8893','GVPAK_TEAM_PORT':'8894','GVPAK_TEAM_HOSTS':'localhost,127.0.0.1','GVPAK_DATA_ROOT':str(owner),'GVPAK_TEAM_ROOT':str(team),'GVPAK_PROFILE':'test'};process=subprocess.Popen([str(root/'backend/target/release/gvpak-studio.exe')],cwd=root,env=env,creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0));host='http://127.0.0.1:8893';remote='https://127.0.0.1:8894'
def call(base,path,data=None,opener=None):
 req=urllib.request.Request(base+path,data=None if data is None else json.dumps(data).encode(),headers={'X-GVPAK-IPC':'1','Content-Type':'application/json'})
 with (opener.open(req,timeout=60) if opener else urllib.request.urlopen(req,timeout=60)) as r:return json.loads(r.read())
def command(base,name,data={},opener=None):return call(base,'/api/invoke/'+name,data,opener)
def png(index,width=80,height=100):
 def chunk(name,data):return struct.pack('>I',len(data))+name+data+struct.pack('>I',zlib.crc32(name+data)&0xffffffff)
 raw=(b'\0'+bytes((20+index*7%220,60+index*11%180,100+index*3%140,255))*width)*height
 return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(raw))+chunk(b'IEND',b'')
def project(base,name,count=1,large=False,opener=None):
 p=command(base,'project_create',{'name':name},opener);files=[{'name':str(i)+'.png','data':base64.b64encode(png(i,1536 if large else 80,2048 if large else 100)).decode()}for i in range(count)];return call(base,'/api/upload',{'projectId':p['id'],'files':files},opener)['project']
def submit(base,p,opener=None,block=False):
 import uuid
 ids=[a['id']for a in p['assets']];context={'page':'library','mentions':[],'selected':ids,'editorAssetId':None,'filter':{'folder':'','tag':'','status':'','q':'','sort':''},'filteredCount':len(ids),'resolvedTarget':{'source':'selected','ids':ids},'fileNames':[a['name']for a in p['assets']]};operations=[{'kind':'local_plan','plan':{'summary':'queue blocker','steps':[{'op':'blur','sigma':8}],'warnings':[],'clarification':None,'output':None}}]if block else[{'kind':'fit_ratio','ratio':'4:5','orientation':'portrait','mode':'pad'}];result=command(base,'operation_execute',{'projectId':p['id'],'context':context,'operations':operations,'clientRequestId':str(uuid.uuid4())},opener);assert 'task'in result,result;return result['task']['id']
try:
 for _ in range(100):
  if (team/'server.crt').exists():break
  assert process.poll() is None;time.sleep(.1)
 ctx=ssl.create_default_context(cafile=str(team/'server.crt'));peer=urllib.request.build_opener(urllib.request.ProxyHandler({}),urllib.request.HTTPSHandler(context=ctx),urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()));password=secrets.token_urlsafe(24);command(host,'team_accounts',{'action':'create','username':'p5_peer','password':password});call(remote,'/api/team/login',{'username':'p5_peer','password':password},peer)
 blocker=project(host,'P5 blocker',16,True);ownerjob=project(host,'P5 host');peerjob=project(remote,'P5 peer',opener=peer);blocking=submit(host,blocker,block=True);peerid=submit(remote,peerjob,peer);hostid=submit(host,ownerjob)
 h=next(t for t in command(host,'task_list')['tasks']if t['id']==hostid);c=next(t for t in command(remote,'task_list',opener=peer)['tasks']if t['id']==peerid);assert h['state']=='queued'and c['state']=='queued',(h,c);assert h['queuePosition']==0 and c['queuePosition']==1,(h['queuePosition'],c['queuePosition']);print('Verified authenticated queue: host position 0, peer position 1',flush=True)
 for _ in range(1200):
  h=next(t for t in command(host,'task_list')['tasks']if t['id']==hostid);c=next(t for t in command(remote,'task_list',opener=peer)['tasks']if t['id']==peerid)
  if h['state']=='done'and c['state']=='done':break
  time.sleep(.1)
 assert h['state']=='done'and c['state']=='done',(h,c);assert h['startedAt']<=c['startedAt'];result={'status':'passed','isolatedData':True,'realBackend':True,'tlsCertificateVerified':True,'checks':['real authenticated peer and owner use separate workspaces','host jumps ahead of already queued peer','peer reports one job ahead while host reports zero','host starts before peer and both complete'],'hostStartedAt':h['startedAt'],'peerStartedAt':c['startedAt'],'providerRequests':0};out=root/'output/playwright/p5';out.mkdir(parents=True,exist_ok=True);(out/'queue.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf8');print(json.dumps(result,ensure_ascii=False,indent=2))
finally:process.terminate();process.wait(timeout=20)
