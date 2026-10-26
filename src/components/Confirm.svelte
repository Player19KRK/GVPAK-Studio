<script>
 import {onMount} from 'svelte';import {studio} from '../studio.svelte';let dialog;const decision=studio.confirm;onMount(()=>dialog.showModal());
</script>
<dialog bind:this={dialog} class="decision-dialog" oncancel={e=>{e.preventDefault();decision.cancel();}}><h2>{decision.title}</h2><p>当前图片有未保存的修改。</p><div class="decision-actions"><button onclick={decision.cancel}>取消</button><button class="line-button" onclick={()=>decision.run(false)}>放弃当前修改</button><button class="primary" onclick={()=>decision.run(true)}>先保存当前版本</button></div></dialog>
