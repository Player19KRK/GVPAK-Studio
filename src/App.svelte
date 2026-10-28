<script>
 import {startExecutionPrefs} from './stores/execution-prefs.svelte';
 import {connectChat} from './stores/chat.svelte';
 import {commitEditorEdit} from './services/editor.js';
 import EditorTools from './components/EditorTools.svelte';
 import VersionBar from './components/VersionBar.svelte';
 import {editor as editorState} from './stores/editor.svelte';
 import {onMount,untrack,tick} from 'svelte';
 import {SlidersHorizontal,Crop,RotateCw} from 'lucide-svelte';
 import AppShell from './components/AppShell.svelte';
 import FoundationPage from './components/FoundationPage.svelte';
 import RatioBar from './components/RatioBar.svelte';
 import Canvas from './components/Canvas.svelte';
 import Inspector from './components/Inspector.svelte';
 import CoverflowHost from './components/CoverflowHost.svelte';
 import Generate from './components/Generate.svelte';
 import {shell,routing,startRouting,navigate,saveShell} from './app/shell.svelte';
 import {editorUrl} from './lib/router.ts';
 import {installApplicationShortcuts} from './lib/shortcuts.ts';
 import {ratios} from './lib/ratio.ts';
 import {layout,updateViewport} from './layout.svelte';
 import {studio,current,asset,assets,select,run,save,notify,isDirty,thumbnails} from './studio.svelte';
 import {project as projectStore} from './stores/project.svelte';
 import {selection} from './stores/selection.svelte';
 import {assets as assetStore} from './stores/assets.svelte';
 import {chooseRatioCommand,swapRatioCommand,historyCommand,trashSelectionCommand} from './services/commands.js';
 import {initializeWorkspace,disposeWorkspace,stopCurrentTask} from './services/workspace.svelte';
 let compareBeforeHold=false,compareSplit=50,appliedEditorId='',shortcuts;
 let editorRouteValid=$derived(studio.projects.some(project=>project.assets.some(asset=>asset.id===routing.current.assetId&&!asset.trashed)));
 const compareKeys=new Set();
 const selectProject=id=>projectStore.select(id);
 onMount(()=>{
  const rememberedProject=shell.projectId,stopRouting=startRouting(),stopChatSync=connectChat(),stopPrefs=startExecutionPrefs();let live=true;
  studio.inspector=window.innerWidth>900;updateViewport();window.addEventListener('resize',updateViewport);
  shortcuts=installApplicationShortcuts({getContext:()=>({page:routing.current.page,busy:!!studio.busy}),onError:error=>studio.error=error instanceof Error?error.message:String(error),actions:{
   'composer.focus':async()=>{if(routing.current.page==='generate'){document.querySelector('.generation-form textarea[aria-label="图片生成要求"],.np-page textarea[aria-label="新品套图要求"]')?.focus();return;}if(routing.current.page==='home'){document.querySelector('[data-composer]')?.focus();return;}if(!['library','editor'].includes(routing.current.page))return;shell.chatCollapsed=false;saveShell();await tick();document.querySelector('[data-composer]')?.focus();},
   'chat.toggle':()=>{if(routing.current.page==='generate'){const wasOpen=studio.workflowChat==='generate'&&!shell.chatCollapsed;studio.workflowChat='generate';shell.chatCollapsed=wasOpen;saveShell();return;}if(!['library','editor'].includes(routing.current.page))return;shell.chatCollapsed=!shell.chatCollapsed;saveShell();},
   'history.undo':()=>historyCommand('undo'),'history.redo':()=>historyCommand('redo'),
   'task.escape':()=>{studio.picking=false;return stopCurrentTask();},
   'editor.save':saveNow,'library.selectAll':()=>selection.set(assetStore.filteredIds),
   'library.trash':trashSelectionCommand,'library.open':()=>{const id=selection.selectedIds[0]||studio.aid;if(id)navigate(editorUrl(id));},
   ...Object.fromEntries(ratios.map((ratio,index)=>[`editor.ratio.${index+1}`,()=>chooseRatioCommand(ratio)])),
   'editor.ratio.swap':swapRatioCommand,'editor.compare':compareHeld,
   'editor.previous':()=>step(-1),'editor.next':()=>step(1),
   'editor.brush':()=>{studio.paintTool=null;studio.tool='mask';studio.inspectorTab='properties';studio.inspector=true;},
   'editor.tool.previous':()=>cycleTool(-1),'editor.tool.next':()=>cycleTool(1),
   'editor.mask.smaller':()=>editorState.mask.size=Math.max(2,Number(editorState.mask.size)-4),'editor.mask.larger':()=>editorState.mask.size=Math.min(300,Number(editorState.mask.size)+4),'editor.fit':()=>{studio.zoom=1;studio.canvasFit=true;studio.pan={x:0,y:0};},
  }});
  initializeWorkspace().then(()=>{if(!live||!studio.ready)return;const project=studio.projects.find(p=>p.id===rememberedProject);if(project&&routing.current.page!=='editor')selectProject(project.id);});
  return()=>{stopPrefs();stopChatSync();live=false;shortcuts.destroy();stopRouting();window.removeEventListener('resize',updateViewport);disposeWorkspace();};
 });
 $effect(()=>{if(studio.ready&&studio.pid){shell.projectId=studio.pid;saveShell();}});
 $effect(()=>{routing.current.page;routing.current.assetId;shortcuts?.releaseHeld();});
 $effect(()=>{
  const route=routing.current;
  if(route.page!=='editor'){appliedEditorId='';if(route.page==='generate')studio.mode='generate';return;}
  if(!studio.ready||route.assetId===appliedEditorId)return;
  const project=studio.projects.find(p=>p.assets.some(a=>a.id===route.assetId&&!a.trashed));
  if(!project)return;
  untrack(()=>{if(studio.pid!==project.id)selectProject(project.id);if(studio.aid!==route.assetId)select(route.assetId);studio.mode='edit';studio.view='canvas';appliedEditorId=route.assetId;});
 });
 $effect(()=>{const id=studio.aid;if(studio.ready&&routing.current.page==='editor'&&appliedEditorId&&id&&id!==appliedEditorId){appliedEditorId=id;untrack(()=>navigate(editorUrl(id),{replace:true}));}});
 $effect(()=>{if(routing.current.page==='editor'&&studio.ready&&studio.view!=='canvas')untrack(()=>navigate('/library'));});
 $effect(()=>{if(studio.view==='projects')for(const p of studio.projects)thumbnails(p);});
 function panelOpen(value){return studio.inspector&&!layout.focus&&studio.tool===value&&studio.inspectorTab==='adjust';}
 function tool(value,toggle=false){if(studio.busy)return;studio.paintTool=null;studio.picking=false;if(toggle&&panelOpen(value)){studio.inspector=false;return;}layout.focus=false;studio.tool=value;studio.mode='edit';studio.view='canvas';studio.inspectorTab='adjust';studio.inspector=true;}
 function closeInspector(){studio.inspector=false;document.querySelector(`[data-tool="${studio.tool}"]`)?.focus();}
 async function saveNow(){if(!isDirty())return;await commitEditorEdit();}
 function step(direction){const ids=editorState.coverflowIds??(selection.selectedIds.length?selection.selectedIds:assetStore.filteredIds),i=ids.indexOf(studio.aid);if(ids[i+direction])select(ids[i+direction]);}
 function cycleTool(direction){const values=['adjust','crop','transform'],index=values.indexOf(studio.tool);tool(values[(index+direction+values.length)%values.length]);}
 function compareHeld(event,phase){if(phase==='press'){if(!compareKeys.size){compareBeforeHold=studio.compare;compareSplit=studio.split;}compareKeys.add(event.code);studio.compare=true;studio.split=100;}else{compareKeys.delete(event.code);if(!compareKeys.size){studio.compare=compareBeforeHold;studio.split=compareSplit;}}}
</script>
<AppShell projectName={current()?.name||''} assetId={studio.aid} selectedCount={studio.selected.length} projects={studio.projects} projectId={studio.pid} onprojectchange={id=>{selectProject(id);shell.projectId=id;saveShell();if(routing.current.page==='editor'&&studio.aid)navigate(editorUrl(studio.aid));}}>
 {#if routing.current.page==='generate'&&studio.ready}<Generate/>
 {:else if routing.current.page==='editor'&&studio.ready&&asset()&&editorRouteValid}
  <div class="legacy-editor">
   <div class="app-workspace">
    <nav class="toolrail" aria-label="图片工具"><EditorTools/><button data-tool="adjust" onclick={()=>tool('adjust',true)} title="调色" aria-label="调色"><SlidersHorizontal size={20}/></button><button data-tool="crop" onclick={()=>tool('crop',true)} title="裁切" aria-label="裁切"><Crop size={20}/></button><button data-tool="transform" onclick={()=>tool('transform',true)} title="旋转与翻转" aria-label="旋转与翻转"><RotateCw size={20}/></button></nav>
    <main class="editing" aria-label="图片编辑"><div class="editor-canvas"><div class="editor-ratio-strip"><RatioBar disabled={!!studio.busy} onerror={e=>studio.error=e.message}/></div><Canvas/></div><VersionBar/>{#if !layout.focus}<CoverflowHost/>{/if}</main>
    {#if studio.inspector&&!layout.focus}<Inspector onclose={closeInspector} historyEnabled={false}/>{/if}
   </div>
   <div class="editor-status" role="status">{studio.busy||studio.error||studio.draftStatus||'就绪'}</div>
  </div>
 {:else}
  <FoundationPage ready={studio.ready} error={studio.error} retry={() => initializeWorkspace()}/>
 {/if}
</AppShell>

<style>.editor-ratio-strip{display:flex;align-items:center;min-height:40px;padding:4px 12px;box-sizing:border-box;background:var(--bg-elev);border-bottom:1px solid var(--border)}</style>
