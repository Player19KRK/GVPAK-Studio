export const runningStates = new Set(['running','waiting','submitted','submitting','cancelling']);
export function shouldApplyTaskPatch(current:any,patch:any){if(!current)return true;const oldTime=Number(current.updatedAt),newTime=Number(patch.updatedAt),timed=Number.isFinite(oldTime)&&Number.isFinite(newTime),newer=timed&&newTime>oldTime;if(timed&&newTime<oldTime)return false;const terminal=(state:string)=>['done','partial','failed','cancelled'].includes(state);if(!newer&&terminal(current.state)&&patch.state&&!terminal(patch.state))return false;if(!newer&&patch.state==='queued'&&current.state&&current.state!=='queued')return false;if(!newer&&Number.isFinite(current.done)&&Number.isFinite(patch.done)&&patch.done<current.done)return false;return true;}
export function taskState(value: string) { return ({completed:'done',attention:'failed'} as Record<string,string>)[value] || value; }
export function taskGroups(values: any[]) {
  return {running:values.filter(task=>runningStates.has(task.state)),queued:values.filter(task=>task.state==='queued').sort((a,b)=>(a.queuePosition??Infinity)-(b.queuePosition??Infinity)),recent:values.filter(task=>!runningStates.has(task.state)&&task.state!=='queued').sort((a,b)=>(b.updatedAt??b.startedAt??0)-(a.updatedAt??a.startedAt??0)).slice(0,20)};
}
export function quotaView(quota: {used:number;limit:number}|null) {
  if (!quota || !Number.isFinite(quota.used)||!Number.isFinite(quota.limit)||quota.used<0||quota.limit<=0) return null;
  const percent=Math.min(100,Math.max(0,quota.used/quota.limit*100));return {...quota,percent,warning:percent>95};
}
export function newProductTask(job:any) {
 return {id:job.id,type:'new-product',projectId:job.projectId,state:taskState(job.status||job.state),done:job.done??job.tasks?.filter((item:any)=>item.state==='done').length??0,total:job.total??job.tasks?.length??0,targetCount:job.total??job.tasks?.length??0,queuePosition:job.queuePosition??null,startedAt:job.createdAt?job.createdAt*1000:null,updatedAt:job.updatedAt?job.updatedAt*1000:Date.now(),resultAssetIds:job.tasks?.filter((item:any)=>item.result).map((item:any)=>item.result.assetId)||[],error:job.error||''};
}
