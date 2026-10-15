import {api} from './api';import {PreviewCache} from './preview-cache';
export const previewCache=new PreviewCache();
const pending=new Map();let running=false,queue=[];const stats={requests:0,completed:0,discarded:0,lastRequestMs:0,byCommand:{}};let published=null;
const aborted=()=>new DOMException('Preview no longer needed','AbortError');
async function decode(blob){if(typeof createImageBitmap==='function')return createImageBitmap(blob,{premultiplyAlpha:'none',colorSpaceConversion:'none'});const url=URL.createObjectURL(blob);try{const image=new Image();image.src=url;await image.decode();return image;}finally{URL.revokeObjectURL(url);}}
export function requestPreview(key,command,args,{priority=0,signal}={}){
 const hit=previewCache.get(key);if(hit)return signal?.aborted?Promise.reject(aborted()):Promise.resolve(hit);
 if(signal?.aborted)return Promise.reject(aborted());
 return new Promise((resolve,reject)=>{let job=pending.get(key);if(!job){job={key,command,args,priority,clients:new Set(),started:false};pending.set(key,job);queue.push(job);}else job.priority=Math.min(job.priority,priority);
 const client={resolve,reject,signal,abort:null};client.abort=()=>{job.clients.delete(client);reject(aborted());if(!job.started&&!job.clients.size){pending.delete(key);queue=queue.filter(j=>j!==job);stats.discarded++;}};job.clients.add(client);signal?.addEventListener('abort',client.abort,{once:true});pump();});
}
async function pump(){if(running)return;running=true;while(queue.length){queue.sort((a,b)=>a.priority-b.priority);const job=queue.shift();if(!job.clients.size){pending.delete(job.key);continue;}job.started=true;const started=performance.now();try{stats.requests++;stats.byCommand[job.command]=(stats.byCommand[job.command]||0)+1;const blob=await api(job.command,job.args);const bitmap=await decode(blob);const entry=previewCache.put(job.key,blob,bitmap);stats.completed++;stats.lastRequestMs=performance.now()-started;for(const c of job.clients){c.signal?.removeEventListener('abort',c.abort);if(!c.signal?.aborted)c.resolve(entry);}}catch(e){for(const c of job.clients){c.signal?.removeEventListener('abort',c.abort);if(!c.signal?.aborted)c.reject(e);}}finally{pending.delete(job.key);}}running=false;}
export function publishPreview(entry){if(published===entry)return;previewCache.retain(entry);previewCache.release(published);published=entry;}
export function previewStats(){return {...previewCache.stats(),...stats,queued:queue.length,running};}
export function disposePreviews(){for(const job of queue)for(const c of job.clients)c.abort();queue=[];previewCache.release(published);published=null;previewCache.clear();}

