<script>
 import {studio,scope,scopeLabel,liveAssets,asset} from '../studio.svelte';
</script>
<div class="scope-picker"><label>Apply to<select aria-label="Operation scope" bind:value={studio.batchScope} disabled={!!studio.busy}><option value="auto">{studio.selected.length?`Selected images (${studio.selected.length})`:'Current image'}</option>{#if studio.selected.length}<option value="current">Current image only</option>{/if}<option value="project">All project images ({liveAssets().length})</option></select></label><span>{scope().length} {scope().length===1?'image':'images'}</span><div class="scope-thumbs">{#each scope().slice(0,4) as id}<img src={studio.thumbs[id]} alt=""/>{/each}{#if scope().length>4}<b>+{scope().length-4}</b>{/if}</div></div>
