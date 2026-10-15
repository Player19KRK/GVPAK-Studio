export const EDIT_DOUBLE_CLICK_MS = 200;
export function isEditDoubleClick(previous, next) {
 return !!previous && next.detail === 2 && previous.detail === 1 &&
  previous.id === next.id && next.time >= previous.time &&
  next.time - previous.time <= EDIT_DOUBLE_CLICK_MS &&
  Math.hypot(next.x - previous.x, next.y - previous.y) <= 6;
}
