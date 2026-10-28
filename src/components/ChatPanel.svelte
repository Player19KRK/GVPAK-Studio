<script>
 import {onMount,tick,untrack} from 'svelte';import {chat,newConversation,connectChat,retryChat,latestRunningRow,stopChat} from '../stores/chat.svelte';import {selection} from '../stores/selection.svelte';import {project} from '../stores/project.svelte';import {assets} from '../stores/assets.svelte';import {navigate} from '../app/shell.svelte';import {studio} from '../studio.svelte';
 import {shortcutScope} from '../lib/shortcuts.ts';import Button from '../ui/Button.svelte';import Composer from './Composer.svelte';import ChatText from './ChatText.svelte';import ExecRow from './ExecRow.svelte';import AssetGridCard from './AssetGridCard.svelte';import ErrorCard from './ErrorCard.svelte';
 let {onshow=null,onexport=null}=$props();let list,atBottom=$state(true),unread=$state(false);
 const messages=$derived(chat.messages[chat.activeId]||[]);
 onMount(connectChat);
 function show(row){if(row.projectId&&project.currentId!==row.projectId)project.select(row.projectId);assets.setFilter({folder:'',tag:'',status:'',q:''});selection.set(row.resultAssetIds||[]);if(onshow)onshow(row);else navigate('/library');}
 function exportRow(row){if(row.projectId&&project.currentId!==row.projectId)project.select(row.projectId);selection.set(row.resultAssetIds||[]);if(onexport)onexport(row);else{studio.batchScope='selected';studio.mode='export';navigate('/library');}}
 function scroll(){atBottom=list.scrollHeight-list.scrollTop-list.clientHeight<48;if(atBottom)unread=false;}
 async function bottom(){await tick();if(list){list.scrollTop=list.scrollHeight;atBottom=true;unread=false;}}
 $effect(()=>{messages.map(message=>[message.id,message.content,message.state,message.done].join(':')).join('|');untrack(()=>{if(atBottom)void bottom();else unread=true;});});
</script>
<section class="chat-panel" aria-label="对话" use:shortcutScope={{name:'chat',exclusive:false,bindings:[{key:'Escape',action:()=>{const row=latestRunningRow();if(row)void stopChat(row);}}]}}>
 <header><select aria-label="当前对话" value={chat.activeId||''} onchange={event=>{chat.activeId=event.currentTarget.value;chat.editingId=null;atBottom=true;void bottom();}}>{#if !chat.conversations.length}<option value="">对话</option>{/if}{#each chat.conversations as conversation}<option value={conversation.id}>{conversation.title||'新对话'}</option>{/each}</select><Button variant="subtle" onclick={()=>{newConversation();atBottom=true;}}>+ 新对话</Button></header>
 <div class="chat-messages" bind:this={list} onscroll={scroll} role="log" aria-label="对话消息" aria-live="polite">
 {#if !messages.length}<div class="chat-empty"><p>选择图片，直接说要做什么。</p><span>也可以输入 /比例 16:9 竖，或拖入图片。</span></div>{/if}
 {#each messages as message (message.id)}<div class="chat-message">{#if message.type==='text'&&message.content}<ChatText content={message.content} role={message.role}/>{:else if message.type==='exec'}<ExecRow row={message} onshow={show} onexport={exportRow}/>{:else if message.type==='assets'}<AssetGridCard ids={message.ids} onshow={ids=>show({resultAssetIds:ids})}/>{:else if message.type==='error'}<ErrorCard message={message.message} onretry={()=>retryChat(message.row)} onskip={()=>message.type='skipped'}/>{/if}</div>{/each}
 </div>
 {#if unread}<div class="chat-new"><Button onclick={bottom}>↓ 新消息</Button></div>{/if}
 {#if chat.error}<ErrorCard message={chat.error} onretry={()=>{chat.error='';const row=messages.filter(m=>m.type==='exec'&&!m.receipt&&m.state==='failed').at(-1);if(row)void retryChat(row);}} onskip={()=>chat.error=''}/>{/if}
 <Composer/>
</section>
<style>.chat-panel{display:flex;flex-direction:column;min-width:0;height:100%;min-height:0;color:var(--text);background:var(--bg-elev);font:13px system-ui,sans-serif}.chat-panel>header{display:flex;align-items:center;gap:8px;height:40px;min-height:40px;padding:0 12px;border-bottom:1px solid var(--border)}.chat-panel select{flex:1;min-width:0;font-size:12px;color:var(--text);background:var(--bg-elev);border:0;padding:6px;border-radius:6px}.chat-panel select:focus-visible{outline:2px solid var(--focus)}.chat-messages{flex:1;min-height:0;overflow:auto;overscroll-behavior:contain;padding:16px;scrollbar-gutter:stable}.chat-message{margin-bottom:16px}.chat-message:empty{display:none}.chat-empty{display:flex;flex-direction:column;gap:8px;color:var(--text-dim);justify-content:center;min-height:160px;line-height:1.8}.chat-empty p{font-size:13px;margin:0}.chat-empty span{font-size:12px}.chat-new{display:flex;justify-content:center;padding:4px}.chat-panel>header :global(button){font-size:12px}</style>
