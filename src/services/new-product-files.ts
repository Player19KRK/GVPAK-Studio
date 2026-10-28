import {withStore} from './idb.ts';
import {hashBlob} from './hash.ts';
import {fileNameOnly} from '../lib/target.ts';
export type ProductDirectory = FileSystemDirectoryHandle & {queryPermission(options:{mode:'readwrite'}):Promise<PermissionState>};
const imageFile=(file:File)=>/\.(png|jpe?g|webp)$/i.test(file.name);
export async function chooseProductDirectory(limit=200) {
  const picker=(window as any).showDirectoryPicker;if(!picker)throw Error('请使用Chrome或Edge，或直接拖入原图。');
  const directory:ProductDirectory=await picker.call(window,{mode:'readwrite'}),files:File[]=[],relativeFolders=new Map<File,string>();
  async function read(handle:any,prefix=''){for await(const entry of handle.values()){if(entry.kind==='directory'){if(!entry.name.startsWith('.')&&entry.name!=='新品套图')await read(entry,prefix?prefix+'/'+entry.name:entry.name);}else{const file=await entry.getFile();if(imageFile(file)){files.push(file);relativeFolders.set(file,prefix);}}if(files.length>limit)throw Error(`文件夹图片超过${limit}张，请分批导入。`);}}
  await read(directory);return {directory,files,relativeFolders,label:directory.name};
}
export async function droppedProductFiles(transfer:DataTransfer,limit=200) {
  const entries=[...transfer.items].map(item=>(item as any).webkitGetAsEntry?.()).filter(Boolean),files:File[]=[];
  async function read(entry:any){if(entry.isFile){const file:File=await new Promise((resolve,reject)=>entry.file(resolve,reject));if(imageFile(file))files.push(file);}else if(!entry.name.startsWith('.')&&entry.name!=='新品套图'){const reader=entry.createReader();while(true){const children:any[]=await new Promise((resolve,reject)=>reader.readEntries(resolve,reject));if(!children.length)break;for(const child of children)await read(child);}}if(files.length>limit)throw Error(`拖入图片超过${limit}张，请分批导入。`);}
  if(entries.length)for(const entry of entries)await read(entry);else files.push(...[...transfer.files].filter(imageFile));
  return files;
}
export async function uniqueProductFiles(files:File[],limit=40) {
  const rows:{file:File;hash:string;aliases:string[]}[]=[],seen=new Map<string,typeof rows[number]>();
  for(const file of files.filter(imageFile)){
    if(file.size>30*1024*1024)throw Error(`${file.name}超过30MB，请缩小后重试。`);
    const hash=await hashBlob(file),existing=seen.get(hash);if(existing){existing.aliases.push(fileNameOnly(file.name));continue;}
    const row={file,hash,aliases:[]};rows.push(row);seen.set(hash,row);
  }
  if(!rows.length)throw Error('没有找到PNG、JPEG或WebP原图。');if(rows.length>limit)throw Error(`单次最多${limit}张不同原图，请分批选择。`);return rows;
}
const scope=()=>`new-product:${(window as any).__gvpakTeamUser?.id||'host'}`;
export async function rememberProductOutput(directory:ProductDirectory){await withStore('gvpak-new-product-files','handles','readwrite',store=>store.put(directory,scope()));}
export async function restoreProductOutput(){const handle=await withStore<ProductDirectory>('gvpak-new-product-files','handles','readonly',store=>store.get(scope())).catch(()=>null);return handle&&await handle.queryPermission({mode:'readwrite'}).catch(()=> 'denied')==='granted'?handle:null;}
export async function writeProductResult(directory:ProductDirectory,jobId:string,result:{name:string;sha256:string},blob:Blob){
  if(await directory.queryPermission({mode:'readwrite'})!=='granted')throw Error('保存目录授权已失效，请重新选择文件夹。');
  const parent=await directory.getDirectoryHandle('新品套图',{create:true}),folder=await parent.getDirectoryHandle(jobId.slice(0,8),{create:true});
  const original=fileNameOnly(result.name),dot=original.lastIndexOf('.'),stem=dot>0?original.slice(0,dot):original,ext=dot>0?original.slice(dot):'';let name=original;
  for(let index=2;;index++){
    try{const old=await folder.getFileHandle(name);if(await hashBlob(await old.getFile())===result.sha256)return name;name=`${stem} (${index})${ext}`;}
    catch(error){if((error as DOMException).name==='NotFoundError')break;throw error;}
  }
  const handle=await folder.getFileHandle(name,{create:true}),writer=await handle.createWritable();
  try{await writer.write(blob);await writer.close();}catch(error){await writer.abort().catch(()=>{});throw error;}
  if(await hashBlob(await handle.getFile())!==result.sha256)throw Error('本机保存校验未通过，工作区结果仍保留。');return name;
}
const encoder=new TextEncoder();
function crc32(bytes:Uint8Array){let value=0xffffffff;for(const byte of bytes){value^=byte;for(let bit=0;bit<8;bit++)value=(value>>>1)^(0xedb88320&-(value&1));}return(value^0xffffffff)>>>0;}
function header(size:number){const bytes=new Uint8Array(size);return{bytes,view:new DataView(bytes.buffer)};}
export async function productZip(files:{name:string;blob:Blob}[]) {
  const locals:BlobPart[]=[],central:BlobPart[]=[];let offset=0,centralSize=0;
  for(const file of files){const name=encoder.encode(fileNameOnly(file.name)),bytes=new Uint8Array(await file.blob.arrayBuffer()),crc=crc32(bytes),local=header(30),entry=header(46);
    local.view.setUint32(0,0x04034b50,true);local.view.setUint16(4,20,true);local.view.setUint16(6,0x0800,true);local.view.setUint16(12,0x21,true);local.view.setUint32(14,crc,true);local.view.setUint32(18,bytes.length,true);local.view.setUint32(22,bytes.length,true);local.view.setUint16(26,name.length,true);
    entry.view.setUint32(0,0x02014b50,true);entry.view.setUint16(4,20,true);entry.view.setUint16(6,20,true);entry.view.setUint16(8,0x0800,true);entry.view.setUint16(14,0x21,true);entry.view.setUint32(16,crc,true);entry.view.setUint32(20,bytes.length,true);entry.view.setUint32(24,bytes.length,true);entry.view.setUint16(28,name.length,true);entry.view.setUint32(42,offset,true);
    locals.push(local.bytes,name,bytes);central.push(entry.bytes,name);offset+=30+name.length+bytes.length;centralSize+=46+name.length;
  }
  const end=header(22);end.view.setUint32(0,0x06054b50,true);end.view.setUint16(8,files.length,true);end.view.setUint16(10,files.length,true);end.view.setUint32(12,centralSize,true);end.view.setUint32(16,offset,true);return new Blob([...locals,...central,end.bytes],{type:'application/zip'});
}
export function downloadProductBlob(blob:Blob,name:string){const url=URL.createObjectURL(blob),link=document.createElement('a');link.href=url;link.download=fileNameOnly(name);link.click();setTimeout(()=>URL.revokeObjectURL(url),30000);}
