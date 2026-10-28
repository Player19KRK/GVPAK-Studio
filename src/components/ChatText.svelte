<script>
 let {content='',role='assistant'}=$props();
 // Text is always escaped by Svelte; only a small native Markdown vocabulary is rendered.
 const parts=text=>text.split(/(\*\*[^*]+\*\*|`[^`]+`|\[[^\]]+\]\(https?:\/\/[^ )]+\))/g);
</script>
<div class="chat-text" class:user={role==='user'}>{#each content.split('\n') as line}{#if line.startsWith('### ')}<h3>{line.slice(4)}</h3>{:else if line.startsWith('## ')}<h3>{line.slice(3)}</h3>{:else if line.startsWith('# ')}<h3>{line.slice(2)}</h3>{:else}<p>{#each parts(line) as part}{#if part.startsWith('**')&&part.endsWith('**')}<strong>{part.slice(2,-2)}</strong>{:else if part.startsWith('`')&&part.endsWith('`')}<code>{part.slice(1,-1)}</code>{:else if /^\[[^\]]+\]\(https?:\/\//.test(part)}<a href={part.slice(part.indexOf('](')+2,-1)} target="_blank" rel="noreferrer">{part.slice(1,part.indexOf(']('))}</a>{:else}{part}{/if}{/each}</p>{/if}{/each}</div>
<style>.chat-text{font-size:13px;line-height:1.8;overflow-wrap:anywhere}.chat-text p{margin:0 0 6px;white-space:pre-wrap}.chat-text h3{font-size:14px;margin:8px 0}.chat-text code{background:var(--bg-sunken);padding:2px 4px;border-radius:4px;font-size:12px}.chat-text a{color:var(--accent)}.chat-text.user{padding:10px 12px;background:var(--bg-sunken);border-radius:8px}.chat-text.user p:last-child{margin-bottom:0}</style>
