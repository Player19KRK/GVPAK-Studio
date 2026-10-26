import {mount} from 'svelte';
import '@fontsource/barlow-condensed/600.css';
import '@fontsource/barlow-condensed/700.css';
import Login from './components/Login.svelte';
import './style.css';
async function enter(user){if(user){window.__gvpakTeamUser=user;document.body.classList.add('team-client');}const {default:App}=await import('./App.svelte');mount(App,{target:document.getElementById('app')});}
async function boot(){try{const res=await fetch('/api/team/session',{cache:'no-store'});if(!res.ok)throw Error('Unable to reach the service');const session=await res.json();if(session.team&&!session.user)mount(Login,{target:document.getElementById('app')});else await enter(session.user);}catch(e){document.getElementById('app').textContent='无法连接 GVPAK Studio。请刷新页面；若仍失败，请联系管理员。';}


}
boot();
