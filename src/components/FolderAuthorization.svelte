<script>
 import {onMount,untrack} from 'svelte';import {project} from '../stores/project.svelte';import Button from '../ui/Button.svelte';
 let {restore=false}=$props();let error=$state(''),busy=$state(false);
 $effect(()=>{const id=project.currentId;if(restore&&id)untrack(()=>{void project.restoreDirectory().catch(e=>error=e.message);});});
 onMount(()=>{const check=()=>{if(document.visibilityState==='visible')void project.checkPermission().catch(e=>error=e.message);};window.addEventListener('focus',check);document.addEventListener('visibilitychange',check);return()=>{window.removeEventListener('focus',check);document.removeEventListener('visibilitychange',check);};});
 async function authorize(){busy=true;error='';try{await project.authorizeDirectory();}catch(e){if(e.name!=='AbortError')error=e.message;}finally{busy=false;}}
</script>
{#if project.directoryHandle&&project.permission!=='granted'}<div class="folder-authorization" role="status"><span>文件夹授权已失效{error?'：'+error:''}</span><Button disabled={busy} onclick={authorize}>重新授权</Button></div>{/if}
<style>.folder-authorization{position:absolute;z-index:40;left:0;right:0;top:48px;height:32px;display:flex;align-items:center;gap:12px;padding:0 16px;background:var(--warning);color:var(--warning-text);font:12px system-ui,sans-serif}.folder-authorization :global(button){min-height:24px;padding:2px 8px}.folder-authorization span{overflow:hidden;white-space:nowrap;text-overflow:ellipsis}</style>
