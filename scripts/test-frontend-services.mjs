import test from 'node:test';
import assert from 'node:assert/strict';
import {createApi} from '../src/services/api.ts';
import {CapabilityUnavailable,taskFromJob} from '../src/services/adapters/real.ts';
import {eventBus,connectSse,normalizeEvent} from '../src/services/realtime.ts';
import {createFileService} from '../src/services/fs.ts';
import {createWorkerClient} from '../src/services/worker-client.ts';
import {normalizeProjects} from '../src/services/normalize.ts';
const context={page:'library',mentions:[],selected:['a'],editorAssetId:null,filter:{folder:'',tag:'',status:'',q:'',sort:''},filteredCount:1,resolvedTarget:{source:'selected',ids:['a']},fileNames:['a.png']};
test('normalization preserves asset identity and leaves absent backend metadata unknown',()=>{
 const asset={id:'a',name:'C:\\private\\a.png',sha256:'hash',sourcePath:'C:\\private\\a.png',versions:[{id:'v'}]},projects=[{id:'p',assets:[asset]}];
 assert.equal(normalizeProjects(projects),projects);assert.equal(projects[0].assets[0],asset);
 assert.equal(asset.fileName,'a.png');assert.equal(asset.hash,'hash');assert.equal(asset.relativePath,null);assert.equal(asset.cutout,null);assert.equal(asset.descriptionStatus,'unknown');assert.equal(asset.versionCount,2);assert.equal(asset.savedVersionCount,1);
});
test('real adapter uses approved read and execution endpoints without mock fallback',async()=>{
 const calls=[];const api=createApi('real',{fetcher:async(url,options)=>{
  calls.push({url,options});const name=url.split('/').at(-1);
  return Response.json(({session:{team:false},bootstrap:{projects:[],dataRoot:'test'},model_catalog:{catalog:{models:[]}},hybrid_jobs:{jobs:[]},operation_execute:{accepted:true},chat_execute:{accepted:true}})[name]);
 }});
 assert.equal((await api.readSession()).team,false);assert.deepEqual((await api.bootstrap()).projects,[]);
 assert.deepEqual(calls.map(call=>call.url).sort(),['/api/team/session','/api/invoke/bootstrap','/api/invoke/model_catalog','/api/invoke/hybrid_jobs'].sort());
 assert.equal((await api.execute({context,operations:[{kind:'center'}]})).accepted,true);assert.equal((await api.chat({context,instruction:'test'})).accepted,true);
 assert.equal(calls.length,6);assert.throws(()=>createApi('other'));
});
test('mock adapter has isolated events, explicit task progression/cancel and no network traffic',async()=>{
 const api=createApi('mock',{fetcher:()=>{throw Error('Mock must not fetch');}}),events=[];
 const unsubscribe=api.subscribe(event=>events.push(event));const result=await api.execute({context,operations:[{kind:'center'}]});
 assert.equal(result.mock,true);assert.equal(result.task.state,'running');await Promise.resolve();assert.equal(events[0].type,'task.progress');
 await api.stopTask({id:result.task.id,type:'mock'});assert.equal(events.at(-1).payload.state,'cancelled');
 unsubscribe();assert.equal((await api.listTasks())[0].state,'cancelled');
 assert.deepEqual((await createApi('mock').bootstrap()).tasks,[]);
 await assert.rejects(api.execute({context:{...context,resolvedTarget:{source:'filter',ids:[]}}}),/选择图片/);
});
test('missing queue/start information stays unknown instead of showing invented values',()=>{
 const task=taskFromJob({id:'old',createdAt:20,state:'queued',items:[{stage:'done'},{stage:'pending'}]});
 assert.equal(task.queuePosition,null);assert.equal(task.startedAt,null);assert.equal(task.done,1);assert.equal(task.total,2);
});
test('SSE normalizes legacy envelopes, reports invalid messages and releases its connection',()=>{
 const source={close(){this.closed=true;}},events=[],errors=[];
 const stop=connectSse(event=>events.push(event),error=>errors.push(error),()=>source);
 source.onmessage({data:JSON.stringify({event:'job-progress',payload:{done:2,total:4}})});
 source.onmessage({data:'not json'});source.onopen();source.onerror();stop();
 assert.equal(events[0].type,'legacy.progress');assert(!events[0].payload.taskId);assert.equal(events[1].type,'connection.open');assert.equal(errors.length,2);assert.equal(source.closed,true);
 assert.equal(normalizeEvent({event:'task.queue',payload:{taskId:'t',queuePosition:2}}).type,'task.queue');
 const bus=eventBus();let count=0;const off=bus.subscribe(()=>count++);bus.emit({type:'test'});off();bus.emit({type:'test'});assert.equal(count,1);
});
function directory(){
 const dirs=new Map(),files=new Map();let permission='granted',requested=0;
 const handle={name:'test',dirs,files,queryPermission:async()=>permission,requestPermission:async()=>{requested++;return permission='granted';},async getDirectoryHandle(name,{create}={}){if(!dirs.has(name)){if(!create)throw new DOMException('missing','NotFoundError');dirs.set(name,directory().handle);}return dirs.get(name);},async getFileHandle(name,{create}={}){if(!files.has(name)){if(!create)throw new DOMException('missing','NotFoundError');files.set(name,{value:null,async createWritable(){return {write:async value=>{this.value=value;},close:async()=>{},abort:async()=>{}};}});}return files.get(name);}};
 return {handle,setPermission(value){permission=value;},get requested(){return requested;}};
}
test('folder restore never asks permission; explicit authorization builds gvpak folders and avoids overwrite',async()=>{
 const folder=directory(),service=createFileService('test',{get:async()=>folder.handle,put:async()=>{}});
 folder.setPermission('prompt');assert.equal(await service.restore(),'prompt');assert.equal(folder.requested,0);
 await assert.rejects(service.ensureStructure(),/授权/);assert.equal(folder.handle.dirs.size,0);
 assert.equal(await service.permission(true),'granted');assert.equal(folder.requested,1);await service.ensureStructure();
 assert.deepEqual([...folder.handle.dirs.get('.gvpak').dirs.keys()],['versions','trash','cache']);
 const blob=new Blob(['test']);assert.equal((await service.writeFile(folder.handle,'C:\\private\\image.jpg',blob)).name,'image.jpg');
 assert.equal((await service.writeFile(folder.handle,'image.jpg',blob)).name,'image (2).jpg');
 assert.equal((await service.writeFile(folder.handle,'image.jpg',blob)).name,'image (3).jpg');
 folder.setPermission('denied');await assert.rejects(service.writeFile(folder.handle,'other.jpg',blob),/授权/);
});
test('Worker RPC matches out-of-order requests and rejects pending work on failure/disposal',async()=>{
 let worker;const client=createWorkerClient(()=>worker={sent:[],postMessage(data){this.sent.push(data);},terminate(){this.stopped=true;}});
 const first=client.call({blob:new Blob(['a'])}),second=client.call({blob:new Blob(['b'])});
 worker.onmessage({data:{id:2,result:'b'}});worker.onmessage({data:{id:1,result:'a'}});assert.deepEqual(await Promise.all([first,second]),['a','b']);
 const failed=client.call({});worker.onerror();await assert.rejects(failed,/失败/);assert.equal(worker.stopped,true);
 const disposed=client.call({});client.close();await assert.rejects(disposed,/停止/);
});

test('an explicitly picked directory can be attached and restored without another permission request',async()=>{const folder=directory();let saved=null;const storage={get:async()=>saved,put:async value=>saved=value};const service=createFileService('bound-project',storage);assert.equal(await service.attach(folder.handle),'granted');assert.equal(folder.requested,0);await service.ensureStructure();const restored=createFileService('bound-project',storage);assert.equal(await restored.restore(),'granted');assert.equal(restored.handle,folder.handle);assert.equal(folder.requested,0);});

test('explicit folder selection wins over an older in-flight restore',async()=>{const folder=directory();let finish;const service=createFileService('race',{get:()=>new Promise(resolve=>finish=resolve),put:async()=>{}});const restore=service.restore();await service.attach(folder.handle);finish(null);await restore;assert.equal(service.handle,folder.handle);});


test('pipeline planning keeps mode/model outside the exact SelectionContext contract',async()=>{let received;const api=createApi('real',{fetcher:async(_url,init)=>{received=JSON.parse(init.body);return Response.json({text:'fixture'});}});await api.chat({projectId:'p',context:{...context,sourcePath:'never'},instruction:'产品主图',pipeline:true,execution:'batch',model:'gemini-nano-banana-2.1',clientRequestId:'r'});assert.equal(received.pipeline,true);assert.equal(received.execution,'batch');assert.equal(received.model,'gemini-nano-banana-2.1');assert.equal(Object.keys(received.context).length,8);assert(!JSON.stringify(received).includes('sourcePath'));assert.deepEqual(received.context.fileNames,['a.png']);});
