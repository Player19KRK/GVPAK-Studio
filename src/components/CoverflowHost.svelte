<script>
 import {untrack,onDestroy} from 'svelte';import Coverflow from './Coverflow.svelte';import CoverflowStatus from './CoverflowStatus.svelte';import {studio,select,thumbnails,current} from '../studio.svelte';import {selection} from '../stores/selection.svelte';import {assets} from '../stores/assets.svelte';import {editor} from '../stores/editor.svelte';import {tasks} from '../stores/tasks.svelte';import {coverflowSource} from '../lib/coverflow-source.ts';
 let frozenIds=$state(null),frozenSource=$state(null),hovered=false,focused=false,lastProject='';
 const results=$derived(Object.values(tasks.byId).filter(task=>task.projectId===studio.pid&&['done','partial','cancelled'].includes(task.state)&&task.resultAssetIds?.length).sort((a,b)=>(b.updatedAt||0)-(a.updatedAt||0))[0]?.resultAssetIds||[]);
 const desired=$derived(coverflowSource(selection.selectedIds,assets.filteredIds,results,assets.availableIds));
 const items=$derived((frozenIds||desired.ids).map(id=>assets.index[id]).filter(asset=>asset&&!asset.trashed&&assets.availableIds.has(asset.id)));
 function lock(){if(frozenIds===null){frozenIds=[...desired.ids];frozenSource=desired.source;}}function unlock(){if(!hovered&&!focused){frozenIds=null;frozenSource=null;}}
 $effect(()=>{const pid=studio.pid;if(pid!==lastProject){lastProject=pid;frozenIds=null;frozenSource=null;hovered=focused=false;}});
 $effect(()=>{const ids=items.map(a=>a.id),id=studio.aid;editor.coverflowIds=ids;if(ids.length&&!ids.includes(id))untrack(()=>select(ids[0],true));const p=current();if(p&&id){const index=ids.indexOf(id);untrack(()=>{void thumbnails(p,false,ids.slice(Math.max(0,index-7),index+8));});}});
 onDestroy(()=>editor.coverflowIds=null);
</script>
<!-- svelte-ignore a11y_no_static_element_interactions (The host only stabilizes its read-only input list while interacting with the original module.) -->
<div class="coverflow-host" data-source={frozenSource||desired.source} onpointerenter={()=>{hovered=true;lock();}} onpointerleave={()=>{hovered=false;unlock();}} onfocusin={()=>{focused=true;lock();}} onfocusout={event=>{if(!event.currentTarget.contains(event.relatedTarget)){focused=false;unlock();}}}>
 <Coverflow {items}/><CoverflowStatus {items}/>
</div>
<style>.coverflow-host{position:relative;flex-shrink:0;min-width:0}</style>
