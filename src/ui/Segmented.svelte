<script>
  let { label, options = [], value, onchange = () => {}, disabled = false } = $props();
</script>
<div class="ui-segmented" role="group" aria-label={label}>
  {#each options as option (option.value)}
    <button type="button" class="ui-control ui-segment" aria-pressed={value === option.value} disabled={disabled || option.disabled} onclick={() => onchange(option.value)}>{option.label}</button>
  {/each}
</div>
