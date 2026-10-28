import {shell,saveShell} from '../app/shell.svelte';
export const prefs = {
  get chatWidth() { return shell.chatWidth; }, set chatWidth(value) { shell.chatWidth = Math.max(360, Math.min(520, value));saveShell(); },
  get chatCollapsed() { return shell.chatCollapsed; }, set chatCollapsed(value) { shell.chatCollapsed = value;saveShell(); },
  get treeWidth() { return shell.treeWidth; }, set treeWidth(value) { shell.treeWidth = Math.max(180, Math.min(320, value));saveShell(); },
  get gridSize() { return shell.gridSize; }, set gridSize(value) { if ([120, 160, 200].includes(value)) { shell.gridSize = value;saveShell(); } },
  get view() { return shell.view; }, set view(value) { if (['grid', 'list'].includes(value)) { shell.view = value;saveShell(); } },
};
