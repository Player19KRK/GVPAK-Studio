import test from 'node:test';import assert from 'node:assert/strict';
let files=[],store=new Map(),acks=[],pickerCalls=0,failWrite=false,failClose=false,corrupt=false,failAck=false;
globalThis.window={__gvpakTeamUser:{id:'fixture'},isSecureContext:true,dispatchEvent(){},confirm:()=>true,showDirectoryPicker:async()=>{pickerCalls++;return directory;}};
const directory={name:'同事电脑目录',queryPermission:async()=> 'granted',getFileHandle:async(name,options)=>{
 if(!options&&!store.has(name)){const e=Error();e.name='NotFoundError';throw e;}
 return {getFile:async()=>new Blob([corrupt?'bad':store.get(name)]),createWritable:async()=>{let buffer;return {write:async b=>{if(failWrite)throw Error('disk full');buffer=await b.text();},close:async()=>{if(failClose)throw Error('close failed');store.set(name,buffer);},abort:async()=>{}}}};
}};
globalThis.fetch=async(url,options)=>{if(url==='/api/files')return {ok:true,json:async()=>({files})};if(url==='/api/files/saved'){const body=JSON.parse(options.body);acks.push(body);if(failAck)return {ok:false};files=files.filter(f=>f.url.split('/').at(-1)!==body.id);return {ok:true};}return {ok:true,blob:async()=>new Blob(['image-data'])};};
const app=await import('../src/local-downloads.js');
test('directory permission is requested from the direct user action',async()=>{const task=app.chooseDirectory();assert.equal(pickerCalls,1);assert.equal(await task,'同事电脑目录');});
test('acknowledges only after close and read-back hash validation',async()=>{acks=[];await app.saveFile({name:'a.webp',url:'/api/download/a'});assert.equal(acks.length,1);assert.equal(acks[0].bytes,10);assert.equal(acks[0].sha256.length,64);assert.equal(store.get('a.webp'),'image-data');});
test('write, close and verification failures never acknowledge deletion',async()=>{for(const mode of ['write','close','corrupt']){acks=[];failWrite=mode==='write';failClose=mode==='close';corrupt=mode==='corrupt';await assert.rejects(app.saveFile({name:mode+'.webp',url:'/api/download/'+mode}));assert.equal(acks.length,0);}failWrite=false;failClose=false;corrupt=false;});
test('cleanup failure retries acknowledgement without writing a second copy',async()=>{acks=[];failAck=true;await assert.rejects(app.saveFile({name:'retry.webp',url:'/api/download/retry'}));const count=store.size;failAck=false;await app.saveFile({name:'retry.webp',url:'/api/download/retry'});assert.equal(store.size,count);assert.equal(acks.length,2);});
test('new files are automatically saved once and removed from pending list',async()=>{acks=[];files=[{name:'auto.webp',url:'/api/download/auto'}];await app.saveNewFiles();await app.saveNewFiles();assert.equal(acks.length,1);assert.equal(files.length,0);});

test('ordinary browser download never cleans without explicit saved confirmation',async()=>{const originalPicker=window.showDirectoryPicker,oldTimer=globalThis.setTimeout;window.showDirectoryPicker=undefined;let clicks=0;globalThis.document={createElement:()=>({click(){clicks++;}})};globalThis.setTimeout=(fn,ms)=>ms===30000?0:oldTimer(fn,ms);const manual=await import('../src/local-downloads.js?manual');acks=[];await manual.chooseDirectory();await manual.saveFile({name:'manual.webp',url:'/api/download/manual'});assert.equal(clicks,1);assert.equal(acks.length,0);window.confirm=()=>false;assert.equal(await manual.confirmSaved({name:'manual.webp',url:'/api/download/manual'}),false);assert.equal(acks.length,0);window.confirm=()=>true;assert.equal(await manual.confirmSaved({name:'manual.webp',url:'/api/download/manual'}),true);assert.equal(acks.length,1);window.showDirectoryPicker=originalPicker;globalThis.setTimeout=oldTimer;});

test('refresh restores an authorized directory but never requests expired permission automatically',async()=>{
 const dbValues=new Map();globalThis.indexedDB={open(){const request={};queueMicrotask(()=>{request.result={close(){},transaction(){const tx={};tx.objectStore=()=>({put(value,key){const r={};queueMicrotask(()=>{dbValues.set(key,value);r.result=key;r.onsuccess?.();tx.oncomplete?.();});return r;},get(key){const r={};queueMicrotask(()=>{r.result=dbValues.get(key);r.onsuccess?.();tx.oncomplete?.();});return r;}});return tx;}};request.onsuccess();});return request;}};
 files=[];await app.chooseDirectory();const restored=await import('../src/local-downloads.js?restored');await restored.restoreDirectory();assert.equal(restored.directoryName(),'同事电脑目录');
 directory.queryPermission=async()=> 'prompt';const expired=await import('../src/local-downloads.js?expired');await expired.restoreDirectory();assert.equal(expired.directoryName(),'');directory.queryPermission=async()=> 'granted';
 window.__gvpakTeamUser={id:'different-account'};const other=await import('../src/local-downloads.js?other-account');await other.restoreDirectory();assert.equal(other.directoryName(),'');window.__gvpakTeamUser={id:'fixture'};
});


test('save as opens the file picker during the click and verifies before cleanup',async()=>{
 let picked=0;window.showSaveFilePicker=async options=>{picked++;assert.equal(options.suggestedName,'desktop.webp');return directory.getFileHandle('desktop.webp',{create:true});};
 acks=[];const work=app.saveFileAs({name:'desktop.webp',url:'/api/download/desktop'});assert.equal(picked,1);await work;assert.equal(store.get('desktop.webp'),'image-data');assert.equal(acks.length,1);
 window.showSaveFilePicker=async()=>{throw Object.assign(Error(),{name:'AbortError'});};acks=[];assert.equal(await app.saveFileAs({name:'cancel.webp',url:'/api/download/cancel'}),null);assert.equal(acks.length,0);
 delete window.showSaveFilePicker;
});
