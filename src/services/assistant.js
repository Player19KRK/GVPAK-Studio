import {createAssistantApi} from './assistant-api.ts';
import {getApi} from './api.ts';import {invoke} from './http.ts';import {serializeSelectionContext} from '../lib/target.ts';import {selection} from '../stores/selection.svelte';import {studio} from '../studio.svelte';import {sendChat} from '../stores/chat.svelte';import {shell,saveShell} from '../app/shell.svelte';
let singleton;export const getAssistantApi=()=>singleton||=createAssistantApi(getApi().mode);
export function executeLocalPlan(plan,context=selection.snapshot()){studio.workflowChat='assistant';shell.chatCollapsed=false;saveShell();return sendChat(plan.summary?.trim()||'执行本地处理步骤',{projectId:studio.pid,context,operations:[{kind:'local_plan',plan:JSON.parse(JSON.stringify(plan))}],followResults:false});}

export function executePipeline(instruction,{execution='realtime',model=null}={},context=selection.snapshot()){studio.workflowChat='assistant';shell.chatCollapsed=false;saveShell();return sendChat(instruction,{projectId:studio.pid,context,pipeline:true,execution,model,followResults:false});}
