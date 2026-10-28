<script>import Button from '../ui/Button.svelte';let {message,oncontinue=()=>{},oncancel=()=>{}}=$props();</script>
<div class="confirm-row" role="status"><span>{message}</span><Button variant="primary" onclick={oncontinue}>继续</Button><Button onclick={oncancel}>取消</Button></div>
<style>.confirm-row{display:flex;align-items:center;gap:8px;padding:8px;background:var(--warning);color:var(--warning-text);font-size:12px;border-radius:6px}.confirm-row span{flex:1}</style>
