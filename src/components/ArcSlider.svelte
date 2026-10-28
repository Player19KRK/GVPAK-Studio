<script>
 import {keyboard} from '../lib/shortcuts.ts';
 import {arcPath,polar} from '../paint.js';
 let {label,value,min=0,max=100,step=1,start=6,end=54,radius=104,row=103,side='right',shape='size',unit='',onchange}=$props();
 let dragging=$state(false);
 let angle=$derived(start+(end-start)*(value-min)/(max-min));
 let knob=$derived(polar(radius,angle));
 function set(v){onchange(Math.max(min,Math.min(max,Math.round(v/step)*step)));}
 function position(e){const svg=e.currentTarget.ownerSVGElement,p=svg.createSVGPoint();p.x=e.clientX;p.y=e.clientY;const local=p.matrixTransform(svg.getScreenCTM().inverse());let delta=(Math.atan2(local.y,local.x)*180/Math.PI-start+720)%360;const span=end-start;if(delta>span)delta=delta>(span+360)/2?0:span;set(min+delta/span*(max-min));}
 function down(e){if(e.button!==0)return;e.preventDefault();e.stopPropagation();dragging=true;e.currentTarget.setPointerCapture(e.pointerId);position(e);}
 function keys(e){if(['ArrowUp','ArrowRight','ArrowDown','ArrowLeft','Home','End'].includes(e.key)){e.preventDefault();e.stopPropagation();set(e.key==='Home'?min:e.key==='End'?max:value+(['ArrowUp','ArrowRight'].includes(e.key)?step:-step));}}
 function wheel(node){const handle=e=>{e.preventDefault();e.stopPropagation();if(e.deltaY)set(value+(e.deltaY<0?step:-step));};node.addEventListener('wheel',handle,{passive:false});return {destroy(){node.removeEventListener('wheel',handle);}};}
</script>
<g class="arc-slider" class:dragging role="slider" tabindex="0" aria-label={label} aria-valuemin={min} aria-valuemax={max} aria-valuenow={value} aria-valuetext={`${value}${unit}`} use:keyboard={{keydown: keys}} onpointerdown={down} onpointermove={e=>{if(dragging)position(e);}} onpointerup={()=>dragging=false} onpointercancel={()=>dragging=false} onlostpointercapture={()=>dragging=false} use:wheel>
 <path class="arc-hit" d={arcPath(radius,start,end)}/>
 <path class="arc-track" d={arcPath(radius,start,end)}/>
 {#if angle>start}<path class="arc-fill" d={arcPath(radius,start,angle)}/>{/if}
 <g class="arc-handle" class:opacity-handle={shape==='opacity'} transform={`translate(${knob[0]} ${knob[1]})`}>
  {#if shape==='hardness'}<path class="arc-knob" d="M 0 -4.5 L 4.5 0 L 0 4.5 L -4.5 0 Z"/>
  {:else if shape==='opacity'}<circle class="arc-knob arc-knob-outline" r="4.5"/><path class="arc-half" d="M 0 -4.5 A 4.5 4.5 0 0 1 0 4.5 Z"/>
  {:else}<circle class="arc-knob" r="4"/>{/if}
 </g>
 <text class="arc-description" text-anchor="end" x={side==='right'?20:-8} y={row}>{label}</text>
 <text class="arc-reading" text-anchor="start" x={side==='right'?28:0} y={row}>{value}{unit}</text>
</g>
