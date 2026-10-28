<script>
 import {onMount} from 'svelte';import ChatPanel from '../../src/components/ChatPanel.svelte';import {studio} from '../../src/studio.svelte';import {selection,startSelectionContext} from '../../src/stores/selection.svelte';import {appEvents} from '../../src/services/realtime.ts';import {appendMessage,newConversation,chat} from '../../src/stores/chat.svelte';
 const edit={adjustmentOpacity:1,adjustmentEnabled:true,exposure:0,contrast:1,saturation:1,temperature:0,rotation:0,flipH:false,flipV:false,crop:null,strokes:[]};
 studio.projects=[{id:'fixture-project',name:'Fixture',assets:[{id:'a',name:'正面.png',edit,versions:[],width:160,height:90,sha256:'a',trashed:false},{id:'b',name:'内托.png',edit,versions:[],width:160,height:90,sha256:'b',trashed:false}]}];studio.pid='fixture-project';studio.aid='a';studio.ready=true;selection.set(['a']);
 onMount(()=>{const stop=startSelectionContext();newConversation();window.chatFixture={emit(type,payload){appEvents.emit({type,payload});},append(content){appendMessage({type:'text',role:'assistant',content});},clearSelection(){selection.clear();studio.aid='';},get(){return JSON.parse(JSON.stringify(chat));}};return stop;});
</script>
<div style="height:700px;width:400px;margin:0 auto;border:1px solid var(--border)"><ChatPanel/></div>
