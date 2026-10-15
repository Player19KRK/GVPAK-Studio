export async function api(name,args={}){const res=await fetch(`/api/invoke/${name}`,{method:'POST',headers:{'Content-Type':'application/json','X-Edge-IPC':'1'},body:JSON.stringify(args)});if(!res.ok){const e=await res.json().catch(()=>({error:res.statusText}));throw Error(e.error||'Request failed');}return res.headers.get('content-type')?.includes('octet-stream')?res.blob():res.json();}
export async function imageUrl(name,args){return URL.createObjectURL(await api(name,args));}
export async function upload(projectId,files){const items=[];for(const file of files){const data=await new Promise((resolve,reject)=>{const r=new FileReader();r.onload=()=>resolve(r.result.split(',')[1]);r.onerror=reject;r.readAsDataURL(file);});items.push({name:file.name,data});}const res=await fetch('/api/upload',{method:'POST',headers:{'Content-Type':'application/json','X-Edge-IPC':'1'},body:JSON.stringify({projectId,files:items})});const result=await res.json();if(!res.ok)throw Error(result.error);return result;}
export const initialEdit={adjustmentOpacity:1,adjustmentEnabled:true,exposure:0,contrast:1,saturation:1,temperature:0,rotation:0,flipH:false,flipV:false,crop:null};
export const clone=x=>JSON.parse(JSON.stringify(x));
export const bytes=n=>n>=1e6?`${(n/1e6).toFixed(1)} MB`:`${Math.round(n/1000)} KB`;
export const rgb=hex=>[1,3,5].map(i=>parseInt(hex.slice(i,i+2),16));
export const hex=arr=>'#'+arr.map(n=>Math.round(n).toString(16).padStart(2,'0')).join('');
export function stored(key,fallback){try{return JSON.parse(localStorage.getItem('edge-'+key))??fallback;}catch{return fallback;}}
export const persist=(key,v)=>localStorage.setItem('edge-'+key,JSON.stringify(v));
