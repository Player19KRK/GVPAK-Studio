export const defaultPreferences = {
  projectId: '', chatWidth: 400, chatCollapsed: false, treeWidth: 220, gridSize: 160, view: 'grid',
  filter: { folder: '', tag: '', status: '', q: '', sort: '' },
};
export const filterKeys = ['folder', 'tag', 'status', 'q', 'sort'] as const;
export function normalizePreferences(value: unknown) {
  const input = value && typeof value === 'object' ? value as Record<string, unknown> : {};
  const clamp = (n: unknown, fallback: number, min: number, max: number) =>
    typeof n === 'number' && Number.isFinite(n) ? Math.min(max, Math.max(min, Math.round(n))) : fallback;
  const rawFilter = input.filter && typeof input.filter === 'object' ? input.filter as Record<string, unknown> : {};
  return {
    projectId: typeof input.projectId === 'string' ? input.projectId : '',
    chatWidth: clamp(input.chatWidth, 400, 360, 520),
    chatCollapsed: input.chatCollapsed === true,
    treeWidth: clamp(input.treeWidth, 220, 180, 320),
    gridSize: [120, 160, 200].includes(input.gridSize as number) ? input.gridSize : 160,
    view: input.view === 'list' ? 'list' : 'grid',
    filter: Object.fromEntries(filterKeys.map(key => [key, typeof rawFilter[key] === 'string' ? rawFilter[key] : ''])),
  };
}
export function readPreferences(storage: Pick<Storage, 'getItem'>, key: string) {
  try { return normalizePreferences(JSON.parse(storage.getItem(key) || 'null')); }
  catch { return normalizePreferences(null); }
}
export function writePreferences(storage: Pick<Storage, 'setItem'>, key: string, value: unknown) {
  try { storage.setItem(key, JSON.stringify(normalizePreferences(value))); return true; }
  catch { return false; }
}
