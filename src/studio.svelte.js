import {loadThumbnailSource,thumbnailInput,thumbnailSourceKey} from './services/thumbnail-cache.ts';
import {isManagedTask} from './lib/task-ownership.js';
import {appEvents} from './services/realtime.ts';
import {untrack} from 'svelte';
import {normalizeEdit} from './preview-cache';
import {disposePreviews} from './preview-loader';
import {api,imageUrl,initialEdit,clone,upload,stored,persist,isMock} from './api';
export const studio=$state({projects:[],projectWarnings:[],pid:'',aid:'',draft:{...initialEdit},drafts:{},draftBases:{},draftStatus:'',thumbs:{},selected:[],batchScope:'auto',mode:'edit',workflowChat:null,tool:'adjust',paintTool:null,paintMenuOpen:false,picking:false,spacePan:false,brush:{color:'#e32a32',size:24,hardness:.8,opacity:1},eraser:{size:40,hardness:.8},view:'canvas',trash:false,compare:false,split:50,zoom:1,canvasFit:true,pan:{x:0,y:0},background:stored('background','paper'),preview:'',previewAssetId:'',original:'',previewBusy:false,previewSignature:'',previewEngine:'checking',gpuPreviewEnabled:stored('gpu-preview-enabled',true),previewDiagnostics:{},busy:'',progress:null,error:'',startupState:'idle',startupError:'',toast:'',toastAction:null,dataRoot:'',ready:false,history:{},redo:{},jobs:[],notices:[],modal:'',report:null,confirm:null,settingsReturn:'',processTab:'auto',catalog:null,currency:stored('currency','CNY'),exchangeRate:stored('exchangeRate',7.2),inspector:true,inspectorTab:'adjust'});
export const current=()=>studio.projects.find(p=>p.id===studio.pid);
export const asset=()=>current()?.assets.find(a=>a.id===studio.aid);
export const assets=()=>current()?.assets.filter(a=>a.trashed===studio.trash)||[];
export const liveAssets=()=>current()?.assets.filter(a=>!a.trashed)||[];
export function scope(){const live=liveAssets();if(studio.batchScope==='selection')return studio.selected.filter(id=>live.some(a=>a.id===id));if(studio.batchScope==='project')return live.map(a=>a.id);if(studio.batchScope!=='current'&&studio.selected.length)return studio.selected.filter(id=>live.some(a=>a.id===id));return live.some(a=>a.id===studio.aid)?[studio.aid]:[];}
export const scopeLabel=()=>studio.batchScope==='project'?`全部 ${scope().length} 张图片`:studio.batchScope!=='current'&&studio.selected.length?`已勾选 ${scope().length} 张`:"当前图片";
export const editFor=a=>studio.drafts[a.id]||a.edit||initialEdit;
const equal=(a,b)=>JSON.stringify(normalizeEdit(a||initialEdit))===JSON.stringify(normalizeEdit(b||initialEdit));
const fingerprint=a=>JSON.stringify({edit:a.edit,base:a.generatedBase||null});
export const isDirty=()=>!!asset()&&!equal(studio.draft,asset().edit);
export const referencePreview=()=>studio.previewAssetId===studio.aid&&studio.previewSignature===JSON.stringify(normalizeEdit(studio.draft))&&studio.preview?studio.preview:studio.thumbs[studio.aid]||'';
let toastTimer,events;
function draftKey(){return `drafts:${studio.dataRoot.toLowerCase().replaceAll('\\','/')}`;}
export function cacheDrafts(){if(!studio.dataRoot)return;try{const items={};for(const [id,edit] of Object.entries(studio.drafts)){const a=studio.projects.flatMap(p=>p.assets).find(a=>a.id===id);if(a&&!equal(edit,a.edit))items[id]={edit,base:studio.draftBases[id]||fingerprint(a)};}persist(draftKey(),{version:1,items,pid:studio.pid,aid:studio.aid});studio.draftStatus="草稿已保存在本机";}catch{studio.draftStatus="草稿未保存";studio.error='Local draft storage is unavailable. Save a version before closing.';}}
function loadDrafts(){const saved=stored(draftKey(),null);if(saved?.version!==1)return null;for(const [id,item] of Object.entries(saved.items||{})){const a=studio.projects.flatMap(p=>p.assets).find(a=>a.id===id);if(a&&item.base===fingerprint(a)&&item.edit){studio.drafts[id]=item.edit;studio.draftBases[id]=item.base;}}return saved;}
export function clearDrafts(ids){for(const id of ids){delete studio.drafts[id];delete studio.draftBases[id];}cacheDrafts();}
export function notify(text,action=null,details={}){appEvents.emit({type:'ui.receipt',payload:{text,action,open:details.open??!!action,...details}});}
export async function run(label,work){if(studio.busy)return;studio.busy=label;studio.error='';try{return await work();}catch(e){studio.error=e.message||String(e);}finally{studio.busy='';studio.progress=null;}}
const thumbnailFingerprint=a=>JSON.stringify([a.sha256,a.generatedBase,a.width,a.height,normalizeEdit(a.edit||initialEdit)]);
function invalidateThumbnails(projects,rootChanged=false){
 const next=new Map(projects.flatMap(p=>p.assets).map(a=>[a.id,a]));
 const previous=new Map(studio.projects.flatMap(p=>p.assets).map(a=>[a.id,a]));
 for(const id of Object.keys(studio.thumbs)){
  if(rootChanged||!next.has(id)||!previous.has(id)||thumbnailFingerprint(next.get(id))!==thumbnailFingerprint(previous.get(id))){
   URL.revokeObjectURL(studio.thumbs[id]);delete studio.thumbs[id];
  }
 }
}
export function updateProject(project){const i=studio.projects.findIndex(p=>p.id===project.id);const projects=[...studio.projects];if(i<0)projects.push(project);else projects[i]=project;invalidateThumbnails(projects);studio.projects=projects;}
const thumbnailRequests=new Map();
export function thumbnails(project=current(),force=false,onlyIds=null){
 // Cache reads must not become dependencies of the caller's project-library effect.
 return untrack(async()=>{
  if(!project)return;
  const root=studio.dataRoot;
  const items=project.assets.filter(a=>!onlyIds||onlyIds.includes(a.id)).sort((a,b)=>a.id===studio.aid?-1:b.id===studio.aid?1:0);
  for(const a of items){
   if(studio.dataRoot!==root)return;
   if(studio.thumbs[a.id]&&!force)continue;
   const revision=thumbnailFingerprint(a),key=JSON.stringify([root,project.id,a.id,revision]);
   try{
    let request=thumbnailRequests.get(key);
    // A save can require a fresh thumbnail after an older request finishes.
    if(request&&force){await request;request=thumbnailRequests.get(key);}
    if(!request){
     request=(async()=>{
      const source=thumbnailInput(root,project.id,a),sourceKey=thumbnailSourceKey(source);
      const {blob}=await loadThumbnailSource(source,()=>api('thumbnail',{projectId:project.id,assetId:a.id}),{force,isCurrent:()=>{const latest=studio.projects.find(p=>p.id===project.id)?.assets.find(item=>item.id===a.id);return studio.dataRoot===root&&!!latest&&thumbnailSourceKey(thumbnailInput(root,project.id,latest))===sourceKey;}});
      const url=URL.createObjectURL(blob);
      const currentAsset=studio.projects.find(p=>p.id===project.id)?.assets.find(item=>item.id===a.id);
      if(studio.dataRoot!==root||!currentAsset||thumbnailFingerprint(currentAsset)!==revision){URL.revokeObjectURL(url);return;}
      const old=studio.thumbs[a.id];studio.thumbs[a.id]=url;
      if(old)URL.revokeObjectURL(old);
     })().finally(()=>thumbnailRequests.delete(key));
     thumbnailRequests.set(key,request);
    }
    await request;
   }catch{}
  }
 });
}
export function remember(){if(!studio.aid||studio.busy)return;const h=studio.history[studio.aid]||[];h.push(clone(studio.draft));studio.history[studio.aid]=h.slice(-60);studio.redo[studio.aid]=[];}
function stash(){const a=asset();if(!a)return;if(equal(studio.draft,a.edit)){delete studio.drafts[a.id];delete studio.draftBases[a.id];}else{studio.drafts[a.id]=clone(studio.draft);studio.draftBases[a.id]??=fingerprint(a);}cacheDrafts();}
export function change(patch,checkpoint=true){if(!studio.aid||studio.busy)return;if(checkpoint)remember();studio.draft={...studio.draft,...patch};stash();}
export function undo(){if(studio.busy)return;studio.history[studio.aid]??=[];const h=studio.history[studio.aid];if(!h.length&&isDirty())h.push(clone(asset().edit));if(!h.length)return;studio.redo[studio.aid]??=[];studio.redo[studio.aid].push(clone(studio.draft));studio.draft=h.pop();stash();}
export function redo(){if(studio.busy)return;const h=studio.redo[studio.aid];if(!h?.length)return;studio.history[studio.aid]??=[];studio.history[studio.aid].push(clone(studio.draft));studio.draft=h.pop();stash();}
export function select(id,force=false){if(studio.busy&&!force)return;studio.aid=id;studio.draft=clone(editFor(asset()||{id,edit:initialEdit}));studio.pan={x:0,y:0};studio.zoom=1;studio.canvasFit=true;studio.compare=false;studio.preview='';studio.previewAssetId='';cacheDrafts();}
export function selectProject(id){studio.pid=id;studio.selected=[];studio.batchScope='auto';studio.trash=false;studio.view='canvas';select(current()?.assets.find(a=>!a.trashed)?.id||'',true);thumbnails();}
export function toggle(id){studio.selected=studio.selected.includes(id)?studio.selected.filter(x=>x!==id):[...studio.selected,id];studio.batchScope='auto';}
export function clearSelection(){studio.selected=[];studio.batchScope='auto';}
export async function refresh(snapshot=null,{loadThumbnails=true}={}){const b=snapshot||await api('bootstrap');const newRoot=studio.dataRoot!==b.dataRoot;invalidateThumbnails(b.projects,newRoot);studio.projectWarnings=b.warnings||[];studio.projects=b.projects;studio.dataRoot=b.dataRoot;let last=null;if(newRoot){studio.drafts={};studio.draftBases={};studio.history={};studio.redo={};last=loadDrafts();}if(!current()){const p=b.projects.find(p=>p.id===last?.pid)||b.projects[0];studio.pid=p?.id||'';const a=p?.assets.find(a=>a.id===last?.aid&&!a.trashed)||p?.assets.find(a=>!a.trashed);select(a?.id||'',true);}else if(!asset()||asset().trashed)select(liveAssets()[0]?.id||'',true);studio.selected=studio.selected.filter(id=>current()?.assets.some(a=>a.id===id&&!a.trashed));if(loadThumbnails){await thumbnails(current(),false,asset()?[studio.aid]:[]);void thumbnails();}}
export async function save(ids=[studio.aid],pid=studio.pid){let count=0;const savedIds=[],index=new Map((studio.projects.find(p=>p.id===pid)?.assets||[]).filter(a=>!a.trashed).map(a=>[a.id,a]));for(const id of ids){const a=index.get(id);if(!a)continue;const edit=studio.drafts[id]||(id===studio.aid?studio.draft:a.edit);if(equal(edit,a.edit))continue;const p=await api('edit_commit',{projectId:pid,assetId:id,edit:clone(edit)});updateProject(p);delete studio.drafts[id];delete studio.draftBases[id];if(id===studio.aid)studio.draft=clone(p.assets.find(a=>a.id===id).edit);count++;savedIds.push(id);}cacheDrafts();if(count)await thumbnails(current(),true,savedIds);return count;}
function importResult(result,oldIds){updateProject(result.project);const added=result.project.assets.filter(a=>!a.trashed&&!oldIds.has(a.id));studio.selected=[];studio.batchScope='auto';studio.trash=false;studio.mode='edit';studio.view='canvas';select(added[0]?.id||studio.aid||result.project.assets.find(a=>!a.trashed)?.id||'',true);notify(`${result.imported} imported${result.skipped.length?` · ${result.skipped.length} duplicates skipped`:''}`);if(result.errors.length)studio.error=result.errors.join('\n');}
async function ensureProject(){if(current())return;const p=await api('project_create',{name:'Untitled project'});updateProject(p);selectProject(p.id);}
export async function importImages(folder=false){await run("正在导入",async()=>{await ensureProject();const previous=new Set(current().assets.map(a=>a.id));const result=await api('pick_import',{projectId:studio.pid,folder});if(result){importResult(result,previous);await thumbnails();}});}
export async function dropImages(files){if(!files.length){studio.error='请选择 PNG、JPEG 或 WebP 图片。';return;}await run("正在导入",async()=>{await ensureProject();const previous=new Set(current().assets.map(a=>a.id));const result=await upload(studio.pid,files);importResult(result,previous);await thumbnails();});}
export async function trashSelected(restore=false){const pid=studio.pid;const ids=(studio.selected.length?[...studio.selected]:[studio.aid]).filter(id=>current()?.assets.some(a=>a.id===id&&a.trashed===restore));if(!ids.length)return;await run(restore?'Restoring':'Moving to trash',async()=>{const p=await api('asset_trash',{projectId:pid,ids,restore});updateProject(p);clearSelection();select(assets()[0]?.id||'',true);notify(restore?`${ids.length} restored`:`${ids.length} moved to trash`,restore?null:{label:"撤销",run:async()=>{updateProject(await api('asset_trash',{projectId:pid,ids,restore:true}));}});});}
export async function restoreVersion(versionId=null){const {selectVersion}=await import('./services/editor.js');return selectVersion(versionId);}
export function goSettings(from){studio.settingsReturn=from;studio.mode='settings';}
export async function jobs(){const r=await api('hybrid_jobs');studio.jobs=r.jobs;studio.notices=r.notices;}
export async function initialize({lazyThumbnails=false}={}){if(studio.startupState==='loading')return;studio.startupState='loading';studio.startupError='';studio.ready=false;studio.error='';try{await refresh(null,{loadThumbnails:!lazyThumbnails});if(lazyThumbnails)await thumbnails(current(),false,asset()?[studio.aid]:[]);const m=await api('model_catalog');studio.catalog=m.catalog;if(m.warning)studio.error=m.warning;await jobs();events?.close();if(isMock){events={close(){}};appEvents.emit({type:'connection.open',payload:{mock:true}});studio.ready=true;studio.startupState='ready';return;}events=new EventSource('/api/events');events.onopen=()=>appEvents.emit({type:'connection.open',payload:null});events.onerror=()=>appEvents.emit({type:'connection.error',payload:null});events.onmessage=e=>{const {event,payload}=JSON.parse(e.data);appEvents.emit({type:event,payload});if(event==='team-cleanup'){refresh();jobs().catch(()=>{});notify(payload.reason==='saved'?(payload.summary?.deferredAssets?'结果已保存到本机。仍有相关任务或待保存结果，原图暂时保留。':'图片已保存到本机，主机原图和结果已清理。'):'24 小时保留期已结束，临时图片已清理。');}if(event==='team-retention-error')studio.error=payload.message;if(event==='team-queue'&&studio.busy)studio.busy=payload.state==='queued'?'正在排队，等待主机处理':'正在处理';if(event==='job-progress')studio.progress=payload;if(event.startsWith('hybrid-'))jobs().catch(()=>{});if(event==='hybrid-completed'&&!isManagedTask(payload.id)){refresh();if(document.hidden&&'Notification' in window&&Notification.permission==='granted')new Notification('GVPAK STUDIO · Task completed',{body:payload.recipe?.summary||'Your images are ready.'});}};studio.ready=true;studio.startupState='ready';}catch(e){studio.error=e.message;studio.startupError=e.message;studio.startupState='error';}}
export function dispose(){cacheDrafts();disposePreviews();events?.close();clearTimeout(toastTimer);Object.values(studio.thumbs).forEach(URL.revokeObjectURL);}
