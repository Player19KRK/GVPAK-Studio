<script>
 import {onMount} from 'svelte';import {studio} from '../studio.svelte';let dialog;const decision=studio.confirm;onMount(()=>dialog.showModal());
</script>
<dialog bind:this={dialog} class="decision-dialog" oncancel={e=>{e.preventDefault();decision.cancel();}}><h2>{decision.title}</h2><p>The current image has an unsaved draft.</p><div class="decision-actions"><button onclick={decision.cancel}>Cancel</button><button class="line-button" onclick={()=>decision.run(false)}>Replace draft</button><button class="primary" onclick={()=>decision.run(true)}>Save version first</button></div></dialog>
