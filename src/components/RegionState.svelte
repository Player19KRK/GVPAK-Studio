<script>
 import Button from '../ui/Button.svelte';
 let {state='loading',title='',message='',onretry=null,compact=false}=$props();
</script>
<div class="region-state" class:compact data-state={state} role={state==='error'?'alert':'status'} aria-live="polite" aria-busy={state==='loading'}>
 {#if title}<h2>{title}</h2>{/if}
 {#if state==='loading'}<div class="region-skeleton" aria-hidden="true"><span></span><span></span><span></span></div>{/if}
 {#if message}<p>{message}</p>{/if}
 {#if state==='error'&&onretry}<Button onclick={onretry}>重试</Button>{/if}
</div>
<style>.region-state{box-sizing:border-box;max-width:640px;margin:48px auto;padding:24px;color:var(--text);font:13px system-ui,sans-serif;display:flex;flex-direction:column;align-items:flex-start;gap:16px}.region-state h2{font-size:14px;line-height:1.5;margin:0;font-weight:600}.region-state p{font-size:13px;line-height:1.7;margin:0;color:var(--text-dim);overflow-wrap:anywhere}.region-state[data-state=error] p{color:var(--danger)}.region-state.compact{margin:0;padding:12px 0;gap:12px;max-width:none}.region-skeleton{width:min(400px,100%);display:grid;gap:12px}.region-skeleton span{display:block;height:12px;min-width:100px;border-radius:4px;background:var(--bg-sunken)}.region-skeleton span:nth-child(2){width:75%}.region-skeleton span:nth-child(3){width:48%}</style>
