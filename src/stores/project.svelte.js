import {isMock} from '../api';
import {studio,selectProject} from '../studio.svelte';
import {createFileService} from '../services/fs.ts';
const directory = $state({ handle: null, permission: 'prompt', recentIds: [] });
let files = null;
export const project = {
  get currentId() { return studio.pid; },
  get items() { return studio.projects; },
  get directoryHandle() { return directory.handle; },
  get permission() { return directory.permission; },
  get recentIds() { return directory.recentIds; },
  select(id) { if (!studio.projects.some(item => item.id === id)) return;selectProject(id);files = null;directory.handle = null;directory.permission = 'prompt';directory.recentIds = [id, ...directory.recentIds.filter(value => value !== id)].slice(0, 20); },
  async restoreDirectory() {if(isMock){directory.handle=null;directory.permission='prompt';return;}
    const id = studio.pid;
    const active = createFileService(`${window.__gvpakTeamUser?.id || 'host'}:${id}`);files = active;
    const permission = await active.restore();
    if (studio.pid === id && files === active) { directory.handle = active.handle;directory.permission = permission; }
  },
  async attachDirectory(handle) {if(isMock)return;
    const id=studio.pid;files ||= createFileService(`${window.__gvpakTeamUser?.id||'host'}:${id}`);const active=files;const permission=await active.attach(handle);if(studio.pid!==id||files!==active)return;directory.handle=active.handle;directory.permission=permission;if(permission==='granted')await active.ensureStructure();
  },
  async checkPermission() { if (!files || !directory.handle) return;const id=studio.pid,active=files;const permission=await active.permission();if(studio.pid===id&&files===active)directory.permission=permission; },
  async authorizeDirectory({chooseNew=false}={}) {if(isMock)throw Error('模拟模式不授权或写入真实输出目录。');
    const id = studio.pid;
    files ||= createFileService(`${window.__gvpakTeamUser?.id || 'host'}:${id}`);
    const active = files;
    const permission = !chooseNew&&active.handle ? await active.permission(true) : await active.choose();
    if (studio.pid !== id) return;
    directory.handle = active.handle;directory.permission = permission;
    if (permission === 'granted') await active.ensureStructure();
  },
};
