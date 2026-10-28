use std::{collections::HashMap,net::SocketAddr,path::{Path as FsPath,PathBuf},sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,AtomicU64,AtomicUsize,Ordering}},time::{Duration,Instant}};
use axum::{Router,Json,extract::{State,Path,ConnectInfo,DefaultBodyLimit},http::{HeaderMap,StatusCode,header},response::{Response,IntoResponse,Sse,sse::{Event,KeepAlive}},routing::{get,post}};
use serde::{Serialize,Deserialize};
use serde_json::{Value,json};
use sha2::{Sha256,Digest};
use base64::{Engine,engine::general_purpose::{STANDARD,URL_SAFE_NO_PAD}};
use tokio::sync::{Semaphore,broadcast};
use crate::{Shared,AppState,Store,Res,transport::AppHandle};
const COOKIE:&str="gvpak_team";
const ROUNDS:u32=600_000;
#[derive(Clone,Serialize,Deserialize)]
struct Account{id:String,username:String,salt:String,hash:String,enabled:bool}
#[derive(Clone)]
struct Workspace{shared:Shared,hub:AppHandle,root:PathBuf,wait_cancel:Arc<AtomicU64>}
#[derive(Clone,Serialize,Deserialize)]
struct Session{uid:String,expires:u64,remember:bool,account_salt:String}
const SESSION_SECONDS:u64=12*60*60;
const DEVICE_SECONDS:u64=30*24*60*60;
fn session_now()->u64{std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()}
fn load_sessions(root:&FsPath,accounts:&[Account])->HashMap<String,Session>{
 let sessions:HashMap<String,Session>=std::fs::read(root.join("device-sessions.json")).ok().and_then(|b|serde_json::from_slice(&b).ok()).unwrap_or_default();
 sessions.into_iter().filter(|(_,s)|s.remember&&s.expires>session_now()&&accounts.iter().any(|a|a.enabled&&a.id==s.uid&&a.salt==s.account_salt)).collect()
}
fn persist_sessions(root:&FsPath,sessions:&HashMap<String,Session>)->Res<()>{
 let remembered=sessions.iter().filter(|(_,s)|s.remember&&s.expires>session_now()).collect::<HashMap<_,_>>();
 let bytes=serde_json::to_vec(&remembered).map_err(|e|e.to_string())?;
 let mut temp=tempfile::NamedTempFile::new_in(root).map_err(|e|e.to_string())?;
 std::io::Write::write_all(&mut temp,&bytes).map_err(|e|e.to_string())?;
 temp.as_file().sync_all().map_err(|e|e.to_string())?;
 temp.persist(root.join("device-sessions.json")).map_err(|e|e.to_string())?;Ok(())
}
struct Team{root:PathBuf,feedback:PathBuf,profile:String,hosts:Vec<String>,accounts:Mutex<Vec<Account>>,sessions:Mutex<HashMap<String,Session>>,spaces:Mutex<HashMap<String,Workspace>>,queue:Arc<Semaphore>,pending:AtomicUsize,login_slots:Arc<Semaphore>,attempts:Mutex<HashMap<std::net::IpAddr,(Instant,u32)>>}
static TEAM:OnceLock<Arc<Team>>=OnceLock::new();
static LIVE_TLS:OnceLock<axum_server::tls_rustls::RustlsConfig>=OnceLock::new();
static CERT_WRITE:tokio::sync::Mutex<()>=tokio::sync::Mutex::const_new(());
pub(crate) fn settings_root()->Option<PathBuf>{TEAM.get().map(|t|t.root.clone())}
pub(crate) fn quota_usage(owner:&Shared)->Res<Value>{let mut used=json!({"host":crate::frontend_settings::used(owner)?});if let Some(t)=TEAM.get(){let accounts=t.accounts.lock().map_err(|_|"WORKSPACE_UNAVAILABLE")?.clone();for account in accounts{let w=t.space(&account)?;used[&account.id]=json!(crate::frontend_settings::used(&w.shared)?);}}Ok(used)}
fn cert_details(root:&FsPath)->Res<Value>{let bytes=std::fs::read(root.join("GVPAK-Team.cer")).map_err(|_|"CERTIFICATE_UNAVAILABLE")?;let fingerprint=Sha256::digest(&bytes).iter().map(|b|format!("{b:02X}")).collect::<Vec<_>>().join(":");Ok(json!({"fingerprint":fingerprint,"algorithm":"SHA-256"}))}
async fn regenerate_at(root:&FsPath,names:Vec<String>,tls:Option<&axum_server::tls_rustls::RustlsConfig>)->Res<Value>{
 let _guard=CERT_WRITE.lock().await;let _=rustls::crypto::ring::default_provider().install_default();let rcgen::CertifiedKey{cert,key_pair}=rcgen::generate_simple_self_signed(names).map_err(|_|"CERTIFICATE_GENERATION_FAILED")?;
 let pem=cert.pem().into_bytes();let key=key_pair.serialize_pem().into_bytes();let files=[("server.crt",pem.clone()),("server.key",key.clone()),("GVPAK-Team.cer",cert.der().to_vec())];let previous=files.iter().map(|(name,_)|(*name,std::fs::read(root.join(name)).ok())).collect::<Vec<_>>();
 let outcome=(||->Res<()>{for(name,bytes)in &files{crate::frontend_settings::atomic_bytes(root,name,bytes)?;}Ok(())})();
 if let Err(error)=outcome{for(name,bytes)in previous{if let Some(bytes)=bytes{let _=crate::frontend_settings::atomic_bytes(root,name,&bytes);}}return Err(error)}
 if let Some(tls)=tls{if tls.reload_from_pem(pem,key).await.is_err(){for(name,bytes)in previous{if let Some(bytes)=bytes{let _=crate::frontend_settings::atomic_bytes(root,name,&bytes);}}return Err("CERTIFICATE_RELOAD_FAILED".into())}}
 cert_details(root)
}
pub(crate) async fn certificate(name:&str)->Res<Value>{let t=TEAM.get().ok_or("TEAM_NOT_READY")?;match name{
 "certificate_status"=>cert_details(&t.root),
 "certificate_regenerate"=>{let names:Vec<String>=serde_json::from_slice(&std::fs::read(t.root.join("hosts.json")).map_err(|_|"CERTIFICATE_UNAVAILABLE")?).map_err(|_|"CERTIFICATE_UNAVAILABLE")?;regenerate_at(&t.root,names,LIVE_TLS.get()).await},
 "certificate_installer"=>{let bytes=std::fs::read(t.root.join("GVPAK-Team.cer")).map_err(|_|"CERTIFICATE_UNAVAILABLE")?;let encoded=STANDARD.encode(bytes);let script=format!("@echo off\r\nchcp 65001 >nul\r\npowershell.exe -NoProfile -Command \"$certFile=Join-Path $env:TEMP ('gvpak-'+[guid]::NewGuid()+'.cer');try{{[IO.File]::WriteAllBytes($certFile,[Convert]::FromBase64String('{encoded}'));certutil.exe -hashfile $certFile SHA256;certutil.exe -user -addstore Root $certFile;if($LASTEXITCODE -ne 0){{exit 1}}}}finally{{Remove-Item -LiteralPath $certFile -ErrorAction SilentlyContinue}}\"\r\nif errorlevel 1 (echo 证书安装失败，请将此窗口内容发给主机管理员。) else (echo GVPAK STUDIO 证书已安装到当前用户。)\r\npause\r\n");Ok(json!({"name":"安装证书.cmd","content":script,"fingerprint":cert_details(&t.root)?["fingerprint"]}))},
 _=>Err("UNKNOWN_COMMAND".into())}}
// HTTP/2 carries the host in :authority, not necessarily in the Host header.
// Normalize only the protocol-owned URI authority; never trust forwarding headers.
fn normalize_authority(req:&mut axum::extract::Request)->bool{
 if req.version()!=axum::http::Version::HTTP_2{return true}
 let Some(authority)=req.uri().authority().map(|a|a.as_str().to_string()) else{return false};
 if req.uri().scheme_str()!=Some("https"){return false}
 if let Some(host)=req.headers().get(header::HOST){if host.to_str().ok()!=Some(authority.as_str()){return false}}
 let Ok(host)=authority.parse() else{return false};req.headers_mut().insert(header::HOST,host);true
}
fn err(status:StatusCode,code:&str)->Response{(status,Json(json!({"error":code}))).into_response()}
fn hash(password:&str,salt:&str)->String{let mut out=[0u8;32];pbkdf2::pbkdf2_hmac::<Sha256>(password.as_bytes(),salt.as_bytes(),ROUNDS,&mut out);STANDARD.encode(out)}
fn equal(a:&str,b:&str)->bool{a.len()==b.len()&&a.bytes().zip(b.bytes()).fold(0u8,|n,(a,b)|n|(a^b))==0}
fn username_ok(s:&str)->bool{(3..=32).contains(&s.len())&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-')}
fn file_ok(s:&str)->bool{!s.is_empty()&&!s.contains(['/', '\\', ':'])&&s!="."&&s!=".."&&!s.ends_with(['.',' '])}
impl Team{
 fn persist(&self,accounts:&[Account])->Res<()>{let p=self.root.join("accounts.json");let temp=self.root.join("accounts.pending");std::fs::write(&temp,serde_json::to_vec(accounts).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;std::fs::rename(temp,p).map_err(|e|e.to_string())}
 fn allowed(&self,h:&HeaderMap,write:bool)->bool{
  let Some(host)=h.get(header::HOST).and_then(|v|v.to_str().ok()) else{return false};
  if !self.hosts.iter().any(|x|x==host){return false}
  if write&&h.get("x-gvpak-ipc").and_then(|v|v.to_str().ok())!=Some("1"){return false}
  h.get(header::ORIGIN).is_none_or(|v|v.to_str().ok()==Some(format!("https://{host}").as_str()))
 }
 fn account(&self,h:&HeaderMap)->Option<Account>{
  let token=h.get(header::COOKIE)?.to_str().ok()?.split(';').find_map(|v|v.trim().strip_prefix(&format!("{COOKIE}=")))?;
  let key=format!("{:x}",Sha256::digest(token));let session=self.sessions.lock().ok()?.get(&key)?.clone();
  if session.expires<=session_now(){return None}
  self.accounts.lock().ok()?.iter().find(|a|a.id==session.uid&&a.enabled&&a.salt==session.account_salt).cloned()
 }
 fn space(&self,a:&Account)->Res<Workspace>{let mut spaces=self.spaces.lock().map_err(|_|"WORKSPACE_UNAVAILABLE")?;if let Some(s)=spaces.get(&a.id){return Ok(s.clone())}
  let root=self.root.join("workspaces").join(&a.id);let store=Store::open(root.clone(),&self.profile)?;
  std::fs::create_dir_all(root.join("exports")).map_err(|e|e.to_string())?;
  let shared=Arc::new(AppState{store:Mutex::new(store),cancel:AtomicBool::new(false),local_job:Mutex::new(()),preview_epoch:AtomicU64::new(0)});
  let (events,_)=broadcast::channel(128);let hub=AppHandle{events};if a.enabled{crate::hybrid::spawn_queued(hub.clone(),shared.clone(),Some(self.queue.clone()));crate::new_product::resume(shared.clone(),hub.clone());crate::frontend_ops::resume(shared.clone(),hub.clone());}
  let w=Workspace{shared,hub,root,wait_cancel:Arc::new(AtomicU64::new(0))};spaces.insert(a.id.clone(),w.clone());Ok(w)
 }
}
pub async fn manage(v:Value)->Response{
 let Some(t)=TEAM.get() else{return err(StatusCode::SERVICE_UNAVAILABLE,"TEAM_NOT_READY")};
 let action=v["action"].as_str().unwrap_or("list");
 if action=="list"{let accounts=t.accounts.lock().unwrap();let sessions=t.sessions.lock().unwrap();let now=session_now();return Json(json!({"accounts":accounts.iter().map(|a|json!({"id":a.id,"username":a.username,"enabled":a.enabled,"loggedIn":a.enabled && sessions.values().any(|s|s.uid==a.id && s.expires>now)})).collect::<Vec<_>>(),"urls":t.hosts.iter().map(|h|format!("https://{h}")).collect::<Vec<_>>()})).into_response()}
 let name=v["username"].as_str().unwrap_or("").to_ascii_lowercase();if !username_ok(&name){return err(StatusCode::BAD_REQUEST,"USERNAME_3_TO_32_LETTERS_DIGITS_DASH")}
 if action=="disable"{let mut accounts=t.accounts.lock().unwrap();let mut next=accounts.clone();let Some(a)=next.iter_mut().find(|a|a.username==name) else{return err(StatusCode::NOT_FOUND,"ACCOUNT_NOT_FOUND")};a.enabled=false;let uid=a.id.clone();if t.persist(&next).is_err(){return err(StatusCode::INTERNAL_SERVER_ERROR,"ACCOUNT_SAVE_FAILED")};*accounts=next;t.sessions.lock().unwrap().retain(|_,s|s.uid!=uid);return Json(json!({"ok":true})).into_response()}
 if action!="create"&&action!="reset"{return err(StatusCode::BAD_REQUEST,"INVALID_ACTION")}
 let password=v["password"].as_str().unwrap_or("").to_string();if password.len()<12||password.len()>128{return err(StatusCode::BAD_REQUEST,"PASSWORD_12_TO_128_CHARACTERS")}
 let salt=uuid::Uuid::new_v4().to_string();let salt2=salt.clone();let hashed=tokio::task::spawn_blocking(move||hash(&password,&salt2)).await.unwrap();
 let mut accounts=t.accounts.lock().unwrap();let mut next=accounts.clone();
 if action=="create"{if next.len()>=64||next.iter().any(|a|a.username==name){return err(StatusCode::CONFLICT,"ACCOUNT_EXISTS_OR_LIMIT")};next.push(Account{id:uuid::Uuid::new_v4().to_string(),username:name.clone(),salt,hash:hashed,enabled:true});}
 else{let Some(a)=next.iter_mut().find(|a|a.username==name) else{return err(StatusCode::NOT_FOUND,"ACCOUNT_NOT_FOUND")};a.salt=salt;a.hash=hashed;a.enabled=true;}
 if t.persist(&next).is_err(){return err(StatusCode::INTERNAL_SERVER_ERROR,"ACCOUNT_SAVE_FAILED")};*accounts=next;
 let a=accounts.iter().find(|a|a.username==name).unwrap();t.sessions.lock().unwrap().retain(|_,s|s.uid!=a.id);Json(json!({"ok":true,"username":name})).into_response()
}
async fn session(State(t):State<Arc<Team>>,h:HeaderMap)->Response{if !t.allowed(&h,false){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")};Json(match t.account(&h){Some(a)=>json!({"team":true,"user":{"id":a.id,"username":a.username}}),None=>json!({"team":true,"user":null})}).into_response()}
async fn login(State(t):State<Arc<Team>>,ConnectInfo(peer):ConnectInfo<SocketAddr>,h:HeaderMap,Json(v):Json<Value>)->Response{
 if !t.allowed(&h,true){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")}
 {let mut attempts=t.attempts.lock().unwrap();attempts.retain(|_,(at,_)|at.elapsed()<Duration::from_secs(60));if attempts.len()>1024{return err(StatusCode::TOO_MANY_REQUESTS,"TRY_LATER")};let a=attempts.entry(peer.ip()).or_insert((Instant::now(),0));a.1+=1;if a.1>10{return err(StatusCode::TOO_MANY_REQUESTS,"TRY_AGAIN_IN_ONE_MINUTE")}}
 let name=v["username"].as_str().unwrap_or("").to_ascii_lowercase();let pass=v["password"].as_str().unwrap_or("").to_string();if pass.len()>128{return err(StatusCode::UNAUTHORIZED,"INVALID_LOGIN")}
 let account=t.accounts.lock().unwrap().iter().find(|a|a.username==name&&a.enabled).cloned();let candidate=account.clone();
 let Ok(_slot)=t.login_slots.clone().try_acquire_owned() else{return err(StatusCode::TOO_MANY_REQUESTS,"TRY_LATER")};
 let valid=tokio::task::spawn_blocking(move||{let salt=candidate.as_ref().map(|a|a.salt.as_str()).unwrap_or("invalid-user-salt");let actual=hash(&pass,salt);candidate.is_some_and(|a|equal(&actual,&a.hash))}).await.unwrap_or(false);
 if !valid{return err(StatusCode::UNAUTHORIZED,"INVALID_LOGIN")};let a=account.unwrap();
 let remember=v["remember"].as_bool().unwrap_or(false);let seconds=if remember{DEVICE_SECONDS}else{SESSION_SECONDS};
 let accounts=t.accounts.lock().unwrap();if !accounts.iter().any(|current|current.id==a.id&&current.enabled&&current.salt==a.salt){return err(StatusCode::UNAUTHORIZED,"INVALID_LOGIN")}
 let token=format!("{}{}",uuid::Uuid::new_v4().simple(),uuid::Uuid::new_v4().simple());let mut sessions=t.sessions.lock().unwrap();let mut next=sessions.clone();next.retain(|_,s|s.expires>session_now());
 if let Some(old)=h.get(header::COOKIE).and_then(|x|x.to_str().ok()).and_then(|cookies|cookies.split(';').find_map(|v|v.trim().strip_prefix(&format!("{COOKIE}=")))){next.remove(&format!("{:x}",Sha256::digest(old)));}
 if next.len()>=512{return err(StatusCode::TOO_MANY_REQUESTS,"SESSION_LIMIT")};next.insert(format!("{:x}",Sha256::digest(&token)),Session{uid:a.id.clone(),expires:session_now()+seconds,remember,account_salt:a.salt.clone()});
 if persist_sessions(&t.root,&next).is_err(){return err(StatusCode::INTERNAL_SERVER_ERROR,"SESSION_SAVE_FAILED")};*sessions=next;
 ([(header::SET_COOKIE,format!("{COOKIE}={token}; HttpOnly; Secure; SameSite=Strict; Path=/{}",if remember{format!("; Max-Age={seconds}")}else{String::new()}))],Json(json!({"user":{"id":a.id,"username":a.username}}))).into_response()
}
async fn logout(State(t):State<Arc<Team>>,h:HeaderMap)->Response{if !t.allowed(&h,true){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")};if let Some(cookie)=h.get(header::COOKIE).and_then(|x|x.to_str().ok()){if let Some(token)=cookie.split(';').find_map(|v|v.trim().strip_prefix(&format!("{COOKIE}="))){let mut sessions=t.sessions.lock().unwrap();let mut next=sessions.clone();next.remove(&format!("{:x}",Sha256::digest(token)));if persist_sessions(&t.root,&next).is_err(){return err(StatusCode::INTERNAL_SERVER_ERROR,"SESSION_SAVE_FAILED")};*sessions=next;}}([(header::SET_COOKIE,format!("{COOKIE}=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0"))],Json(json!({"ok":true}))).into_response()}
fn allowed_command(s:&str)->bool{matches!(s,"execution_preferences"|"session_status"|"task_list"|"task_stop"|"operation_execute"|"operation_result"|"selection_export_preview"|"assistant_context_plan"|"chat_execute"|"assets_metadata"|"new_product_capabilities"|"new_product_analyze"|"new_product_start"|"new_product_jobs"|"new_product_cancel"|"new_product_result"|"bootstrap"|"model_catalog"|"hybrid_plan"|"hybrid_start"|"hybrid_jobs"|"hybrid_cancel"|"hybrid_ack"|"assistant_key_status"|"assistant_tools"|"assistant_plan"|"assistant_preview"|"assistant_apply"|"assistant_presets"|"assistant_save_preset"|"project_create"|"project_rename"|"project_delete"|"project_restore"|"preview"|"preview_source"|"thumbnail"|"original_preview"|"generated_preview"|"adopt_generated"|"restore_version"|"generation_details"|"edit_commit"|"asset_trash"|"export_preview"|"export_run"|"cancel_job"|"credential_status"|"poe_key_status"|"generate")}
fn heavy(s:&str)->bool{matches!(s,"assistant_preview"|"assistant_apply"|"export_run"|"generate"|"edit_commit"|"asset_trash"|"adopt_generated"|"restore_version")}
fn safe_error(e:&str)->String{let code=e.split(':').next().unwrap_or("");if !code.is_empty()&&code.len()<80&&code.bytes().all(|b|b.is_ascii_uppercase()||b==b'_'){code.into()}else{"PROCESSING_FAILED".into()}}
fn export_relative(path:&FsPath,exports:&FsPath)->Option<String>{
 let rel=path.strip_prefix(exports).ok()?;let parts=rel.components().map(|c|match c{std::path::Component::Normal(s)=>s.to_str().filter(|s|file_ok(s)),_=>None}).collect::<Option<Vec<_>>>()?;
 if parts.is_empty(){return None}Some(parts.join("/"))
}
fn resolve_export(exports:&FsPath,id:&str)->Option<PathBuf>{
 let rel=String::from_utf8(URL_SAFE_NO_PAD.decode(id).ok()?).ok()?;
 if rel.split('/').any(|s|!file_ok(s)){return None}
 let root=exports.canonicalize().ok()?;let path=root.join(rel).canonicalize().ok()?;
 (path.starts_with(&root)&&path.is_file()).then_some(path)
}
fn download_url(path:&str,exports:&FsPath)->Option<String>{Some(format!("/api/download/{}",URL_SAFE_NO_PAD.encode(export_relative(FsPath::new(path),exports)?)))}
fn list_exports(dir:&FsPath,root:&FsPath,out:&mut Vec<Value>){if let Ok(entries)=std::fs::read_dir(dir){for e in entries.flatten(){let Ok(kind)=e.file_type() else{continue};if kind.is_symlink(){continue}if kind.is_dir(){list_exports(&e.path(),root,out)}else if kind.is_file(){if !e.path().extension().and_then(|s|s.to_str()).is_some_and(|s|["png","jpg","jpeg","webp"].contains(&s.to_ascii_lowercase().as_str())){continue}let Ok(real)=e.path().canonicalize() else{continue};if !real.starts_with(root){continue}let name=e.file_name().to_string_lossy().to_string();out.push(json!({"name":name,"bytes":e.metadata().map(|m|m.len()).unwrap_or(0),"url":download_url(&e.path().to_string_lossy(),root)}));}}}}

fn scrub(v:&mut Value,uid:&str,exports:&FsPath){match v{
 Value::Object(o)=>{if let Some(path)=o.get("path").and_then(Value::as_str){if download_url(path,exports).is_some(){if let Some(name)=FsPath::new(path).file_name().and_then(|n|n.to_str()){let name=name.to_string();o.insert("downloadName".into(),json!(name));}}}for key in ["sourcePath","source_path","savedPath","outputDir","output_dir"]{o.remove(key);}if o.contains_key("dataRoot"){o.insert("dataRoot".into(),json!(format!("team/{uid}")));}
  for (k,x) in o.iter_mut(){if k=="path"||k=="exported"{if let Some(p)=x.as_str(){*x=download_url(p,exports).map(Value::String).unwrap_or(Value::Null);continue;}}if k=="error"{if let Some(e)=x.as_str(){*x=json!(safe_error(e));continue;}}scrub(x,uid,exports);}},
 Value::Array(a)=>for x in a{scrub(x,uid,exports)},Value::String(s)=>{if s.contains(":\\")||s.contains(":/")&&!s.starts_with("http"){*s="Host path hidden".into();}},_=>{}
}}
struct Waiting<'a>(&'a AtomicUsize);impl Drop for Waiting<'_>{fn drop(&mut self){self.0.fetch_sub(1,Ordering::Relaxed);}}
async fn permit(t:&Arc<Team>,w:&Workspace)->Res<tokio::sync::OwnedSemaphorePermit>{let count=t.pending.fetch_add(1,Ordering::Relaxed);let waiting=Waiting(&t.pending);let _=count;let _=w.hub.emit("team-queue",json!({"state":"queued"}));let generation=w.wait_cancel.load(Ordering::Relaxed);let canceled=async{loop{tokio::time::sleep(Duration::from_millis(100)).await;if w.wait_cancel.load(Ordering::Relaxed)!=generation{return;}}};let p=tokio::select!{p=t.queue.clone().acquire_owned()=>p.map_err(|_|"QUEUE_CLOSED")?,_=canceled=>return Err("TASK_CANCELLED".into())};if w.wait_cancel.load(Ordering::Relaxed)!=generation{return Err("TASK_CANCELLED".into())};drop(waiting);let _=w.hub.emit("team-queue",json!({"state":"running"}));Ok(p)}
async fn invoke(State(t):State<Arc<Team>>,Path(name):Path<String>,h:HeaderMap,Json(mut v):Json<Value>)->Response{
 if !t.allowed(&h,true){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")};let Some(a)=t.account(&h) else{return err(StatusCode::UNAUTHORIZED,"LOGIN_REQUIRED")};if !allowed_command(&name){return err(StatusCode::FORBIDDEN,"HOST_OPERATION_BLOCKED")}
 if name=="session_status"{return Json(json!({"account":{"id":a.id,"name":a.username},"isHost":false,"isLocal":false,"quota":t.space(&a).ok().and_then(|w|crate::frontend_settings::quota(&w.shared).ok().flatten())})).into_response()}
 let w=match t.space(&a){Ok(w)=>w,Err(_)=>return err(StatusCode::INTERNAL_SERVER_ERROR,"WORKSPACE_UNAVAILABLE")};let exports=w.root.join("exports");
 if name=="cancel_job"{w.wait_cancel.fetch_add(1,Ordering::Relaxed);}
 if name=="export_run"{v["preset"]["outputDir"]=json!(exports);}if name=="hybrid_start"||name=="assistant_apply"{v["outputDir"]=json!(exports);}
 let _p=if heavy(&name){match permit(&t,&w).await{Ok(p)=>Some(p),Err(e)=>return err(StatusCode::TOO_MANY_REQUESTS,&e)}}else{None};
 if matches!(name.as_str(),"operation_execute"|"chat_execute"){v["_frontendHost"]=json!(false);}
 let shared=w.shared.clone();let notice=w.hub.clone();let command=name.clone();let args=v.clone();
 match crate::dispatch(w.shared,w.hub,name,v).await{Ok(r)=>{if r.headers().get(header::CONTENT_TYPE).is_some_and(|v|v.to_str().unwrap_or("").contains("json")){let Ok(bytes)=axum::body::to_bytes(r.into_body(),64*1024*1024).await else{return err(StatusCode::INTERNAL_SERVER_ERROR,"RESPONSE_TOO_LARGE")};let Ok(mut value)=serde_json::from_slice::<Value>(&bytes) else{return err(StatusCode::INTERNAL_SERVER_ERROR,"INVALID_RESPONSE")};if let Ok(root)=crate::retention::TempRoot::new(&t.root,&a.id){if let Err(_)=crate::retention::record(&root,&shared,&command,&args,&value,crate::retention::now()){let _=notice.emit("team-retention-error",json!({"message":"操作已完成，但临时文件登记失败。主机副本暂时保留，请联系管理员。"}));}}if command=="task_list"{if let Some(items)=value["tasks"].as_array_mut(){for task in items{task["accountId"]=json!(a.id);}}}scrub(&mut value,&a.id,&exports);Json(value).into_response()}else{r}},Err(e)=>err(StatusCode::BAD_REQUEST,&safe_error(&e))}
}
async fn upload(State(t):State<Arc<Team>>,h:HeaderMap,Json(v):Json<Value>)->Response{
 if !t.allowed(&h,true){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")};let Some(a)=t.account(&h) else{return err(StatusCode::UNAUTHORIZED,"LOGIN_REQUIRED")};let w=match t.space(&a){Ok(w)=>w,Err(_)=>return err(StatusCode::INTERNAL_SERVER_ERROR,"WORKSPACE_UNAVAILABLE")};let _p=match permit(&t,&w).await{Ok(p)=>p,Err(e)=>return err(StatusCode::TOO_MANY_REQUESTS,&e)};let exports=w.root.join("exports");let shared=w.shared.clone();let notice=w.hub.clone();
 let result=tokio::task::spawn_blocking(move||->Res<Value>{let _job=crate::begin_local_job(&w.shared)?;let id=v["projectId"].as_str().ok_or("PROJECT_REQUIRED")?;let items=v["files"].as_array().ok_or("FILES_REQUIRED")?;if items.is_empty(){return Err("UPLOAD_LIMIT".into())};let uploads=w.root.join("uploads");std::fs::create_dir_all(&uploads).map_err(|_|"TEMP_UNAVAILABLE")?;let temp=tempfile::tempdir_in(uploads).map_err(|_|"TEMP_UNAVAILABLE")?;let mut paths=Vec::new();for (i,item) in items.iter().enumerate(){let name=item["name"].as_str().ok_or("NAME_REQUIRED")?;if !file_ok(name){return Err("INVALID_NAME".into())};let ext=FsPath::new(name).extension().and_then(|s|s.to_str()).unwrap_or("").to_ascii_lowercase();if !["png","jpg","jpeg","webp"].contains(&ext.as_str()){return Err("UNSUPPORTED_IMAGE".into())};let dir=temp.path().join(i.to_string());std::fs::create_dir(&dir).map_err(|_|"TEMP_UNAVAILABLE")?;let p=dir.join(name);let bytes=STANDARD.decode(item["data"].as_str().ok_or("DATA_REQUIRED")?).map_err(|_|"INVALID_DATA")?;if bytes.len()>30*1024*1024{return Err("FILE_TOO_LARGE".into())};std::fs::write(&p,bytes).map_err(|_|"TEMP_UNAVAILABLE")?;paths.push(p.to_string_lossy().to_string());}
 let report=crate::locked(&w.shared)?.import(id,paths,&w.shared.cancel,|_,_|{})?;serde_json::to_value(report).map_err(|_|"INVALID_RESPONSE".into())}).await;
 match result{Ok(Ok(mut v))=>{if let Ok(root)=crate::retention::TempRoot::new(&t.root,&a.id){if let Err(_)=crate::retention::record(&root,&shared,"upload",&json!({}),&v,crate::retention::now()){let _=notice.emit("team-retention-error",json!({"message":"图片已上传，但临时文件登记失败。主机副本暂时保留，请联系管理员。"}));}}scrub(&mut v,&a.id,&exports);Json(v).into_response()},Ok(Err(e))=>err(StatusCode::BAD_REQUEST,&safe_error(&e)),_=>err(StatusCode::INTERNAL_SERVER_ERROR,"UPLOAD_FAILED")}
}
async fn files(State(t):State<Arc<Team>>,h:HeaderMap)->Response{if !t.allowed(&h,false){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")};let Some(a)=t.account(&h) else{return err(StatusCode::UNAUTHORIZED,"LOGIN_REQUIRED")};let Ok(w)=t.space(&a) else{return err(StatusCode::INTERNAL_SERVER_ERROR,"WORKSPACE_UNAVAILABLE")};let root=w.root.join("exports").canonicalize().unwrap();let mut files=vec![];list_exports(&root,&root,&mut files);if let Ok(temp)=crate::retention::TempRoot::new(&t.root,&a.id){if let Ok(status)=crate::retention::file_status(&temp,&w.shared,crate::retention::now()){files.retain_mut(|f|{let rel=f["url"].as_str().and_then(|u|u.strip_prefix("/api/download/")).and_then(|id|URL_SAFE_NO_PAD.decode(id).ok()).and_then(|b|String::from_utf8(b).ok());if let Some(s)=rel.and_then(|r|status.get(&r)){f["expiresAt"]=s["expiresAt"].clone();return s["saved"]!=true}true});}}Json(json!({"files":files,"temporary":true,"retentionHours":24})).into_response()}

async fn download(State(t):State<Arc<Team>>,Path(id):Path<String>,h:HeaderMap)->Response{if !t.allowed(&h,false){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")};let Some(a)=t.account(&h) else{return err(StatusCode::UNAUTHORIZED,"LOGIN_REQUIRED")};let Ok(w)=t.space(&a) else{return err(StatusCode::INTERNAL_SERVER_ERROR,"WORKSPACE_UNAVAILABLE")};let Some(path)=resolve_export(&w.root.join("exports"),&id) else{return err(StatusCode::NOT_FOUND,"FILE_NOT_FOUND")};let name=path.file_name().unwrap().to_string_lossy().to_string();let Ok(bytes)=tokio::fs::read(path).await else{return err(StatusCode::NOT_FOUND,"FILE_NOT_FOUND")};{let encoded=name.bytes().map(|b|if b.is_ascii_alphanumeric()||b"-._~".contains(&b){(b as char).to_string()}else{format!("%{b:02X}")}).collect::<String>();([(header::CONTENT_TYPE,"application/octet-stream".to_string()),(header::CONTENT_DISPOSITION,format!("attachment; filename*=UTF-8''{encoded}"))],bytes).into_response()}}
async fn saved(State(t):State<Arc<Team>>,h:HeaderMap,Json(v):Json<Value>)->Response{
 if !t.allowed(&h,true){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")};let Some(a)=t.account(&h) else{return err(StatusCode::UNAUTHORIZED,"LOGIN_REQUIRED")};let Ok(w)=t.space(&a) else{return err(StatusCode::INTERNAL_SERVER_ERROR,"WORKSPACE_UNAVAILABLE")};
 let Some(id)=v["id"].as_str() else{return err(StatusCode::BAD_REQUEST,"INVALID_FILE")};let Some(rel)=URL_SAFE_NO_PAD.decode(id).ok().and_then(|b|String::from_utf8(b).ok()) else{return err(StatusCode::BAD_REQUEST,"INVALID_FILE")};if rel.split('/').any(|s|!file_ok(s)){return err(StatusCode::BAD_REQUEST,"INVALID_FILE")}
 let Some(sha)=v["sha256"].as_str().filter(|s|s.len()==64&&s.bytes().all(|b|b.is_ascii_hexdigit())) else{return err(StatusCode::BAD_REQUEST,"SAVED_FILE_MISMATCH")};let Some(bytes)=v["bytes"].as_u64() else{return err(StatusCode::BAD_REQUEST,"SAVED_FILE_MISMATCH")};
 let _p=match permit(&t,&w).await{Ok(p)=>p,Err(e)=>return err(StatusCode::BAD_REQUEST,&e)};
 let Ok(root)=crate::retention::TempRoot::new(&t.root,&a.id) else{return err(StatusCode::FORBIDDEN,"RETENTION_BOUNDARY_REJECTED")};let path=root.path().join("exports").join(rel);let sha=sha.to_string();let shared=w.shared.clone();
 match tokio::task::spawn_blocking(move||crate::retention::acknowledge(&root,&shared,&path,&sha,bytes,crate::retention::now())).await{Ok(Ok(summary))=>{let _=w.hub.emit("team-cleanup",json!({"reason":"saved","summary":summary}));Json(json!({"ok":true,"summary":summary})).into_response()},Ok(Err(e))=>err(StatusCode::BAD_REQUEST,&safe_error(&e)),_=>err(StatusCode::INTERNAL_SERVER_ERROR,"CLEANUP_FAILED")}
}
fn start_cleanup(t:Arc<Team>){tokio::spawn(async move{loop{
 let accounts=t.accounts.lock().unwrap().clone();for account in accounts{let Ok(w)=t.space(&account) else{continue};let Ok(root)=crate::retention::TempRoot::new(&t.root,&account.id) else{continue};let Ok(_permit)=t.queue.clone().acquire_owned().await else{return};let shared=w.shared.clone();match tokio::task::spawn_blocking(move||crate::retention::sweep(&root,&shared,crate::retention::now())).await{Ok(Ok(s))=>{if s.removed_assets>0||s.removed_files>0{let _=w.hub.emit("team-cleanup",json!({"reason":"expired","summary":s}));}},_=>{let _=w.hub.emit("team-retention-error",json!({"message":"临时文件清理失败，请联系管理员。"}));}}}
 tokio::time::sleep(Duration::from_secs(60)).await;
}});}
async fn events(State(t):State<Arc<Team>>,h:HeaderMap)->Response{if !t.allowed(&h,false){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")};let Some(a)=t.account(&h) else{return err(StatusCode::UNAUTHORIZED,"LOGIN_REQUIRED")};let Ok(w)=t.space(&a) else{return err(StatusCode::INTERNAL_SERVER_ERROR,"WORKSPACE_UNAVAILABLE")};let rx=w.hub.events.subscribe();let uid=a.id;let exports=w.root.join("exports");let t2=t.clone();let headers=h.clone();let stream=futures_util::stream::unfold((rx,uid,exports,t2,headers),|(mut rx,uid,exports,t,h)|async move{loop{if t.account(&h).is_none(){return None};match tokio::time::timeout(Duration::from_secs(15),rx.recv()).await{Ok(Ok(mut v))=>{scrub(&mut v,&uid,&exports);return Some((Ok::<_,std::convert::Infallible>(Event::default().data(v.to_string())),(rx,uid,exports,t,h)))},Ok(Err(broadcast::error::RecvError::Closed))=>return None,_=>continue}}});Sse::new(stream).keep_alive(KeepAlive::default()).into_response()}
async fn feedback(State(t):State<Arc<Team>>,h:HeaderMap,Json(v):Json<Value>)->Response{
 if !t.allowed(&h,true){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")};let Some(a)=t.account(&h) else{return err(StatusCode::UNAUTHORIZED,"LOGIN_REQUIRED")};
 save_feedback(&t,&a.username,v)
}
pub fn host_feedback(v:Value)->Response{let Some(t)=TEAM.get() else{return err(StatusCode::SERVICE_UNAVAILABLE,"FEEDBACK_UNAVAILABLE")};save_feedback(t,"主机",v)}
fn save_feedback(t:&Team,username:&str,v:Value)->Response{
 let text=v["description"].as_str().unwrap_or("");if text.trim().is_empty()||text.len()>32000{return err(StatusCode::BAD_REQUEST,"DESCRIPTION_REQUIRED")};
 let id=uuid::Uuid::new_v4().to_string();let parent=t.feedback.join(username);if std::fs::create_dir_all(&parent).is_err(){return err(StatusCode::INTERNAL_SERVER_ERROR,"FEEDBACK_SAVE_FAILED")};
 let Ok(temp)=tempfile::tempdir_in(&parent) else{return err(StatusCode::INTERNAL_SERVER_ERROR,"FEEDBACK_SAVE_FAILED")};
 if let Some(files)=v["images"].as_array(){for (i,item) in files.iter().enumerate(){let Some(data)=item["data"].as_str() else{return err(StatusCode::BAD_REQUEST,"INVALID_IMAGE")};let Ok(bytes)=STANDARD.decode(data) else{return err(StatusCode::BAD_REQUEST,"INVALID_IMAGE")};if bytes.len()>12*1024*1024{return err(StatusCode::BAD_REQUEST,"IMAGE_TOO_LARGE")};let mut reader=image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().unwrap();let mut limits=image::Limits::default();limits.max_image_width=Some(16000);limits.max_image_height=Some(16000);limits.max_alloc=Some(128*1024*1024);reader.limits(limits);let Ok(im)=reader.decode() else{return err(StatusCode::BAD_REQUEST,"INVALID_IMAGE")};if im.save(temp.path().join(format!("图片-{}.png",i+1))).is_err(){return err(StatusCode::INTERNAL_SERVER_ERROR,"FEEDBACK_SAVE_FAILED")}}}
 let timestamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();let body=format!("账号：{}\n提交时间（Unix）：{}\n问题编号：{}\n\n{}",username,timestamp,id,text);
 if std::fs::write(temp.path().join("问题说明.txt"),body).is_err(){return err(StatusCode::INTERNAL_SERVER_ERROR,"FEEDBACK_SAVE_FAILED")};
 if std::fs::rename(temp.path(),parent.join(format!("{timestamp}-{id}"))).is_err(){return err(StatusCode::INTERNAL_SERVER_ERROR,"FEEDBACK_SAVE_FAILED")};Json(json!({"ok":true,"id":id})).into_response()
}
pub fn launch(project:PathBuf,owner:Shared){
 let _=rustls::crypto::ring::default_provider().install_default();
 let root=std::env::var_os("GVPAK_TEAM_ROOT").map(PathBuf::from).unwrap_or_else(||project.join("backend/team-private"));if std::fs::create_dir_all(&root).is_err(){return};let profile=crate::locked(&owner).unwrap().profile.clone();
 let mut names=std::env::var("GVPAK_TEAM_HOSTS").unwrap_or("localhost,127.0.0.1".into()).split(',').map(str::trim).filter(|x|!x.is_empty()).map(str::to_string).collect::<Vec<_>>();names.sort();names.dedup();let port=std::env::var("GVPAK_TEAM_PORT").unwrap_or("8788".into());
 let accounts:Vec<Account>=match std::fs::read(root.join("accounts.json")){Ok(b)=>match serde_json::from_slice(&b){Ok(a)=>a,Err(_)=>return},Err(e) if e.kind()==std::io::ErrorKind::NotFound=>vec![],_=>return};
 let restored_sessions=load_sessions(&root,&accounts);
 let t=Arc::new(Team{root:root.clone(),feedback:std::env::var_os("GVPAK_FEEDBACK_ROOT").map(PathBuf::from).unwrap_or_else(||project.join("问题汇总")),profile,hosts:names.iter().map(|n|format!("{n}:{port}")).collect(),accounts:Mutex::new(accounts),sessions:Mutex::new(restored_sessions),spaces:Mutex::new(HashMap::new()),queue:Arc::new(Semaphore::new(1)),pending:AtomicUsize::new(0),login_slots:Arc::new(Semaphore::new(2)),attempts:Mutex::new(HashMap::new())});if TEAM.set(t.clone()).is_err(){return};start_cleanup(t.clone());
 // Resume only this team's jobs. Each worker competes for the same processing slot.
 for a in t.accounts.lock().unwrap().clone().iter().filter(|a|a.enabled){let _=t.space(a);}
 tokio::spawn(async move{
  let result=async{let cert=root.join("server.crt");let key=root.join("server.key");let names_file=root.join("hosts.json");let expected=serde_json::to_vec(&names).unwrap();let previous=std::fs::read(&names_file).ok().and_then(|b|serde_json::from_slice::<Vec<String>>(&b).ok()).map(|mut v|{v.sort();v.dedup();v});if !cert.exists()||!key.exists()||previous.as_ref()!=Some(&names){
   let rcgen::CertifiedKey{cert:certificate,key_pair}=rcgen::generate_simple_self_signed(names.clone()).map_err(|e|e.to_string())?;std::fs::write(&cert,certificate.pem()).map_err(|e|e.to_string())?;std::fs::write(&key,key_pair.serialize_pem()).map_err(|e|e.to_string())?;std::fs::write(root.join("GVPAK-Team.cer"),certificate.der()).map_err(|e|e.to_string())?;std::fs::write(names_file,expected).map_err(|e|e.to_string())?;
  }
  let tls=axum_server::tls_rustls::RustlsConfig::from_pem_file(cert,key).await.map_err(|e|e.to_string())?;
  let _=LIVE_TLS.set(tls.clone());
  let router=Router::new().route("/api/team/session",get(session)).route("/api/team/login",post(login)).route("/api/team/logout",post(logout)).route("/api/feedback",post(feedback)).route("/api/files",get(files)).route("/api/files/saved",post(saved)).route("/api/download/{id}",get(download)).route("/api/invoke/{name}",post(invoke)).route("/api/upload",post(upload)).route("/api/events",get(events)).layer(DefaultBodyLimit::max(80*1024*1024)).layer(axum::middleware::from_fn(|mut req:axum::extract::Request,next:axum::middleware::Next|async move{if !normalize_authority(&mut req){return err(StatusCode::FORBIDDEN,"ORIGIN_REJECTED")};let mut res=next.run(req).await;res.headers_mut().insert(header::CACHE_CONTROL,"no-store".parse().unwrap());res.headers_mut().insert("x-content-type-options","nosniff".parse().unwrap());res.headers_mut().insert("x-frame-options","DENY".parse().unwrap());res})).with_state(t).merge(crate::transport::static_pages(project.join("dist")));
  let addr=format!("0.0.0.0:{port}").parse::<SocketAddr>().map_err(|e|e.to_string())?;
  axum_server::bind_rustls(addr,tls).serve(router.into_make_service_with_connect_info::<SocketAddr>()).await.map_err(|e|e.to_string())
  }.await;
  if let Err(e)=result{let _=std::fs::write(root.join("startup-error.log"),e);}
 });
}
#[cfg(test)]mod tests{use super::*;
 fn auth_team(root:PathBuf)->Arc<Team>{Arc::new(Team{feedback:root.join("reports"),root,profile:"test".into(),hosts:vec!["localhost:8788".into()],accounts:Mutex::new(vec![Account{id:"user-a".into(),username:"alice".into(),salt:"test-salt".into(),hash:hash("test-password-123","test-salt"),enabled:true}]),sessions:Mutex::new(HashMap::new()),spaces:Mutex::new(HashMap::new()),queue:Arc::new(Semaphore::new(1)),pending:AtomicUsize::new(0),login_slots:Arc::new(Semaphore::new(2)),attempts:Mutex::new(HashMap::new())})}
 fn auth_headers(cookie:Option<&str>)->HeaderMap{let mut h=HeaderMap::new();h.insert(header::HOST,"localhost:8788".parse().unwrap());h.insert("x-gvpak-ipc","1".parse().unwrap());if let Some(c)=cookie{h.insert(header::COOKIE,c.parse().unwrap());}h}
 async fn device_login(t:Arc<Team>,remember:bool)->String{
  let response=login(State(t),ConnectInfo("127.0.0.1:12345".parse().unwrap()),auth_headers(None),Json(json!({"username":"alice","password":"test-password-123","remember":remember}))).await;
  assert_eq!(response.status(),StatusCode::OK);let cookie=response.headers()[header::SET_COOKIE].to_str().unwrap();
  if remember{assert!(cookie.contains("Max-Age=2592000"));}else{assert!(!cookie.contains("Max-Age="));}
  for flag in ["HttpOnly","Secure","SameSite=Strict"]{assert!(cookie.contains(flag));}
  cookie.split(';').next().unwrap().into()
 }
 #[tokio::test]async fn remembered_devices_survive_restart_and_logout_is_device_local(){
  let temp=tempfile::tempdir().unwrap();let t=auth_team(temp.path().into());
  let first=device_login(t.clone(),true).await;let second=device_login(t.clone(),true).await;let temporary=device_login(t.clone(),false).await;
  assert!(t.account(&auth_headers(Some(&temporary))).is_some());
  let disk=std::fs::read_to_string(temp.path().join("device-sessions.json")).unwrap();
  assert!(!disk.contains(first.split('=').nth(1).unwrap()));assert!(!disk.contains("test-password-123"));
  let restart=auth_team(temp.path().into());*restart.sessions.lock().unwrap()=load_sessions(temp.path(),&restart.accounts.lock().unwrap());
  assert!(restart.account(&auth_headers(Some(&first))).is_some());assert!(restart.account(&auth_headers(Some(&second))).is_some());assert!(restart.account(&auth_headers(Some(&temporary))).is_none());
  let response=logout(State(restart.clone()),auth_headers(Some(&first))).await;assert_eq!(response.status(),StatusCode::OK);assert!(response.headers()[header::SET_COOKIE].to_str().unwrap().contains("Max-Age=0"));
  *restart.sessions.lock().unwrap()=load_sessions(temp.path(),&restart.accounts.lock().unwrap());
  assert!(restart.account(&auth_headers(Some(&first))).is_none());assert!(restart.account(&auth_headers(Some(&second))).is_some());
  restart.accounts.lock().unwrap()[0].salt="new-password-salt".into();assert!(restart.account(&auth_headers(Some(&second))).is_none());assert!(load_sessions(temp.path(),&restart.accounts.lock().unwrap()).is_empty());
 }
 #[tokio::test]async fn remembered_devices_reject_expired_and_disabled_accounts(){
  let temp=tempfile::tempdir().unwrap();let t=auth_team(temp.path().into());let cookie=device_login(t.clone(),true).await;
  t.accounts.lock().unwrap()[0].enabled=false;assert!(t.account(&auth_headers(Some(&cookie))).is_none());assert!(load_sessions(temp.path(),&t.accounts.lock().unwrap()).is_empty());
  t.accounts.lock().unwrap()[0].enabled=true;t.sessions.lock().unwrap().values_mut().for_each(|s|s.expires=session_now());
  assert!(t.account(&auth_headers(Some(&cookie))).is_none());persist_sessions(temp.path(),&t.sessions.lock().unwrap()).unwrap();assert!(load_sessions(temp.path(),&t.accounts.lock().unwrap()).is_empty());
 }
 #[tokio::test]async fn failed_device_storage_never_issues_a_login_cookie(){
  let temp=tempfile::tempdir().unwrap();let t=auth_team(temp.path().join("missing-directory"));let response=login(State(t.clone()),ConnectInfo("127.0.0.1:12345".parse().unwrap()),auth_headers(None),Json(json!({"username":"alice","password":"test-password-123","remember":true}))).await;
  assert_eq!(response.status(),StatusCode::INTERNAL_SERVER_ERROR);assert!(response.headers().get(header::SET_COOKIE).is_none());assert!(t.sessions.lock().unwrap().is_empty());
 }
 #[test]fn host_commands_are_never_allowed(){for name in ["pick_import","import_paths","pick_output","data_root_switch","credential_set","assistant_key_set","credential_delete","team_accounts","hybrid_open_output"]{assert!(!allowed_command(name));}}
 #[tokio::test]async fn settings_and_certificate_commands_return_403_even_for_authenticated_team_users(){let temp=tempfile::tempdir().unwrap();let t=auth_team(temp.path().into());let cookie=device_login(t.clone(),false).await;for name in ["settings_read","settings_patch","certificate_status","certificate_regenerate","certificate_installer"]{let response=invoke(State(t.clone()),Path(name.into()),auth_headers(Some(&cookie)),Json(json!({"patch":{"costCny":0.01},"_frontendHost":true}))).await;assert_eq!(response.status(),StatusCode::FORBIDDEN,"{}",name);}}
 #[tokio::test]async fn certificate_rotation_reloads_tls_and_preserves_names_and_other_data(){let temp=tempfile::tempdir().unwrap();let names=vec!["localhost".into(),"127.0.0.1".into()];std::fs::write(temp.path().join("accounts.json"),b"preserved-account-fixture").unwrap();let first=regenerate_at(temp.path(),names.clone(),None).await.unwrap();let tls=axum_server::tls_rustls::RustlsConfig::from_pem_file(temp.path().join("server.crt"),temp.path().join("server.key")).await.unwrap();let previous=std::fs::read(temp.path().join("GVPAK-Team.cer")).unwrap();let second=regenerate_at(temp.path(),names,Some(&tls)).await.unwrap();assert_ne!(first["fingerprint"],second["fingerprint"]);assert_ne!(previous,std::fs::read(temp.path().join("GVPAK-Team.cer")).unwrap());assert_eq!(std::fs::read(temp.path().join("accounts.json")).unwrap(),b"preserved-account-fixture");assert_eq!(second["algorithm"],"SHA-256");assert_eq!(second["fingerprint"].as_str().unwrap().len(),95);}
 #[test]fn filenames_cannot_escape(){for name in ["../secret","..\\secret","C:\\secret","image.png:secret","..",""]{assert!(!file_ok(name));}assert!(file_ok("my photo.webp"));}
 #[test]fn response_hides_host_paths(){let mut v=json!({"sourcePath":"C:/secret","dataRoot":"C:/private","path":"C:/private/out/a.webp","nested":{"outputDir":"C:/private"}});scrub(&mut v,"a",FsPath::new("C:/private/out"));assert_eq!(v["dataRoot"],"team/a");assert!(v.get("sourcePath").is_none());assert!(!v.to_string().contains("C:/"));}
 #[tokio::test]
 async fn processing_slot_queues_and_releases(){
  let temp=tempfile::tempdir().unwrap();let root=temp.path().join("workspace");let shared=Arc::new(AppState{store:Mutex::new(Store::open(root.clone(),"test").unwrap()),cancel:AtomicBool::new(false),local_job:Mutex::new(()),preview_epoch:AtomicU64::new(0)});let (events,_)=broadcast::channel(8);let w=Workspace{shared,root:root.clone(),hub:AppHandle{events},wait_cancel:Arc::new(AtomicU64::new(0))};
  let t=Arc::new(Team{feedback:root.join("reports"),root,profile:"test".into(),hosts:vec![],accounts:Mutex::new(vec![]),sessions:Mutex::new(HashMap::new()),spaces:Mutex::new(HashMap::new()),queue:Arc::new(Semaphore::new(1)),pending:AtomicUsize::new(0),login_slots:Arc::new(Semaphore::new(2)),attempts:Mutex::new(HashMap::new())});
  let first=permit(&t,&w).await.unwrap();let t2=t.clone();let w2=w.clone();let mut next=tokio::spawn(async move{permit(&t2,&w2).await.unwrap()});
  assert!(tokio::time::timeout(Duration::from_millis(30),&mut next).await.is_err());assert_eq!(t.pending.load(Ordering::Relaxed),1);drop(first);
  let second=tokio::time::timeout(Duration::from_secs(1),next).await.unwrap().unwrap();assert_eq!(t.pending.load(Ordering::Relaxed),0);drop(second);assert_eq!(t.queue.available_permits(),1);
  let held=permit(&t,&w).await.unwrap();let t3=t.clone();let w3=w.clone();let canceled=tokio::spawn(async move{permit(&t3,&w3).await});tokio::time::sleep(Duration::from_millis(20)).await;w.wait_cancel.fetch_add(1,Ordering::Relaxed);assert!(tokio::time::timeout(Duration::from_secs(1),canceled).await.unwrap().unwrap().is_err());assert_eq!(t.pending.load(Ordering::Relaxed),0);drop(held);
 }

 #[test]fn http2_authority_preserves_origin_checks(){
  let mut req=axum::http::Request::builder().version(axum::http::Version::HTTP_2).uri("https://192.0.2.17:8788/api/team/session").body(axum::body::Body::empty()).unwrap();
  assert!(normalize_authority(&mut req));assert_eq!(req.headers()[header::HOST],"192.0.2.17:8788");
  req.headers_mut().insert(header::HOST,"evil.invalid".parse().unwrap());assert!(!normalize_authority(&mut req));
  let mut bad=axum::http::Request::builder().version(axum::http::Version::HTTP_2).uri("http://192.0.2.17:8788/api/team/session").body(axum::body::Body::empty()).unwrap();assert!(!normalize_authority(&mut bad));
 }

}
