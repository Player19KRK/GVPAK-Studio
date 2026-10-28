<script>
 import {studio,asset} from '../studio.svelte';import {editor} from '../stores/editor.svelte';import {executeCurrentOperation,currentEditorBusy} from '../services/editor.js';
 let points=$state([]),pending=$state(false),cursor=$state(null),node=$state();
 const active=$derived(studio.tool==='mask'&&!studio.compare&&!currentEditorBusy()&&!pending);
 function point(event){const r=node.getBoundingClientRect();return [Math.max(0,Math.min(1,(event.clientX-r.left)/r.width)),Math.max(0,Math.min(1,(event.clientY-r.top)/r.height))];}
 function down(event){if(!active||event.button!==0)return;event.preventDefault();event.stopPropagation();node.setPointerCapture(event.pointerId);points=[point(event)];}
 function move(event){cursor=point(event);if(points.length&&points.length<4096)points=[...points,cursor];}
 async function finish(event){if(!points.length)return;const stroke=points;points=[];if(event.type==='pointercancel')return;pending=true;try{const r=node.getBoundingClientRect();await executeCurrentOperation([{kind:'mask',mode:editor.mask.mode,radius:Math.min(.5,editor.mask.size/Math.min(r.width,r.height)/2),points:stroke}],editor.mask.mode==='erase'?'擦除掩膜':'恢复掩膜');}catch(error){studio.error=error.message;}finally{pending=false;}}
</script>
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div bind:this={node} class="mask-layer" class:active onpointerdown={down} onpointermove={move} onpointerup={finish} onpointercancel={finish} onpointerleave={()=>{if(!points.length)cursor=null;}} aria-hidden="true">
 {#if active&&cursor}<i style:left={`${cursor[0]*100}%`} style:top={`${cursor[1]*100}%`} style:width={`${editor.mask.size}px`} style:height={`${editor.mask.size}px`}></i>{/if}
 {#if points.length}<svg viewBox="0 0 100 100" preserveAspectRatio="none"><polyline points={points.map(p=>`${p[0]*100},${p[1]*100}`).join(' ')} fill="none" stroke="var(--accent)" stroke-width=".5"/></svg>{/if}
</div>
<style>.mask-layer{position:absolute;inset:0;pointer-events:none;z-index:5}.mask-layer.active{pointer-events:auto;touch-action:none;cursor:crosshair}.mask-layer i{position:absolute;box-sizing:border-box;border:1px solid white;outline:1px solid #222;border-radius:50%;transform:translate(-50%,-50%);pointer-events:none}.mask-layer svg{width:100%;height:100%;opacity:.7;pointer-events:none}</style>
