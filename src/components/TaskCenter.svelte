<script>
 import RegionState from './RegionState.svelte';
 import {onMount,tick} from 'svelte';
 import {LoaderCircle,ChevronDown} from 'lucide-svelte';
 import {tasks} from '../stores/tasks.svelte';import {session} from '../stores/session.svelte';
 import {taskGroups,quotaView} from '../lib/task-view.ts';
 import {monitorTasks,refreshTaskList,stopTask} from '../services/task-monitor.js';
 import {shortcutScope} from '../lib/shortcuts.ts';import Button from '../ui/Button.svelte';
 let {onresult=()=>{},readSession=false,includeNewProduct=false}=$props();
 let open=$state(false),error=$state(''),pending=$state(''),trigger;
 const groups=$derived(taskGroups(Object.values(tasks.byId))),active=$derived(groups.running.length+groups.queued.length),quota=$derived(quotaView(session.quota));
 const labels={queued:'排队中',running:'正在处理',waiting:'等待异步结果',submitted:'已提交',submitting:'正在提交',cancelling:'正在停止后续提交',done:'已完成',partial:'部分完成',failed:'失败',cancelled:'已取消'};
 onMount(()=>monitorTasks({readSession,includeNewProduct}));
 async function retry(){error='';try{await refreshTaskList();}catch(e){error=e.message;}}
 async function close(){open=false;await tick();trigger?.focus();}
 async function stop(task){pending=task.id;error='';try{await stopTask(task);}catch(e){error=e.message;}finally{pending='';}}
</script>
<div class="task-center" use:shortcutScope={{name:'task-drawer',exclusive:open,bindings:open?[{key:'Escape',action:close}]:[]}}>
 <button class="ui-control task-trigger" bind:this={trigger} aria-expanded={open} aria-controls="task-drawer" onclick={()=>open=!open} title="查看运行中和排队任务">{#if active}<span class="task-spin"><LoaderCircle size={15}/></span>{/if}任务 {active}<ChevronDown size={12}/></button>
 <span class:quota-warning={quota?.warning} class="task-quota" title={quota?`已用 ${quota.used} / 上限 ${quota.limit}`:'当前后端未提供配额'}>配额 {quota?quota.percent.toFixed(0)+'%':'未提供'}</span>
 {#if open}<section id="task-drawer" class="task-drawer" aria-label="任务抽屉">
  <header><h2>任务</h2><span>{tasks.connection==='connected'?'实时更新':tasks.connection==='reconnecting'?'正在重连':'等待连接'}</span><Button variant="subtle" onclick={close}>收起</Button></header>
  {#if error||tasks.error}<RegionState compact state="error" title="无法更新任务列表" message={error||tasks.error} onretry={retry}/>{/if}
  {#if tasks.status==='loading'||tasks.status==='idle'}<RegionState compact state="loading" title="正在读取任务"/>{:else if tasks.status==='ready'&&!Object.keys(tasks.byId).length}<p class="task-empty">暂无任务。处理图片后可在这里查看进度。</p>{/if}
  {#each [['running','运行中'],['queued','排队中'],['recent','最近完成']] as [key,label]}
   {#if groups[key].length}<section class="task-group"><h3>{label}</h3>{#each groups[key] as task (task.id)}<div class="task-item"><div><b>{task.title||(task.type==='new-product'?'新品套图':'图片处理')}</b><span>{labels[task.state]||task.state} · {task.done??0} / {task.total??task.targetCount??0}</span>{#if task.state==='queued'}<small>{task.queuePosition==null?'排队位置待服务端提供':`前面还有 ${task.queuePosition} 个（主机优先）`}</small>{/if}{#if task.error}<small class="task-error">{task.error}</small>{/if}</div>{#if key==='running'||key==='queued'}<Button disabled={pending===task.id||task.state==='cancelling'} onclick={()=>stop(task)}>{key==='queued'?'取消':'停止'}</Button>{:else}<Button onclick={()=>{onresult(task);open=false;}}>查看结果</Button>{/if}</div>{/each}</section>{/if}
  {/each}
 </section>{/if}
</div>
<style>
.task-center{display:flex;align-items:center;gap:12px;color:var(--text);font:13px system-ui,sans-serif}.task-trigger{display:flex;align-items:center;gap:6px;white-space:nowrap;border:0;border-radius:6px;padding:6px 8px;min-height:32px;color:var(--text);background:transparent;cursor:pointer}.task-trigger:hover{background:var(--bg-sunken)}.task-trigger:active{background:var(--accent-soft)}.task-trigger:focus-visible{outline:2px solid var(--focus);outline-offset:2px}.task-quota{font-size:12px;white-space:nowrap;color:var(--text-dim)}.quota-warning{color:var(--warning-text);background:var(--warning);padding:4px 8px;border-radius:6px}.task-drawer{position:fixed;top:48px;right:16px;z-index:60;width:min(600px,calc(100vw - 32px));max-height:60dvh;overflow:auto;padding:16px;background:var(--bg-elev);color:var(--text);border:1px solid var(--border);border-radius:0 0 8px 8px;box-shadow:0 8px 20px #0002}.task-drawer header{display:flex;align-items:center;gap:16px}.task-drawer h2{font-size:14px;margin:0 auto 0 0}.task-drawer header>span,.task-empty{font-size:12px;color:var(--text-dim)}.task-group h3{font-size:12px;color:var(--text-dim);font-weight:500;margin:20px 0 8px}.task-item{display:flex;align-items:center;gap:12px;border-top:1px solid var(--border);padding:12px 0}.task-item>div{flex:1;display:flex;flex-direction:column;gap:6px;min-width:0}.task-item b{font-weight:500;font-size:13px}.task-item span,.task-item small{font-size:12px;color:var(--text-dim)}.task-error{font-size:12px;color:var(--danger)!important;overflow-wrap:anywhere}.task-spin{animation:task-rotate 1s linear infinite}@keyframes task-rotate{to{transform:rotate(360deg)}}@media(prefers-reduced-motion:reduce){.task-spin{animation:none}}
</style>
