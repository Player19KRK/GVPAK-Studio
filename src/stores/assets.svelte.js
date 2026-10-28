import {studio} from '../studio.svelte';
import {shell,saveShell} from '../app/shell.svelte';
const index = $derived.by(() => Object.fromEntries(studio.projects.flatMap(project => project.assets.map(asset => [asset.id, asset]))));
const available = $derived.by(() => new Set((studio.projects.find(project => project.id === studio.pid)?.assets || []).filter(asset => !asset.trashed).map(asset => asset.id)));
const filtered = $derived.by(() => {
  const filter = shell.filter;
  const values = (studio.projects.find(project => project.id === studio.pid)?.assets || []).filter(asset =>
    !asset.trashed && (!filter.folder || (asset.folderId === filter.folder || asset.folderId?.startsWith(filter.folder+'/'))) &&
    (!filter.tag || asset.tags?.includes(filter.tag)) && (!filter.status || (filter.status==='unanalyzed'?asset.descriptionStatus!=='done'&&!asset.description:(asset.processingStatus??'idle') === filter.status)) &&
    (!filter.q || asset.name.toLocaleLowerCase().includes(filter.q.toLocaleLowerCase())));
  if (filter.sort === 'name') values.sort((a, b) => a.name.localeCompare(b.name, 'zh-CN', { numeric: true }));
  else if (filter.sort === 'name-desc') values.sort((a, b) => b.name.localeCompare(a.name, 'zh-CN', { numeric: true }));
  return values.map(asset => asset.id);
});
export const assets = {
  get index() { return index; }, get filteredIds() { return filtered; },
  get availableIds() { return available; },
  get filter() { return shell.filter; },
  setFilter(patch) { shell.filter = { ...shell.filter, ...patch };saveShell(); },
  update(id, patch, projectId=studio.pid) { let asset = index[id];if(!asset&&patch.name&&patch.sha256&&patch.edit&&Array.isArray(patch.versions)){const project=studio.projects.find(project=>project.id===projectId);if(project){project.assets.push({...patch,id});asset=project.assets.at(-1);}}if (asset) { Object.assign(asset, patch, { id });if (patch.versions) {asset.savedVersionCount=patch.versions.length;asset.versionCount=patch.versions.length+1;} } },
};
