<script>
  import Tooltip from './Tooltip.svelte';
  let { label, shortcut = '', children, class: className = '', ...props } = $props();
</script>
<Tooltip text={label} {shortcut}>
  <button {...props} type="button" class={`ui-control ui-icon-button ${className}`} aria-label={label} title={shortcut ? `${label} · ${shortcut}` : label}>
    {@render children?.()}
  </button>
</Tooltip>
