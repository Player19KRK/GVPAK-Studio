import {invoke} from './http.ts';import {serializeSelectionContext} from '../lib/target.ts';
export function createAssistantApi(mode='real',fetcher:typeof fetch=fetch){let presets:Record<string,any>={};return {mode,
 async presets(){return mode==='mock'?structuredClone(presets):invoke('assistant_presets',{},fetcher);},
 async savePreset(name:string,plan:any){if(mode==='mock'){presets[name]=JSON.parse(JSON.stringify(plan));return;}return invoke('assistant_save_preset',{name,plan},fetcher);},
 async legacyJobs(){return mode==='mock'?{jobs:[],notices:[]}:invoke('hybrid_jobs',{},fetcher);},
 async openLegacy(jobId:string){if(mode==='mock')throw Error('模拟模式没有真实输出目录。');return invoke('hybrid_open_output',{jobId},fetcher);},
 async cancelLegacy(jobId:string){return mode==='mock'?undefined:invoke('hybrid_cancel',{jobId},fetcher);},
 async ackLegacy(jobId:string){return mode==='mock'?undefined:invoke('hybrid_ack',{jobId},fetcher);},
 async configured(){return mode==='mock'?false:invoke('assistant_key_status',{},fetcher);},
 async plan(projectId:string,context:any,instruction:string){if(mode==='mock')throw Error('模拟模式不调用模型，请直接添加本地步骤。');return invoke('assistant_context_plan',{projectId,context:serializeSelectionContext(context),instruction:instruction+'\n只使用SelectionContext；未提供尺寸或像素时不推测图像属性。'},fetcher);},
 };}
