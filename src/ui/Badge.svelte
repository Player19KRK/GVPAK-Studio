<script>
  let { tone = 'neutral', children } = $props();
</script>
<span class="ui-badge" data-tone={tone}>{@render children?.()}</span>
