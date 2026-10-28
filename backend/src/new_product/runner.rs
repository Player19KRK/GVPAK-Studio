use super::*;
use futures_util::{stream::FuturesUnordered,StreamExt};
static ACTIVE:OnceLock<Mutex<HashMap<String,Arc<AtomicBool>>>>=OnceLock::new();
static CLOUD:OnceLock<Arc<tokio::sync::Semaphore>>=OnceLock::new();
fn key(root:&Path,id:&str)->String{format!("{}:{id}",root.display())}
pub(super) fn signal(root:&Path,id:&str){if let Some(active)=ACTIVE.get(){if let Some(flag)=active.lock().unwrap().get(&key(root,id)){flag.store(true,Ordering::Relaxed);}}}
fn ready(job:&Job,index:usize)->bool{let task=&job.tasks[index];task.state=="queued"&&task.depends_on.iter().all(|id|job.tasks.iter().any(|task|task.id==*id&&task.state=="done"))}
fn store_image(s:&Shared,job:&Job,index:usize,image:&image::DynamicImage,usage:&Value)->Res<ResultImage>{
 let task=&job.tasks[index];let store=locked(s)?;
 let (project,version_id)=store.save_generated(&job.project_id,&task.reference_ids[0],image,json!({"newProductJobId":job.id,"taskId":task.id,"kind":task.kind,"prompt":task.prompt,"referenceIds":task.reference_ids,"model":IMAGE,"ratio":job.ratio,"resolution":job.resolution,"usageMetadata":usage,"mode":job.mode}))?;
 let version=project.assets.iter().find(|asset|asset.id==task.reference_ids[0]).and_then(|asset|asset.versions.iter().find(|version|version.id==version_id)).ok_or("NEW_PRODUCT_SAVE_RESULT")?;
 let bytes=std::fs::read(store.root.join("projects").join(&job.project_id).join(&version.file)).map_err(|_|"NEW_PRODUCT_SAVE_RESULT")?;
 Ok(ResultImage{asset_id:task.reference_ids[0].clone(),version_id,name:format!("{:02}-{}.png",index+1,task.id),sha256:format!("{:x}",Sha256::digest(&bytes)),bytes:bytes.len()})
}
fn recover_results(s:&Shared,job:&mut Job)->Res<()>{
 let store=locked(s)?;let project=store.load(&job.project_id)?;
 for (index,task) in job.tasks.iter_mut().enumerate(){
  if task.result.is_some(){continue}
  if let Some((asset,version))=project.assets.iter().find_map(|asset|asset.versions.iter().find(|version|version.generation.as_ref().is_some_and(|value|value["newProductJobId"]==job.id&&value["taskId"]==task.id)).map(|version|(asset,version))){
   if let Ok(bytes)=std::fs::read(store.root.join("projects").join(&job.project_id).join(&version.file)){task.result=Some(ResultImage{asset_id:asset.id.clone(),version_id:version.id.clone(),name:format!("{:02}-{}.png",index+1,task.id),sha256:format!("{:x}",Sha256::digest(&bytes)),bytes:bytes.len()});task.state="done".into();continue}
  }
  if task.state=="running"||task.state=="submitting"{task.state="uncertain".into();task.error=Some("服务曾中断，请求结果待确认；不会自动重复提交。".into());}
 }
 Ok(())
}
fn finalize(job:&mut Job,stop:bool){
 for task in &mut job.tasks{if task.state=="queued"{task.state=if stop{"cancelled"}else{"blocked"}.into();task.error=Some(if stop{"已停止，未提交此图。"}else{"依赖的改款定稿未完成，未提交此图。"}.into());}}
 let done=job.tasks.iter().filter(|task|task.state=="done").count();
 job.status=if done==job.tasks.len(){"done"}else if job.tasks.iter().any(|task|task.state=="uncertain"){"attention"}else if done>0{"partial"}else if stop{"cancelled"}else{"failed"}.into();
}
fn accept(s:&Shared,hub:&AppHandle,job:&mut Job,index:usize,value:Res<Value>){
 match value.and_then(|value|{let image=google::decode_image(&value)?;store_image(s,job,index,&image,&value["usageMetadata"])}){
  Ok(result)=>{let aid=result.asset_id.clone();job.tasks[index].state="done".into();job.tasks[index].result=Some(result);job.tasks[index].error=None;if let Ok(project)=locked(s).and_then(|store|store.load(&job.project_id)){if let Some(asset)=project.assets.iter().find(|asset|asset.id==aid){let _=hub.emit("asset.updated",json!({"projectId":job.project_id,"assetId":aid,"patch":asset}));}}},
  Err(error)=>{job.tasks[index].state=if error.starts_with("NEW_PRODUCT_CANCELLED"){"cancelled"}else if error.starts_with("NETWORK_OUTCOME_UNKNOWN"){"uncertain"}else{"failed"}.into();job.tasks[index].error=Some(error);}
 }
}
async fn realtime(s:&Shared,hub:&AppHandle,root:&Path,job:&mut Job,stop:Arc<AtomicBool>,google:Arc<google::Google>)->Res<()>{
 let analysis=read_analysis(root,&job.analysis_id)?;let mut running=FuturesUnordered::new();
 loop{
  while running.len()<job.concurrency&&!stop.load(Ordering::Relaxed){
   let Some(index)=(0..job.tasks.len()).find(|index|ready(job,*index))else{break};
   if source(s,&job.project_id,&job.context)?!=analysis.fingerprints{return Err("NEW_PRODUCT_SOURCE_CHANGED".into())}
   let body=match google::generation_request(s,job,&job.tasks[index]){Ok(body)=>body,Err(error)=>{accept(s,hub,job,index,Err(error));save(root,job,hub)?;continue}};
   job.tasks[index].state="running".into();job.status="running".into();save(root,job,hub)?;
   let google=google.clone();let stop=stop.clone();
   running.push(async move{
    let semaphore=CLOUD.get_or_init(||Arc::new(tokio::sync::Semaphore::new(4))).clone();let _permit=semaphore.acquire_owned().await;
    let result=if stop.load(Ordering::Relaxed){Err("NEW_PRODUCT_CANCELLED".into())}else{google.call(reqwest::Method::POST,&format!("models/{IMAGE}:generateContent"),Some(&body)).await};(index,result)
   });
  }
  let Some((index,result))=running.next().await else{break};accept(s,hub,job,index,result);if stop.load(Ordering::Relaxed){job.status="cancelling".into()}save(root,job,hub)?;
 }
 finalize(job,stop.load(Ordering::Relaxed));Ok(())
}
async fn batch(s:&Shared,hub:&AppHandle,root:&Path,job:&mut Job,stop:Arc<AtomicBool>)->Res<()>{
 let google=google::Google::new()?;let analysis=read_analysis(root,&job.analysis_id)?;let mut cancelled_groups=HashSet::new();
 loop{
  if stop.load(Ordering::Relaxed){for group in &job.batches{if !cancelled_groups.contains(&group.name){google::resource(&group.name,"batches/")?;if google.call(reqwest::Method::POST,&format!("{}:cancel",group.name),None).await.is_ok(){cancelled_groups.insert(group.name.clone());}}}job.status="cancelling".into();}
  for group in job.batches.clone(){
   google::resource(&group.name,"batches/")?;
   let value=match google.call(reqwest::Method::GET,&group.name,None).await{Ok(value)=>value,Err(error)=>{job.error=Some(error);save(root,job,hub)?;continue}};
   let state=value["metadata"]["state"].as_str().unwrap_or("");let finished=value["done"]==true||["SUCCEEDED","FAILED","CANCELLED","EXPIRED"].iter().any(|suffix|state.ends_with(suffix));if !finished{continue}
   let entries=if value["error"].is_null()&&!state.ends_with("CANCELLED")&&!state.ends_with("FAILED") { google.batch_entries(&value).await.ok() }else{None};
   if let Some(entries)=entries{for (key,entry) in crate::hybrid::keyed(&entries,&group.task_ids)?{if let Some(index)=job.tasks.iter().position(|task|task.id==key){if job.tasks[index].state!="done"{accept(s,hub,job,index,Ok(entry));}}}}
   for tid in &group.task_ids{if let Some(task)=job.tasks.iter_mut().find(|task|task.id==*tid&&task.state=="submitted"){task.state=if state.ends_with("CANCELLED")||stop.load(Ordering::Relaxed){"cancelled"}else{"failed"}.into();task.error=Some("批量任务未返回该图片，请检查任务记录。".into());}}
   job.batches.retain(|value|value.name!=group.name);save(root,job,hub)?;
  }
  if !stop.load(Ordering::Relaxed){loop{
   let indices=(0..job.tasks.len()).filter(|index|ready(job,*index)).take(4).collect::<Vec<_>>();if indices.is_empty(){break}
   if source(s,&job.project_id,&job.context)?!=analysis.fingerprints{return Err("NEW_PRODUCT_SOURCE_CHANGED".into())}
   let mut requests=vec![];let mut valid=vec![];
   for index in indices{match google::generation_request(s,job,&job.tasks[index]){Ok(request)=>{requests.push(json!({"request":request,"metadata":{"key":job.tasks[index].id}}));valid.push(index)},Err(error)=>accept(s,hub,job,index,Err(error))}}
   if valid.is_empty(){save(root,job,hub)?;continue}
   for index in &valid{job.tasks[*index].state="submitting".into()}job.status="submitting".into();save(root,job,hub)?;
   let body=json!({"batch":{"displayName":format!("gvpak-new-{}-{}",job.id,job.tasks[valid[0]].id),"inputConfig":{"requests":{"requests":requests}}}});
   match google.call(reqwest::Method::POST,&format!("models/{IMAGE}:batchGenerateContent"),Some(&body)).await{
    Ok(value)=>{let name=value["name"].as_str().ok_or("NEW_PRODUCT_BATCH_ID")?;google::resource(name,"batches/")?;let ids=valid.iter().map(|index|job.tasks[*index].id.clone()).collect();for index in valid{job.tasks[index].state="submitted".into()}job.batches.push(Batch{name:name.into(),task_ids:ids});job.status="waiting".into();},
    Err(error)=>for index in valid{accept(s,hub,job,index,Err(error.clone()));},
   }save(root,job,hub)?;
  }}
  if job.batches.is_empty(){break}
  tokio::time::sleep(std::time::Duration::from_secs(5)).await;
 }
 finalize(job,stop.load(Ordering::Relaxed));Ok(())
}
pub(super) fn spawn(s:Shared,hub:AppHandle,root:PathBuf,mut job:Job){
 let key=key(&root,&job.id);let stop=Arc::new(AtomicBool::new(cancelled(&root,&job.id)));let map=ACTIVE.get_or_init(||Mutex::new(HashMap::new()));
 {let mut active=map.lock().unwrap();if active.contains_key(&key){return}active.insert(key.clone(),stop.clone());}
 tokio::spawn(async move{
  let result=async{recover_results(&s,&mut job)?;save(&root,&mut job,&hub)?;if job.mode=="batch"{batch(&s,&hub,&root,&mut job,stop).await}else{realtime(&s,&hub,&root,&mut job,stop,Arc::new(google::Google::new()?)).await}}.await;
  if let Err(error)=result{for task in &mut job.tasks{if matches!(task.state.as_str(),"running"|"submitting"){task.state="uncertain".into();task.error=Some("已提交的请求结果待确认，不会自动再次提交。".into());}}job.error=Some(error);let stop=cancelled(&root,&job.id);finalize(&mut job,stop);}
  let _=save(&root,&mut job,&hub);ACTIVE.get().unwrap().lock().unwrap().remove(&key);
 });
}
#[cfg(test)]mod tests{
 use super::*;use super::super::tests::{setup,analysis,task,job};use std::sync::atomic::AtomicUsize;use base64::Engine;
 #[tokio::test]async fn realtime_parallelism_and_shared_design_anchor_use_only_local_provider_fixtures(){
  #[derive(Clone)]struct Provider{active:Arc<AtomicUsize>,peak:Arc<AtomicUsize>,requests:Arc<Mutex<Vec<Value>>>,image:String}
  let(_temp,s,pid,aid,context)=setup();let root=folder(&s).unwrap();let analysis=analysis(&s,&pid,&aid,&context);atomic(&root.join("analyses").join(format!("{}.json",analysis.id)),&analysis).unwrap();
  let image=core::png(&image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(8,12,image::Rgba([40,80,100,255])))).unwrap();
  let provider=Provider{active:Arc::new(AtomicUsize::new(0)),peak:Arc::new(AtomicUsize::new(0)),requests:Arc::new(Mutex::new(vec![])),image:base64::engine::general_purpose::STANDARD.encode(image)};
  let app=axum::Router::new().fallback(axum::routing::post(|axum::extract::State(p):axum::extract::State<Provider>,axum::Json(body):axum::Json<Value>|async move{let active=p.active.fetch_add(1,Ordering::Relaxed)+1;p.peak.fetch_max(active,Ordering::Relaxed);p.requests.lock().unwrap().push(body);tokio::time::sleep(std::time::Duration::from_millis(40)).await;p.active.fetch_sub(1,Ordering::Relaxed);axum::Json(json!({"candidates":[{"content":{"parts":[{"inlineData":{"mimeType":"image/png","data":p.image}}]}}]}))})).layer(axum::extract::DefaultBodyLimit::max(20*1024*1024)).with_state(provider.clone());
  let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let base=format!("http://{}/v1beta",listener.local_addr().unwrap());let server=tokio::spawn(async move{axum::serve(listener,app).await.unwrap()});
  let mut anchor=task("anchor",&aid);anchor.anchor=true;let mut tasks=vec![anchor];for index in 0..3{let mut next=task(&format!("scene-{index}"),&aid);next.kind="scene".into();next.depends_on=vec!["anchor".into()];tasks.push(next)}let mut job=job(&analysis,tasks);let(events,_)=tokio::sync::broadcast::channel(16);
  realtime(&s,&AppHandle{events},&root,&mut job,Arc::new(AtomicBool::new(false)),Arc::new(google::Google::testing(base))).await.unwrap();
  assert_eq!(job.status,"done","{}",view(&job));assert!(job.tasks.iter().all(|task|task.result.is_some()));assert_eq!(provider.peak.load(Ordering::Relaxed),3);let requests=provider.requests.lock().unwrap();assert_eq!(requests.len(),4);assert_eq!(requests[0]["contents"][0]["parts"].as_array().unwrap().iter().filter(|part|part["inlineData"].is_object()).count(),1);assert!(requests[1..].iter().all(|request|request["contents"][0]["parts"].as_array().unwrap().iter().filter(|part|part["inlineData"].is_object()).count()==2));
  assert_eq!(source(&s,&pid,&context).unwrap(),analysis.fingerprints);server.abort();
 }
 #[test]fn restart_recovers_saved_images_without_resubmitting_and_flags_uncertain_requests(){
  let(_temp,s,pid,aid,context)=setup();let analysis=analysis(&s,&pid,&aid,&context);let mut job=job(&analysis,vec![task("saved",&aid),task("uncertain",&aid),task("pending",&aid)]);let image=image::DynamicImage::new_rgb8(8,12);store_image(&s,&job,0,&image,&json!({})).unwrap();job.tasks[0].state="running".into();job.tasks[1].state="running".into();recover_results(&s,&mut job).unwrap();assert_eq!(job.tasks[0].state,"done");assert_eq!(job.tasks[1].state,"uncertain");finalize(&mut job,true);assert_eq!(job.status,"attention");assert_eq!(job.tasks[2].state,"cancelled");assert_eq!(locked(&s).unwrap().load(&pid).unwrap().assets[0].versions.len(),1);
 }
}
