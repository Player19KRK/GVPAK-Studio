import fs from 'node:fs/promises';
import path from 'node:path';
import https from 'node:https';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
const base='http://127.0.0.1:8787',hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const call=async(name)=>{const response=await fetch(base+'/api/invoke/'+name,{method:'POST',headers:{'Content-Type':'application/json','X-GVPAK-IPC':'1'},body:'{}'});assert(response.ok,name+' unavailable');return response.json();};
const workspace=await call('bootstrap'),tools=await call('assistant_tools'),capabilities=await call('new_product_capabilities');
assert.equal(tools.available,true);assert(capabilities.analysisModel);assert(capabilities.imageModel);
const index=await fs.readFile('dist/index.html'),assetId=workspace.projects.flatMap(p=>p.assets).find(a=>!a.trashed)?.id;
const routes=['/','/library','/generate','/settings',...(assetId?['/editor/'+encodeURIComponent(assetId)]:[])];
for(const route of routes){const response=await fetch(base+route);assert.equal(response.status,200);assert.equal(hash(Buffer.from(await response.arrayBuffer())),hash(index),'SPA route mismatch '+route);}
for(const route of ['/assets/delivery-file-does-not-exist.js','/api/delivery-endpoint-does-not-exist']){const response=await fetch(base+route);assert.equal(response.status,404);assert.notEqual(hash(Buffer.from(await response.arrayBuffer())),hash(index));}
let files=0;
async function verify(directory){for(const entry of await fs.readdir(directory,{withFileTypes:true})){const file=path.join(directory,entry.name);if(entry.isDirectory())await verify(file);else{const relative=path.relative('.tools/new-product-trial/runtime/dist',file),source=await fs.readFile(file),deployed=await fs.readFile(path.join('dist',relative));assert.equal(hash(deployed),hash(source),relative);const response=await fetch(base+'/'+relative.split(path.sep).map(encodeURIComponent).join('/'));assert.equal(response.status,200,relative);assert.equal(hash(Buffer.from(await response.arrayBuffer())),hash(source),relative);files++;}}}
await verify('.tools/new-product-trial/runtime/dist');
const ca=await fs.readFile('backend/team-private/server.crt');
const team=await new Promise((resolve,reject)=>{https.get('https://127.0.0.1:8788/api/team/session',{ca},response=>{let body='';response.on('data',value=>body+=value);response.on('end',()=>{try{assert.equal(response.statusCode,200);resolve(JSON.parse(body));}catch(error){reject(error);}});}).on('error',reject);});
assert.equal(team.team,true);
const receipt={status:'passed',readonly:true,providerRequests:0,routes,staticFilesVerified:files,openCvVersion:tools.version,newProductConfigured:capabilities.configured,teamTlsCertificateVerified:true,servedIndexSha256:hash(index),projects:workspace.projects.length,assets:workspace.projects.reduce((n,p)=>n+p.assets.length,0),checks:['all supported page routes return the current application index','unknown static and API URLs retain 404 semantics','every shipped build file matches both disk and HTTP bytes','one-click runtime exposes OpenCV and the integrated new product module','team HTTPS validates against its existing certificate']};
await fs.writeFile('output/playwright/finalization/http.json',JSON.stringify(receipt,null,2));console.log(JSON.stringify(receipt,null,2));
