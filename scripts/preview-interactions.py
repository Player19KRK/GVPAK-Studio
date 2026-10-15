from pathlib import Path
p=Path('src/components/Library.svelte');s=p.read_text(encoding='utf-8-sig')
s=s.replace("<script>","<script>\n import {onDestroy,tick} from 'svelte';import ProjectPeek from './ProjectPeek.svelte';",1)
s=s.replace('Pencil,X}', 'Pencil,X,Minus,ZoomIn}')
s=s.replace("import {api,bytes}","import {api,bytes,stored,persist}")
s=s.replace("function openAsset(id)","""let projectZoom=$state(stored('project-grid-zoom',1)),imageZoom=$state(stored('image-grid-zoom',1)),peek=$state(null),peekComponent,library;let openTimer,closeTimer;
 let zoom=$derived(studio.view==='projects'?projectZoom:imageZoom);
 function changeZoom(value){const next=Math.min(1.8,Math.max(.6,Math.round(value*100)/100));if(studio.view==='projects'){projectZoom=next;persist('project-grid-zoom',next);}else{imageZoom=next;persist('image-grid-zoom',next);}}
 function zoomWheel(e){if(!e.ctrlKey)return;e.preventDefault();e.stopPropagation();changeZoom(zoom*Math.exp(-e.deltaY*(e.deltaMode===1?20:1)*.0015));}
 function zoomArea(node){const handler=e=>zoomWheel(e);node.addEventListener('wheel',handler,{passive:false});return{destroy(){node.removeEventListener('wheel',handler);}};}
 function holdPeek(){clearTimeout(closeTimer);}
 function hidePeek(){clearTimeout(openTimer);clearTimeout(closeTimer);peek=null;}
 function leavePeek(){clearTimeout(openTimer);closeTimer=setTimeout(()=>peek=null,220);}
 function schedulePeek(project,anchor){clearTimeout(openTimer);clearTimeout(closeTimer);if(peek?.project.id===project.id)return;openTimer=setTimeout(()=>peek={project,anchor},170);}
 async function cardWheel(e,project){if(e.ctrlKey)return; e.preventDefault();e.stopPropagation();clearTimeout(openTimer);clearTimeout(closeTimer);if(peek?.project.id!==project.id)peek={project,anchor:e.currentTarget.closest('.project-card')};await tick();peekComponent?.scrollBy(e.deltaY*(e.deltaMode===1?20:1));}
 function openPreview(pid,id){hidePeek();selectProject(pid);select(id);}
 onDestroy(()=>{clearTimeout(openTimer);clearTimeout(closeTimer);});
 function openAsset(id)""")
s=s.replace('<div class="library" inert={!!studio.busy}>','<div bind:this={library} class="library" inert={!!studio.busy} use:zoomArea onscroll={hidePeek} style:--project-card-size={`${230*projectZoom}px`} style:--project-stack-height={`${180*projectZoom}px`} style:--image-card-size={`${170*imageZoom}px`} style:--image-thumb-height={`${165*imageZoom}px`}>')
s=s.replace('<div class="button-row">{#if studio.view', '<div class="button-row"><div class="grid-zoom" aria-label="Preview size"><button aria-label="Smaller previews" title="Smaller previews" disabled={zoom<=.6} onclick={()=>changeZoom(zoom-.1)}><Minus size={13}/></button><button class="zoom-readout" aria-label="Reset preview zoom" title="Ctrl + scroll · Reset to 100%" onclick={()=>changeZoom(1)}>{Math.round(zoom*100)}%</button><button aria-label="Larger previews" title="Larger previews" disabled={zoom>=1.8} onclick={()=>changeZoom(zoom+.1)}><ZoomIn size={13}/></button></div>{#if studio.view',1)
s=s.replace('<article class="project-card">','<article class="project-card" onpointerenter={e=>{if(e.pointerType!==\'touch\')schedulePeek(project,e.currentTarget);}} onpointerleave={leavePeek}>')
s=s.replace('class="project-open" oncontextmenu=', 'class="project-open" onfocus={e=>schedulePeek(project,e.currentTarget.closest(\'.project-card\'))} onblur={leavePeek} oncontextmenu=')
start=s.index('onwheel={e=>{e.preventDefault();const count=project.assets')
end=s.index(' onclick={()=>selectProject(project.id)}',start)
s=s[:start]+"onwheel={e=>cardWheel(e,project)}"+s[end:]
s=s.replace('onclick={()=>selectProject(project.id)}','onclick={()=>{hidePeek();selectProject(project.id);}}')
s=s.rstrip()+"\n{#if peek&&studio.view==='projects'}<ProjectPeek bind:this={peekComponent} project={peek.project} thumbs={studio.thumbs} anchor={peek.anchor} onselect={openPreview} onclose={hidePeek} onenter={holdPeek} onleave={leavePeek} onzoom={zoomWheel}/>{/if}\n"
p.write_text(s,encoding='utf-8')
p=Path('src/components/Coverflow.svelte');s=p.read_text(encoding='utf-8-sig').replace('Check,FolderOpen}', 'Check,FolderOpen,Minus,SquareCheck}')
s=s.replace('let accumulator=0;', "let accumulator=0;let selectedCount=$derived(list.filter(a=>studio.selected.includes(a.id)).length);let allSelected=$derived(list.length>0&&selectedCount===list.length);\n function selectAll(){if(studio.busy)return;const ids=list.map(a=>a.id);studio.selected=allSelected?studio.selected.filter(id=>!ids.includes(id)):[...new Set([...studio.selected,...ids])];studio.batchScope='auto';}")
s=s.replace('<span>{list.length} images', '<span class="cf-header-actions"><button class="cf-select-all" class:active={allSelected} role="checkbox" aria-checked={allSelected?true:selectedCount>0?\'mixed\':false} aria-label={allSelected?\'Deselect all images\':\'Select all images\'} title={allSelected?\'Deselect all\':selectedCount?\'Select remaining images\':\'Select all\'} disabled={!list.length||!!studio.busy} onclick={selectAll}>{#if allSelected}<SquareCheck size={13}/>{:else if selectedCount}<Minus size={13}/>{:else}<span class="select-all-square"></span>{/if}All</button>{list.length} images',1)
p.write_text(s,encoding='utf-8')
