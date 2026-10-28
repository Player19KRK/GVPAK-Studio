export const ratios = ['1:1', '3:2', '4:3', '4:5', '16:9'] as const;
export type Ratio = typeof ratios[number];
export type Orientation = 'landscape' | 'portrait';
export type RatioState = { ratio: Ratio; orientation: Orientation };
function requireRatio(ratio: Ratio) { if (!ratios.includes(ratio)) throw Error('不支持这个比例'); }
export function defaultOrientation(ratio: Ratio): Orientation { requireRatio(ratio); return ratio === '4:5' ? 'portrait' : 'landscape'; }
export function chooseRatio(ratio: Ratio): RatioState { return { ratio, orientation: defaultOrientation(ratio) }; }
export function ratioParts(ratio: Ratio, orientation = defaultOrientation(ratio)): [number, number] {
  requireRatio(ratio);
  if (!['landscape', 'portrait'].includes(orientation)) throw Error('方向无效');
  const [a, b] = ratio.split(':').map(Number);
  return orientation === 'portrait' ? [Math.min(a, b), Math.max(a, b)] : [Math.max(a, b), Math.min(a, b)];
}
export function ratioLabel(state: RatioState) { return ratioParts(state.ratio, state.orientation).join(':'); }
export function swapRatio(state: RatioState): RatioState {
  requireRatio(state.ratio);
  return state.ratio === '1:1' ? { ...state } : { ratio: state.ratio, orientation: state.orientation === 'landscape' ? 'portrait' : 'landscape' };
}
export function ratioOutputSize(width: number, height: number, state: RatioState, longEdge?: number) {
  if (![width, height].every(n => Number.isInteger(n) && n > 0)) throw Error('图片尺寸无效');
  const edge = longEdge ?? Math.max(width, height);
  if (!Number.isInteger(edge) || edge < 1 || edge > 16000) throw Error('长边必须为1至16000像素');
  const [a, b] = ratioParts(state.ratio, state.orientation);
  return a >= b ? { width: edge, height: Math.max(1, Math.round(edge * b / a)) } : { width: Math.max(1, Math.round(edge * a / b)), height: edge };
}
