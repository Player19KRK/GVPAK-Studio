import {fetchResponse} from './http.ts';
const post=(url:string,value:any)=>fetchResponse(url,{method:'POST',headers:{'Content-Type':'application/json','X-GVPAK-IPC':'1'},body:JSON.stringify(value)});
export const loginRequest=(input:any)=>post('/api/team/login',input);
export const logoutRequest=()=>fetchResponse('/api/team/logout',{method:'POST',headers:{'X-GVPAK-IPC':'1'}});
export const downloadsRequest=()=>fetchResponse('/api/files',{cache:'no-store'});
export const feedbackRequest=(team:boolean,input:any)=>post(team?'/api/feedback':'/api/invoke/submit_feedback',input);
