import {getSettingsApi} from '../services/settings.ts';
export const executionPrefs=$state({status:'idle',analysisCount:20,generationCount:4,costCny:2,filterCount:50,imageModel:'gemini-nano-banana-2.1',quota:null,error:''});
export async function refreshExecutionPrefs(){try{const value=await getSettingsApi().execution();Object.assign(executionPrefs,value,{status:'ready',error:''});}catch(error){executionPrefs.status='error';executionPrefs.error=error.message;}}
export function startExecutionPrefs(){void refreshExecutionPrefs();const changed=()=>void refreshExecutionPrefs();window.addEventListener('execution-preferences',changed);return()=>window.removeEventListener('execution-preferences',changed);}
