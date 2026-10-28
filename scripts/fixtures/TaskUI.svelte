<script>
 import {onMount} from 'svelte';import TaskCenter from '../../src/components/TaskCenter.svelte';import FolderAuthorization from '../../src/components/FolderAuthorization.svelte';
 import {appEvents} from '../../src/services/realtime.ts';import {setTasks} from '../../src/stores/tasks.svelte';import {session} from '../../src/stores/session.svelte';import {project} from '../../src/stores/project.svelte';import {studio} from '../../src/studio.svelte';
 onMount(()=>{window.taskFixture={seed(){setTasks([{id:'run',state:'running',type:'hybrid',done:1,total:4},{id:'queue',state:'queued',type:'hybrid',queuePosition:2,total:2},{id:'done',state:'done',type:'hybrid',done:3,total:3,resultAssetIds:[]}]);session.quota={used:68,limit:100};},emit(type,payload){appEvents.emit({type,payload});},async restore(){studio.pid='fixture';await project.restoreDirectory();}};});
</script>
<header style="height:48px;display:flex;align-items:center;justify-content:flex-end"><TaskCenter/></header><FolderAuthorization/><p>页面内容</p>
