<script>
 import {onMount} from 'svelte';import {X,ArrowUpRight} from 'lucide-svelte';
 let {project,thumbs,anchor,onselect,onclose,onzoom}=$props();let panel,scroller;let row=$state(0);let pictures=$derived(project.assets.filter(a=>!a.trashed));let layout=$state({left:12,top:12,width:330,height:390,origin:'left top'});
 function position(){const r=anchor?.getBoundingClientRect();if(!r)return;const gap=10,margin=12;const width=Math.min(338,innerWidth-24);const cell=(width-36)/3;const height=Math.min(84+Math.min(3,Math.max(1,Math.ceil(pictures.length/3)))*(cell+6),innerHeight-24);let left,top,origin;
  if(innerWidth-r.right>=width+gap+margin){left=r.right+gap;top=r.top;origin='left top';}
  else if(r.left>=width+gap+margin){left=r.left-width-gap;top=r.top;origin='right top';}
  else if(innerHeight-r.bottom>=height+gap+margin){left=r.left;top=r.bottom+gap;origin='left top';}
  else if(r.top>=height+gap+margin){left=r.left;top=r.top-height-gap;origin='left bottom';}
  else{left=r.left+(r.width-width)/2;top=r.top+Math.min(40,r.height/4);origin='center top';}
  layout={left:Math.max(margin,Math.min(innerWidth-width-margin,left)),top:Math.max(margin,Math.min(innerHeight-height-margin,top)),width,height,origin};
 }
 onMount(()=>{position();panel.showPopover();const update=()=>position();window.addEventListener('resize',update);const observer=new ResizeObserver(update);if(anchor)observer.observe(anchor);return()=>{observer.disconnect();window.removeEventListener('resize',update);};});
 $effect(()=>{project.id;anchor;position();row=0;if(scroller)scroller.scrollTop=0;});
 function pointerMove(e){
  if(e.pointerType==='touch'||anchor?.contains(e.target)||panel?.contains(e.target))return;
  const a=anchor?.getBoundingClientRect(),p=panel?.getBoundingClientRect();if(!a||!p){onclose();return;}
  const inside=(l,t,r,b)=>l<=r&&t<=b&&e.clientX>=l&&e.clientX<=r&&e.clientY>=t&&e.clientY<=b;
  // Keep only the narrow gap between the card and its popover traversable.
  const top=Math.max(a.top,p.top),bottom=Math.min(a.bottom,p.bottom),left=Math.max(a.left,p.left),right=Math.min(a.right,p.right);
  if((p.left>=a.right&&inside(a.right,top,p.left,bottom))||(a.left>=p.right&&inside(p.right,top,a.left,bottom))||(p.top>=a.bottom&&inside(left,a.bottom,right,p.top))||(a.top>=p.bottom&&inside(left,p.bottom,right,a.top)))return;
  onclose();
 }
 function scrollRows(delta){if(!scroller)return;scroller.scrollBy({top:delta,behavior:'instant'});}
 export function scrollBy(delta){scrollRows(delta);}
 function wheel(node){const handler=e=>{e.preventDefault();e.stopPropagation();if(e.ctrlKey){onzoom(e);position();return;}scrollRows(e.deltaY*(e.deltaMode===1?20:e.deltaMode===2?node.clientHeight:1));};node.addEventListener('wheel',handler,{passive:false});return{destroy(){node.removeEventListener('wheel',handler);}};}
</script>
<svelte:window onpointermove={pointerMove} onblur={onclose} onpointerout={e=>{if(!e.relatedTarget)onclose();}} onkeydown={e=>{if(e.key==='Escape')onclose();}}/>
<div bind:this={panel} popover="manual" class="project-peek" role="region" aria-label={`Preview ${project.name}`} style:left={`${layout.left}px`} style:top={`${layout.top}px`} style:width={`${layout.width}px`} style:max-height={`${layout.height}px`} style:--peek-origin={layout.origin} style:--peek-cell={`${(layout.width-36)/3}px`} onpointerleave={pointerMove} onfocusout={e=>{if(!e.currentTarget.contains(e.relatedTarget))onclose();}} use:wheel>
 <header><span>{project.name}</span><button aria-label="关闭项目预览" onclick={onclose}><X size={14}/></button></header>
 <div bind:this={scroller} class="peek-grid" onscroll={()=>{row=Math.floor((scroller.scrollTop+.5)/(scroller.firstElementChild?.getBoundingClientRect().height+6||1));}}>{#each pictures as picture,i (picture.id)}<button aria-label={`Open preview ${picture.name}`} title={picture.name} onclick={()=>onselect(project.id,picture.id)}><img src={thumbs[picture.id]} alt={picture.name} loading={i<9?'eager':'lazy'}/><span>{picture.name}</span></button>{/each}</div>
 {#if !pictures.length}<div class="peek-empty">暂无图片</div>{/if}
 <footer><span>{pictures.length?`${Math.min(row*3+1,pictures.length)}–${Math.min((row+3)*3,pictures.length)} / ${pictures.length}`:'0 张图片'}</span>{#if pictures.length>9}<span>滚动查看其余图片</span>{/if}<ArrowUpRight size={12}/></footer>
</div>

