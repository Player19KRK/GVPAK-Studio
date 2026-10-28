import catalog from '../../../config/models.json' with {type:'json'};
import {fileNameOnly} from '../../lib/target.ts';
const copy=value=>structuredClone(value);
const edit=()=>({adjustmentOpacity:1,adjustmentEnabled:true,exposure:0,contrast:1,saturation:1,temperature:0,rotation:0,flipH:false,flipV:false,crop:null,strokes:[]});
export function createMockWorkspace(projects:any[]){
 const originals=new Map<string,Blob>(),recycled=new Map<string,any>();
 const find=(id:string)=>{const p=projects.find(p=>p.id===id);if(!p)throw Error('模拟项目不存在。');return p;};
 const create=async(name:string)=>{const p={id:crypto.randomUUID(),name,assets:[],createdAt:Date.now()/1000,updatedAt:Date.now()/1000};projects.push(p);return copy(p);};
 async function importFiles(projectId:string,files:File[]){const p=find(projectId),skipped:string[]=[],errors:string[]=[];let imported=0;
  for(const file of files){try{if(file.size>30*1024*1024)throw Error('单张图片不能超过30MB');const hash=[...new Uint8Array(await crypto.subtle.digest('SHA-256',await file.arrayBuffer()))].map(b=>b.toString(16).padStart(2,'0')).join('');const existing=p.assets.find(a=>a.sha256===hash);if(existing){existing.trashed=false;originals.set(existing.id,file);skipped.push(fileNameOnly(file.name));continue;}let width=0,height=0;if(typeof createImageBitmap==='function'){const bitmap=await createImageBitmap(file);width=bitmap.width;height=bitmap.height;bitmap.close();}const id=crypto.randomUUID();p.assets.push({id,name:fileNameOnly(file.name),sourcePath:fileNameOnly(file.name),original:'mock/'+id,sha256:hash,width,height,bytes:file.size,trashed:false,edit:edit(),versions:[],generatedBase:null,folderId:'',relativePath:fileNameOnly(file.name),tags:[],mock:true});originals.set(id,file);imported++;}catch(error){errors.push(fileNameOnly(file.name)+'：'+error.message);}}
  p.updatedAt=Date.now()/1000;return {project:copy(p),imported,skipped,errors};
 }
 async function legacy(name:string,args:any={}){
  if(name==='bootstrap')return {projects:copy(projects),dataRoot:'mock',warnings:[],catalog:copy(catalog),jobs:[],tasks:[]};
  if(name==='model_catalog')return {catalog:copy(catalog)};
  if(name==='hybrid_jobs'||name==='new_product_jobs')return {jobs:[],notices:[]};
  if(name==='session_status')return {team:false,isHost:true,isLocal:true,quota:null,mock:true};
  if(name==='credential_status'||name==='poe_key_status'||name==='assistant_key_status')return false;
  if(name==='assistant_presets')return {};
  if(['export_preview','selection_export_preview','export_run'].includes(name))throw Error('模拟模式不进行导出编码或保存。');
  if(name==='assistant_tools')return {available:false,mock:true,message:'模拟模式未检查本机工具。'};
  if(name==='project_create')return create(args.name);
  if(name==='project_rename'){const p=find(args.projectId);p.name=args.name;return copy(p);}
  if(name==='project_delete'){const index=projects.findIndex(p=>p.id===args.projectId);if(index<0)throw Error('模拟项目不存在。');recycled.set(args.projectId,projects[index]);projects.splice(index,1);return true;}
  if(name==='project_restore'){const p=recycled.get(args.projectId);if(!p)throw Error('模拟回收项目不存在。');projects.push(p);recycled.delete(args.projectId);return true;}
  if(name==='asset_trash'){const p=find(args.projectId);for(const asset of p.assets)if(args.ids.includes(asset.id))asset.trashed=!args.restore;return copy(p);}
  if(['thumbnail','preview','preview_source','original_preview'].includes(name)){const p=find(args.projectId),a=p.assets.find(a=>a.id===args.assetId),blob=originals.get(args.assetId);if(!a||!blob)throw Error('模拟素材尚无原图字节，请导入图片。');if(name==='preview'){const e=args.edit||a.edit;if(e.exposure||e.rotation||e.flipH||e.flipV||e.crop||e.strokes?.length||e.contrast!==1||e.saturation!==1||e.temperature)throw Error('模拟模式不生成编辑后的精确预览。');}return blob;}
  throw Error('模拟模式未执行 '+name+'；真实后台与输出目录保持隔离。');
 }
 return {createProject:create,importFiles,legacy,catalog:copy(catalog)};
}
