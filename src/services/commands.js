import {executeCurrentOperation} from './editor.js';
import {rgb} from '../api';
import {studio,undo,redo,isDirty} from '../studio.svelte';
import {editor} from '../stores/editor.svelte';
import {tasks} from '../stores/tasks.svelte';
import {chooseRatio,swapRatio} from '../lib/ratio.ts';
import {executeSelection} from './workspace.svelte';
let ratioRequest = 0;
export async function applyRatioState(state) {
  const request = ++ratioRequest;
  const operation={kind:'fit_ratio',ratio:state.ratio,orientation:state.orientation,long_edge:editor.properties.longEdge??undefined,mode:editor.properties.mode,background:rgb(editor.properties.paddingColor)};
  const result = await (studio.mode==='edit'&&studio.view==='canvas'?executeCurrentOperation([operation],'调整当前图片比例'):executeSelection([operation]));
  if (request === ratioRequest) editor.ratio = state;
  return result;
}
export const chooseRatioCommand = ratio => applyRatioState(chooseRatio(ratio));
export function swapRatioCommand() { if (editor.ratio.ratio === '1:1') return;return applyRatioState(swapRatio(editor.ratio)); }
export function historyCommand(direction) {
  if(studio.busy)return false;
  if(studio.mode==='edit'&&studio.view==='canvas'){
    if(direction==='undo'&&isDirty()){undo();return;}
    if(direction==='redo'&&studio.redo[studio.aid]?.length){redo();return;}
    if(direction==='redo'&&isDirty())return false;
  }
  const batch = Object.values(tasks.byId).filter(task => task.batchId && task.undoable!==false && task.projectId===studio.pid && ['done','partial'].includes(task.state)&& (direction==='undo'?!task.undone:task.undone)).sort((a,b)=>(b.updatedAt||0)-(a.updatedAt||0))[0];
  if (batch) return executeSelection([{kind:direction,batchId:batch.batchId}]);
  // Input fields keep native history through the centralized shortcut manager.
  if (direction === 'undo') undo();else redo();
}
export function trashSelectionCommand() {
  if (!studio.selected.length) return false;
  return executeSelection([{kind:'trash'}]);
}
