<script>
  let { tone = 'info', children, actions } = $props();
</script>
<div class="ui-banner" data-tone={tone} role={tone === 'danger' ? 'alert' : 'status'}>
  <div class="ui-banner-content">{@render children?.()}</div>
  {@render actions?.()}
</div>
