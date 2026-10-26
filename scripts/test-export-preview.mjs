import test from 'node:test';
import assert from 'node:assert/strict';
import {createExportPreview} from '../src/export-preview.js';
const tick=()=>new Promise(resolve=>setTimeout(resolve,0));
const result=id=>({data:btoa('after'),before:btoa('before'),mime:'image/webp',bytes:id,width:id,height:100,status:'ok'});

test('export preview serializes encoding and publishes only the latest queued settings',async()=>{
 const requests=[],states=[],pending=[],dropped=[];
 let urls=0;
 const preview=createExportPreview({request:args=>{requests.push(args.id);return new Promise(resolve=>pending.push(resolve));},onChange:s=>states.push(s),makeURL:()=>`url-${++urls}`,dropURL:u=>dropped.push(u)});
 preview.set({id:1},0);await tick();
 preview.set({id:2},0);await tick();preview.set({id:3},0);await tick();
 assert.deepEqual(requests,[1]);
 pending.shift()(result(1));await tick();assert.deepEqual(requests,[1,3]);
 pending.shift()(result(3));await tick();
 assert.equal(states.at(-1).result.bytes,3);assert.equal(urls,2);
 preview.set(null);assert.deepEqual(dropped,['url-1','url-2']);assert.equal(states.at(-1).result,null);
 preview.dispose();
});
test('export preview errors are visible and late results after disposal allocate no URLs',async()=>{
 let state,resolve,urls=0;
 const preview=createExportPreview({request:()=>new Promise(r=>resolve=r),onChange:s=>state=s,makeURL:()=>{urls++;return 'url';},dropURL:()=>{}});
 preview.set({id:1},0);await tick();preview.dispose();resolve(result(1));await tick();assert.equal(urls,0);
 const failing=createExportPreview({request:async()=>{throw Error('INVALID_EXPORT');},onChange:s=>state=s});
 failing.set({id:1},0);await tick();await tick();assert.equal(state.error,'INVALID_EXPORT');assert.equal(state.loading,false);failing.dispose();
});
