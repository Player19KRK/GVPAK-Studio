import {createSelectionContext} from '../lib/target.ts';import {assets} from '../stores/assets.svelte';import {studio} from '../studio.svelte';import {sendChat} from '../stores/chat.svelte';import {tasks} from '../stores/tasks.svelte';
export function currentEditorBusy(){return !!studio.busy||Object.values(tasks.byId).some(task=>task.projectId===studio.pid&&['queued','running','waiting'].includes(task.state)&&(task.assetIds?.includes(studio.aid)||task.context?.resolvedTarget?.ids?.includes(studio.aid)))||assets.index[studio.aid]?.processingStatus==='running';}
export function currentEditorContext(){return createSelectionContext({page:'editor',mentions:[],selected:[],editorAssetId:studio.aid||null,filter:assets.filter,filteredIds:assets.filteredIds,names:Object.fromEntries(Object.values(assets.index).map(asset=>[asset.id,asset.name]))});}
export function executeCurrentOperation(operations,label){return sendChat(label,{context:currentEditorContext(),projectId:studio.pid,operations,followResults:false});}
export const selectVersion=versionId=>executeCurrentOperation([{kind:'restore',versionId}],'切换到'+(versionId?'历史版本':'原图'));

export function commitEditorEdit(edit=studio.draft,label='保存当前编辑'){return executeCurrentOperation([{kind:'commit_edit',edit:JSON.parse(JSON.stringify(edit))}],label);}
