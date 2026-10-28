import {studio} from '../studio.svelte';
import {assets} from './assets.svelte';
import {routing} from '../app/shell.svelte';
import {createSelectionContext,uniqueIds} from '../lib/target.ts';
const state = $state({ mentions: [], lastAnchorId: null, context: null });
function snapshot() {
  return createSelectionContext({
    page: ['home', 'library', 'editor'].includes(routing.current.page) ? routing.current.page : 'home',
    mentions: state.mentions, selected: studio.selected, editorAssetId: studio.aid || null,
    filteredIds: assets.filteredIds, filter: assets.filter,
    names: Object.fromEntries(Object.entries(assets.index).map(([id, asset]) => [id, asset.name])),
  });
}
export const selection = {
  get selectedIds() { return studio.selected; },
  get mentions() { return state.mentions; },
  get lastAnchorId() { return state.lastAnchorId; },
  get context() { return state.context; }, snapshot,
  set(ids, anchor = ids.at(-1) || null) { const available=assets.availableIds;studio.selected = uniqueIds(ids).filter(id => available.has(id));state.lastAnchorId = anchor;studio.batchScope = 'auto'; },
  toggle(id) { selection.set(studio.selected.includes(id) ? studio.selected.filter(value => value !== id) : [...studio.selected, id], id); },
  clear() { selection.set([], null); },
  mention(ids) { const available=assets.availableIds;state.mentions = uniqueIds(ids).filter(id => available.has(id)); },
  range(id, append = false) {
    const ids = assets.filteredIds, from = ids.indexOf(state.lastAnchorId), to = ids.indexOf(id);
    if (to < 0) return;
    if (from < 0) { selection.set([id], id);return; }
    selection.set([...(append ? studio.selected : []), ...ids.slice(Math.min(from, to), Math.max(from, to) + 1)], state.lastAnchorId);
  },
};
export function startSelectionContext() {
  let frame = 0, latest, projectId = studio.pid;
  const stop = $effect.root(() => {
    $effect(() => { if (projectId !== studio.pid) { projectId = studio.pid;state.mentions = [];state.lastAnchorId = null; }latest = snapshot();if (!frame) frame = requestAnimationFrame(() => { frame = 0;state.context = latest; }); });
  });
  return () => { stop();cancelAnimationFrame(frame); };
}
