import {sendChat} from '../stores/chat.svelte';import {shell,saveShell} from '../app/shell.svelte';
import {untrack} from 'svelte';
import {studio,dispose,refresh} from '../studio.svelte';
import {runningStates} from '../lib/task-view.ts';
import {getApi} from './api.ts';
import {appEvents} from './realtime.ts';
import {session,setSession} from '../stores/session.svelte';
import {project} from '../stores/project.svelte';
import {assets} from '../stores/assets.svelte';
import {tasks,setTasks,applyTaskEvent,taskSnapshotToken} from '../stores/tasks.svelte';
import {startSelectionContext,selection} from '../stores/selection.svelte';
import {disposeHashWorker} from './hash.ts';
import {disposeThumbnailWorker} from './thumbnails.ts';
let disconnect = () => {}, stopContext = () => {}, stopProject = () => {}, epoch = 0;
let activeApi = null;
export async function initializeWorkspace(api = getApi()) {
  const token = ++epoch;disconnect();stopContext();stopProject();studio.error = '';tasks.status = session.status = 'loading';
  try {
    const [snapshot, rawSession] = await Promise.all([api.bootstrap(), api.readSession()]);
    if (token !== epoch) return;
    activeApi = api;
    setSession(rawSession);
    await refresh(snapshot, { loadThumbnails: api.mode === 'real'&&location.pathname.startsWith('/editor/') });
    if (token !== epoch) return;
    studio.catalog = snapshot.catalog;studio.jobs = snapshot.jobs || [];
    setTasks(snapshot.tasks || [],{reset:true});studio.ready = true;
    if (snapshot.modelWarning) studio.error = snapshot.modelWarning;
    stopContext = startSelectionContext();
    disconnect = api.subscribe(event => {
      if (token !== epoch) return;
      appEvents.emit(event);
      if (event.type.startsWith('task.')) applyTaskEvent(event);
      if (event.type === 'asset.updated') { const payload = event.payload;assets.update(payload.assetId || payload.id, payload.asset || payload.patch || payload,payload.projectId); }
      if (event.type === 'legacy.progress') studio.progress = event.payload;
      if (event.type === 'legacy.queue') tasks.connection = 'connected';
      if (event.type === 'connection.open') { tasks.connection = 'connected';tasks.error = ''; }
      if (event.type.startsWith('hybrid-')) api.listTasks().then(values => { if (token === epoch) setTasks(values); }).catch(error => { if (token === epoch) tasks.error = error.message; });
    }, error => { tasks.connection = 'reconnecting';tasks.error = error.message; });
    if (api.mode === 'mock') tasks.connection = 'connected';
    stopProject = $effect.root(() => {
      $effect(() => { const id = project.currentId;if (id) untrack(() => { void project.restoreDirectory().catch(error => { if (token === epoch && project.currentId === id) session.error = error.message; }); }); });
    });
  } catch (error) {
    if (token !== epoch) return;
    studio.error = error.message;tasks.status = session.status = 'error';tasks.error = session.error = error.message;
  }
}
export async function executeSelection(operations,instruction=null){
 const context=untrack(()=>selection.snapshot()),labels={fit_ratio:'调整比例',trash:'删除',undo:'撤销',redo:'重做',cutout:'抠图',white:'白底',center:'居中',export:'导出'};
 shell.chatCollapsed=false;saveShell();return sendChat(instruction||operations.map(op=>labels[op.kind]||op.kind).join(' → '),{projectId:project.currentId,context,operations:instruction===null?operations:null,api:activeApi||getApi()});
}
export async function stopCurrentTask() {
  const task = tasks.byId[tasks.activeId] || Object.values(tasks.byId).find(value => runningStates.has(value.state)||value.state==='queued');
  if (!task || !runningStates.has(task.state)&&task.state!=='queued') return false;
  const api = activeApi || getApi();await api.stopTask(task);const token=taskSnapshotToken();setTasks(await api.listTasks(),{since:token});return true;
}
export function disposeWorkspace() { epoch++;activeApi=null;disconnect();stopContext();stopProject();disposeHashWorker();disposeThumbnailWorker();dispose(); }
