<script>
 import {onDestroy} from 'svelte';
 let {axis,value=$bindable(),min,max,label,onfinish=()=>{},onstart=()=>{},reset=()=>{}}=$props();let drag=null;
 function start(e){if(e.button!==0)return;e.preventDefault();onstart();drag={point:axis==='width'?e.clientX:e.clientY,value:Math.min(max,Math.max(min,value)),element:e.currentTarget};e.currentTarget.setPointerCapture(e.pointerId);document.body.dataset.resizingPanel=axis;}
 function move(e){if(!drag)return;value=Math.round(Math.min(max,Math.max(min,drag.value+drag.point-(axis==='width'?e.clientX:e.clientY))));}
 function finish(){if(!drag)return;drag=null;delete document.body.dataset.resizingPanel;onfinish();}
 function keys(e){let delta=0;if(axis==='width'){if(e.key==='ArrowLeft')delta=1;if(e.key==='ArrowRight')delta=-1;}else{if(e.key==='ArrowUp')delta=1;if(e.key==='ArrowDown')delta=-1;}if(!delta&&!['Home','End'].includes(e.key))return;e.preventDefault();e.stopPropagation();onstart();value=e.key==='Home'?min:e.key==='End'?max:Math.min(max,Math.max(min,Math.min(max,Math.max(min,value))+delta*(e.shiftKey?40:10)));onfinish();}
 onDestroy(()=>{delete document.body.dataset.resizingPanel;});
</script>
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (Focusable resize separators intentionally support pointer and keyboard input.) -->
<div class={`panel-resizer ${axis}`} role="separator" tabindex="0" aria-label={label} aria-orientation={axis==='width'?'vertical':'horizontal'} aria-valuemin={min} aria-valuemax={Math.round(max)} aria-valuenow={Math.min(max,Math.max(min,value))} aria-valuetext={`${Math.round(Math.min(max,Math.max(min,value)))} pixels`} title="Drag to resize · Double-click to reset" onpointerdown={start} onpointermove={move} onpointerup={finish} onpointercancel={finish} onlostpointercapture={finish} onkeydown={keys} ondblclick={()=>{reset();onfinish();}}><span></span></div>


