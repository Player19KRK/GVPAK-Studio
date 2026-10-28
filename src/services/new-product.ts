import {createApi,getApi} from './api.ts';
import {invoke} from './http.ts';
import {appEvents} from './realtime.ts';
import {uniqueProductFiles} from './new-product-files.ts';
import {serializeSelectionContext} from '../lib/target.ts';
export function createNewProductApi(mode='real',fetcher:typeof fetch=fetch,shared:any=null){
  const mock=mode==='mock'?(shared||createApi('mock',{seed:{allowTaskSimulation:false}})):null;
  const call=(name:string,args:any={})=>mock?mock.legacy(name,args):invoke(name,args,fetcher);

  return {
    mode,
    async capabilities(){return mode==='mock'?{configured:false,mock:true,analysisModel:'gemini-3.8-flash',imageModel:'gemini-nano-banana-2.1',maxInputs:40,maxConcurrency:4}:call('new_product_capabilities');},
    async importFiles(files:File[],label:string,projectId:string|null,options:{limit?:number;projectName?:string;loadThumbnails?:boolean}={}){
      const rows=await uniqueProductFiles(files,options.limit||40);
      const state=await import('../studio.svelte');let project=projectId?state.studio.projects.find((item:any)=>item.id===projectId):null;
      if(!project)project=await call('project_create',{name:options.projectName||`新品 · ${label||'图片套图'}`});
      const existing=new Set(project.assets.map((asset:any)=>asset.sha256)),missing=rows.filter(row=>!existing.has(row.hash));
      if(mode==='mock')project=(await mock.importFiles(project.id,rows.map(row=>row.file))).project;
      else if(missing.length){const {upload}=await import('../api.js');project=(await upload(project.id,missing.map(row=>row.file))).project;}
      state.updateProject(project);state.studio.pid=project.id;const assets=rows.map(row=>{const asset=project.assets.find((asset:any)=>asset.sha256===row.hash&&!asset.trashed);if(!asset)throw Error('部分素材导入失败，请重试。');return{id:asset.id,name:asset.name,hash:row.hash,aliases:row.aliases,sourceFile:row.file,previewUrl:URL.createObjectURL(row.file)};});
      state.studio.selected=assets.map(row=>row.id);state.select(assets[0].id,true);if(options.loadThumbnails!==false)await state.thumbnails(project,false,assets.map(row=>row.id));return{projectId:project.id,assets};
    },
    async analyze(input:any){if(mode==='mock')throw Error('模拟模式不进行图像识别；请切换真实模式后识别。');return call('new_product_analyze',{...input,context:serializeSelectionContext(input.context)});},
    async start(input:any){if(mode==='mock')throw Error('模拟模式不生成套图或假结果。');return call('new_product_start',{...input,context:serializeSelectionContext(input.context)});},
    async jobs(projectId?:string){return mode==='mock'?{jobs:[]}:call('new_product_jobs',projectId?{projectId}:{});},
    async cancel(jobId:string){if(mode==='mock')throw Error('模拟模式没有生成任务。');return call('new_product_cancel',{jobId});},
    result(jobId:string,taskId:string):Promise<Blob>{if(mode==='mock')return Promise.reject(Error('模拟任务不会返回假图片。'));return call('new_product_result',{jobId,taskId});},
    subscribe(listener:()=>void){return appEvents.subscribe(event=>{if(event.type.startsWith('new-product.'))listener();});},
    async saveKey(key:string){if(mode==='mock')throw Error('模拟模式不保存真实API密钥。');return call('credential_set',{key});},
  };
}
let singleton:ReturnType<typeof createNewProductApi>;
export function getNewProductApi(){const mode=(import.meta as any).env?.VITE_FRONTEND_DATA_MODE||'real';if(!['real','mock'].includes(mode))throw Error('数据模式无效。');return singleton||=createNewProductApi(mode,fetch,getApi());}
