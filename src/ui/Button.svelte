<script>
  import Spinner from './Spinner.svelte';
  let { variant = 'default', busy = false, disabled = false, children, class: className = '', type = 'button', ...props } = $props();
</script>
<button {...props} {type} class={`ui-control ui-button ${className}`} data-variant={variant} disabled={disabled || busy} aria-busy={busy || undefined}>
  {#if busy}<Spinner label="正在处理" />{/if}
  {@render children?.()}
</button>
