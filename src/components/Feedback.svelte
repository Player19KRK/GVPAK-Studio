<script>
 import {feedbackRequest} from '../services/account.ts';
 import {keyboard} from '../lib/shortcuts.ts';
 import {onDestroy} from 'svelte';import {ShieldCheck,X,ImagePlus} from 'lucide-svelte';import {isTeam} from '../api';
 let open=$state(false),items=$state([]),description=$state(''),busy=$state(false),message=$state(''),x=$state(24),y=$state(90);let drag=null,moved=false;let picker=$state();let hovering=$state(false);
 function add(files){for(const file of files){if(!file.type.startsWith('image/'))continue;if(file.size>12*1024*1024){message='单张截图不能超过 12 MB。';continue;}items.push({file,url:URL.createObjectURL(file)});}}
 function remove(i){URL.revokeObjectURL(items[i].url);items.splice(i,1);}
 function down(e){if(e.button!==0)return;drag={x:e.clientX,y:e.clientY,startX:x,startY:y};moved=false;e.currentTarget.setPointerCapture(e.pointerId);}
 function move(e){if(!drag)return;if(Math.hypot(e.clientX-drag.x,e.clientY-drag.y)>5)moved=true;x=Math.max(8,Math.min(innerWidth-60,drag.startX-(e.clientX-drag.x)));y=Math.max(32,Math.min(innerHeight-60,drag.startY-(e.clientY-drag.y)));}
 async function submit(){busy=true;message='';try{const images=[];for(const item of items){const data=await new Promise((resolve,reject)=>{const r=new FileReader();r.onload=()=>resolve(r.result.split(',')[1]);r.onerror=reject;r.readAsDataURL(item.file);});images.push({data});}const r=await feedbackRequest(isTeam,{description,images});if(!r.ok)throw Error('提交失败。请减少截图大小后重试。');const result=await r.json();message='已提交。问题编号：'+result.id;items.forEach(i=>URL.revokeObjectURL(i.url));items=[];description='';}catch(e){message=e.message;}finally{busy=false;}}
 onDestroy(()=>items.forEach(i=>URL.revokeObjectURL(i.url)));
</script>
<button class="feedback-fab" style:right={`${x}px`} style:bottom={`${y}px`} aria-label="提交问题（可拖动）" title="提交问题 · 按住可拖动" onpointerdown={down} onpointermove={move} onpointerup={()=>drag=null} onpointercancel={()=>drag=null} onclick={()=>{if(!moved)open=!open;}}><ShieldCheck size={23}/></button>
{#if open}
<div class="feedback-board" class:dragging={hovering} role="dialog" tabindex="-1" aria-label="提交问题"
 use:keyboard={{keydown: e=>{if(e.key==='Escape')open=false;}}}
 onpaste={e=>{const files=[...(e.clipboardData?.files||[])];if(files.length){e.preventDefault();if(!busy)add(files);}}}
 ondragover={e=>{e.preventDefault();hovering=true;}}
 ondragleave={e=>{if(!e.currentTarget.contains(e.relatedTarget))hovering=false;}}
 ondrop={e=>{e.preventDefault();hovering=false;if(!busy)add([...e.dataTransfer.files]);}}>
 <header><h2>提交问题</h2><button class="close-board" aria-label="关闭问题面板" title="关闭问题面板 · Esc" onclick={()=>open=false}><X size={22}/></button></header>
 <div class="feedback-content">
 {#if items.length}<div class="attachment-heading"><span>图片 · {items.length}</span><span>点击 × 删除</span></div><div class="attachment-previews">{#each items as item,i (item.url)}<figure><img src={item.url} alt={`截图 ${i+1}`}/><button disabled={busy} aria-label={`删除截图 ${i+1}`} title="删除图片" onclick={()=>remove(i)}><X size={16}/></button></figure>{/each}</div>{/if}
 <div class="feedback-compose">
 <textarea aria-label="问题描述" bind:value={description} disabled={busy} maxlength="8000" placeholder="你进行了什么操作？
出现了什么问题？"></textarea>
 <div class="compose-tools"><button disabled={busy} onclick={()=>picker.click()}><ImagePlus size={19}/><span>添加图片</span></button><span>拖入图片 · Ctrl + V 粘贴</span></div>
 </div>
 <input class="attachment-input" bind:this={picker} type="file" accept="image/*" multiple onchange={e=>{add([...e.target.files]);e.target.value='';}}/>
 {#if message}<p class="feedback-message" role="status">{message}</p>{/if}
 </div>
 <footer><span>请勿提交密码或密钥</span><button class="primary" disabled={busy||!description.trim()} onclick={submit}>{busy?'正在提交…':'提交问题'}</button></footer>
 {#if hovering}<div class="drop-indicator"><ImagePlus size={32}/><span>松开鼠标，添加图片</span></div>{/if}
</div>{/if}
<style>
.feedback-board{--paper:var(--bg-elev);--line:var(--border);--red:var(--accent);--ink:var(--text);color-scheme:light dark;position:fixed;right:24px;bottom:82px;width:min(760px,calc(100vw - 32px));max-height:calc(100dvh - 112px);display:flex;flex-direction:column;background:var(--paper);border:1px solid var(--line);box-shadow:0 18px 55px #17251c30;z-index:111;animation:board-in .12s ease-out;color:var(--ink)}
header{display:flex;align-items:center;justify-content:space-between;padding:24px 28px 20px;border-bottom:1px solid var(--line)}h2{font-size:24px;font-weight:600;margin:0}.close-board{width:36px;height:36px;display:grid;place-items:center}.close-board:hover{background:var(--bg-sunken)}
.feedback-content{padding:24px 28px;overflow:auto;min-height:0}.feedback-compose{background:var(--bg-elev);border:1px solid var(--line)}.feedback-compose:focus-within{border-color:var(--focus)}textarea{display:block;width:100%;min-height:240px;border:0!important;background:transparent;padding:20px!important;resize:vertical;box-sizing:border-box;font:inherit;font-size:15px;line-height:1.8;outline:none;color:inherit}textarea::placeholder{color:var(--text-dim);opacity:1}.compose-tools{display:flex;align-items:center;justify-content:space-between;padding:12px 16px;gap:12px;border-top:1px solid var(--border)}.compose-tools button{display:flex;align-items:center;gap:8px;font-size:13px;padding:6px}.compose-tools button:hover{color:var(--red)}.compose-tools>span,footer>span,.attachment-heading{font-size:12px;color:var(--text-dim)}.attachment-input{display:none!important}
.attachment-heading{display:flex;justify-content:space-between;margin-bottom:12px}.attachment-previews{display:grid;grid-template-columns:repeat(auto-fill,minmax(130px,1fr));gap:12px;margin-bottom:20px;max-height:240px;overflow:auto}figure{position:relative;margin:0;background:var(--bg-sunken);border:1px solid var(--line);height:116px}figure img{width:100%;height:100%;object-fit:contain}figure button{position:absolute;top:5px;right:5px;width:28px;height:28px;display:grid;place-items:center;background:var(--paper);border:1px solid var(--line)}figure button:hover{background:var(--red);color:white}footer{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:16px 28px;border-top:1px solid var(--line)}footer>span{color:var(--red)}footer .primary{min-width:132px;justify-content:center}.feedback-message{font-size:13px;overflow-wrap:anywhere;margin:16px 0 0}.drop-indicator{position:absolute;inset:10px;border:2px dashed var(--red);background:var(--bg-elev);display:flex;flex-direction:column;gap:16px;align-items:center;justify-content:center;color:var(--red);pointer-events:none}button:focus-visible{outline:2px solid var(--red);outline-offset:3px}button:disabled{opacity:.45;cursor:default}@keyframes board-in{from{transform:translateY(12px);opacity:.5}to{transform:translateY(0);opacity:1}}@media(max-width:600px){.feedback-board{right:16px;bottom:16px;max-height:calc(100dvh - 32px)}header,.feedback-content{padding:18px}footer{padding:14px 18px}.compose-tools>span{display:none}textarea{min-height:200px}}@media(prefers-reduced-motion:reduce){.feedback-board{animation:none}}
</style>
