<script>
  import {studio} from '../studio.svelte';
  import { PanelRightClose, PanelRightOpen, ChevronRight, Settings2 } from 'lucide-svelte';
  import { shell, routing, navigate, saveShell } from '../app/shell.svelte.js';
  import { editorUrl, libraryUrl } from '../lib/router.ts';
  import IconButton from '../ui/IconButton.svelte';
  import Resizer from '../ui/Resizer.svelte';
  import Button from '../ui/Button.svelte';
  import ChatPanel from './ChatPanel.svelte';
  import TaskCenter from './TaskCenter.svelte';import FolderAuthorization from './FolderAuthorization.svelte';
  import {selection} from '../stores/selection.svelte';
  import {chat} from '../stores/chat.svelte';
  let { children, projectName = '', assetId = '', selectedCount = 0, projects = [], projectId = '', onprojectchange = () => {} } = $props();
  let chatVisible = $derived(['library', 'editor'].includes(routing.current.page)||routing.current.page==='generate'&&studio.workflowChat==='generate');
  let chatOpen = $derived(chatVisible && !shell.chatCollapsed);
  const isLocalHost = window.__gvpakLocalHost === true;
  function follow(event, url) {
    if (event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
    event.preventDefault(); navigate(url);
  }
</script>
<div class="foundation-shell" style:--chat-w={`${chatOpen ? shell.chatWidth : 0}px`} style:--tree-w={`${shell.treeWidth}px`}>
  <header class="shell-header">
    <a class="shell-brand" href="/" onclick={event => follow(event, '/')} aria-label="GVPAK Studio 首页"><b>GVPAK</b><span>STUDIO</span></a>
    {#if projects.length}<select class="ui-control shell-project-select" aria-label="当前项目" value={projectId} onchange={event => onprojectchange(event.currentTarget.value)}>{#each projects as project (project.id)}<option value={project.id}>{project.name}</option>{/each}</select>{:else}<Button class="shell-project" onclick={() => navigate('/library')} title={projectName || '打开素材库'}>{projectName || '选择项目'}</Button>{/if}
    <nav class="shell-nav" aria-label="主导航">
      <a href="/" aria-current={routing.current.page === 'home' ? 'page' : undefined} onclick={event => follow(event, '/')}>首页</a>
      <a href={libraryUrl(shell.filter)} aria-current={routing.current.page === 'library' ? 'page' : undefined} onclick={event => follow(event, libraryUrl(shell.filter))}>素材库</a>
      {#if assetId}<a href={editorUrl(assetId)} aria-current={routing.current.page === 'editor' ? 'page' : undefined} onclick={event => follow(event, editorUrl(assetId))}>编辑器</a>{/if}
      <a href="/generate" aria-current={routing.current.page === 'generate' ? 'page' : undefined} onclick={event => follow(event, '/generate')}>生成图片</a>
    </nav>
    <div class="shell-header-end">
      <TaskCenter onresult={task=>{if(task.type==='new-product')navigate('/generate');else{selection.set(task.resultAssetIds||[]);navigate('/library');}}}/>
      <span>{window.__gvpakTeamUser?.username || '本机工作区'}</span>
      {#if isLocalHost}<IconButton label="设置" onclick={() => navigate('/settings')}><Settings2 size={16}/></IconButton>{/if}
      {#if chatVisible}<IconButton label={chatOpen ? '收起对话栏' : '展开对话栏'} shortcut="Ctrl+J" aria-expanded={chatOpen} aria-controls="shell-chat" onclick={() => { shell.chatCollapsed = !shell.chatCollapsed; saveShell(); }}>{#if chatOpen}<PanelRightClose size={16}/>{:else}<PanelRightOpen size={16}/>{/if}</IconButton>{/if}
    </div>
  </header>
  <FolderAuthorization/>
  <div class="shell-body">
    <main class="shell-main" id="main-content" aria-label="工作区">{@render children?.()}</main>
    {#if chatOpen}
      <aside class="shell-chat" id="shell-chat" aria-label="对话">
        <Resizer label="对话栏宽度" bind:value={shell.chatWidth} min={360} max={520} initial={400} direction={-1} onchange={saveShell}/>
        <ChatPanel/>
      </aside>
    {:else if chatVisible}
      <button type="button" class="ui-control shell-expand" aria-label="展开对话栏" aria-expanded="false" title="展开对话栏 · Ctrl+J" onclick={() => { shell.chatCollapsed = false; saveShell(); }}><ChevronRight size={16}/></button>
    {/if}
  </div>
</div>
