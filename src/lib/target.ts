export type Filter = { folder: string; tag: string; status: string; q: string; sort: string };
export type TargetInput = { mentions: readonly string[]; selected: readonly string[]; editorAssetId: string | null; filteredIds: readonly string[] };
export type SelectionContext = {
  page: 'home' | 'library' | 'editor'; mentions: string[]; selected: string[];
  editorAssetId: string | null; filter: Filter; filteredCount: number;
  resolvedTarget: { source: 'mention' | 'selected' | 'editor' | 'filter'; ids: string[] };
  fileNames: string[];
};
export const uniqueIds = (ids: readonly string[]) => [...new Set(ids.filter(id => typeof id === 'string' && id.length > 0))];
export function resolveTarget(input: TargetInput): SelectionContext['resolvedTarget'] {
  const mentions = uniqueIds(input.mentions), selected = uniqueIds(input.selected);
  if (mentions.length) return { source: 'mention', ids: mentions };
  if (selected.length) return { source: 'selected', ids: selected };
  if (input.editorAssetId) return { source: 'editor', ids: [input.editorAssetId] };
  return { source: 'filter', ids: uniqueIds(input.filteredIds) };
}
export function fileNameOnly(name: string) {
  return String(name).split(/[\\/]/).at(-1)?.replace(/^[a-z]:/i, '') || '';
}
export function createSelectionContext(input: TargetInput & {
  page: SelectionContext['page']; filter: Filter; names: Readonly<Record<string, string>>;
}): SelectionContext {
  const editorAssetId = input.page === 'editor' ? input.editorAssetId : null;
  const target = resolveTarget({ ...input, editorAssetId });
  return {
    page: input.page, mentions: uniqueIds(input.mentions), selected: uniqueIds(input.selected), editorAssetId,
    filter: { folder: input.filter.folder, tag: input.filter.tag, status: input.filter.status, q: input.filter.q, sort: input.filter.sort },
    filteredCount: uniqueIds(input.filteredIds).length, resolvedTarget: target,
    fileNames: target.ids.map(id => fileNameOnly(input.names[id] || '')),
  };
}
// Explicit field selection prevents a store object or sourcePath from leaking into a request.
export function serializeSelectionContext(context: SelectionContext): SelectionContext {
  return {
    page: context.page, mentions: uniqueIds(context.mentions), selected: uniqueIds(context.selected),
    editorAssetId: context.editorAssetId, filter: {
      folder: context.filter.folder, tag: context.filter.tag, status: context.filter.status,
      q: context.filter.q, sort: context.filter.sort,
    }, filteredCount: context.filteredCount,
    resolvedTarget: { source: context.resolvedTarget.source, ids: uniqueIds(context.resolvedTarget.ids) },
    fileNames: context.fileNames.map(fileNameOnly),
  };
}
