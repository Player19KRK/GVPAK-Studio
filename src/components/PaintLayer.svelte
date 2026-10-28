<script>
 import {currentEditorBusy} from '../services/editor.js';
 import {onDestroy} from 'svelte';
 import {studio,asset,change,notify} from '../studio.svelte';
 import {clone,hex,rgb} from '../api';
 import {normalizeEdit,outputGeometry,previewKey} from '../preview-cache';
 import {previewCache,requestPreview} from '../preview-loader';
 import {renderPaint,applyStroke,sourcePoint,strokeRadius} from '../paint.js';
 let canvas=$state(),cursor=$state(null),cursorSize=$state(0);
 let active=$derived(!!studio.paintTool&&!studio.spacePan&&!studio.paintMenuOpen&&!studio.compare&&!currentEditorBusy()&&studio.tool!=='crop');
 let edit=$derived(normalizeEdit(studio.tool==='crop'?{...studio.draft,crop:null,rotation:0,flipH:false,flipV:false}:studio.draft));
 let currentStroke=null,completed=null,geometry=null,source=null,frame=0,published=null;
 function draw(){cancelAnimationFrame(frame);frame=requestAnimationFrame(()=>{if(!canvas||!completed||!source)return;const pixels=currentStroke?applyStroke(completed.slice(),geometry.width,geometry.height,currentStroke,edit,source.width,source.height):completed;canvas.getContext('2d').putImageData(new ImageData(pixels,geometry.width,geometry.height),0,0);});}
 $effect(()=>{
  const a=asset(),e=edit,node=canvas;if(!a||!node)return;
  currentStroke=null;source={id:a.id,width:a.width,height:a.height};geometry=outputGeometry(a.width,a.height,e);
  node.width=geometry.width;node.height=geometry.height;
  completed=renderPaint(geometry.width,geometry.height,e.strokes,e,a.width,a.height);draw();
 });
 $effect(()=>{if(!active){currentStroke=null;cursor=null;draw();}});
 // Publish the precise composite for histogram, AI reference, and the image eyedropper.
 $effect(()=>{
  const a=asset(),e=normalizeEdit(studio.draft),root=studio.dataRoot,pid=studio.pid;
  if(!a||!e.strokes.length)return;
  const key=previewKey(root,pid,a,e),signature=JSON.stringify(e),controller=new AbortController();let live=true;
  const timer=setTimeout(()=>requestPreview(key,'preview',{projectId:pid,assetId:a.id,edit:e},{priority:1,signal:controller.signal}).then(entry=>{
   if(!live)return;previewCache.retain(entry);previewCache.release(published);published=entry;
   studio.preview=entry.url;studio.previewAssetId=a.id;studio.previewSignature=signature;
  }).catch(error=>{if(live&&error.name!=='AbortError'&&!error.message.includes('PREVIEW_SUPERSEDED'))studio.error=error.message;}),220);
  return()=>{live=false;clearTimeout(timer);controller.abort();};
 });
 function point(e){const r=canvas.getBoundingClientRect();return [Math.max(0,Math.min(1,(e.clientX-r.left)/r.width)),Math.max(0,Math.min(1,(e.clientY-r.top)/r.height))];}
 function hover(e){if(!active||studio.picking){cursor=null;return;}const local=point(e);cursor={x:local[0]*100,y:local[1]*100};const brush=studio.paintTool==='eraser'?studio.eraser:studio.brush;cursorSize=strokeRadius(brush,edit,source.width,source.height,canvas.getBoundingClientRect().width)*2/canvas.getBoundingClientRect().width*100;}
 async function sample(e){
  const a=asset(),pid=studio.pid,snapshot=normalizeEdit(studio.draft),signature=JSON.stringify(snapshot),p=point(e);
  try{const entry=await requestPreview(previewKey(studio.dataRoot,pid,a,snapshot),'preview',{projectId:pid,assetId:a.id,edit:snapshot},{priority:0});
   if(!studio.picking||studio.aid!==a.id||JSON.stringify(normalizeEdit(studio.draft))!==signature)return;
   const work=document.createElement('canvas');work.width=1;work.height=1;const ctx=work.getContext('2d',{willReadFrequently:true});
   ctx.drawImage(entry.bitmap,Math.min(entry.width-1,Math.floor(p[0]*entry.width)),Math.min(entry.height-1,Math.floor(p[1]*entry.height)),1,1,0,0,1,1);
   const pixel=ctx.getImageData(0,0,1,1).data;if(!pixel[3]){studio.error="此位置透明，请选择有颜色的位置";return;}
   studio.brush.color=hex([...pixel].slice(0,3));studio.picking=false;studio.paintTool='brush';
  }catch(error){studio.error=error.message;}
 }
 function down(e){if(e.button!==0||!active)return;e.stopPropagation();e.preventDefault();if(studio.picking){sample(e);return;}
  canvas.closest('.canvas-stage')?.focus({preventScroll:true});canvas.setPointerCapture(e.pointerId);const settings=studio.paintTool==='eraser'?studio.eraser:studio.brush;
  currentStroke={points:[sourcePoint(point(e),edit,source.width,source.height)],color:rgb(studio.brush.color),size:settings.size,hardness:settings.hardness,opacity:studio.paintTool==='eraser'?1:settings.opacity,erase:studio.paintTool==='eraser'};draw();
 }
 function move(e){hover(e);if(!currentStroke)return;e.stopPropagation();for(const event of e.getCoalescedEvents?.().length?e.getCoalescedEvents():[e]){
  const p=sourcePoint(point(event),edit,source.width,source.height),last=currentStroke.points.at(-1);
  if(Math.hypot((p[0]-last[0])*source.width,(p[1]-last[1])*source.height)>.25)currentStroke.points.push(p);
 }draw();}
 function finish(e){if(!currentStroke)return;e.stopPropagation();const stroke=currentStroke;currentStroke=null;change({strokes:[...(studio.draft.strokes||[]),clone(stroke)]});}
 function cancel(){currentStroke=null;draw();}
 onDestroy(()=>{cancelAnimationFrame(frame);previewCache.release(published);});
</script>
<!-- svelte-ignore a11y_no_noninteractive_element_interactions, a11y_no_interactive_element_to_noninteractive_role (Canvas pointer input has explicit tool and parameter controls.) -->
<canvas bind:this={canvas} class="paint-layer" class:interactive={active} class:picking={studio.picking} role="application" aria-label="绘画图层" tabindex="0" onpointerdown={down} onpointermove={move} onpointerup={finish} onpointercancel={cancel} onlostpointercapture={cancel} onpointerleave={()=>{cursor=null;}}></canvas>
{#if cursor&&active&&!studio.picking}<span class="paint-cursor" style:left={`${cursor.x}%`} style:top={`${cursor.y}%`} style:width={`${cursorSize}%`}></span>{/if}
