import { readPreferences, writePreferences } from '../lib/preferences.ts';
import { parseRoute, libraryUrl } from '../lib/router.ts';
const key = `gvpak-ui:${typeof window !== 'undefined' ? window.__gvpakTeamUser?.id || 'host' : 'host'}`;
const storage = typeof localStorage === 'undefined' ? { getItem: () => null, setItem: () => {} } : localStorage;
export const shell = $state(readPreferences(storage, key));
export const routing = $state({ current: parseRoute(typeof location === 'undefined' ? '/' : location.href) });
export function saveShell() { return writePreferences(storage, key, shell); }
export function navigate(url, { replace = false } = {}) {
  if (!url.startsWith('/') || url.startsWith('//')) return;
  const next = parseRoute(url);
  if (next.page === 'settings' && !window.__gvpakLocalHost) url = '/';
  if (location.pathname + location.search !== url) history[replace ? 'replaceState' : 'pushState']({}, '', url);
  sync();
}
function sync() {
  const next = parseRoute(location.href);
  if (next.page === 'settings' && !window.__gvpakLocalHost) { navigate('/', { replace: true }); return; }
  routing.current = next;
  if (next.page === 'library') { shell.filter = next.filter; saveShell(); }
}
export function startRouting() {
  if (location.pathname === '/library' && !location.search) navigate(libraryUrl(shell.filter), { replace: true });
  else sync();
  window.addEventListener('popstate', sync);
  return () => window.removeEventListener('popstate', sync);
}
