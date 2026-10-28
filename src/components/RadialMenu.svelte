<script>
 import {keyboard} from '../lib/shortcuts.ts';
 import {onMount,tick} from 'svelte';
 import {fade} from 'svelte/transition';
 import {Palette,Brush,Eraser,Pipette,ArrowLeft,X} from 'lucide-svelte';
 import {studio} from '../studio.svelte';
 import {sectorPath,polar,paletteColor} from '../paint.js';
 import {radialPlacement} from '../radial-layout.js';
 import ArcSlider from './ArcSlider.svelte';
 let {anchor,onclose}=$props();
 let page=$state('root'),host=$state(),hoverColor=$state(''),placement=$state(null);
 const duration=window.matchMedia('(prefers-reduced-motion: reduce)').matches?0:150;
 const items=[{id:'color',name:"调色",icon:Palette},{id:'brush',name:"画笔",icon:Brush},{id:'eraser',name:"橡皮擦",icon:Eraser}];
 function place(){const stage=document.querySelector('.canvas-stage')?.getBoundingClientRect();if(!stage)return;placement=radialPlacement(page,anchor,stage,{width:innerWidth,height:innerHeight});}
 $effect(()=>{page;anchor;place();});
 onMount(()=>{place();tick().then(()=>host?.querySelector('[tabindex="0"]')?.focus());const stage=document.querySelector('.canvas-stage');const observer=new ResizeObserver(place);if(stage)observer.observe(stage);window.visualViewport?.addEventListener('resize',place);return()=>{observer.disconnect();window.visualViewport?.removeEventListener('resize',place);};});
 async function enter(id){page=id;hoverColor='';if(id==='brush'||id==='eraser'){studio.paintTool=id;studio.picking=false;studio.compare=false;if(studio.tool==='crop')studio.tool='adjust';}place();await tick();host.querySelector('[tabindex="0"]')?.focus();}
 function back(){if(page==='root')onclose();else enter('root');}
 function choose(color){studio.brush.color=color;studio.paintTool='brush';studio.picking=false;studio.compare=false;if(studio.tool==='crop')studio.tool='adjust';onclose();}
 function pick(){studio.picking=true;studio.paintTool='brush';studio.compare=false;if(studio.tool==='crop')studio.tool='adjust';onclose();}
 function keys(e){if(e.key==='Escape'){e.preventDefault();e.stopPropagation();back();}else if(e.key==='Tab'){const focusable=[...host.querySelectorAll('[tabindex="0"],button')];const index=focusable.indexOf(document.activeElement);e.preventDefault();focusable[(index+(e.shiftKey?-1:1)+focusable.length)%focusable.length]?.focus();}}
 function activate(e,fn){if(e.key==='Enter'||e.key===' '){e.preventDefault();e.stopPropagation();fn();}}
</script>
<svelte:window onresize={place}/>
{#if placement}<div class="paint-menu-shade" role="presentation" onpointerdown={e=>{if(e.target===e.currentTarget){e.preventDefault();e.stopPropagation();onclose();}}} oncontextmenu={e=>{e.preventDefault();onclose();}}>
 <div bind:this={host} class="paint-wheel" class:parameter-mode={page==='brush'||page==='eraser'} style:left={`${placement.left}px`} style:top={`${placement.top}px`} style:width={`${placement.width}px`} style:height={`${placement.height}px`} role="dialog" aria-modal="true" aria-label="绘画工具" tabindex="-1" use:keyboard={{keydown: keys}}>
  {#if page==='root'||page==='color'}<div class="wheel-glass"></div>{/if}
  <svg viewBox={placement.viewBox} class="wheel-svg" role="group" aria-label={page==='root'?"绘画工具":page==='color'?"颜色选择":`${page} settings`}>

   {#key page}<g in:fade={{duration}}>
    {#if page==='root'}
     {#each items as item,i}
      {@const start=-150+i*120}{@const location=polar(70,start+60)}
<!-- svelte-ignore a11y_click_events_have_key_events (Keyboard activation uses the central keyboard action.) -->
      <g class="wheel-sector" role="button" tabindex="0" aria-label={item.name} onclick={()=>enter(item.id)} use:keyboard={{keydown: e=>activate(e,()=>enter(item.id))}}>
       <path d={sectorPath(30,104,start+1.2,start+118.8)}/>
       <g transform={`translate(${location[0]-9} ${location[1]-15})`}><item.icon size={18} strokeWidth={1.5}/></g>
       <text x={location[0]} y={location[1]+15}>{item.name}</text>
      </g>
     {/each}
    {:else if page==='color'}
     {#each Array(72) as _,i}
      {@const ring=Math.floor(i/24)}{@const start=-90+(i%24)*15}{@const inner=40+ring*32}{@const color=paletteColor(i)}
      {#if i===69}
       {@const point=polar(inner+15,start+7.5)}
<!-- svelte-ignore a11y_click_events_have_key_events (Keyboard activation uses the central keyboard action.) -->
       <g class="wheel-swatch wheel-eyedropper" role="button" tabindex="0" aria-label="从图片取色" onclick={pick} use:keyboard={{keydown: e=>activate(e,pick)}}>
        <path d={sectorPath(inner,inner+30,start+.6,start+14.4)}/>
        <g transform={`translate(${point[0]-7} ${point[1]-7})`}><Pipette size={14} strokeWidth={1.6}/></g>
       </g>
      {:else}
<!-- svelte-ignore a11y_click_events_have_key_events (Keyboard activation uses the central keyboard action.) -->
       <path class="wheel-swatch" class:selected={studio.brush.color.toLowerCase()===color} d={sectorPath(inner,inner+30,start+.6,start+14.4)} fill={color} role="button" tabindex="0" aria-label={`Use ${color}`} onpointerenter={()=>hoverColor=color} onpointerleave={()=>hoverColor=''} onfocus={()=>hoverColor=color} onblur={()=>hoverColor=''} onclick={()=>choose(color)} use:keyboard={{keydown: e=>activate(e,()=>choose(color))}}/>
      {/if}
     {/each}
    {:else}
     {@const settings=page==='brush'?studio.brush:studio.eraser}
     {@const isBrush=page==='brush'}{@const location=polar(70,isBrush?30:150)}
     {@const start=isBrush?-8:145}{@const end=isBrush?35:188}{@const side=isBrush?'right':'left'}
<!-- svelte-ignore a11y_click_events_have_key_events (Keyboard activation uses the central keyboard action.) -->
     <g class="parameter-tool" role="button" tabindex="0" aria-label="返回绘画工具" onclick={back} use:keyboard={{keydown: e=>activate(e,back)}}>
      <circle class="parameter-tool-hit" cx={location[0]} cy={location[1]} r="25"/>
      <g transform={`translate(${location[0]-9} ${location[1]-15})`}>{#if isBrush}<Brush size={18} strokeWidth={1.5}/>{:else}<Eraser size={18} strokeWidth={1.5}/>{/if}</g>
      <text x={location[0]} y={location[1]+15}>{isBrush?"画笔":"橡皮擦"}</text>
     </g>
     <ArcSlider label="大小" value={settings.size} min={1} max={512} {start} {end} {side} radius={104} row={63} shape="size" unit="px" onchange={v=>settings.size=v}/>
     <ArcSlider label="硬度" value={Math.round(settings.hardness*100)} {start} {end} {side} radius={128} row={85} shape="hardness" unit="%" onchange={v=>settings.hardness=v/100}/>
     {#if isBrush}<ArcSlider label="不透明度" value={Math.round(settings.opacity*100)} {start} {end} {side} radius={152} row={107} shape="opacity" unit="%" onchange={v=>settings.opacity=v/100}/>{/if}

    {/if}
   </g>{/key}
  </svg>
  {#if page==='root'||page==='color'}<button class="wheel-center" aria-label={page==='root'?"关闭绘画工具":"返回绘画工具"} title={page==='root'?"关闭":"返回"} onclick={back}>{#if page==='root'}<X size={15} strokeWidth={1.5}/>{:else}<ArrowLeft size={15} strokeWidth={1.5}/>{/if}</button>{/if}
  {#if page==='color'}<div class="wheel-caption" aria-live="polite">{hoverColor||studio.brush.color}</div>{/if}
 </div>
</div>{/if}
