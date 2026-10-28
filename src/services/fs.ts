import {withStore} from './idb.ts';
import {fileNameOnly} from '../lib/target.ts';
type DirectoryHandle = FileSystemDirectoryHandle & { queryPermission(options: {mode: 'readwrite'}): Promise<PermissionState>; requestPermission(options: {mode: 'readwrite'}): Promise<PermissionState> };
type PickerWindow = Window & { showDirectoryPicker(options: { mode: 'readwrite' }): Promise<DirectoryHandle> };
export function createFileService(scope: string, storage = {
  get: () => withStore<DirectoryHandle>('gvpak-project-folders', 'handles', 'readonly', store => store.get(scope)),
  put: (handle: DirectoryHandle) => withStore('gvpak-project-folders', 'handles', 'readwrite', store => store.put(handle, scope)),
}) {
  let handle: DirectoryHandle | null = null,revision=0;
  async function permission(request = false): Promise<PermissionState> {
    if (!handle) return 'prompt';
    const active=handle;const state = await active.queryPermission({ mode: 'readwrite' }).catch(() => 'prompt' as PermissionState);if(handle!==active)return permission(false);
    return request && state !== 'granted' ? handle.requestPermission({ mode: 'readwrite' }).catch(() => 'denied' as PermissionState) : state;
  }
  return {
    get handle() { return handle; }, permission,
    async restore() { const token=revision,stored=await storage.get().catch(()=>null);if(token===revision)handle=stored;return permission(); },
    async attach(selected:DirectoryHandle) { revision++;handle=selected;await storage.put(selected);return permission(); },
    async choose() {
      const picker = (window as PickerWindow).showDirectoryPicker;
      if (!picker) throw Error('请使用支持文件夹授权的 Chrome 或 Edge。');
      const selected = await picker.call(window, { mode: 'readwrite' });revision++;handle = selected;await storage.put(selected);return permission();
    },
    async ensureStructure() {
      if (!handle || await permission() !== 'granted') throw Error('请先重新授权文件夹。');
      const root = await handle.getDirectoryHandle('.gvpak', { create: true });
      for (const name of ['versions', 'trash', 'cache']) await root.getDirectoryHandle(name, { create: true });
    },
    async writeFile(directory: FileSystemDirectoryHandle, requestedName: string, blob: Blob) {
      if (!handle || await permission() !== 'granted') throw Error('文件夹授权已失效，请重新授权。');
      const original = fileNameOnly(requestedName);if (!original || original === '.' || original === '..') throw Error('文件名无效。');
      const dot = original.lastIndexOf('.'), stem = dot > 0 ? original.slice(0, dot) : original, extension = dot > 0 ? original.slice(dot) : '';
      let name = original;
      for (let index = 2; ; index++) {
        try { await directory.getFileHandle(name);name = `${stem} (${index})${extension}`; }
        catch (error) { if ((error as DOMException).name === 'NotFoundError') break;throw error; }
      }
      const file = await directory.getFileHandle(name, { create: true }), writable = await file.createWritable();
      try { await writable.write(blob);await writable.close(); }
      catch (error) { await writable.abort().catch(() => {});throw error; }
      return { name, handle: file };
    },
  };
}
