<script>
  import { keyboard } from '../lib/shortcuts.ts';
  let { label, value = $bindable(), min, max, initial, axis = 'width', direction = 1, onchange = () => {} } = $props();
  let drag = null;
  const clamp = number => Math.min(max, Math.max(min, Math.round(number)));
  function set(number) { value = clamp(number); onchange(value); }
  function down(event) {
    if (event.button !== 0) return;
    event.preventDefault(); event.currentTarget.focus();
    drag = { point: axis === 'width' ? event.clientX : event.clientY, value, id: event.pointerId };
    event.currentTarget.setPointerCapture(event.pointerId);
  }
  function move(event) {
    if (!drag || drag.id !== event.pointerId) return;
    set(drag.value + ((axis === 'width' ? event.clientX : event.clientY) - drag.point) * direction);
  }
  function keys(event) {
    const before = axis === 'width' ? 'ArrowLeft' : 'ArrowUp';
    const after = axis === 'width' ? 'ArrowRight' : 'ArrowDown';
    if (![before, after, 'Home', 'End'].includes(event.key)) return;
    event.preventDefault(); event.stopPropagation();
    set(event.key === 'Home' ? min : event.key === 'End' ? max : value + (event.key === after ? 1 : -1) * direction * (event.shiftKey ? 40 : 10));
  }
</script>
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (A keyboard-operable resize separator.) -->
<div class="ui-control ui-resizer" data-axis={axis} role="separator" tabindex="0" aria-label={label} aria-orientation={axis === 'width' ? 'vertical' : 'horizontal'} aria-valuemin={min} aria-valuemax={max} aria-valuenow={value} aria-valuetext={`${value} 像素`} use:keyboard={{ keydown: keys }} onpointerdown={down} onpointermove={move} onpointerup={() => drag = null} onpointercancel={() => drag = null} onlostpointercapture={() => drag = null} ondblclick={() => set(initial)} title="拖动调整，方向键微调，双击恢复默认值"></div>
