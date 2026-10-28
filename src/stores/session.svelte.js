import {canOpenSettings} from '../lib/router.ts';
export const session = $state({ account: null, isHost: false, isLocal: false, quota: null, status: 'loading', error: '' });
export function setSession(value, hostname = location.hostname) {
  session.account = value.account || value.user || null;
  session.isHost = value.isHost ?? value.team === false;
  session.isLocal = value.isLocal ?? canOpenSettings(value, hostname);
  session.quota = value.quota || null;
  session.status = 'ready';session.error = '';
}
