<script>
 import {shell,saveShell} from '../app/shell.svelte';
 import {studio,asset} from '../studio.svelte';import {editor} from '../stores/editor.svelte';import {executeCurrentOperation,currentEditorBusy} from '../services/editor.js';
 let pending=$state(false);async function run(kind,label){pending=true;try{await executeCurrentOperation([{kind}],label);}catch(error){studio.error=error.message;}finally{pending=false;}}
 function mask(){shell.chatCollapsed=true;saveShell();studio.paintTool=null;studio.tool='mask';studio.inspector=true;studio.inspectorTab='properties';}
</script>
<div class="editor-tools" aria-label="主体处理工具"><button disabled={!asset()||pending||currentEditorBusy()} title="抠图" aria-label="抠图" onclick={()=>run('cutout','抠图')}>抠图</button><button disabled={!asset()||pending||currentEditorBusy()} title="主体居中" aria-label="主体居中" onclick={()=>run('center','主体居中')}>居中</button><button disabled={!asset()||pending||currentEditorBusy()} class:chosen={studio.tool==='mask'} title="掩膜加/擦 · B" aria-label="掩膜画笔" onclick={mask}>掩膜</button><button disabled={!asset()||pending||currentEditorBusy()} title="识别主体并旋转纠偏" aria-label="旋转纠偏" onclick={()=>run('deskew','主体自动摆正')}>摆正</button></div>
<style>.editor-tools{display:flex;flex-direction:column;gap:4px}.editor-tools button{font-size:11px;min-height:32px;padding:4px 2px}.editor-tools button.chosen{background:var(--accent-soft);color:var(--accent)}</style>
