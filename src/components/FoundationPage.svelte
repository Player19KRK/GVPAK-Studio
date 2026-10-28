<script>
 import Settings from './Settings.svelte';
  import AssetLibrary from './AssetLibrary.svelte';
  import HomePage from './HomePage.svelte';
  import { FolderOpen, Image, AlertCircle } from 'lucide-svelte';
  import { shell, routing, navigate, saveShell } from '../app/shell.svelte.js';
  import Button from '../ui/Button.svelte';
  import Banner from '../ui/Banner.svelte';
  import Spinner from '../ui/Spinner.svelte';
  import Resizer from '../ui/Resizer.svelte';
  let { ready = false, error = '', retry = () => {} } = $props();
</script>
{#if !ready && error}
  <section class="foundation-error"><h1>工作区暂时无法打开</h1><Banner tone="danger"><AlertCircle size={16}/><span>{error}</span>{#snippet actions()}<Button onclick={retry}>重试</Button>{/snippet}</Banner></section>
{:else if !ready}
  <section class="foundation-loading" aria-busy="true" aria-label="正在加载工作区"><div class="foundation-loading-label"><Spinner/><span>正在打开工作区</span></div><div aria-hidden="true"><div class="foundation-skeleton"></div><div class="foundation-skeleton"></div><div class="foundation-skeleton"></div></div></section>
{:else if routing.current.page === 'home'}
  <HomePage/>
{:else if routing.current.page === 'library'}
  <AssetLibrary/>
{:else if routing.current.page === 'settings'}
  <Settings/>
{:else if routing.current.page === 'not-found'}
  <section class="foundation-empty"><h1>找不到这个页面</h1><p>请检查地址，或返回首页。</p><Button onclick={() => navigate('/')}>返回首页</Button></section>
{:else}
  <section class="foundation-empty"><h1>图片暂时无法打开</h1><p>请回到素材库重新选择。</p><Button onclick={() => navigate('/library')}>打开素材库</Button></section>
{/if}
