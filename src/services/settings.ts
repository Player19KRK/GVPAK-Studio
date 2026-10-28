import {invoke} from './http.ts';
const defaults=()=>({analysisCount:20,generationCount:4,costCny:2,filterCount:50,imageModel:'gemini-nano-banana-2.1',quotas:{}});
export function createSettingsApi(mode='real',fetcher:typeof fetch=fetch,{allowSensitiveSimulation=true}={}){let mock=defaults(),credentials={google:false,deepseek:false,poe:false};const call=(name:string,args:any={})=>invoke(name,args,fetcher);const names={google:'credential',deepseek:'assistant_key',poe:'poe_key'};
 return {mode,
 read:async()=>mode==='mock'?structuredClone(mock):call('settings_read'),
 execution:async()=>mode==='mock'?{...structuredClone(mock),quota:null}:call('execution_preferences'),
 patch:async(patch:any)=>{if(mode==='mock'){mock={...mock,...patch,quotas:{...mock.quotas,...patch.quotas}};return structuredClone(mock);}return call('settings_patch',{patch});},
 credentials:async()=>{if(mode==='mock')return {...credentials};const [google,deepseek,poe]=await Promise.all([call('credential_status'),call('assistant_key_status'),call('poe_key_status')]);return {google,deepseek,poe};},
 setCredential:async(provider:keyof typeof names,key:string)=>{if(mode==='mock'){if(!allowSensitiveSimulation)throw Error('模拟模式不保存真实API密钥。');credentials[provider]=true;return;}await call(names[provider]+'_set',{key});},
 tools:async()=>mode==='mock'?{available:false,message:'模拟模式未检查本机工具'}:call('assistant_tools'),
 certificate:async()=>mode==='mock'?{fingerprint:null,algorithm:'SHA-256',mock:true}:call('certificate_status'),
 regenerate:async()=>{if(mode==='mock'){if(!allowSensitiveSimulation)throw Error('模拟模式不生成或更换证书。');return {fingerprint:null,mock:true};}return call('certificate_regenerate');},
 installer:async()=>{if(mode==='mock')throw Error('模拟模式不生成真实证书脚本。');return call('certificate_installer');},
 accounts:async(input:any={})=>mode==='mock'?{accounts:[],urls:[]}:call('team_accounts',input),
 };
}
let singleton:ReturnType<typeof createSettingsApi>;
export const getSettingsApi=()=>singleton||=createSettingsApi((import.meta as ImportMeta&{env?:Record<string,string>}).env?.VITE_FRONTEND_DATA_MODE||'real',fetch,{allowSensitiveSimulation:false});
