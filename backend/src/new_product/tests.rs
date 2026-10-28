use super::*;
pub(super) fn setup()->(tempfile::TempDir,Shared,String,String,Value){
 let temp=tempfile::tempdir().unwrap();let store=core::Store::open(temp.path().join("data"),"test").unwrap();let project=store.create("new product fixture".into()).unwrap();
 let file=temp.path().join("refill.png");image::RgbaImage::from_pixel(8,12,image::Rgba([110,140,120,255])).save(&file).unwrap();
 let imported=store.import(&project.id,vec![file.to_string_lossy().into()],&AtomicBool::new(false),|_,_|{}).unwrap();let aid=imported.project.assets[0].id.clone();
 let shared=Arc::new(crate::AppState{store:Mutex::new(store),cancel:AtomicBool::new(false),local_job:Mutex::new(()),preview_epoch:std::sync::atomic::AtomicU64::new(0)});
 let context=json!({"page":"library","mentions":[],"selected":[aid],"editorAssetId":null,"filter":{"folder":"","tag":"","status":"","q":"","sort":""},"filteredCount":1,"resolvedTarget":{"source":"selected","ids":[aid]},"fileNames":["refill.png"]});(temp,shared,project.id,aid,context)
}
pub(super) fn analysis(s:&Shared,pid:&str,aid:&str,context:&Value)->Analysis{Analysis{id:uuid::Uuid::new_v4().to_string(),project_id:pid.into(),context:context.clone(),fingerprints:source(s,pid,context).unwrap(),brief:"fixture".into(),facts:"plastic shell; wet-pressed pulp insert".into(),data:json!({"items":[{"assetId":aid,"productId":"p1","role":"assembled","contents":"unknown"}],"products":[{"id":"p1","sceneIds":[aid],"humanIds":[aid],"explodedIds":[aid],"explodedReady":true}],"questions":[]})}}
pub(super) fn task(id:&str,aid:&str)->Task{Task{id:id.into(),kind:"white".into(),product_id:"p1".into(),title:id.into(),prompt:format!("TASK {id}: preserve product geometry"),reference_ids:vec![aid.into()],depends_on:vec![],enabled:true,anchor:false,state:"queued".into(),error:None,result:None}}
pub(super) fn job(analysis:&Analysis,tasks:Vec<Task>)->Job{Job{id:uuid::Uuid::new_v4().to_string(),project_id:analysis.project_id.clone(),analysis_id:analysis.id.clone(),request_hash:"fixture".into(),context:analysis.context.clone(),tasks,mode:"realtime".into(),concurrency:3,resolution:"2K".into(),ratio:"4:5".into(),status:"queued".into(),created_at:now(),updated_at:now(),estimated_cny:Some(1.),batches:vec![],error:None}}
#[test]fn selection_context_rejects_paths_and_extra_objects(){
 let(_temp,s,pid,_aid,context)=setup();let a=source(&s,&pid,&context).unwrap();let b=source(&s,&pid,&context).unwrap();assert_eq!(digest(&a),digest(&b));
 let mut wrong=context.clone();wrong["fileNames"]=json!(["C:/private/refill.png"]);assert!(source(&s,&pid,&wrong).is_err());wrong=context.clone();wrong["sourcePath"]=json!("secret");assert!(source(&s,&pid,&wrong).is_err());wrong=context;wrong["filter"]["folder"]=json!("C:\\private");assert!(source(&s,&pid,&wrong).is_err());
}
#[test]fn empty_refill_requires_a_content_answer_and_foreign_references_fail(){
 let(_temp,s,pid,aid,context)=setup();let mut analysis=analysis(&s,&pid,&aid,&context);analysis.data["items"][0]["role"]=json!("refill");analysis.data["items"][0]["contents"]=json!("empty");ensure_questions(&mut analysis.data);
 assert_eq!(analysis.data["questions"].as_array().unwrap().len(),1);let tasks=vec![task("white-a",&aid)];assert!(validate_tasks(&tasks,&analysis,&json!({})).is_err());assert!(validate_tasks(&tasks,&analysis,&json!({"contents-p1":{"choice":"empty"}})).is_ok());
 let mut foreign=tasks;foreign[0].reference_ids=vec!["foreign".into()];assert!(validate_tasks(&foreign,&analysis,&json!({"contents-p1":{"choice":"empty"}})).is_err());
}
#[test]fn optional_model_question_cannot_skip_required_exploded_clarification(){
 let(_temp,s,pid,aid,context)=setup();let mut analysis=analysis(&s,&pid,&aid,&context);analysis.data["products"][0]["explodedReady"]=json!(false);analysis.data["questions"]=json!([{"id":"model-q","productId":"p1","kind":"exploded","question":"是否需要爆炸图","options":[],"assetIds":[aid],"affectedKinds":[],"required":false}]);ensure_questions(&mut analysis.data);assert_eq!(analysis.data["questions"][0]["required"],true);assert_eq!(analysis.data["questions"][0]["affectedKinds"],json!(["exploded"]));let mut task=task("exploded",&aid);task.kind="exploded".into();assert!(validate_tasks(&[task],&analysis,&json!({})).is_err());
}
#[test]fn dependencies_must_exist_and_cannot_cycle(){
 let(_temp,s,pid,aid,context)=setup();let analysis=analysis(&s,&pid,&aid,&context);let mut a=task("anchor-a",&aid);let mut b=task("anchor-b",&aid);a.anchor=true;b.anchor=true;a.depends_on=vec![b.id.clone()];b.depends_on=vec![a.id.clone()];assert!(validate_tasks(&[a,b],&analysis,&json!({})).is_err());
 let mut a=task("a",&aid);a.depends_on=vec!["absent".into()];assert!(validate_tasks(&[a],&analysis,&json!({})).is_err());
}
#[tokio::test]async fn generation_count_gate_runs_before_credentials_or_provider_submission(){
 let(_temp,s,pid,aid,context)=setup();let root=folder(&s).unwrap();let analysis=analysis(&s,&pid,&aid,&context);atomic(&root.join("analyses").join(format!("{}.json",analysis.id)),&analysis).unwrap();
 let tasks=(0..5).map(|index|task(&format!("white-{index}"),&aid)).collect::<Vec<_>>();let(events,_)=tokio::sync::broadcast::channel(8);let jid=uuid::Uuid::new_v4().to_string();
 let response=invoke(s,AppHandle{events},"new_product_start",json!({"projectId":pid,"analysisId":analysis.id,"context":context,"manifest":{"tasks":tasks},"answers":{},"mode":"realtime","resolution":"2K","ratio":"4:5","concurrency":3,"clientRequestId":jid,"approvedGeneration":false})).await.unwrap();
 let body=axum::body::to_bytes(response.into_body(),1024*1024).await.unwrap();let value:Value=serde_json::from_slice(&body).unwrap();assert_eq!(value["confirmation"]["count"],5);assert!(!job_path(&root,&jid).exists());
}
#[test]fn jobs_are_isolated_by_the_owning_workspace(){
 let(_one,s,pid,aid,context)=setup();let(_two,other,_,_,_)=setup();let analysis=analysis(&s,&pid,&aid,&context);let job=job(&analysis,vec![task("a",&aid)]);atomic(&job_path(&folder(&s).unwrap(),&job.id),&job).unwrap();assert!(read_job(&folder(&other).unwrap(),&job.id).is_err());assert!(read_job(&folder(&s).unwrap(),"../outside").is_err());
}
#[test]fn installed_opencv_corrects_non_quarter_turn_subject_on_two_tone_background(){
 assert_eq!(crate::assistant::opencv_status()["available"],true);
 let angle=19_f64.to_radians();let mut image=image::RgbImage::new(500,500);
 for(y,row)in image.rows_mut().enumerate(){for(x,pixel)in row.enumerate(){let dx=x as f64-250.;let dy=y as f64-250.;let u=dx*angle.cos()+dy*angle.sin();let v=-dx*angle.sin()+dy*angle.cos();*pixel=if u.abs()<55.&&v.abs()<120.{image::Rgb([240,242,239])}else if y<220{image::Rgb([135,95,65])}else{image::Rgb([190,165,130])};}}
 let result=crate::assistant::opencv(image::DynamicImage::ImageRgb8(image),"deskew",45.).unwrap().to_rgb8();
 let mut points=vec![];for(x,y,pixel)in result.enumerate_pixels(){if pixel.0.iter().all(|v|*v>210){points.push((x as f64,y as f64));}}
 let n=points.len() as f64;assert!(n>10000.);let mx=points.iter().map(|p|p.0).sum::<f64>()/n;let my=points.iter().map(|p|p.1).sum::<f64>()/n;
 let xx=points.iter().map(|p|(p.0-mx).powi(2)).sum::<f64>();let yy=points.iter().map(|p|(p.1-my).powi(2)).sum::<f64>();let xy=points.iter().map(|p|(p.0-mx)*(p.1-my)).sum::<f64>();let degrees=(0.5*(2.*xy).atan2(xx-yy)).to_degrees().abs();assert!(degrees.min((90.-degrees).abs())<2.,"remaining tilt {degrees}");
}

#[test]fn original_folder_references_and_analysis_cache_do_not_drift_after_generated_versions_are_adopted(){let(_temp,s,pid,aid,context)=setup();let fingerprint=source(&s,&pid,&context).unwrap();let original=google::input(&s,&pid,&aid,1600).unwrap();let changed=image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(90,60,image::Rgba([240,20,20,255])));let version=locked(&s).unwrap().save_generated(&pid,&aid,&changed,json!({"fixture":true})).unwrap().1;locked(&s).unwrap().adopt_generated(&pid,&aid,&version).unwrap();assert_eq!(source(&s,&pid,&context).unwrap(),fingerprint);assert_eq!(google::input(&s,&pid,&aid,1600).unwrap(),original);let current=locked(&s).unwrap().load(&pid).unwrap();assert_eq!(current.assets[0].generated_base,Some(version));}
