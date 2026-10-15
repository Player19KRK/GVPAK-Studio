import {stored,persist} from './api';
export const layout=$state({inspectorWidth:stored('inspector-width',280),coverflowHeight:stored('coverflow-height',144),coverflowCollapsed:stored('coverflow-collapsed',false),focus:false,viewportWidth:typeof innerWidth==='number'?innerWidth:1280,viewportHeight:typeof innerHeight==='number'?innerHeight:800});
export const inspectorMax=()=>Math.max(220,Math.min(480,layout.viewportWidth<=900?layout.viewportWidth-64:layout.viewportWidth*.45));
export const coverflowMax=()=>Math.max(100,Math.min(360,layout.viewportHeight-(layout.viewportWidth<=650?76:48)-22-220));
export function saveLayout(){persist('inspector-width',layout.inspectorWidth);persist('coverflow-height',layout.coverflowHeight);persist('coverflow-collapsed',layout.coverflowCollapsed);}
export function updateViewport(){layout.viewportWidth=innerWidth;layout.viewportHeight=innerHeight;}
export function resetLayout(){layout.inspectorWidth=280;layout.coverflowHeight=144;layout.coverflowCollapsed=false;layout.focus=false;saveLayout();}
