//! Independent new-product workflow. Existing editing / hybrid commands are unchanged.
mod google;
mod runner;
use crate::{locked,core,Shared,Res,transport::AppHandle};
use axum::response::{IntoResponse,Response};
use serde::{Serialize,Deserialize};
use serde_json::{Value,json};
use sha2::{Sha256,Digest};
use std::{path::{Path,PathBuf},collections::{HashMap,HashSet,BTreeMap},sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,Ordering}},io::Write};
pub const VISION:&str="gemini-3.8-flash";
pub const IMAGE:&str="gemini-nano-banana-2.1";
pub(super) fn now()->u64{std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()}
pub(super) fn digest(v:&impl Serialize)->String{format!("{:x}",Sha256::digest(serde_json::to_vec(v).unwrap_or_default()))}
fn clean_name(s:&str)->String{s.rsplit(['/', '\\']).next().unwrap_or("").to_string()}
fn id(v:&Value,key:&str)->Res<String>{let s=v[key].as_str().ok_or_else(||format!("NEW_PRODUCT_ARGUMENT: {key}"))?;uuid::Uuid::parse_str(s).map_err(|_|"NEW_PRODUCT_INVALID_ID")?;Ok(s.into())}
pub(super) fn folder(s:&Shared)->Res<PathBuf>{let root=locked(s)?.root.join("new-product-jobs");std::fs::create_dir_all(root.join("analyses")).map_err(|_|"NEW_PRODUCT_STORAGE")?;Ok(root)}
pub(super) fn atomic(path:&Path,value:&impl Serialize)->Res<()>{let mut file=tempfile::NamedTempFile::new_in(path.parent().ok_or("NEW_PRODUCT_STORAGE")?).map_err(|e|e.to_string())?;file.write_all(&serde_json::to_vec_pretty(value).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;file.as_file().sync_all().map_err(|e|e.to_string())?;crate::frontend_ops::persist_temp(file,path.to_path_buf())}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(super) struct Analysis{pub id:String,pub project_id:String,pub context:Value,pub fingerprints:BTreeMap<String,String>,pub data:Value,pub brief:String,pub facts:String}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(super) struct ResultImage{pub asset_id:String,pub version_id:String,pub name:String,pub sha256:String,pub bytes:usize}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(super) struct Task{
 pub id:String,pub kind:String,pub product_id:String,pub title:String,pub prompt:String,pub reference_ids:Vec<String>,
 #[serde(default)]pub depends_on:Vec<String>,#[serde(default="yes")]pub enabled:bool,#[serde(default)]pub anchor:bool,
 #[serde(default)]pub state:String,#[serde(default)]pub error:Option<String>,#[serde(default)]pub result:Option<ResultImage>,
}
fn yes()->bool{true}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(super) struct Batch{pub name:String,pub task_ids:Vec<String>}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub(super) struct Job{
 pub id:String,pub project_id:String,pub analysis_id:String,pub request_hash:String,pub context:Value,pub tasks:Vec<Task>,
 pub mode:String,pub concurrency:usize,pub resolution:String,pub ratio:String,pub status:String,pub created_at:u64,pub updated_at:u64,
 pub estimated_cny:Option<f64>,#[serde(default)]pub batches:Vec<Batch>,#[serde(default)]pub error:Option<String>,
}
pub(super) fn terminal(s:&str)->bool{matches!(s,"done"|"partial"|"failed"|"cancelled"|"attention")}
pub(super) fn job_path(root:&Path,id:&str)->PathBuf{root.join(format!("{id}.json"))}
pub(super) fn cancelled(root:&Path,id:&str)->bool{root.join(format!("{id}.cancel")).exists()}
pub(super) fn save(root:&Path,job:&mut Job,hub:&AppHandle)->Res<()>{job.updated_at=now();atomic(&job_path(root,&job.id),job)?;let _=hub.emit("new-product.progress",json!({"jobId":job.id,"state":job.status,"done":job.tasks.iter().filter(|task|task.state=="done").count(),"total":job.tasks.len()}));Ok(())}
pub(super) fn view(job:&Job)->Value{json!({"id":job.id,"projectId":job.project_id,"analysisId":job.analysis_id,"mode":job.mode,"concurrency":job.concurrency,"resolution":job.resolution,"ratio":job.ratio,"status":job.status,"createdAt":job.created_at,"updatedAt":job.updated_at,"estimatedCny":job.estimated_cny,"tasks":job.tasks,"error":job.error})}
pub(super) fn read_job(root:&Path,id:&str)->Res<Job>{uuid::Uuid::parse_str(id).map_err(|_|"NEW_PRODUCT_INVALID_ID")?;serde_json::from_slice(&std::fs::read(job_path(root,id)).map_err(|_|"NEW_PRODUCT_JOB_NOT_FOUND")?).map_err(|_|"NEW_PRODUCT_JOB_CORRUPT".into())}
fn read_analysis(root:&Path,id:&str)->Res<Analysis>{uuid::Uuid::parse_str(id).map_err(|_|"NEW_PRODUCT_INVALID_ID")?;let mut analysis:Analysis=serde_json::from_slice(&std::fs::read(root.join("analyses").join(format!("{id}.json"))).map_err(|_|"NEW_PRODUCT_ANALYSIS_NOT_FOUND")?).map_err(|_|"NEW_PRODUCT_ANALYSIS_CORRUPT")?;ensure_questions(&mut analysis.data);Ok(analysis)}
fn source(s:&Shared,pid:&str,context:&Value)->Res<BTreeMap<String,String>>{
 let object=context.as_object().ok_or("NEW_PRODUCT_CONTEXT")?;
 let keys=["page","mentions","selected","editorAssetId","filter","filteredCount","resolvedTarget","fileNames"];
 if object.len()!=keys.len()||object.keys().any(|key|!keys.contains(&key.as_str())){return Err("NEW_PRODUCT_CONTEXT".into())}
 if !["home","library","editor"].contains(&context["page"].as_str().unwrap_or(""))||!["mention","selected","editor","filter"].contains(&context["resolvedTarget"]["source"].as_str().unwrap_or("")){return Err("NEW_PRODUCT_CONTEXT".into())}
 let filter=context["filter"].as_object().ok_or("NEW_PRODUCT_CONTEXT")?;if filter.len()!=5||["folder","tag","status","q","sort"].iter().any(|key|!filter.get(*key).is_some_and(Value::is_string))||context["filter"]["folder"].as_str().unwrap_or("").contains(['/', '\\', ':']){return Err("NEW_PRODUCT_CONTEXT".into())}
 let ids=context["resolvedTarget"]["ids"].as_array().ok_or("NEW_PRODUCT_CONTEXT")?;
 if ids.is_empty()||ids.len()>40{return Err("NEW_PRODUCT_INPUT_LIMIT: 单次需要1至40张不同素材".into())}
 let names=context["fileNames"].as_array().ok_or("NEW_PRODUCT_CONTEXT")?;
 if names.len()!=ids.len()||names.iter().any(|v|v.as_str().is_none_or(|s|s.contains(['/', '\\', ':']))){return Err("NEW_PRODUCT_FILE_NAMES".into())}
 let project=locked(s)?.load(pid)?;let mut result=BTreeMap::new();
 for id in ids{let id=id.as_str().ok_or("NEW_PRODUCT_CONTEXT")?;let asset=project.assets.iter().find(|asset|asset.id==id&&!asset.trashed).ok_or("ASSET_NOT_FOUND")?;if result.insert(id.into(),digest(&json!([asset.sha256,core::Edit::default(),Option::<String>::None]))).is_some(){return Err("DUPLICATE_ASSET".into())}}
 Ok(result)
}
fn require_shape(data:&Value,ids:&BTreeMap<String,String>)->Res<()>{
 let products=data["products"].as_array().filter(|v|!v.is_empty()).ok_or("NEW_PRODUCT_ANALYSIS_INVALID")?;
 let product_ids:HashSet<&str>=products.iter().filter_map(|p|p["id"].as_str()).collect();
 let items=data["items"].as_array().ok_or("NEW_PRODUCT_ANALYSIS_INVALID")?;let mut seen=HashSet::new();
 for item in items{let aid=item["assetId"].as_str().ok_or("NEW_PRODUCT_ANALYSIS_INVALID")?;if !ids.contains_key(aid)||!seen.insert(aid)||!product_ids.contains(item["productId"].as_str().unwrap_or("")){return Err("NEW_PRODUCT_ANALYSIS_REFERENCES".into())}}
 if seen.len()!=ids.len(){return Err("NEW_PRODUCT_ANALYSIS_INCOMPLETE".into())}
 for product in products{for field in ["sceneIds","humanIds","explodedIds"]{let refs=product[field].as_array().ok_or("NEW_PRODUCT_ANALYSIS_INVALID")?;for id in refs{if !items.iter().any(|item|item["assetId"]==*id&&item["productId"]==product["id"]){return Err("NEW_PRODUCT_ANALYSIS_REFERENCES".into())}}}}
 for q in data["questions"].as_array().ok_or("NEW_PRODUCT_ANALYSIS_INVALID")?{if !product_ids.contains(q["productId"].as_str().unwrap_or("")){return Err("NEW_PRODUCT_ANALYSIS_QUESTION".into())}for id in q["assetIds"].as_array().ok_or("NEW_PRODUCT_ANALYSIS_QUESTION")?{if !id.as_str().is_some_and(|id|ids.contains_key(id)){return Err("NEW_PRODUCT_ANALYSIS_QUESTION".into())}}}
 Ok(())
}
fn ensure_questions(data:&mut Value){
 let products=data["products"].as_array().cloned().unwrap_or_default();let items=data["items"].as_array().cloned().unwrap_or_default();
 let Some(questions)=data["questions"].as_array_mut()else{return};
 for question in questions.iter_mut(){if question["kind"]=="contents"{question["options"]=json!([{"id":"add","label":"加入膏体/内容物"},{"id":"empty","label":"保持空托"},{"id":"custom","label":"我来补充"}]);question["required"]=json!(true);}else if question["kind"]=="exploded"{question["options"]=json!([{"id":"visible","label":"只用已见部件，我会指定参考图"},{"id":"skip","label":"本次跳过爆炸图"}]);}}
 for product in products{let pid=product["id"].as_str().unwrap_or("p");
  for question in questions.iter_mut().filter(|question|question["productId"]==pid){if question["kind"]=="exploded"&&product["explodedReady"]!=true{question["required"]=json!(true);question["affectedKinds"]=json!(["exploded"]);}if question["kind"]=="contents"&&items.iter().any(|item|item["productId"]==pid&&item["role"]=="refill"&&item["contents"]=="empty"){question["required"]=json!(true);question["affectedKinds"]=json!(["white","scene","human","exploded"]);}}

  if items.iter().any(|item|item["productId"]==pid&&item["role"]=="refill"&&item["contents"]=="empty")&&!questions.iter().any(|q|q["productId"]==pid&&q["kind"]=="contents"){
   questions.push(json!({"id":format!("contents-{pid}"),"productId":pid,"kind":"contents","question":"识别到空的替换装纸塑内托，成品图是否需要加入膏体或内容物？","options":[{"id":"add","label":"加入膏体/内容物"},{"id":"empty","label":"保持空托"},{"id":"custom","label":"我来补充"}],"assetIds":items.iter().filter(|i|i["productId"]==pid&&i["role"]=="refill").map(|i|i["assetId"].clone()).collect::<Vec<_>>(),"affectedKinds":["white","scene","human","exploded"],"required":true}));
  }
  if product["explodedReady"]!=true&&!questions.iter().any(|q|q["productId"]==pid&&q["kind"]=="exploded"){questions.push(json!({"id":format!("exploded-{pid}"),"productId":pid,"kind":"exploded","question":"拆解结构证据不足，爆炸图如何处理？","options":[{"id":"visible","label":"只用已见部件，我会指定参考图"},{"id":"skip","label":"本次跳过爆炸图"}],"assetIds":product["explodedIds"],"affectedKinds":["exploded"],"required":true}));}
 }
}
fn validate_tasks(tasks:&[Task],analysis:&Analysis,answers:&Value)->Res<()>{
 if tasks.is_empty()||tasks.len()>64{return Err("NEW_PRODUCT_TASK_LIMIT".into())}
 let ids:HashSet<&str>=tasks.iter().map(|task|task.id.as_str()).collect();if ids.len()!=tasks.len(){return Err("NEW_PRODUCT_DUPLICATE_TASK".into())}
 for task in tasks{
  if task.id.is_empty()||task.id.len()>80||!task.id.bytes().all(|c|c.is_ascii_alphanumeric()||b"-_".contains(&c))||!matches!(task.kind.as_str(),"white"|"scene"|"human"|"exploded")||task.prompt.trim().is_empty()||task.prompt.len()>24000||task.reference_ids.is_empty()||task.reference_ids.len()>13||task.reference_ids.len()+task.depends_on.len()>14{return Err("NEW_PRODUCT_TASK_INVALID".into())}
  for aid in &task.reference_ids{if !analysis.data["items"].as_array().unwrap().iter().any(|item|item["assetId"]==*aid&&item["productId"]==task.product_id){return Err("NEW_PRODUCT_REFERENCE_INVALID".into())}}
  for dependency in &task.depends_on{if !tasks.iter().any(|other|other.id==*dependency&&other.product_id==task.product_id&&other.anchor&&other.kind=="white")||*dependency==task.id{return Err("NEW_PRODUCT_DEPENDENCY_INVALID".into())}}
 }
 let mut done=HashSet::new();for _ in 0..tasks.len(){for task in tasks{if task.depends_on.iter().all(|id|done.contains(id)){done.insert(task.id.clone());}}}if done.len()!=tasks.len(){return Err("NEW_PRODUCT_DEPENDENCY_CYCLE".into())}
 for q in analysis.data["questions"].as_array().unwrap(){
  if q["required"]==false{continue}let relevant=tasks.iter().any(|task|q["productId"]==task.product_id&&q["affectedKinds"].as_array().is_some_and(|kinds|kinds.iter().any(|kind|kind==&task.kind)));
  let answer=&answers[q["id"].as_str().unwrap_or("")];if relevant&&(answer["choice"].as_str().unwrap_or("").is_empty()||(answer["choice"]=="custom"&&answer["text"].as_str().unwrap_or("").trim().is_empty())){return Err("NEW_PRODUCT_CLARIFICATION_REQUIRED".into())}
 }
 Ok(())
}
pub(super) fn result_bytes(s:&Shared,job:&Job,task:&Task)->Res<Vec<u8>>{
 let result=task.result.as_ref().ok_or("NEW_PRODUCT_RESULT_NOT_READY")?;let store=locked(s)?;let project=store.load(&job.project_id)?;
 let version=project.assets.iter().find(|asset|asset.id==result.asset_id).and_then(|asset|asset.versions.iter().find(|version|version.id==result.version_id)).ok_or("NEW_PRODUCT_RESULT_NOT_FOUND")?;
 std::fs::read(store.root.join("projects").join(&job.project_id).join(&version.file)).map_err(|_|"NEW_PRODUCT_RESULT_NOT_FOUND".into())
}
pub async fn invoke(s:Shared,hub:AppHandle,name:&str,v:Value)->Res<Response>{
 let root=folder(&s)?;
 let response=match name{
  "new_product_capabilities"=>json!({"configured":crate::credential_status()?,"analysisModel":VISION,"imageModel":IMAGE,"resolutions":["1K","2K","4K"],"modes":["realtime","batch"],"maxInputs":40,"maxTasks":64,"maxConcurrency":4}),
  "new_product_analyze"=>{
   let pid=id(&v,"projectId")?;let fingerprints=source(&s,&pid,&v["context"])?;
   if fingerprints.len() as u64>crate::frontend_settings::read(&s)?.analysis_count&&v["approvedAnalysis"]!=true{return Ok(axum::Json(json!({"confirmation":{"stage":"analysis","reason":"describe_count","count":fingerprints.len(),"message":format!("将识别{}张不同原图，是否继续？",fingerprints.len())}})).into_response())}
   let brief=v["brief"].as_str().unwrap_or("").to_string();let facts=v["productFacts"].as_str().unwrap_or("").to_string();if brief.len()>12000||facts.len()>6000{return Err("NEW_PRODUCT_BRIEF_TOO_LONG".into())}
   let cache=digest(&json!([pid,fingerprints,brief,facts,v["appearance"],VISION]));let cache_path=root.join("analyses").join(format!("cache-{cache}.json"));
   let record=if let Ok(bytes)=std::fs::read(&cache_path){serde_json::from_slice::<Analysis>(&bytes).map_err(|_|"NEW_PRODUCT_ANALYSIS_CORRUPT")?}else{
    let mut data=google::analyze(s.clone(),&pid,&v["context"],&brief,&facts,&v["appearance"]).await?;require_shape(&data,&fingerprints)?;ensure_questions(&mut data);
    let record=Analysis{id:uuid::Uuid::new_v4().to_string(),project_id:pid,context:v["context"].clone(),fingerprints,data,brief,facts};atomic(&root.join("analyses").join(format!("{}.json",record.id)),&record)?;atomic(&cache_path,&record)?;record
   };let mut data=record.data;ensure_questions(&mut data);data["analysisId"]=json!(record.id);data
  },
  "new_product_start"=>{
   let pid=id(&v,"projectId")?;let analysis=read_analysis(&root,&id(&v,"analysisId")?)?;if analysis.project_id!=pid||source(&s,&pid,&v["context"])?!=analysis.fingerprints{return Err("NEW_PRODUCT_SOURCE_CHANGED".into())}
   let mut tasks:Vec<Task>=serde_json::from_value(v["manifest"]["tasks"].clone()).map_err(|_|"NEW_PRODUCT_TASK_INVALID")?;tasks.retain(|task|task.enabled);validate_tasks(&tasks,&analysis,&v["answers"])?;
   let mode=v["mode"].as_str().unwrap_or("realtime");let resolution=v["resolution"].as_str().unwrap_or("2K");let ratio=v["ratio"].as_str().unwrap_or("1:1");
   if !["realtime","batch"].contains(&mode)||!["1K","2K","4K"].contains(&resolution)||!["1:1","3:2","2:3","4:3","3:4","4:5","5:4","16:9","9:16"].contains(&ratio){return Err("NEW_PRODUCT_OPTIONS".into())}
   let estimated=match resolution{"1K"=>Some(0.04),"2K"=>Some(0.06),_=>None}.map(|usd|usd*tasks.len() as f64*7.2*if mode=="batch"{0.5}else{1.});
   let jid=id(&v,"clientRequestId")?;let request_hash=digest(&json!([pid,analysis.id,v["manifest"],v["answers"],mode,resolution,ratio,v["concurrency"]]));
   if job_path(&root,&jid).exists(){let job=read_job(&root,&jid)?;if job.request_hash!=request_hash{return Err("NEW_PRODUCT_REQUEST_CONFLICT".into())}return Ok(axum::Json(json!({"job":view(&job)})).into_response())}
   let settings=crate::frontend_settings::read(&s)?;let near_quota=crate::frontend_settings::near_quota(&s,tasks.len())?;
   if v["approvedGeneration"]!=true&&(near_quota||tasks.len() as u64>settings.generation_count||estimated.is_none_or(|value|value>settings.cost_cny)){return Ok(axum::Json(json!({"confirmation":{"stage":"generation","reason":"generation_cost_or_count","count":tasks.len(),"estimatedCny":estimated,"message":format!("{}将生成{}张图片{}，是否继续？",if near_quota{"本账号配额将达到95%以上。"}else{""},tasks.len(),estimated.map(|v|format!("，参考估算约¥{v:.2}")).unwrap_or("，费用尚不能准确估算".into()))}})).into_response())}
   if !crate::credential_status()?{return Err("KEY_NOT_CONFIGURED".into())}for task in &mut tasks{task.state="queued".into();task.error=None;task.result=None;}
   let mut job=Job{id:jid,project_id:pid,analysis_id:analysis.id,request_hash,context:v["context"].clone(),tasks,mode:mode.into(),concurrency:v["concurrency"].as_u64().unwrap_or(3).clamp(1,4) as usize,resolution:resolution.into(),ratio:ratio.into(),status:"queued".into(),created_at:now(),updated_at:now(),estimated_cny:estimated,batches:vec![],error:None};
   save(&root,&mut job,&hub)?;runner::spawn(s.clone(),hub.clone(),root.clone(),job.clone());json!({"job":view(&job)})
  },
  "new_product_jobs"=>{
   let pid=v["projectId"].as_str();let mut jobs=vec![];
   for entry in std::fs::read_dir(&root).map_err(|_|"NEW_PRODUCT_STORAGE")?.flatten(){if entry.path().extension().is_some_and(|ext|ext=="json"){if let Ok(bytes)=std::fs::read(entry.path()){if let Ok(job)=serde_json::from_slice::<Job>(&bytes){if pid.is_none_or(|pid|pid==job.project_id){jobs.push(job)}}}}}jobs.sort_by_key(|job|std::cmp::Reverse(job.created_at));json!({"jobs":jobs.iter().take(20).map(|job|{let mut value=view(job);value["stopRequested"]=json!(cancelled(&root,&job.id));value["context"]=job.context.clone();value}).collect::<Vec<_>>()})
  },
  "new_product_cancel"=>{let jid=id(&v,"jobId")?;let job=read_job(&root,&jid)?;std::fs::write(root.join(format!("{jid}.cancel")),b"stop").map_err(|_|"NEW_PRODUCT_STORAGE")?;runner::signal(&root,&jid);json!({"job":view(&job),"stopRequested":true})},
  "new_product_result"=>{let job=read_job(&root,&id(&v,"jobId")?)?;let tid=v["taskId"].as_str().ok_or("NEW_PRODUCT_ARGUMENT")?;let task=job.tasks.iter().find(|task|task.id==tid).ok_or("NEW_PRODUCT_TASK_NOT_FOUND")?;return Ok(([(axum::http::header::CONTENT_TYPE,"application/octet-stream")],result_bytes(&s,&job,task)?).into_response())},
  _=>return Err("UNKNOWN_COMMAND".into()),
 };Ok(axum::Json(response).into_response())
}
pub fn resume(s:Shared,hub:AppHandle){if let Ok(root)=folder(&s){if let Ok(entries)=std::fs::read_dir(&root){for entry in entries.flatten(){if let Ok(bytes)=std::fs::read(entry.path()){if let Ok(job)=serde_json::from_slice::<Job>(&bytes){if !terminal(&job.status){runner::spawn(s.clone(),hub.clone(),root.clone(),job)}}}}}}}
#[cfg(test)]mod tests;

pub(crate) async fn generate_reference(image:&image::DynamicImage,prompt:&str,resolution:&str,ratio:&str)->Res<(image::DynamicImage,Value)>{google::single_reference(image,prompt,resolution,ratio).await}
