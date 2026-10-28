import {getApi} from './services/api.ts';
import {invoke} from './services/http.ts';
import {readLocalSetting} from './branding-migration.js';
import {chooseDirectory} from './local-downloads.js';
export const teamUser=typeof window==='undefined'?null:window.__gvpakTeamUser||null;
export const isTeam=!!teamUser;
export const isMock=import.meta.env?.VITE_FRONTEND_DATA_MODE==='mock';
export async function api(name,args={}){if(isTeam||isMock){if(name==='pick_output'&&!isMock)return chooseDirectory();if(name==='pick_import')return pickUpload(args.projectId,args.folder);if(name==='hybrid_open_output'){window.dispatchEvent(new Event('team-downloads'));return null;}}
try{return await invoke(name,args);}catch(e){if(e.status===401&&isTeam){location.reload();throw Error('Session expired');}throw Error(errorText(e.message));}}
export async function imageUrl(name,args){return URL.createObjectURL(await api(name,args));}
export async function upload(projectId,files){if(isMock)return getApi().importFiles(projectId,files);let total=null,items=[],size=0;
 async function send(){if(!items.length)return;const r=await fetch('/api/upload',{method:'POST',headers:{'Content-Type':'application/json','X-GVPAK-IPC':'1'},body:JSON.stringify({projectId,files:items})});const result=await r.json();if(!r.ok)throw Error(`已导入 ${total?.imported||0} 张。${result.error||'上传失败，请重试剩余图片。'}`);total={...result,imported:(total?.imported||0)+result.imported,skipped:[...(total?.skipped||[]),...result.skipped],errors:[...(total?.errors||[]),...result.errors]};items=[];size=0;}
 for(const file of files){if(file.size>30*1024*1024)throw Error(`${file.name} 超过单张 30 MB。请缩小此文件后再上传。`);if(size+file.size>42*1024*1024||items.length>=50)await send();const data=await new Promise((resolve,reject)=>{const r=new FileReader();r.onload=()=>resolve(r.result.split(',')[1]);r.onerror=reject;r.readAsDataURL(file);});items.push({name:file.name,data});size+=file.size;}await send();return total;}
export const initialEdit={adjustmentOpacity:1,adjustmentEnabled:true,exposure:0,contrast:1,saturation:1,temperature:0,rotation:0,flipH:false,flipV:false,crop:null,strokes:[]};
export const clone=x=>JSON.parse(JSON.stringify(x));
export const bytes=n=>n>=1e6?`${(n/1e6).toFixed(1)} MB`:`${Math.round(n/1000)} KB`;
export const rgb=hex=>[1,3,5].map(i=>parseInt(hex.slice(i,i+2),16));
export const hex=arr=>'#'+arr.map(n=>Math.round(n).toString(16).padStart(2,'0')).join('');
export function stored(key,fallback){if(isTeam&&key==='assistant-output')return '浏览器下载目录';try{let value=JSON.parse(isTeam?localStorage.getItem('gvpak-team-'+teamUser.id+'-'+key):readLocalSetting(localStorage,key))??fallback;if(isTeam&&key==='export')value={...value,outputDir:'浏览器下载目录'};return value;}catch{return fallback;}}

export const persist=(key,v)=>localStorage.setItem((isTeam?'gvpak-team-'+teamUser.id+'-':'gvpak-studio-')+key,JSON.stringify(v));

function pickUpload(projectId,folder){return new Promise((resolve,reject)=>{const input=document.createElement('input');input.type='file';input.multiple=true;input.accept='image/png,image/jpeg,image/webp';if(folder)input.setAttribute('webkitdirectory','');input.style.display='none';document.body.append(input);input.oncancel=()=>{input.remove();resolve(null);};input.onchange=async()=>{try{const files=[...input.files].filter(f=>/\.(png|jpe?g|webp)$/i.test(f.name));resolve(files.length?await upload(projectId,files):null);}catch(e){reject(e);}finally{input.remove();}};input.click();});}

export function errorText(message){const code=String(message).split(':')[0];return ({LOGIN_REQUIRED:'请先登录。',INVALID_LOGIN:'用户名或密码不正确。',ORIGIN_REJECTED:'连接地址未获允许。请使用管理员提供的网址。',HOST_OPERATION_BLOCKED:'此操作只能由主机管理员执行。',WORKSPACE_UNAVAILABLE:'工作区暂时不可用，请刷新后重试。',PROCESSING_FAILED:'处理失败，请提交问题给管理员。',POE_API_ERROR:'Poe 请求失败，请管理员检查密钥、额度和模型权限。',NO_IMAGE:'供应商未返回可读取的图片，请查看任务记录。',IMAGE_DOWNLOAD_FAILED:'生成结果下载失败，请查看供应商任务记录，避免重复生成。',POE_KEY_REQUIRED:'请联系主机管理员配置 Poe API 密钥。',KEY_NOT_CONFIGURED:'请先配置生图服务的 API 密钥。',TASK_CANCELLED:'任务已取消。',FILE_TOO_LARGE:'单张图片超过大小限制，请缩小后重试。',PROJECT_HAS_ACTIVE_TASKS:'项目仍有任务正在处理，请先取消任务。',PROJECT_NOT_FOUND:'项目不存在或不属于当前账号。',ASSET_NOT_FOUND:'图片不存在或不属于当前账号。',EXPORT_TARGET_UNREACHABLE:'未达到文件大小要求，请提高上限或调整尺寸。',SAMPLE_STALE:'图片已修改，请重新测试一张。',SESSION_LIMIT:'登录会话过多，请稍后重试。',TRY_AGAIN_IN_ONE_MINUTE:'登录尝试过多，请一分钟后重试。'})[code]||message;}
