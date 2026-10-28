<script>
  let { text, shortcut = '', children } = $props();
</script>
<span class="ui-tooltip" title={shortcut ? `${text} · ${shortcut}` : text}>
  {@render children?.()}
</span>
