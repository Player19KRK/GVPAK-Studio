import { filterKeys } from './preferences.ts';
export function parseRoute(input: string) {
  const url = new URL(input, 'http://localhost');
  const path = url.pathname.replace(/\/+$/, '') || '/';
  const filter = Object.fromEntries(filterKeys.map(key => [key, url.searchParams.get(key) || '']));
  if (path === '/') return { page: 'home', assetId: null, filter };
  if (path === '/library') return { page: 'library', assetId: null, filter };
  if (path === '/generate') return { page: 'generate', assetId: null, filter };
  if (path === '/settings') return { page: 'settings', assetId: null, filter };
  const editor = path.match(/^\/editor\/([^/]+)$/);
  if (editor) {
    try { return { page: 'editor', assetId: decodeURIComponent(editor[1]), filter }; }
    catch { /* A malformed address is an unknown page. */ }
  }
  return { page: 'not-found', assetId: null, filter };
}
export function libraryUrl(filter: Record<string, string>) {
  const params = new URLSearchParams();
  for (const key of filterKeys) if (filter[key]) params.set(key, filter[key]);
  const query = params.toString();
  return '/library' + (query ? '?' + query : '');
}
export const editorUrl = (assetId: string) => '/editor/' + encodeURIComponent(assetId);
export function canOpenSettings(session: { team?: boolean }, hostname: string) {
  return session.team === false && ['localhost', '127.0.0.1', '[::1]'].includes(hostname);
}
