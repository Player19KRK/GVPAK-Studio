<script>
  let { label = '正在加载' } = $props();
</script>
<span class="ui-spinner" role="status"><svg viewBox="0 0 16 16" fill="none" aria-hidden="true"><circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="2" opacity=".2" /><path d="M8 2a6 6 0 0 1 6 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" /></svg><span class="ui-sr-only">{label}</span></span>
