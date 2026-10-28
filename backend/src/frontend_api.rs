//! Additive P4 read/cancel contracts; old commands keep their existing behavior.
use crate::{Shared,Res,locked,transport::AppHandle};
use serde_json::{Value,json};
use axum::response::{Response,IntoResponse};
fn state(value:&str)->&str{match value{"completed"=>"done","attention"=>"failed",other=>other}}
pub(crate) fn list(s:&Shared)->Res<Vec<Value>> {
 let root=locked(s)?.root.clone();let operations=crate::frontend_ops::list(s)?;let operation_ids=operations.iter().filter_map(|task|task["id"].as_str()).collect::<std::collections::HashSet<_>>();let mut tasks=vec![];
 for job in crate::hybrid::list(&root)? {if operation_ids.contains(job.id.as_str()){continue}
  tasks.push(json!({"id":job.id,"type":"hybrid","projectId":job.project_id,"accountId":"host","state":state(&job.state),"targetCount":job.items.len(),"done":job.items.iter().filter(|item|item.stage=="done").count(),"total":job.items.len(),"queuePosition":null,"startedAt":null,"updatedAt":job.updated_at*1000,"resultAssetIds":job.items.iter().filter(|item|item.stage=="done").map(|item|item.asset_id.clone()).collect::<Vec<_>>(),"error":job.error}));
 }
 let directory=root.join("new-product-jobs");if directory.is_dir(){for entry in std::fs::read_dir(directory).map_err(|e|e.to_string())?.flatten(){if entry.path().extension().is_none_or(|ext|ext!="json"){continue}let Ok(bytes)=std::fs::read(entry.path()) else{continue};let Ok(job)=serde_json::from_slice::<Value>(&bytes) else{continue};let Some(items)=job["tasks"].as_array() else{continue};tasks.push(json!({"id":job["id"],"type":"new-product","projectId":job["projectId"],"accountId":"host","state":state(job["status"].as_str().unwrap_or("failed")),"targetCount":items.len(),"done":items.iter().filter(|item|item["state"]=="done").count(),"total":items.len(),"queuePosition":null,"startedAt":null,"updatedAt":job["updatedAt"].as_u64().unwrap_or(0)*1000,"resultAssetIds":items.iter().filter_map(|item|item["result"]["assetId"].as_str()).collect::<Vec<_>>(),"error":job["error"]}));}}
 tasks.extend(operations);
 tasks.sort_by_key(|task|std::cmp::Reverse(task["updatedAt"].as_u64().unwrap_or(0)));Ok(tasks)
}
pub(crate) async fn dispatch(s:Shared,hub:AppHandle,name:&str,args:Value)->Res<Response> {
 match name {
  "selection_export_preview"=>{let value=crate::transport::runtime::spawn_blocking(move||crate::frontend_export::preview(&s,args)).await.map_err(|_|"PREVIEW_UNAVAILABLE")??;Ok(axum::Json(value).into_response())},
  "assistant_context_plan"=>{let pid=args["projectId"].as_str().ok_or("PROJECT_REQUIRED")?;crate::frontend_ops::context_ids(&s,pid,&args["context"],false)?;if cfg!(test){return Err("TEST_PROVIDER_REQUIRED".into())}Ok(axum::Json(crate::assistant::plan(args["instruction"].as_str().ok_or("INSTRUCTION_REQUIRED")?.into(),args["context"].clone()).await?).into_response())},
  "operation_result"=>Ok(([(axum::http::header::CONTENT_TYPE,"application/octet-stream")],crate::frontend_output::result(&s,args["taskId"].as_str().ok_or("TASK_REQUIRED")?,args["assetId"].as_str().ok_or("ASSET_REQUIRED")?)?).into_response()),
  "settings_read"=>{let mut value=serde_json::to_value(crate::frontend_settings::read(&s)?).map_err(|_|"SETTINGS_UNAVAILABLE")?;value["usage"]=crate::team::quota_usage(&s)?;Ok(axum::Json(value).into_response())},
  "settings_patch"=>{let saved=crate::frontend_settings::patch(&s,&args["patch"])?;hub.emit("session.updated",json!({"quota":crate::frontend_settings::quota(&s)?}))?;Ok(axum::Json(saved).into_response())},
  "execution_preferences"=>{let settings=crate::frontend_settings::read(&s)?;Ok(axum::Json(json!({"analysisCount":settings.analysis_count,"generationCount":settings.generation_count,"costCny":settings.cost_cny,"filterCount":settings.filter_count,"imageModel":settings.image_model,"quota":crate::frontend_settings::quota(&s)?})).into_response())},
  "session_status"=>Ok(axum::Json(json!({"account":null,"isHost":true,"isLocal":true,"quota":crate::frontend_settings::quota(&s)?})).into_response()),
  "task_list"=>Ok(axum::Json(json!({"tasks":list(&s)?})).into_response()),
  "assets_metadata"=>{let pid=args["projectId"].as_str().ok_or("PROJECT_REQUIRED")?;let entries:Vec<crate::frontend_assets::Entry>=serde_json::from_value(args["entries"].clone()).map_err(|_|"INVALID_METADATA")?;let ids=entries.iter().map(|entry|entry.asset_id.clone()).collect::<Vec<_>>();let project=locked(&s)?.metadata(pid,entries)?;for asset in project.assets.iter().filter(|a|ids.contains(&a.id)){hub.emit("asset.updated",json!({"projectId":pid,"assetId":asset.id,"patch":asset}))?;}Ok(axum::Json(json!({"project":project})).into_response())},
  "operation_execute"=>Ok(axum::Json(crate::frontend_ops::execute(s,hub,args.clone(),args["_frontendHost"].as_bool().unwrap_or(true))?).into_response()),
  "chat_execute"=>{
   let pid=args["projectId"].as_str().ok_or("PROJECT_REQUIRED")?;crate::frontend_ops::context_ids(&s,pid,&args["context"],false)?;
   let rid=args["clientRequestId"].as_str().ok_or("REQUEST_ID_REQUIRED")?;uuid::Uuid::parse_str(rid).map_err(|_|"INVALID_REQUEST_ID")?;
   let key=crate::frontend_ops::plan_key(&args);let cached=crate::frontend_ops::cached_plan(&s,rid,&key)?;
   if let Some(task)=crate::frontend_ops::existing(&s,rid)?{if cached.is_none(){return Err("REQUEST_ID_CONFLICT".into())}return Ok(axum::Json(json!({"task":task,"batchId":rid})).into_response())}
   let plan=if let Some(plan)=cached{plan}else{
    if args["confirmToken"].is_string(){return Err("CONFIRMATION_EXPIRED".into())}
    let root=crate::frontend_ops::planning_context(&s,rid,pid,&args["context"])?;let cancel=root.join(format!("{rid}.cancel"));
    let outcome=tokio::select!{result=async{if args["pipeline"]==true{let mode=args["execution"].as_str().unwrap_or("realtime");if !matches!(mode,"batch"|"realtime"){return Err("INVALID_PIPELINE_MODE".into())}crate::frontend_pipeline::plan(&s,args["instruction"].as_str().ok_or("INSTRUCTION_REQUIRED")?,&args["context"],mode,args["model"].as_str(),rid,&hub).await}else{crate::frontend_chat::plan(args["instruction"].as_str().ok_or("INSTRUCTION_REQUIRED")?,&args["context"],rid,&hub).await}}=>result,_=async{loop{if cancel.exists(){break}tokio::time::sleep(std::time::Duration::from_millis(50)).await;}}=>Err("TASK_CANCELLED".into())};
    let _=std::fs::remove_file(root.join(format!("{rid}.planning")));let plan=outcome?;if cancel.exists(){return Err("TASK_CANCELLED".into())}crate::frontend_ops::cache_plan(&s,rid,&key,&plan)?;plan
   };
   hub.emit("chat.plan",json!({"requestId":rid,"operations":plan["operations"],"text":plan["text"]}))?;
   if plan["operations"].as_array().is_none_or(|ops|ops.is_empty()){return Ok(axum::Json(json!({"text":plan["text"]})).into_response())}
   let mut input=args.clone();input["operations"]=plan["operations"].clone();Ok(axum::Json(crate::frontend_ops::execute(s,hub,input,args["_frontendHost"].as_bool().unwrap_or(true))?).into_response())
  },
  "task_stop"=>{let id=args["taskId"].as_str().ok_or("TASK_REQUIRED")?;let task=match list(&s)?.into_iter().find(|task|task["id"]==id){Some(task)=>task,None=>{let task=crate::frontend_ops::stop(&s,id)?;return Ok(axum::Json(json!({"task":task})).into_response())}};
   if matches!(task["state"].as_str(),Some("done"|"partial"|"failed"|"cancelled")){return Ok(axum::Json(json!({"task":task})).into_response())}
   if task["type"]=="operation"{let result=crate::frontend_ops::stop(&s,id)?;hub.emit("task.progress",&result)?;return Ok(axum::Json(json!({"task":result})).into_response())}
   if task["type"]=="new-product"{crate::new_product::invoke(s.clone(),hub.clone(),"new_product_cancel",json!({"jobId":id})).await?;}else{crate::hybrid::request_cancel(&locked(&s)?.root,id)?;}
   let mut next=task;next["state"]=json!("cancelling");hub.emit("task.progress",&next)?;Ok(axum::Json(json!({"task":next})).into_response())
  },
  _=>Err("UNKNOWN_COMMAND".into())
 }
}
#[cfg(test)]mod tests{
 use super::*;
 fn setup()->(tempfile::TempDir,Shared){let temp=tempfile::tempdir().unwrap();let store=crate::core::Store::open(temp.path().join("data"),"test").unwrap();let s=std::sync::Arc::new(crate::AppState{store:std::sync::Mutex::new(store),cancel:std::sync::atomic::AtomicBool::new(false),local_job:std::sync::Mutex::new(()),preview_epoch:std::sync::atomic::AtomicU64::new(0)});(temp,s)}
 #[tokio::test]async fn task_snapshot_has_no_context_paths_and_isolates_workspaces(){
  let(_one,s)=setup();let(_two,other)=setup();let root=locked(&s).unwrap().root.join("new-product-jobs");std::fs::create_dir_all(&root).unwrap();let id=uuid::Uuid::new_v4().to_string();
  let job=json!({"id":id,"projectId":"p","status":"done","updatedAt":1,"tasks":[{"state":"done","result":{"assetId":"a","name":"test.png"}}],"context":{"fileNames":["private.png"]},"outputDir":"private-path"});std::fs::write(root.join(format!("{id}.json")),job.to_string()).unwrap();
  let tasks=list(&s).unwrap();assert_eq!(tasks.len(),1);assert_eq!(tasks[0]["done"],1);assert_eq!(tasks[0]["resultAssetIds"],json!(["a"]));assert_eq!(tasks[0]["queuePosition"],Value::Null);assert!(!tasks[0].to_string().contains("private"));assert!(list(&other).unwrap().is_empty());
  let(events,_)=tokio::sync::broadcast::channel(8);let hub=AppHandle{events};assert!(dispatch(other,hub.clone(),"task_stop",json!({"taskId":id})).await.is_err());
  let response=dispatch(s,hub,"task_stop",json!({"taskId":id})).await.unwrap();let bytes=axum::body::to_bytes(response.into_body(),4096).await.unwrap();let value:Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(value["task"]["state"],"done");assert!(!root.join(format!("{id}.cancel")).exists());
 }

 #[test]fn legacy_terminal_states_are_translated_without_faking_queue_positions(){assert_eq!(state("completed"),"done");assert_eq!(state("queued"),"queued");assert_eq!(state("attention"),"failed");}
 #[test]fn cached_scene_plan_confirms_without_replanning_or_repeating_request(){crate::transport::runtime::block_on(async{
  let(temp,s)=setup();let project=locked(&s).unwrap().create("cached scene".into()).unwrap();let files=(0..5).map(|i|{let p=temp.path().join(format!("ref-{i}.png"));image::RgbaImage::from_pixel(8,8,image::Rgba([20+i,60,90,255])).save(&p).unwrap();p.to_string_lossy().to_string()}).collect();let project=locked(&s).unwrap().import(&project.id,files,&std::sync::atomic::AtomicBool::new(false),|_,_|{}).unwrap().project;let ids=project.assets.iter().map(|a|a.id.clone()).collect::<Vec<_>>();let context=json!({"page":"home","mentions":[],"selected":ids,"editorAssetId":null,"filter":{"folder":"","tag":"","status":"","q":"","sort":""},"filteredCount":5,"resolvedTarget":{"source":"selected","ids":ids},"fileNames":project.assets.iter().map(|a|a.name.clone()).collect::<Vec<_>>()});let id=uuid::Uuid::new_v4().to_string();let mut args=json!({"projectId":project.id,"context":context,"instruction":"生成场景图","clientRequestId":id});let key=crate::frontend_ops::plan_key(&args);let plan=json!({"text":"","operations":[{"kind":"generate","prompt":"clean Scandinavian still life preserving product","ratio":"1:1","orientation":"landscape","resolution":"1K"}]});crate::frontend_ops::cache_plan(&s,&id,&key,&plan).unwrap();let(events,_)=tokio::sync::broadcast::channel(128);let hub=AppHandle{events};let first=dispatch(s.clone(),hub.clone(),"chat_execute",args.clone()).await.unwrap();let bytes=axum::body::to_bytes(first.into_body(),65536).await.unwrap();let response:Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(response["confirmation"]["count"],5);args["confirmToken"]=response["confirmation"]["token"].clone();let approved=dispatch(s.clone(),hub.clone(),"chat_execute",args.clone()).await.unwrap();let bytes=axum::body::to_bytes(approved.into_body(),65536).await.unwrap();let response:Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(response["task"]["id"],id);for _ in 0..1500{if list(&s).unwrap().iter().any(|t|t["id"]==id&&t["state"]=="failed"){break}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}let tasks=list(&s).unwrap();let task=tasks.iter().find(|t|t["id"]==id).unwrap();assert_eq!(task["state"],"failed");assert_eq!(task["failed"],5);dispatch(s.clone(),hub,"chat_execute",args).await.unwrap();assert_eq!(list(&s).unwrap().len(),1);
 });}
}
