#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod assistant;
mod core;
mod hybrid;
mod models;
use base64::Engine;
use core::{Edit, ExportItem, ExportPreset, ImportReport, Project, Res, Store};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
use transport::{State, AppHandle};
mod transport;
use axum::response::IntoResponse;
struct AppState {
    store: Mutex<Store>,
    cancel: AtomicBool,
    preview_epoch: AtomicU64,
}
type Shared = Arc<AppState>;
fn locked(s: &Shared) -> Res<std::sync::MutexGuard<'_, Store>> {
    s.store.lock().map_err(|_| "STORE_UNAVAILABLE".into())
}

fn bootstrap(s: State<Shared>) -> Res<Value> {
    let store = locked(&s)?;
    Ok(
        json!({"projects":store.list()?,"dataRoot":store.root,"profile":store.profile,"version":"0.2.0"}),
    )
}

fn model_catalog(s: State<Shared>) -> Res<Value> {
    let store = locked(&s)?;
    let (catalog, warning) = models::load(&store.root);
    Ok(json!({"catalog":catalog,"warning":warning}))
}

async fn hybrid_plan(
    s: State<'_, Shared>,
    project_id: String,
    ids: Vec<String>,
    instruction: String,
) -> Res<Value> {
    let context = {
        let store = locked(&s)?;
        let p = store.load(&project_id)?;
        if ids.is_empty() || ids.len() > 500 {
            return Err("INVALID_SCOPE".into());
        }
        let mut dims = std::collections::BTreeMap::<String, usize>::new();
        for id in ids {
            let a = p
                .assets
                .iter()
                .find(|a| a.id == id && !a.trashed)
                .ok_or("ASSET_NOT_FOUND")?;
            *dims.entry(format!("{}x{}", a.width, a.height)).or_default() += 1;
        }
        json!({"dimensionCounts":dims})
    };
    hybrid::plan(instruction, context).await
}

fn hybrid_start(
    s: State<Shared>,
    project_id: String,
    ids: Vec<String>,
    instruction: String,
    recipe: hybrid::Recipe,
    output_dir: String,
    approved: bool,
    client_request_id: String,
) -> Res<hybrid::Job> {
    hybrid::start(
        &s,
        project_id,
        ids,
        instruction,
        recipe,
        output_dir,
        approved,
        client_request_id,
    )
}

fn hybrid_jobs(s: State<Shared>) -> Res<Value> {
    let store = locked(&s)?;
    Ok(json!({"jobs":hybrid::list(&store.root)?,"notices":hybrid::notices(&store.root)?}))
}

fn hybrid_cancel(s: State<Shared>, job_id: String) -> Res<()> {
    hybrid::request_cancel(&locked(&s)?.root, &job_id)
}

fn hybrid_ack(s: State<Shared>, job_id: String) -> Res<()> {
    hybrid::acknowledge(&locked(&s)?.root, &job_id)
}

fn hybrid_open_output(s: State<Shared>, job_id: String) -> Res<()> {
    let store = locked(&s)?;
    let job = hybrid::list(&store.root)?
        .into_iter()
        .find(|j| j.id == job_id)
        .ok_or("JOB_NOT_FOUND")?;
    if !std::path::Path::new(&job.output_dir).is_dir() {
        return Err("OUTPUT_NOT_READY".into());
    }
    std::process::Command::new("explorer.exe")
        .arg(job.output_dir)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn assistant_key_status() -> Res<bool> {
    match assistant::credential()?.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(_) => Err("KEYRING_UNAVAILABLE".into()),
    }
}

fn assistant_key_set(key: String) -> Res<()> {
    if key.trim().len() < 12 {
        return Err("INVALID_KEY".into());
    }
    assistant::credential()?
        .set_password(key.trim())
        .map_err(|_| "KEYRING_UNAVAILABLE".into())
}

fn assistant_key_delete() -> Res<()> {
    match assistant::credential()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("KEYRING_UNAVAILABLE".into()),
    }
}

async fn assistant_tools() -> Res<Value> {
    transport::runtime::spawn_blocking(|| Ok(assistant::opencv_status()))
        .await
        .map_err(|e| e.to_string())?
}

async fn assistant_plan(
    s: State<'_, Shared>,
    project_id: String,
    ids: Vec<String>,
    instruction: String,
) -> Res<Value> {
    let (context, root) = {
        let store = locked(&s)?;
        let p = store.load(&project_id)?;
        if ids.is_empty() || ids.len() > 500 {
            return Err("INVALID_SCOPE".into());
        }
        let mut dimensions = std::collections::BTreeMap::<String, usize>::new();
        for id in &ids {
            let a = p
                .assets
                .iter()
                .find(|a| a.id == *id && !a.trashed)
                .ok_or("ASSET_NOT_FOUND")?;
            *dimensions
                .entry(format!("{}x{}", a.width, a.height))
                .or_default() += 1;
        }
        (
            json!({"selectedCount":ids.len(),"dimensionCounts":dimensions}),
            store.root.clone(),
        )
    };
    let mut result = assistant::plan(instruction, context).await?;
    let id = uuid::Uuid::new_v4().to_string();
    result["id"] = json!(id);
    let dir = root.join("assistant-plans");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(
        dir.join(format!("{id}.json")),
        serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(result)
}

async fn assistant_preview(
    s: State<'_, Shared>,
    project_id: String,
    ids: Vec<String>,
    plan: assistant::Plan,
) -> Res<Value> {
    let state = s.inner().clone();
    transport::runtime::spawn_blocking(move || {
        assistant::preview(&*locked(&state)?, &project_id, ids, plan)
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn assistant_apply(
    s: State<'_, Shared>,
    app: AppHandle,
    sample_id: String,
    output_dir: Option<String>,
) -> Res<Value> {
    let state = s.inner().clone();
    state.cancel.store(false, Ordering::Relaxed);
    transport::runtime::spawn_blocking(move || {
        assistant::apply(
            &*locked(&state)?,
            &sample_id,
            output_dir,
            &state.cancel,
            |done, total| {
                let _ = app.emit(
                    "job-progress",
                    json!({"kind":"local-assistant","done":done,"total":total}),
                );
            },
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

fn assistant_presets(s: State<Shared>) -> Res<Value> {
    Ok(json!(assistant::presets(&*locked(&s)?)?))
}

fn assistant_save_preset(s: State<Shared>, name: String, plan: assistant::Plan) -> Res<()> {
    assistant::save_preset(&*locked(&s)?, name, plan)
}

fn project_create(s: State<Shared>, name: String) -> Res<Project> {
    locked(&s)?.create(name)
}

fn project_rename(s: State<Shared>, project_id: String, name: String) -> Res<Project> {
    locked(&s)?.rename(&project_id, name)
}

async fn pick_import(
    s: State<'_, Shared>,
    app: AppHandle,
    project_id: String,
    folder: bool,
) -> Res<Option<ImportReport>> {
    let s = s.inner().clone();
    transport::runtime::spawn_blocking(move || {
        let paths = if folder {
            rfd::FileDialog::new()
                .set_title("选择图片文件夹")
                .pick_folder()
                .map(|x| vec![x])
        } else {
            rfd::FileDialog::new()
                .set_title("导入图片")
                .add_filter("图像", &["png", "jpg", "jpeg", "webp"])
                .pick_files()
        };
        if let Some(paths) = paths {
            s.cancel.store(false, Ordering::Relaxed);
            Ok(Some(locked(&s)?.import(
                &project_id,
                paths.iter().map(|p| p.to_string_lossy().into()).collect(),
                &s.cancel,
                |done, total| {
                    let _ = app.emit(
                        "job-progress",
                        json!({"kind":"import","done":done,"total":total}),
                    );
                },
            )?))
        } else {
            Ok(None)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn import_paths(
    s: State<'_, Shared>,
    app: AppHandle,
    project_id: String,
    paths: Vec<String>,
) -> Res<ImportReport> {
    let s = s.inner().clone();
    transport::runtime::spawn_blocking(move || {
        s.cancel.store(false, Ordering::Relaxed);
        locked(&s)?.import(&project_id, paths, &s.cancel, |done, total| {
            let _ = app.emit(
                "job-progress",
                json!({"kind":"import","done":done,"total":total}),
            );
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn preview(
    s: State<'_, Shared>,
    project_id: String,
    asset_id: String,
    edit: Edit,
) -> Res<Vec<u8>> {
    let s = s.inner().clone();
    let epoch = s.preview_epoch.fetch_add(1, Ordering::Relaxed) + 1;
    transport::runtime::spawn_blocking(move || {
        let store = locked(&s)?;
        if s.preview_epoch.load(Ordering::Relaxed) != epoch {
            return Err("PREVIEW_SUPERSEDED".into());
        }
        Ok(Vec::from(store.preview(
            &project_id,
            &asset_id,
            edit,
        )?))
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn preview_source(s: State<'_, Shared>, project_id: String, asset_id: String) -> Res<Vec<u8>> {
    let s = s.inner().clone();
    transport::runtime::spawn_blocking(move || locked(&s)?.preview_source(&project_id, &asset_id)).await.map_err(|e| e.to_string())?
}
fn thumbnail(s: State<Shared>, project_id: String, asset_id: String) -> Res<Vec<u8>> {
    Ok(Vec::from(
        locked(&s)?.thumb(&project_id, &asset_id)?,
    ))
}

fn original_preview(
    s: State<Shared>,
    project_id: String,
    asset_id: String,
) -> Res<Vec<u8>> {
    Ok(Vec::from(
        locked(&s)?.original_preview(&project_id, &asset_id)?,
    ))
}

fn generated_preview(
    s: State<Shared>,
    project_id: String,
    asset_id: String,
    version_id: String,
) -> Res<Vec<u8>> {
    Ok(Vec::from(locked(&s)?.generated_preview(
        &project_id,
        &asset_id,
        &version_id,
    )?))
}

fn adopt_generated(
    s: State<Shared>,
    project_id: String,
    asset_id: String,
    version_id: String,
) -> Res<Project> {
    locked(&s)?.adopt_generated(&project_id, &asset_id, &version_id)
}

fn restore_version(
    s: State<Shared>,
    project_id: String,
    asset_id: String,
    version_id: Option<String>,
) -> Res<Project> {
    locked(&s)?.restore_version(&project_id, &asset_id, version_id.as_deref())
}

fn generation_details(
    s: State<Shared>,
    project_id: String,
    asset_id: String,
    version_id: String,
) -> Res<Value> {
    let mut metadata = locked(&s)?.generation_metadata(&project_id, &asset_id, &version_id)?;
    if let Ok(price) = serde_json::from_value::<models::Model>(metadata["priceSnapshot"].clone()) {
        let mut bounds = json!(models::cost_bounds(&price, &metadata["usageMetadata"]));
        let factor = metadata["batchFactor"].as_f64().unwrap_or(1.0);
        if bounds.is_object() {
            for field in ["minimumUsd", "maximumUsd"] {
                if let Some(n) = bounds[field].as_f64() {
                    bounds[field] = json!(n * factor);
                }
            }
        }
        metadata["costBounds"] = bounds;
    }
    Ok(metadata)
}

async fn edit_commit(
    s: State<'_, Shared>,
    project_id: String,
    asset_id: String,
    edit: Edit,
) -> Res<Project> {
    let s = s.inner().clone();
    transport::runtime::spawn_blocking(move || locked(&s)?.commit(&project_id, &asset_id, edit))
        .await
        .map_err(|e| e.to_string())?
}

fn asset_trash(
    s: State<Shared>,
    project_id: String,
    ids: Vec<String>,
    restore: bool,
) -> Res<Project> {
    locked(&s)?.trash(&project_id, ids, restore)
}

async fn pick_output() -> Res<Option<String>> {
    transport::runtime::spawn_blocking(|| {
        Ok(rfd::FileDialog::new()
            .set_title("选择导出目录")
            .pick_folder()
            .map(|p| p.to_string_lossy().into()))
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn export_run(
    s: State<'_, Shared>,
    app: AppHandle,
    project_id: String,
    ids: Vec<String>,
    preset: ExportPreset,
) -> Res<Vec<ExportItem>> {
    let s = s.inner().clone();
    transport::runtime::spawn_blocking(move || {
        s.cancel.store(false, Ordering::Relaxed);
        locked(&s)?.export(&project_id, ids, preset, &s.cancel, |done, total| {
            let _ = app.emit(
                "job-progress",
                json!({"kind":"export","done":done,"total":total}),
            );
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

fn cancel_job(s: State<Shared>) {
    s.cancel.store(true, Ordering::Relaxed)
}
fn credential() -> Res<keyring::Entry> {
    keyring::Entry::new(
        &format!(
            "EDGE-STUDIO-{}",
            std::env::var("GVPAK_PROFILE").unwrap_or("dev".into())
        ),
        "google-official",
    )
    .map_err(|_| "KEYRING_UNAVAILABLE: Windows 凭据管理器不可用".into())
}

fn credential_status() -> Res<bool> {
    match credential()?.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(_) => Err("KEYRING_UNAVAILABLE".into()),
    }
}

fn credential_set(key: String) -> Res<()> {
    if key.trim().len() < 12 {
        return Err("INVALID_KEY: 密钥长度不正确".into());
    }
    credential()?
        .set_password(key.trim())
        .map_err(|_| "KEYRING_UNAVAILABLE: 密钥保存失败".into())
}

fn credential_delete() -> Res<()> {
    match credential()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("KEYRING_UNAVAILABLE".into()),
    }
}

async fn credential_test() -> Res<String> {
    let key = credential()?
        .get_password()
        .map_err(|_| "KEY_NOT_CONFIGURED")?;
    let response = reqwest::Client::new()
        .get("https://generativelanguage.googleapis.com/v1beta/models")
        .header("x-goog-api-key", key)
        .timeout(std::time::Duration::from_secs(25))
        .send()
        .await
        .map_err(|_| "NETWORK_ERROR: 无法连接 Google 官方接口")?;
    if response.status().is_success() {
        Ok("Google 连接成功（免费模型列表接口）".into())
    } else {
        Err(format!(
            "API_AUTH_ERROR: HTTP {}",
            response.status().as_u16()
        ))
    }
}

async fn generate(
    s: State<'_, Shared>,
    project_id: String,
    asset_id: String,
    edit: Edit,
    prompt: String,
    model: String,
    resolution: String,
    ratio: String,
    approved: bool,
    estimated_input_tokens: Option<u32>,
    estimated_output_tokens: Option<u32>,
) -> Res<Value> {
    if !approved {
        return Err("COST_APPROVAL_REQUIRED".into());
    }
    if prompt.trim().is_empty() || prompt.len() > 16000 {
        return Err("INVALID_PROMPT".into());
    }
    let catalog = models::load(&locked(&s)?.root).0;
    let capability = catalog
        .models
        .iter()
        .find(|m| m.id == model)
        .ok_or("INVALID_MODEL_PARAMETERS: 模型未配置")?;
    if !capability.resolutions.contains(&resolution) || !capability.ratios.contains(&ratio) {
        return Err("INVALID_MODEL_PARAMETERS: 模型不支持当前分辨率或宽高比".into());
    }
    let key = credential()?
        .get_password()
        .map_err(|_| "KEY_NOT_CONFIGURED: 请先配置 Google 密钥")?;
    let state = s.inner().clone();
    let pid = project_id.clone();
    let input_asset_id = asset_id.clone();
    let request_root = locked(&s)?.root.clone();
    let input_edit = edit.clone();
    let estimated_usd = capability
        .image_usd
        .get(&resolution)
        .copied()
        .flatten()
        .map(|price| {
            price
                + estimated_input_tokens.unwrap_or(1000) as f64 * capability.input_usd_per_million
                    / 1e6
                + estimated_output_tokens.unwrap_or(1000) as f64 * capability.text_usd_per_million
                    / 1e6
        });
    let image = transport::runtime::spawn_blocking(move || {
        locked(&state)?.preview(&pid, &input_asset_id, edit)
    })
    .await
    .map_err(|e| e.to_string())??;
    let body = json!({"contents":[{"role":"user","parts":[{"text":prompt},{"inlineData":{"mimeType":"image/png","data":base64::engine::general_purpose::STANDARD.encode(image)}}]}],"generationConfig":{"responseModalities":["TEXT","IMAGE"],"imageConfig":{"aspectRatio":ratio,"imageSize":resolution}}});
    let response = reqwest::Client::new()
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"
        ))
        .header("x-goog-api-key", key)
        .json(&body)
        .timeout(std::time::Duration::from_secs(180))
        .send()
        .await
        .map_err(|_| "NETWORK_OUTCOME_UNKNOWN: 请求未取得结果，请检查账户用量后再决定是否重试")?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "GOOGLE_API_ERROR: HTTP {}；未自动重试",
            status.as_u16()
        ));
    }
    let value: Value = response.json().await.map_err(|_| "INVALID_API_RESPONSE")?;
    let parts = value["candidates"][0]["content"]["parts"]
        .as_array()
        .ok_or("NO_IMAGE: Google 未返回图片，可能触发安全限制")?;
    let data = parts
        .iter()
        .find_map(|p| p["inlineData"]["data"].as_str())
        .ok_or("NO_IMAGE: Google 未返回图像")?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|_| "INVALID_IMAGE")?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(600_000_000);
    limits.max_image_width = Some(16000);
    limits.max_image_height = Some(16000);
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| "INVALID_IMAGE")?;
    reader.limits(limits);
    let generated = reader
        .decode()
        .map_err(|_| "INVALID_IMAGE: 返回图片无法解码")?;
    let store = locked(&s)?;
    if store.root != request_root {
        return Err("DATA_ROOT_CHANGED: 请求期间切换了数据目录，未将结果写入其他环境".into());
    }
    let staging = store.root.join("generated");
    std::fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    let name = uuid::Uuid::new_v4().to_string();
    let file = staging.join(format!("generated-{name}.png"));
    generated.save(&file).map_err(|e| e.to_string())?;
    let actual_usd = models::actual_usd(capability, &value["usageMetadata"]);
    let cost_bounds = models::cost_bounds(capability, &value["usageMetadata"]);
    std::fs::write(staging.join(format!("{name}-usage.json")),serde_json::to_vec_pretty(&json!({"model":model,"usageMetadata":value["usageMetadata"],"resolution":resolution,"prompt":prompt,"actualUsd":actual_usd,"priceSnapshot":capability})).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let metadata = json!({"model":model,"prompt":prompt,"resolution":resolution,"ratio":ratio,"inputEdit":input_edit,"usageMetadata":value["usageMetadata"],"estimatedUsd":estimated_usd,"actualUsd":actual_usd,"costBounds":cost_bounds,"priceSnapshot":capability});
    let (project, version_id) =
        store.save_generated(&project_id, &asset_id, &generated, metadata)?;
    Ok(
        json!({"project":project,"versionId":version_id,"assetId":asset_id,"usage":value["usageMetadata"],"savedPath":file,"actualUsd":actual_usd}),
    )
}

fn settings_path() -> Res<PathBuf> {
    Ok(std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("APP_PATH_UNAVAILABLE")?
        .join(format!(
            "edge-settings-{}.json",
            std::env::var("GVPAK_PROFILE").unwrap_or("dev".into())
        )))
}

fn data_root_switch(s: State<Shared>, path: String) -> Res<Value> {
    let mut store = locked(&s)?;
    if hybrid::active(&store.root) {
        return Err("ACTIVE_JOBS: 请等待或取消后台任务后再切换数据目录".into());
    }
    let new = Store::open(PathBuf::from(&path), &store.profile)?;
    let value = json!({"dataRoot":new.root,"profile":new.profile});
    std::fs::write(
        settings_path()?,
        serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    *store = new;
    Ok(json!({"projects":store.list()?,"dataRoot":store.root,"profile":store.profile}))
}


#[tokio::main]
async fn main() {
    let executable=std::env::current_exe().expect("Executable path");
    let project_root=executable.ancestors().find(|p|p.join("dist/index.html").exists()).map(|p|p.to_path_buf()).unwrap_or_else(||std::env::current_dir().unwrap());
    let root = std::env::var_os("GVPAK_DATA_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            settings_path()
                .ok()
                .and_then(|p| std::fs::read(p).ok())
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                .and_then(|v| v["dataRoot"].as_str().map(PathBuf::from))
                .unwrap_or_else(|| project_root.join("backend/studio-data"))
        });
    let profile = std::env::var("GVPAK_PROFILE").unwrap_or("dev".into());
    let store = match Store::open(root, &profile) {
        Ok(s) => s,
        Err(e) => {
            rfd::MessageDialog::new()
                .set_title("EDGE")
                .set_description(&e)
                .set_level(rfd::MessageLevel::Error)
                .show();
            return;
        }
    };
    if store
        .list()
        .expect("project catalog read failed")
        .is_empty()
    {
        let p = store
            .create("Architecture".into())
            .expect("demo project");
        let sample = store.root.join("demo-sources");
        std::fs::create_dir_all(&sample).expect("demo directory");
        for (name, data) in [
            (
                "Cloister.jpg",
                include_bytes!("../demo/cloister.jpg").as_slice(),
            ),
            (
                "Gothic arches.jpg",
                include_bytes!("../demo/arches.jpg").as_slice(),
            ),
            (
                "Gallery.jpg",
                include_bytes!("../demo/gallery.jpg").as_slice(),
            ),
        ] {
            std::fs::write(sample.join(name), data).expect("demo file");
        }
        store
            .import(
                &p.id,
                vec![sample.to_string_lossy().into()],
                &AtomicBool::new(false),
                |_, _| {},
            )
            .expect("demo import");
    }
    let shared = Arc::new(AppState {
        store: Mutex::new(store),
        cancel: AtomicBool::new(false),
        preview_epoch: AtomicU64::new(0),
    });

    transport::serve(shared,project_root).await;
}

fn arg<T:serde::de::DeserializeOwned>(v:&Value,key:&str)->Res<T>{serde_json::from_value(v.get(key).cloned().unwrap_or(Value::Null)).map_err(|e|format!("INVALID_ARGUMENT {key}: {e}"))}
fn binary(bytes:Vec<u8>)->axum::response::Response{([(axum::http::header::CONTENT_TYPE,"application/octet-stream")],bytes).into_response()}
pub(crate) async fn dispatch(s:Shared,hub:AppHandle,name:String,v:Value)->Res<axum::response::Response>{match name.as_str(){
"bootstrap" => Ok(axum::Json(serde_json::to_value(bootstrap(State(&s))?).map_err(|e|e.to_string())?).into_response()),
"model_catalog" => Ok(axum::Json(serde_json::to_value(model_catalog(State(&s))?).map_err(|e|e.to_string())?).into_response()),
"hybrid_plan" => Ok(axum::Json(serde_json::to_value(hybrid_plan(State(&s),arg(&v,"projectId")?,arg(&v,"ids")?,arg(&v,"instruction")?).await?).map_err(|e|e.to_string())?).into_response()),
"hybrid_start" => Ok(axum::Json(serde_json::to_value(hybrid_start(State(&s),arg(&v,"projectId")?,arg(&v,"ids")?,arg(&v,"instruction")?,arg(&v,"recipe")?,arg(&v,"outputDir")?,arg(&v,"approved")?,arg(&v,"clientRequestId")?)?).map_err(|e|e.to_string())?).into_response()),
"hybrid_jobs" => Ok(axum::Json(serde_json::to_value(hybrid_jobs(State(&s))?).map_err(|e|e.to_string())?).into_response()),
"hybrid_cancel" => Ok(axum::Json(serde_json::to_value(hybrid_cancel(State(&s),arg(&v,"jobId")?)?).map_err(|e|e.to_string())?).into_response()),
"hybrid_ack" => Ok(axum::Json(serde_json::to_value(hybrid_ack(State(&s),arg(&v,"jobId")?)?).map_err(|e|e.to_string())?).into_response()),
"hybrid_open_output" => Ok(axum::Json(serde_json::to_value(hybrid_open_output(State(&s),arg(&v,"jobId")?)?).map_err(|e|e.to_string())?).into_response()),
"assistant_key_status" => Ok(axum::Json(serde_json::to_value(assistant_key_status()?).map_err(|e|e.to_string())?).into_response()),
"assistant_key_set" => Ok(axum::Json(serde_json::to_value(assistant_key_set(arg(&v,"key")?)?).map_err(|e|e.to_string())?).into_response()),
"assistant_key_delete" => Ok(axum::Json(serde_json::to_value(assistant_key_delete()?).map_err(|e|e.to_string())?).into_response()),
"assistant_tools" => Ok(axum::Json(serde_json::to_value(assistant_tools().await?).map_err(|e|e.to_string())?).into_response()),
"assistant_plan" => Ok(axum::Json(serde_json::to_value(assistant_plan(State(&s),arg(&v,"projectId")?,arg(&v,"ids")?,arg(&v,"instruction")?).await?).map_err(|e|e.to_string())?).into_response()),
"assistant_preview" => Ok(axum::Json(serde_json::to_value(assistant_preview(State(&s),arg(&v,"projectId")?,arg(&v,"ids")?,arg(&v,"plan")?).await?).map_err(|e|e.to_string())?).into_response()),
"assistant_apply" => Ok(axum::Json(serde_json::to_value(assistant_apply(State(&s),hub.clone(),arg(&v,"sampleId")?,arg(&v,"outputDir")?).await?).map_err(|e|e.to_string())?).into_response()),
"assistant_presets" => Ok(axum::Json(serde_json::to_value(assistant_presets(State(&s))?).map_err(|e|e.to_string())?).into_response()),
"assistant_save_preset" => Ok(axum::Json(serde_json::to_value(assistant_save_preset(State(&s),arg(&v,"name")?,arg(&v,"plan")?)?).map_err(|e|e.to_string())?).into_response()),
"project_create" => Ok(axum::Json(serde_json::to_value(project_create(State(&s),arg(&v,"name")?)?).map_err(|e|e.to_string())?).into_response()),
"project_rename" => Ok(axum::Json(serde_json::to_value(project_rename(State(&s),arg(&v,"projectId")?,arg(&v,"name")?)?).map_err(|e|e.to_string())?).into_response()),
"pick_import" => Ok(axum::Json(serde_json::to_value(pick_import(State(&s),hub.clone(),arg(&v,"projectId")?,arg(&v,"folder")?).await?).map_err(|e|e.to_string())?).into_response()),
"import_paths" => Ok(axum::Json(serde_json::to_value(import_paths(State(&s),hub.clone(),arg(&v,"projectId")?,arg(&v,"paths")?).await?).map_err(|e|e.to_string())?).into_response()),
"preview" => Ok(binary(preview(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?,arg(&v,"edit")?).await?)),
"preview_source" => Ok(binary(preview_source(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?).await?)),
"thumbnail" => Ok(binary(thumbnail(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?)?)),
"original_preview" => Ok(binary(original_preview(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?)?)),
"generated_preview" => Ok(binary(generated_preview(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?,arg(&v,"versionId")?)?)),
"adopt_generated" => Ok(axum::Json(serde_json::to_value(adopt_generated(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?,arg(&v,"versionId")?)?).map_err(|e|e.to_string())?).into_response()),
"restore_version" => Ok(axum::Json(serde_json::to_value(restore_version(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?,arg(&v,"versionId")?)?).map_err(|e|e.to_string())?).into_response()),
"generation_details" => Ok(axum::Json(serde_json::to_value(generation_details(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?,arg(&v,"versionId")?)?).map_err(|e|e.to_string())?).into_response()),
"edit_commit" => Ok(axum::Json(serde_json::to_value(edit_commit(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?,arg(&v,"edit")?).await?).map_err(|e|e.to_string())?).into_response()),
"asset_trash" => Ok(axum::Json(serde_json::to_value(asset_trash(State(&s),arg(&v,"projectId")?,arg(&v,"ids")?,arg(&v,"restore")?)?).map_err(|e|e.to_string())?).into_response()),
"pick_output" => Ok(axum::Json(serde_json::to_value(pick_output().await?).map_err(|e|e.to_string())?).into_response()),
"export_run" => Ok(axum::Json(serde_json::to_value(export_run(State(&s),hub.clone(),arg(&v,"projectId")?,arg(&v,"ids")?,arg(&v,"preset")?).await?).map_err(|e|e.to_string())?).into_response()),
"cancel_job" => Ok(axum::Json(serde_json::to_value(cancel_job(State(&s))).map_err(|e|e.to_string())?).into_response()),
"credential_status" => Ok(axum::Json(serde_json::to_value(credential_status()?).map_err(|e|e.to_string())?).into_response()),
"credential_set" => Ok(axum::Json(serde_json::to_value(credential_set(arg(&v,"key")?)?).map_err(|e|e.to_string())?).into_response()),
"credential_delete" => Ok(axum::Json(serde_json::to_value(credential_delete()?).map_err(|e|e.to_string())?).into_response()),
"credential_test" => Ok(axum::Json(serde_json::to_value(credential_test().await?).map_err(|e|e.to_string())?).into_response()),
"generate" => Ok(axum::Json(serde_json::to_value(generate(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?,arg(&v,"edit")?,arg(&v,"prompt")?,arg(&v,"model")?,arg(&v,"resolution")?,arg(&v,"ratio")?,arg(&v,"approved")?,arg(&v,"estimatedInputTokens")?,arg(&v,"estimatedOutputTokens")?).await?).map_err(|e|e.to_string())?).into_response()),
"data_root_switch" => Ok(axum::Json(serde_json::to_value(data_root_switch(State(&s),arg(&v,"path")?)?).map_err(|e|e.to_string())?).into_response()),
_=>Err("UNKNOWN_COMMAND".into())}}
