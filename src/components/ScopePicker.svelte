<script>
 import {studio,scope,scopeLabel,liveAssets,asset} from '../studio.svelte';
</script>
<div class="scope-picker"><label>处理范围<select aria-label="处理范围" bind:value={studio.batchScope} disabled={!!studio.busy}><option value="auto">{studio.selected.length?`Selected 张图片 (${studio.selected.length})`:"当前图片"}</option>{#if studio.selected.length}<option value="current">仅当前图片</option>{/if}<option value="project">项目全部图片（{liveAssets().length}）</option><option value="selection">已勾选图片（{studio.selected.length}）</option></select></label><span>{scope().length} {scope().length===1?'张图片':'张图片'}</span><div class="scope-thumbs">{#each scope().slice(0,4) as id}<img src={studio.thumbs[id]} alt=""/>{/each}{#if scope().length>4}<b>+{scope().length-4}</b>{/if}</div></div>
