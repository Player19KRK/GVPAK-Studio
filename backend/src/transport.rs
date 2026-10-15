use std::{convert::Infallible,ops::Deref,path::PathBuf,sync::OnceLock,time::Duration};
use axum::{Router,Json,extract::{State as AxumState,Path,DefaultBodyLimit},http::{HeaderMap,StatusCode},response::{IntoResponse,Response,Sse,sse::{Event,KeepAlive}},routing::{get,post}};
use serde::Serialize;
use serde_json::{Value,json};
use tokio::sync::broadcast;
use base64::Engine;
use crate::{Shared,Res};
pub struct State<'a,T>(pub &'a T);
impl<'a,T> State<'a,T>{pub fn inner(&self)->&'a T{self.0}}
impl<T> Deref for State<'_,T>{type Target=T;fn deref(&self)->&T{self.0}}
impl<'a,T> From<&'a T> for State<'a,T>{fn from(v:&'a T)->Self{Self(v)}}
#[derive(Clone)]
pub struct AppHandle{events:broadcast::Sender<Value>}
impl AppHandle {pub fn emit<T:Serialize>(&self,name:&str,payload:T)->Res<()>{let _=self.events.send(json!({"event":name,"payload":payload}));Ok(())}}
pub mod runtime{
 pub use tokio::task::spawn_blocking;
 pub fn block_on<F:std::future::Future>(f:F)->F::Output{static RT:super::OnceLock<tokio::runtime::Runtime>=super::OnceLock::new();RT.get_or_init(||tokio::runtime::Runtime::new().unwrap()).block_on(f)}
}
#[derive(Clone)]struct Server{shared:Shared,hub:AppHandle}
fn allowed(h:&HeaderMap)->bool{
 h.get("x-edge-ipc").and_then(|x|x.to_str().ok())==Some("1") && h.get("origin").is_none_or(|v|matches!(v.to_str(),Ok("http://127.0.0.1:8787")|Ok("http://localhost:8787")|Ok("http://127.0.0.1:5173")|Ok("http://localhost:5173")))
}
async fn invoke(AxumState(s):AxumState<Server>,Path(name):Path<String>,h:HeaderMap,Json(v):Json<Value>)->Response{
 if !allowed(&h){return(StatusCode::FORBIDDEN,Json(json!({"error":"ORIGIN_REJECTED"}))).into_response()}
 match crate::dispatch(s.shared,s.hub,name,v).await{Ok(r)=>r,Err(e)=>(StatusCode::BAD_REQUEST,Json(json!({"error":e}))).into_response()}
}
async fn upload(AxumState(s):AxumState<Server>,h:HeaderMap,Json(v):Json<Value>)->Response{
 if !allowed(&h){return StatusCode::FORBIDDEN.into_response()}
 let result=tokio::task::spawn_blocking(move||->Res<Value>{
  let id=v["projectId"].as_str().ok_or("PROJECT_REQUIRED")?;let items=v["files"].as_array().ok_or("FILES_REQUIRED")?;
  let temp=tempfile::tempdir().map_err(|e|e.to_string())?;let mut paths=Vec::new();
  for (i,item) in items.iter().enumerate(){let name=PathBuf::from(item["name"].as_str().ok_or("NAME_REQUIRED")?);let file=name.file_name().ok_or("INVALID_NAME")?.to_string_lossy();let dir=temp.path().join(i.to_string());std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;let path=dir.join(file.as_ref());let bytes=base64::engine::general_purpose::STANDARD.decode(item["data"].as_str().ok_or("DATA_REQUIRED")?).map_err(|_|"INVALID_DATA")?;std::fs::write(&path,bytes).map_err(|e|e.to_string())?;paths.push(path.to_string_lossy().to_string());}
  s.shared.cancel.store(false,std::sync::atomic::Ordering::Relaxed);
  let report=crate::locked(&s.shared)?.import(id,paths,&s.shared.cancel,|done,total|{let _=s.hub.emit("job-progress",json!({"kind":"import","done":done,"total":total}));})?;
  serde_json::to_value(report).map_err(|e|e.to_string())
 }).await;
 match result{Ok(Ok(v))=>Json(v).into_response(),Ok(Err(e))=>(StatusCode::BAD_REQUEST,Json(json!({"error":e}))).into_response(),Err(e)=>(StatusCode::INTERNAL_SERVER_ERROR,Json(json!({"error":e.to_string()}))).into_response()}
}
async fn events(AxumState(s):AxumState<Server>)->impl IntoResponse{
 let rx=s.hub.events.subscribe();
 let stream=futures_util::stream::unfold(rx,|mut rx|async{loop{match rx.recv().await{Ok(v)=>return Some((Ok::<_,Infallible>(Event::default().data(v.to_string())),rx)),Err(broadcast::error::RecvError::Lagged(_))=>continue,Err(_)=>return None}}});
 Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
pub async fn serve(shared:Shared,root:PathBuf){
 let (sender,_)=broadcast::channel(128);let hub=AppHandle{events:sender};crate::hybrid::spawn(hub.clone(),shared.clone());
 let server=Server{shared,hub};let router=Router::new().route("/api/health",get(||async{Json(json!({"service":"edge-rust","version":"2.0.0","status":"ok"}))})).route("/api/invoke/{name}",post(invoke)).route("/api/upload",post(upload)).route("/api/events",get(events)).layer(DefaultBodyLimit::max(128*1024*1024)).with_state(server).fallback_service(tower_http::services::ServeDir::new(root.join("dist")));
 let port=std::env::var("EDGE_PORT").unwrap_or("8787".into());let listener=tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await.expect("EDGE port is available");println!("EDGE Studio 2: http://127.0.0.1:{port}");axum::serve(listener,router).await.unwrap();
}

