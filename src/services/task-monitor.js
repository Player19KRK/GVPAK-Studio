import {markManagedTask} from '../lib/task-ownership.js';
import {queueTaskOutputs} from '../stores/output.svelte';
import {getApi} from './api.ts';
import {appEvents,normalizeEvent} from './realtime.ts';
import {tasks,setTasks,applyTaskEvent,taskSnapshotToken} from '../stores/tasks.svelte';
import {session,setSession} from '../stores/session.svelte';
import {getNewProductApi} from './new-product.ts';
import {newProductTask} from '../lib/task-view.ts';
export async function refreshTaskList(api=getApi(),{isCurrent=()=>true}={}) {
 const token=taskSnapshotToken(),current=()=>isCurrent()&&token.epoch===taskSnapshotToken().epoch;
 if(current()){if(tasks.status==='idle'||tasks.status==='error')tasks.status='loading';tasks.error='';}
 try{const values=await api.listTasks();if(!current()||setTasks(values,{since:token})===false)return null;return values;}
 catch(error){if(current()){tasks.status='error';tasks.error=error.message;}throw error;}
}
export function monitorTasks({api=getApi(),events=appEvents,readSession=false,includeNewProduct=false}={}) {
 let alive=true,refreshing=false,pending=false;const notified=new Set();
 const refresh=async()=>{if(refreshing){pending=true;return;}refreshing=true;try{const values=await refreshTaskList(api,{isCurrent:()=>alive});if(alive&&values)for(const task of values){markManagedTask(task);void queueTaskOutputs(task);}}catch(error){}finally{refreshing=false;if(pending&&alive){pending=false;void refresh();}}};
 if(readSession)void (api.readStatus?api.readStatus():api.readSession()).then(value=>{if(alive)setSession(value);}).catch(error=>{if(alive)session.error=error.message;});
 void refresh();
 if(includeNewProduct)void getNewProductApi().jobs().then(value=>{if(alive)for(const job of value.jobs)applyTaskEvent({type:'task.progress',payload:newProductTask(job)});}).catch(error=>{if(alive)tasks.error=error.message;});
 const off=events.subscribe(raw=>{const event=normalizeEvent(raw);if(!event)return;
  if(event.type.startsWith('task.')){if(event.type==='task.done'&&event.payload?.pipeline&&!event.payload.undone&&['done','partial','failed'].includes(event.payload.state)&&!notified.has(event.payload.id)){notified.add(event.payload.id);if(document.hidden&&'Notification'in window&&Notification.permission==='granted'){try{new Notification('GVPAK STUDIO',{body:event.payload.summary||'处理任务状态已更新'});}catch{}}}markManagedTask(event.payload);applyTaskEvent(event);void queueTaskOutputs(tasks.byId[event.payload.id]);if(readSession&&event.type==='task.done')void (api.readStatus?api.readStatus():api.readSession()).then(value=>{if(alive)setSession(value);}).catch(()=>{});}
  if(event.type==='new-product.progress'){applyTaskEvent({type:'task.progress',payload:{id:event.payload.jobId,type:'new-product',state:event.payload.state,done:event.payload.done,total:event.payload.total}});void refresh();}
  if(event.type==='session.updated'){if(event.payload?.quota!==undefined)session.quota=event.payload.quota;}
  if(event.type.startsWith('hybrid-'))void refresh();
  if(event.type==='connection.open'){tasks.connection='connected';void refresh();if(readSession)void (api.readStatus?api.readStatus():api.readSession()).then(value=>{if(alive)setSession(value);}).catch(error=>{if(alive)session.error=error.message;});}
  if(event.type==='connection.error')tasks.connection='reconnecting';
 });return()=>{alive=false;off();};
}
export async function stopTask(task) {
 if(task.type==='new-product')await getNewProductApi().cancel(task.id);else await getApi().stopTask(task);
 // Cancellation acknowledgement does not claim in-flight provider work stopped.
 if(task.type==='new-product')Object.assign(task,{state:'cancelling'});else await refreshTaskList();
}
