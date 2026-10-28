import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {compileModule} from 'svelte/compiler';
let serial=0;
const moduleUrl=code=>'data:text/javascript;base64,'+Buffer.from(code).toString('base64');
async function create(api){
 let source=await fs.readFile('src/stores/new-product.svelte.js','utf8');
 const stub=moduleUrl('export const getNewProductApi=()=>null,chooseProductDirectory=()=>{},restoreProductOutput=async()=>null,rememberProductOutput=()=>{},writeProductResult=()=>{},productZip=()=>{},downloadProductBlob=()=>{};');
 for(const spec of ['../services/new-product.ts','../services/new-product-files.ts'])source=source.replace(JSON.stringify(spec).replaceAll('"',"'"),JSON.stringify(stub));
 for(const name of ['target','ratio','new-product'])source=source.replace("'../lib/"+name+".ts'",JSON.stringify(pathToFileURL(path.resolve('src/lib/'+name+'.ts')).href));
 const code=compileModule(source,{filename:'np-read-'+serial+++'.svelte.js',generate:'client'}).js.code.replace(/['"](svelte(?:\/internal\/client)?)['"]/g,(_,name)=>JSON.stringify(import.meta.resolve(name)));
 return (await import(moduleUrl(code))).createNewProductStore(api);
}
test('new product capabilities distinguish pending, failure, unconfigured and configured without stale readiness',async()=>{
 let release,fail=true;const np=await create({capabilities:async()=>{if(fail)throw Error('read failed');return new Promise(r=>release=r);}});
 await np.reloadCapabilities();assert.equal(np.state.capabilityState,'error');assert.equal(np.state.capabilities,null);
 fail=false;let pending=np.reloadCapabilities();assert.equal(np.state.capabilityState,'loading');release({configured:false});await pending;assert.equal(np.state.capabilityState,'ready');assert.equal(np.state.capabilities.configured,false);assert.equal(np.state.capabilityError,'');
 pending=np.reloadCapabilities();assert.equal(np.state.capabilities,null);release({configured:true});await pending;assert.equal(np.state.capabilities.configured,true);np.dispose();
});
test('disposed new product reads never adopt a late response',async()=>{
 let release;const np=await create({capabilities:()=>new Promise(r=>release=r)}),pending=np.reloadCapabilities();np.dispose();release({configured:true});await pending;assert.equal(np.state.capabilities,null);
});
