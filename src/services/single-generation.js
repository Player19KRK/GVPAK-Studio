import {getApi} from './api.ts';import {studio,current,updateProject,selectProject} from '../studio.svelte';import {currentEditorContext} from './editor.js';import {createSelectionContext} from '../lib/target.ts';import {sendChat} from '../stores/chat.svelte';import {shell,saveShell} from '../app/shell.svelte';
export function textGenerationContext(){return createSelectionContext({page:'library',mentions:[],selected:[],editorAssetId:null,filteredIds:[],filter:{folder:'',tag:'',status:'',q:'',sort:''},names:{}});}
export const singleGenerationConfigured=model=>getApi().credentialStatus(model);
export async function submitSingleGeneration({sourceMode,prompt,model,resolution,ratio},api=getApi()){
 if(!current()){const project=await api.createProject('文字生图');updateProject(project);selectProject(project.id);}
 const context=sourceMode==='text'?textGenerationContext():currentEditorContext();
 studio.workflowChat='generate';shell.chatCollapsed=false;saveShell();
 return sendChat(prompt,{projectId:studio.pid,context,operations:[{kind:'generate',source:sourceMode,model,prompt,resolution,ratio:ratio.ratio,orientation:ratio.orientation}],followResults:false,api});
}
