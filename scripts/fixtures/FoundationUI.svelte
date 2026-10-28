<script>
  import { Settings2 } from 'lucide-svelte';
  import Button from '../../src/ui/Button.svelte';
  import IconButton from '../../src/ui/IconButton.svelte';
  import Segmented from '../../src/ui/Segmented.svelte';
  import Badge from '../../src/ui/Badge.svelte';
  import Tooltip from '../../src/ui/Tooltip.svelte';
  import Resizer from '../../src/ui/Resizer.svelte';
  import Banner from '../../src/ui/Banner.svelte';
  import Spinner from '../../src/ui/Spinner.svelte';
  import Kbd from '../../src/ui/Kbd.svelte';
  let value = $state('a'), width = $state(400), clicks = $state(0);
</script>
<div class="foundation-shell">
  <header class="shell-header"><b>基础组件验收</b></header>
  <main class="foundation-page" style="display:flex;flex-direction:column;align-items:flex-start;gap:24px">
    <Button onclick={() => clicks++}>操作 {clicks}</Button>
    <Button variant="primary">主要操作</Button><Button busy>处理中</Button><Button disabled>不可用</Button>
    <IconButton label="设置" shortcut="Ctrl+,"><Settings2 size={16}/></IconButton>
    <Segmented label="视图" options={[{value:'a',label:'网格'},{value:'b',label:'列表'}]} {value} onchange={next => value=next}/>
    <Badge>状态</Badge><Tooltip text="说明" shortcut="Ctrl+K"><Button>帮助</Button></Tooltip>
    <div style="height:80px;display:flex;gap:16px"><Resizer label="测试宽度" bind:value={width} min={360} max={520} initial={400}/><output aria-label="当前宽度">{width}</output></div>
    <Banner tone="warning">文件夹需要重新授权{#snippet actions()}<Button>重新授权</Button>{/snippet}</Banner>
    <Spinner/><Kbd>Ctrl+K</Kbd>
  </main>
</div>
