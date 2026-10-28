import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
const {chromium}=await import(pathToFileURL(path.resolve(process.argv[2]||process.env.PLAYWRIGHT_MODULE)).href);
const browser=await chromium.launch({headless:true,...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE?{executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE}:{})});
const output='output/playwright/frontend-data';await fs.mkdir(output,{recursive:true});
const report={checks:[],pageErrors:[],requests:[]};
try{
 const context=await browser.newContext({viewport:{width:1280,height:900}});
 await context.route('**/api/**',route=>{
  const url=new URL(route.request().url());report.requests.push(url.pathname);
  const data=({'/api/team/session':{team:false},'/api/invoke/bootstrap':{projects:[],dataRoot:'qa-data',warnings:[]},'/api/invoke/model_catalog':{catalog:{models:[]}},'/api/invoke/hybrid_jobs':{jobs:[],notices:[]}})[url.pathname];
  if(url.pathname==='/api/events')return route.fulfill({contentType:'text/event-stream',body:': testing\n\n'});
  if(!data)throw Error('Unexpected backend request: '+url.pathname);
  return route.fulfill({contentType:'application/json',body:JSON.stringify(data)});
 });
 const page=await context.newPage();page.on('pageerror',error=>report.pageErrors.push(error.message));
 await page.goto(process.env.FRONTEND_QA_URL||'http://127.0.0.1:5173');await page.getByRole('heading',{name:'今天要处理什么？'}).waitFor();
 await page.evaluate(async()=>{
  const load=file=>{const canonical=file.endsWith('.svelte')?file+'.js':file;const actual=performance.getEntriesByType('resource').find(entry=>new URL(entry.name).pathname===canonical)?.name||canonical;return import(actual);};
  const [{studio},{selection},{assets},{tasks,applyTaskEvent},{session},{project},{editor},{prefs},{chat},{createApi}]=await Promise.all([
   load('/src/studio.svelte'),load('/src/stores/selection.svelte'),load('/src/stores/assets.svelte'),load('/src/stores/tasks.svelte'),load('/src/stores/session.svelte'),load('/src/stores/project.svelte'),load('/src/stores/editor.svelte'),load('/src/stores/prefs.svelte'),load('/src/stores/chat.svelte'),load('/src/services/api.ts'),
  ]);
  studio.projects=[{id:'data-project',name:'Data test',assets:[{id:'a',name:'C:\\private\\a.png',trashed:false,tags:['box'],processingStatus:'idle'},{id:'b',name:'/private/b.png',trashed:false,tags:[],processingStatus:'idle'}]}];studio.pid='data-project';studio.aid='a';
  window.__dataQA={studio,selection,assets,tasks,applyTaskEvent,session,project,editor,prefs,chat,createApi};
 });
 const timing=await page.evaluate(async()=>{
  const q=window.__dataQA,start=performance.now();q.selection.set(['b']);q.selection.set(['a','b']);
  await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
  return {ms:performance.now()-start,context:JSON.parse(JSON.stringify(q.selection.context)),sameArray:q.selection.selectedIds===q.studio.selected,sameAsset:q.assets.index.a===q.studio.projects[0].assets[0],sessionReady:q.session.status==='ready',storeNames:['session','project','assets','selection','tasks','chat','editor','prefs'].filter(name=>!!q[name])};
 });
 report.diagnostics=await page.evaluate(()=>({error:window.__dataQA.studio.error,status:window.__dataQA.session.status,resources:performance.getEntriesByType('resource').map(e=>e.name).filter(n=>n.includes('/src/stores/')||n.includes('workspace.svelte')||n.includes('studio.svelte'))}));
 assert(timing.ms<100,`Selection context took ${timing.ms}ms`);assert.deepEqual(timing.context.resolvedTarget.ids,['a','b']);assert.deepEqual(timing.context.fileNames,['a.png','b.png']);assert.equal(timing.sameArray,true);assert.equal(timing.sameAsset,true);assert.equal(timing.storeNames.length,8);assert.equal(timing.sessionReady,true);
 report.selectionMs=timing.ms;report.checks.push('Eight stores load; legacy and new selection/assets share canonical data; SelectionContext updates within 100ms');
 const patch=await page.evaluate(()=>{
  const q=window.__dataQA,b=q.assets.index.b;q.assets.update('a',{processingStatus:'done'});q.applyTaskEvent({type:'task.queue',payload:{taskId:'task-a',queuePosition:2}});q.applyTaskEvent({type:'task.progress',payload:{taskId:'task-a',done:1,total:2}});
  return {unaffected:q.assets.index.b===b,status:q.assets.index.a.processingStatus,task:JSON.parse(JSON.stringify(q.tasks.byId['task-a']))};
 });
 assert.equal(patch.unaffected,true);assert.equal(patch.status,'done');assert.equal(patch.task.state,'running');assert.equal(patch.task.queuePosition,2);report.checks.push('Asset event patches only the matching record; task progress/queue updates retain identity');
 const workers=await page.evaluate(async()=>{
  const hash=await import('/src/services/hash.ts'),thumb=await import('/src/services/thumbnails.ts');
  const abc=await hash.hashBlob(new Blob(['abc']));const canvas=new OffscreenCanvas(320,160);canvas.getContext('2d').fillRect(0,0,320,160);const blob=await canvas.convertToBlob({type:'image/png'}),key=await hash.hashBlob(blob);
  const first=await thumb.thumbnailBlob(blob,key,160);thumb.disposeThumbnailWorker();const second=await thumb.thumbnailBlob(blob,key,160);
  const bitmap=await createImageBitmap(first.blob);const result={abc,first:{width:first.width,height:first.height,cacheHit:first.cacheHit,bytes:first.blob.size},second:{cacheHit:second.cacheHit},decodedWidth:bitmap.width,decodedHeight:bitmap.height};bitmap.close();thumb.disposeThumbnailWorker();hash.disposeHashWorker();return result;
 });
 assert.equal(workers.abc,'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');assert.deepEqual([workers.first.width,workers.first.height],[160,80]);assert.equal(workers.first.cacheHit,false);assert.equal(workers.second.cacheHit,true);assert.equal(workers.decodedWidth,160);assert(workers.first.bytes>0);report.workers=workers;report.checks.push('Real hash Worker SHA-256 matches known digest; thumbnail Worker uses OffscreenCanvas and survives recreation through IndexedDB cache');
 const requestCount=report.requests.length;
 const mock=await page.evaluate(async()=>{
  const url=performance.getEntriesByType('resource').find(entry=>new URL(entry.name).pathname==='/src/services/workspace.svelte.js').name;const {initializeWorkspace}=await import(url);const q=window.__dataQA,api=q.createApi('mock');await initializeWorkspace(api);return {mode:api.mode,count:q.project.items.length,status:q.session.status};
 });
 assert.equal(report.requests.length,requestCount);assert.equal(mock.mode,'mock');assert.equal(mock.status,'ready');assert.equal(mock.count,0);report.checks.push('Switching workspace to the mock adapter produces no HTTP or AI requests');
 assert.deepEqual(report.pageErrors,[]);report.status='passed';console.log(JSON.stringify(report,null,2));
}catch(error){report.status='failed';report.error=error.stack;throw error;}
finally{await fs.writeFile(`${output}/report.json`,JSON.stringify(report,null,2));await browser.close();}
