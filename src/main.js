import {mount} from 'svelte';
import '@fontsource/barlow-condensed/600.css';
import '@fontsource/barlow-condensed/700.css';
import Login from './components/Login.svelte';
import './style.css';
import './styles/tokens.css';
import './styles/base.css';
import './styles/complete-app.css';
import {canOpenSettings} from './lib/router.ts';
import {getApi} from './services/api.ts';
async function enter(user){if(user){window.__gvpakTeamUser=user;document.body.classList.add('team-client');}const {default:App}=import.meta.env.VITE_COMPLETE_APP!=='0'?await import('./components/CompleteStudio.svelte'):await import('./App.svelte');mount(App,{target:document.getElementById('app')});}
async function boot(){let session;try{session=await getApi().readSession();}catch(e){window.__gvpakLocalHost=false;await enter(null);return;}window.__gvpakLocalHost=canOpenSettings(session,location.hostname);if(session.team&&!session.user)mount(Login,{target:document.getElementById('app')});else await enter(session.user);


}
boot();
