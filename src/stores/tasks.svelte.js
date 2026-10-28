import {shouldApplyTaskPatch,runningStates} from '../lib/task-view.ts';
export const tasks = $state({ byId: {}, activeId: null, status: 'idle', error: '', connection: 'connecting' });
let serial=0,epoch=0;const versions=new Map();export const taskSnapshotToken=()=>({serial,epoch});
export function setTasks(values,{since=null,reset=false}={}){if(!reset&&since&&since.epoch!==epoch)return false;if(reset){epoch++;versions.clear();}const next={};for(const value of values){const current=tasks.byId[value.id];next[value.id]=!reset&&current?(shouldApplyTaskPatch(current,value)?{...current,...value}:current):value;}if(!reset&&since!==null){for(const[id,version]of versions){if(version>since.serial&&!next[id]&&tasks.byId[id])next[id]=tasks.byId[id];}}for(const id of versions.keys())if(!next[id])versions.delete(id);tasks.byId=next;if(!runningStates.has(tasks.byId[tasks.activeId]?.state)&&tasks.byId[tasks.activeId]?.state!=='queued')tasks.activeId=Object.values(next).find(task=>runningStates.has(task.state))?.id||Object.values(next).find(task=>task.state==='queued')?.id||null;tasks.status='ready';tasks.error='';return true;}

export function applyTaskEvent(event) {
  const payload = event.payload, id = payload?.taskId || payload?.id;
  if (!id) return;
  const patch = { ...payload, id };
  if (event.type === 'task.progress' && !patch.state) patch.state = 'running';
  if (event.type === 'task.queue' && !patch.state) patch.state = 'queued';
  if (event.type === 'task.failed') patch.state = 'failed';
  if (event.type === 'task.done' && !patch.state) patch.state = 'done';
  if (!shouldApplyTaskPatch(tasks.byId[id],patch)) return false;
  if (tasks.byId[id]) Object.assign(tasks.byId[id], patch);
  else tasks.byId[id] = patch;
  versions.set(id,++serial);
  if (runningStates.has(patch.state) && !tasks.activeId) tasks.activeId = id;
  if (['done', 'partial', 'failed', 'cancelled'].includes(patch.state) && tasks.activeId === id) tasks.activeId = null;
  return true;
}
