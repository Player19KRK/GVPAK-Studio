let directory=null,saving=false,writeQueue=Promise.resolve(),initialized=false;
const receipts=new Map();
const scope=()=>window.__gvpakTeamUser?.id||'none';
const changed=()=>window.dispatchEvent(new Event('team-download-state'));
const report=message=>window.dispatchEvent(new CustomEvent('team-downloads',{detail:{message}}));
export const directoryName=()=>directory?.name||'';
export const pendingDownloads=()=>[...receipts].filter(([,r])=>r.kind==='browser').map(([url])=>url);
function storage(key,value,write=false){if(typeof indexedDB==='undefined')return Promise.resolve(null);return new Promise((resolve,reject)=>{const open=indexedDB.open('gvpak-local-files',1);open.onupgradeneeded=()=>open.result.createObjectStore('preferences');open.onerror=()=>reject(open.error);open.onsuccess=()=>{const db=open.result;const tx=db.transaction('preferences',write?'readwrite':'readonly');const req=write?tx.objectStore('preferences').put(value,scope()+'/'+key):tx.objectStore('preferences').get(scope()+'/'+key);let result;req.onsuccess=()=>result=req.result;tx.oncomplete=()=>{db.close();resolve(result)};tx.onerror=()=>{db.close();reject(tx.error)};};});}
async function persistReceipts(){await storage('receipts',[...receipts],true).catch(()=>{});changed();}
export async function restoreDirectory(){if(initialized)return;initialized=true;const prior=await storage('receipts').catch(()=>null);for(const [url,r]of prior||[])receipts.set(url,r);const handle=await storage('directory').catch(()=>null);if(handle&&await handle.queryPermission({mode:'readwrite'}).catch(()=> 'denied')==='granted')directory=handle;window.dispatchEvent(new Event('team-directory'));changed();}
export async function chooseDirectory(){
 if(!window.showDirectoryPicker||!window.isSecureContext){report('此浏览器不能选择保存文件夹。请下载图片后点击“已保存，清理主机”。未确认的图片保留 24 小时。');return '浏览器下载目录';}
 try{directory=await window.showDirectoryPicker({mode:'readwrite',id:'gvpak-exports'});await storage('directory',directory,true).catch(()=>{});window.dispatchEvent(new Event('team-directory'));void saveNewFiles().catch(e=>report(e.message));return directory.name;}catch(e){if(e.name==='AbortError')return null;throw Error('无法选择文件夹。请确认网站证书可信，并允许写入所选文件夹。');}
}
const digest=async blob=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',await blob.arrayBuffer()))].map(b=>b.toString(16).padStart(2,'0')).join('');
async function acknowledge(file,r){const id=file.url.split('/').at(-1);const response=await fetch('/api/files/saved',{method:'POST',headers:{'Content-Type':'application/json','X-GVPAK-IPC':'1'},body:JSON.stringify({id,sha256:r.sha256,bytes:r.bytes})});if(!response.ok){let code='';try{code=(await response.json()).error;}catch{}if(code==='SAVED_FILE_MISMATCH'){receipts.delete(file.url);await persistReceipts();throw Error('主机结果已更新，旧文件已保留在本机。请重新下载新结果。');}if(code!=='FILE_NOT_FOUND')throw Error('图片已保存到本机，但主机副本尚未清理。稍后会重试；请勿删除本机图片。');}receipts.delete(file.url);await persistReceipts();window.dispatchEvent(new Event('team-save-complete'));}
async function verify(handle,r){const file=await handle.getFile();return file.size===r.bytes&&await digest(file)===r.sha256;}
async function saveOne(file){
 const prior=receipts.get(file.url);
 if(directory&&prior?.kind==='directory'){try{const h=await directory.getFileHandle(prior.localName);if(await verify(h,prior)){await acknowledge(file,prior);return {saved:true};}}catch(e){if(e.name!=='NotFoundError')throw e;}}
 const response=await fetch(file.url);if(!response.ok)throw Error('下载失败，请刷新列表后重试。');const blob=await response.blob();const proof={sha256:await digest(blob),bytes:blob.size};
 if(directory){let name=file.name;try{await directory.getFileHandle(name);const dot=name.lastIndexOf('.');name=(dot<0?name:name.slice(0,dot))+'-'+crypto.randomUUID().slice(0,8)+(dot<0?'':name.slice(dot));}catch(e){if(e.name!=='NotFoundError')throw e;}
  const handle=await directory.getFileHandle(name,{create:true});let out;
  try{out=await handle.createWritable();await out.write(blob);await out.close();}catch(e){try{await out?.abort();}catch{}throw Error('保存失败。主机副本仍保留，请检查文件夹权限和可用空间。');}
  if(!await verify(handle,proof))throw Error('本机文件校验失败，主机副本仍保留。请重新保存。');
  const receipt={...proof,kind:'directory',localName:name};receipts.set(file.url,receipt);await persistReceipts();await acknowledge(file,receipt);return {saved:true};
 }
 const url=URL.createObjectURL(blob),link=document.createElement('a');link.href=url;link.download=file.name;link.click();setTimeout(()=>URL.revokeObjectURL(url),30000);
 receipts.set(file.url,{...proof,kind:'browser'});await persistReceipts();return {saved:false,downloadStarted:true};
}
export function saveFile(file){const work=writeQueue.then(()=>saveOne(file));writeQueue=work.catch(()=>{});return work;}
export function descriptor(url,fallback='图片.webp'){try{const id=url.split('/').at(-1).replaceAll('-','+').replaceAll('_','/');const name=new TextDecoder().decode(Uint8Array.from(atob(id),c=>c.charCodeAt(0))).split('/').at(-1);return {url,name:name||fallback};}catch{return {url,name:fallback};}}
export async function confirmSaved(file){const receipt=receipts.get(file.url);if(!receipt)throw Error('请先使用下载按钮保存图片，再确认清理。');if(!window.confirm('请确认图片已完整保存到你的电脑。继续后，主机上的对应原图和结果将被清理，再次编辑需要重新导入。'))return false;await acknowledge(file,receipt);return true;}
export async function saveNewFiles(){if(!directory||saving)return;saving=true;try{const response=await fetch('/api/files',{cache:'no-store'});if(!response.ok)return;for(const file of (await response.json()).files)await saveFile(file);}finally{saving=false;}}

// Request the handle before yielding so the browser retains the click activation.
export async function saveFileAs(file){
 if(!window.showSaveFilePicker||!window.isSecureContext)return saveFile(file);
 let handle;try{handle=await window.showSaveFilePicker({suggestedName:file.name,startIn:'downloads',id:'gvpak-save-file'});}catch(e){if(e.name==='AbortError')return null;throw Error('无法另存为。请允许浏览器下载，或选择其他保存位置。');}
 const work=writeQueue.then(async()=>{
  const response=await fetch(file.url);if(!response.ok)throw Error('下载失败，主机副本仍保留。');
  const blob=await response.blob(),proof={sha256:await digest(blob),bytes:blob.size};let out;
  try{out=await handle.createWritable();await out.write(blob);await out.close();}catch(e){try{await out?.abort();}catch{}throw Error('保存失败，主机副本仍保留。请检查文件权限和可用空间。');}
  if(!await verify(handle,proof))throw Error('本机文件校验失败，主机副本仍保留。');
  await acknowledge(file,proof);return {saved:true};
 });writeQueue=work.catch(()=>{});return work;
}

export async function useBrowserDownloads(){directory=null;await storage('directory',null,true).catch(()=>{});window.dispatchEvent(new Event('team-directory'));return '浏览器另存为（下载 / 桌面）';}
