// One encoder request at a time; only the latest settings may publish a result.
export function createExportPreview({request,onChange,makeURL=blob=>URL.createObjectURL(blob),dropURL=url=>URL.revokeObjectURL(url)}){
 let revision=0,pending=null,running=false,disposed=false,timer,current=null;
 const blob=(data,type)=>new Blob([Uint8Array.from(atob(data),c=>c.charCodeAt(0))],{type});
 function release(){if(current){dropURL(current.url);dropURL(current.beforeUrl);current=null;}}
 async function pump(){
  if(running||disposed)return;
  running=true;
  try{
   while(pending&&!disposed){
    const job=pending;pending=null;
    try{
     const {data,before,...details}=await request(job.args);
     if(disposed||job.revision!==revision)continue;
     const url=makeURL(blob(data,details.mime));
     let beforeUrl;
     try{beforeUrl=makeURL(blob(before,'image/png'));}catch(error){dropURL(url);throw error;}
     current={...details,url,beforeUrl};
     onChange({loading:false,result:current,error:''});
    }catch(error){if(!disposed&&job.revision===revision)onChange({loading:false,result:null,error:error.message||String(error)});}
   }
  }finally{running=false;}
 }
 return {
  set(args,delay=250){
   if(disposed)return;
   revision++;clearTimeout(timer);pending=null;release();
   onChange({loading:!!args,result:null,error:''});
   if(args){const job={args,revision};timer=setTimeout(()=>{pending=job;void pump();},delay);}
  },
  dispose(){disposed=true;revision++;clearTimeout(timer);pending=null;release();}
 };
}
