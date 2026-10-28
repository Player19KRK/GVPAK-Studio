// Read-only configuration status shared by generation and natural processing.
export function createConfiguration(read){
 const state=$state({status:'idle',configured:false,error:''});let request=0;
 async function load(key){const token=++request;state.status='loading';state.configured=false;state.error='';try{const value=await read(key);if(token!==request)return null;state.configured=!!value;state.status='ready';return state.configured;}catch(error){if(token===request){state.status='error';state.error=error.message||String(error);}return null;}}
 function cancel(){request++;state.status='idle';state.configured=false;state.error='';}
 return {state,load,cancel};
}
