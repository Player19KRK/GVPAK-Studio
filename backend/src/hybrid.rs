use crate::{
    Shared,
    assistant::{self, Output, Plan, Step},
    core::{self, Asset, Edit, ExportPreset, Res},
    locked,
    models::{self, Model},
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{SystemTime, UNIX_EPOCH},
};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Generation {
    #[serde(default)]
    pub mode: String,
    pub prompt: String,
    pub model: String,
    pub resolution: String,
    pub ratio: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Recipe {
    pub summary: String,
    pub before: Vec<Step>,
    pub generation: Option<Generation>,
    pub after: Vec<Step>,
    pub output: Output,
    pub clarification: Option<String>,
    pub warnings: Vec<String>,
}
fn local(steps: Vec<Step>) -> Plan {
    Plan {
        summary: "本地处理".into(),
        steps,
        warnings: vec![],
        clarification: None,
        output: None,
    }
}
impl Recipe {
    pub fn validate(&self) -> Res<()> {
        let mut p = local(self.before.clone());
        p.steps.extend(self.after.clone());
        p.output = Some(self.output.clone());
        p.validate()?;
        if self.summary.len() > 4000 || self.warnings.len() > 20 {
            return Err("INVALID_RECIPE".into());
        }
        if let Some(g) = &self.generation {
            if !["", "batch", "realtime"].contains(&g.mode.as_str()) || g.prompt.trim().is_empty()
                || g.prompt.len() > 16000
                || !["1K","2K","4K"].contains(&g.resolution.as_str())
                || !["1:1", "4:3", "3:4", "16:9", "9:16", "3:2", "2:3"].contains(&g.ratio.as_str())
            {
                return Err("INVALID_GENERATION: 混合任务当前使用 1K 生图".into());
            }
        }
        Ok(())
    }
}
const SYSTEM: &str = r#"You plan a seamless local + Google Batch product-image workflow. Return only json:
{"summary":"中文简短处理说明","before":[],"generation":null,"after":[],"output":{"format":"webp","width":1000,"height":1000,"fit":"pad","maxBytes":800000,"background":[255,255,255]},"clarification":null,"warnings":[]}.
Allowed local steps: {"op":"adjust","exposure":0.3,"contrast":1,"saturation":1,"temperature":0}; {"op":"rotate","degrees":90} (0/90/180/270); {"op":"flip","axis":"horizontal"} (horizontal/vertical); {"op":"crop","x":0,"y":0,"width":1,"height":1} normalized inside [0,1]; {"op":"resize","width":1000,"height":1000,"fit":"pad","background":[255,255,255]}; {"op":"flatten","background":[255,255,255]} fills transparency only; {"op":"denoise","strength":3} (1..20); {"op":"blur","sigma":1}; {"op":"sharpen","sigma":1,"threshold":3}; {"op":"deskew","max_degrees":20}; {"op":"auto_center","padding":0.08}.
Exposure [-3,3], contrast/saturation [0,3], temperature [-1,1], sigma [.1,5], threshold [0,100], padding [0,.35], max_degrees [1,45]. No arbitrary code/paths/URLs. Max 20 local steps total. Sizes <=16000, <=120 million pixels. Formats webp/png/jpeg, fit pad/crop. KB = 1000 bytes. If aspect is not specified, preserve full product with pad (do not ask about pad/crop unless the user's request is contradictory).
If the user requests changing scene/background, adding an animal/object, high-end product photography/relighting or any semantic generation, set generation={"prompt":"complete image-editing prompt","model":"gemini-nano-banana-2.1","resolution":"1K","ratio":"1:1"}. Choose ratio matching output (nearest available 1:1,4:3,3:4,16:9,9:16,3:2,2:3). Generate only one image per source, via half-price asynchronous Google Batch. Prompt must explicitly preserve exact product geometry, material, colors and logo/text, unless user requests their change; permit requested background/animal additions. For an unspecified matching animal, tell the image model to choose a visually harmonious small animal based on the reference product, secondary and not obscuring the product/logo. You do not see pixels: never claim to have analyzed the product. Google receives the actual reference image and can interpret its appearance.
Automatic centering/deskew: use before steps deskew then auto_center for local processing. For generated tasks also explicitly request upright and centered MAIN PRODUCT in generation.prompt, because local subject detection may be uncertain on complex backgrounds. Never center the added animal instead of the product after generation. after is for mild adjustments if requested; output handles final resize/compression. Do not use local flatten to simulate background removal. Reuse one recipe for the whole selection. Output automatic, no per-step confirmations; give clarification only if essential requirements are contradictory or truly unavailable. Don't reject requested generation as unsupported: Google Batch is integrated. No automatic real-time fallback. All user-facing strings Chinese."#;
pub async fn plan(instruction: String, context: Value) -> Res<Value> {
    let mut r = assistant::query(SYSTEM, instruction, context).await?;
    let recipe: Recipe =
        serde_json::from_value(r["plan"].clone()).map_err(|e| format!("INVALID_RECIPE: {e}"))?;
    recipe.validate()?;
    r["recipe"] = json!(recipe);
    r.as_object_mut().unwrap().remove("plan");
    Ok(r)
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub retention_status: Option<String>,
    pub asset_id: String,
    pub key: String,
    pub name: String,
    pub signature: String,
    pub edit: Edit,
    pub stage: String,
    pub version_id: Option<String>,
    pub exported: Option<String>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub usage: Value,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chunk {
    pub display_name: String,
    pub keys: Vec<String>,
    pub state: String,
    pub remote_name: Option<String>,
    pub last_poll: u64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub project_id: String,
    pub instruction: String,
    pub recipe: Recipe,
    pub items: Vec<Item>,
    pub chunks: Vec<Chunk>,
    pub state: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub output_dir: String,
    pub price: Option<Model>,
    pub estimated_usd: Option<f64>,
    pub error: Option<String>,
    #[serde(default)]
    pub cost_bounds: Option<Value>,
}
pub fn terminal(state: &str) -> bool {
    matches!(state, "completed" | "partial" | "failed" | "cancelled")
}
fn signature(a: &Asset) -> String {
    format!("{:x}",Sha256::digest(json!({"original":a.original,"hash":a.sha256,"edit":a.edit,"base":a.generated_base,"trashed":a.trashed}).to_string()))
}
fn dir(root: &Path, id: &str) -> Res<PathBuf> {
    uuid::Uuid::parse_str(id).map_err(|_| "INVALID_JOB_ID")?;
    Ok(root.join("hybrid-jobs").join(id))
}
fn save(root: &Path, job: &Job) -> Res<()> {
    let d = dir(root, &job.id)?;
    fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let mut f = tempfile::NamedTempFile::new_in(&d).map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec_pretty(job).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    f.as_file().sync_all().map_err(|e| e.to_string())?;
    f.persist(d.join("job.json")).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn list(root: &Path) -> Res<Vec<Job>> {
    let path = root.join("hybrid-jobs");
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut jobs = vec![];
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let e = entry.map_err(|e| e.to_string())?;
        if e.file_type().map_err(|e| e.to_string())?.is_dir() {
            if let Ok(bytes) = fs::read(e.path().join("job.json")) {
                if let Ok(j) = serde_json::from_slice::<Job>(&bytes) {
                    jobs.push(j)
                }
            }
        }
    }
    jobs.sort_by_key(|j| std::cmp::Reverse(j.created_at));
    Ok(jobs)
}
pub fn active(root: &Path) -> bool {
    list(root)
        .map(|j| j.iter().any(|j| !terminal(&j.state)))
        .unwrap_or(true)
}
pub fn request_cancel(root: &Path, id: &str) -> Res<()> {
    fs::write(dir(root, id)?.join("cancel.request"), b"cancel").map_err(|e| e.to_string())
}
pub fn acknowledge(root: &Path, id: &str) -> Res<()> {
    fs::write(dir(root, id)?.join("notified"), b"yes").map_err(|e| e.to_string())
}
pub fn notices(root: &Path) -> Res<Vec<Job>> {
    Ok(list(root)?
        .into_iter()
        .filter(|j| terminal(&j.state) && !dir(root, &j.id).unwrap().join("notified").exists())
        .collect())
}
pub fn start(
    s: &Shared,
    pid: String,
    ids: Vec<String>,
    instruction: String,
    recipe: Recipe,
    output: String,
    approved: bool,
    client_request_id: String,
) -> Res<Job> {
    recipe.validate()?;
    if recipe
        .clarification
        .as_ref()
        .is_some_and(|x| !x.trim().is_empty())
    {
        return Err("RECIPE_NEEDS_CLARIFICATION".into());
    }
    if ids.is_empty() {
        return Err("INVALID_SCOPE".into());
    }
    if !Path::new(&output).is_absolute() || !Path::new(&output).is_dir() {
        return Err("OUTPUT_REQUIRED".into());
    }
    let store = locked(s)?;
    let p = store.load(&pid)?;
    let id = client_request_id;
    let target = dir(&store.root, &id)?.join("job.json");
    if target.exists() {
        let existing: Job = serde_json::from_slice(&fs::read(target).map_err(|e| e.to_string())?)
            .map_err(|_| "JOB_CORRUPT")?;
        if existing.project_id != pid
            || existing.instruction != instruction
            || existing
                .items
                .iter()
                .map(|i| i.asset_id.clone())
                .collect::<Vec<_>>()
                != ids
        {
            return Err("REQUEST_ID_CONFLICT".into());
        }
        return Ok(existing);
    }
    let mut items = vec![];
    for aid in ids {
        if items.iter().any(|i: &Item| i.asset_id == aid) {
            return Err("DUPLICATE_ASSET".into());
        }
        let a = p
            .assets
            .iter()
            .find(|a| a.id == aid && !a.trashed)
            .ok_or("ASSET_NOT_FOUND")?;
        items.push(Item {
            retention_status:None,
            asset_id: aid,
            key: uuid::Uuid::new_v4().to_string(),
            name: a.name.clone(),
            signature: signature(a),
            edit: a.edit.clone(),
            stage: "pending".into(),
            version_id: None,
            exported: None,
            error: None,
            warnings: vec![],
            usage: Value::Null,
        });
    }
    let price = if let Some(g) = &recipe.generation {
        if !approved {
            return Err("COST_APPROVAL_REQUIRED".into());
        }
        if crate::image_provider::is_poe(&g.model){crate::image_provider::credential()?.get_password().map_err(|_|"POE_KEY_REQUIRED")?;None}else{crate::credential()?
            .get_password()
            .map_err(|_| "GOOGLE_KEY_REQUIRED: 请先配置 Google 密钥")?;
        let c = models::load(&store.root).0;
        let m = c
            .models
            .into_iter()
            .find(|m| {
                m.id == g.model
                    && m.resolutions.contains(&g.resolution)
                    && m.ratios.contains(&g.ratio)
            })
            .ok_or("INVALID_MODEL_PARAMETERS")?;
        Some(m)}
    } else {
        None
    };
    let estimated = price.as_ref().and_then(|m| {
        m.image_usd.get("1K").copied().flatten().map(|p| {
            (p + 1000. * m.input_usd_per_million / 1e6 + 1000. * m.text_usd_per_million / 1e6)
                * if recipe.generation.as_ref().is_some_and(|g|g.mode=="realtime"){1.0}else{0.5}
                * items.len() as f64
        })
    });
    let output_dir = Path::new(&output)
        .join(format!("GVPAK-{}", &id[..8]))
        .to_string_lossy()
        .into();
    let job = Job {
        id,
        project_id: pid,
        instruction,
        recipe,
        items,
        chunks: vec![],
        state: "queued".into(),
        created_at: now(),
        updated_at: now(),
        output_dir,
        price,
        estimated_usd: estimated,
        error: None,
        cost_bounds: None,
    };
    save(&store.root, &job)?;
    Ok(job)
}
fn apply_steps(
    mut im: image::DynamicImage,
    steps: &[Step],
    fallback: bool,
    warnings: &mut Vec<String>,
) -> Res<image::DynamicImage> {
    for step in steps {
        let backup = if fallback && matches!(step, Step::AutoCenter { .. } | Step::Deskew { .. }) {
            Some(im.clone())
        } else {
            None
        };
        match assistant::process(im, &local(vec![step.clone()]), &AtomicBool::new(false)) {
            Ok(v) => im = v,
            Err(e) => {
                if let Some(b) = backup {
                    warnings.push(format!("本地主体判断不足，已交给生成步骤完成构图：{e}"));
                    im = b;
                } else {
                    return Err(e);
                }
            }
        }
    }
    Ok(im)
}
fn prepare(s: &Shared, root: &Path, job: &mut Job) -> Res<()> {
    job.state = "preparing".into();
    save(root, job)?;
    for i in 0..job.items.len() {
        if dir(root, &job.id)?.join("cancel.request").exists() {
            return Ok(());
        }
        if job.items[i].stage != "pending" {
            continue;
        }
        let result = (|| -> Res<()> {
            let im = {
                let store = locked(s)?;
                let p = store.load(&job.project_id)?;
                let a = p
                    .assets
                    .iter()
                    .find(|a| a.id == job.items[i].asset_id)
                    .ok_or("ASSET_NOT_FOUND")?;
                if signature(a) != job.items[i].signature {
                    return Err("SOURCE_CHANGED: 开始处理前素材已改变".into());
                }
                core::render(store.source(&p, &a.id)?, &job.items[i].edit, None)?
            };
            let im = apply_steps(
                im,
                &job.recipe.before,
                job.recipe.generation.is_some(),
                &mut job.items[i].warnings,
            )?;
            im.save(dir(root, &job.id)?.join(format!("{}-prepared.png", job.items[i].key)))
                .map_err(|e| e.to_string())?;
            Ok(())
        })();
        match result {
            Ok(()) => job.items[i].stage = "ready".into(),
            Err(e) => {
                job.items[i].stage = "failed".into();
                job.items[i].error = Some(e)
            }
        }
        save(root, job)?;
    }
    Ok(())
}
fn request(root: &Path, job: &Job, key: &str) -> Res<Value> {
    let g = job.recipe.generation.as_ref().ok_or("GENERATION_MISSING")?;
    let im = core::decode(&dir(root, &job.id)?.join(format!("{key}-prepared.png")))?
        .thumbnail(1280, 1280);
    let data = base64::engine::general_purpose::STANDARD.encode(core::png(&im)?);
    Ok(
        json!({"request":{"contents":[{"role":"user","parts":[{"text":g.prompt},{"inlineData":{"mimeType":"image/png","data":data}}]}],"generationConfig":{"responseModalities":["TEXT","IMAGE"],"imageConfig":{"aspectRatio":g.ratio,"imageSize":g.resolution},"maxOutputTokens":8192}},"metadata":{"key":key}}),
    )
}
fn resource(name: &str, prefix: &str) -> Res<()> {
    if !name.starts_with(prefix)
        || name.len() <= prefix.len()
        || !name[prefix.len()..]
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
    {
        return Err("INVALID_REMOTE_RESOURCE".into());
    }
    Ok(())
}
async fn google(method: reqwest::Method, path: &str, body: Option<&Value>) -> Res<Value> {
    let key = crate::credential()?
        .get_password()
        .map_err(|_| "GOOGLE_KEY_REQUIRED")?;
    let client = reqwest::Client::new();
    let mut r = client
        .request(
            method,
            format!("https://generativelanguage.googleapis.com/v1beta/{path}"),
        )
        .header("x-goog-api-key", key)
        .timeout(std::time::Duration::from_secs(60));
    if let Some(b) = body {
        r = r.json(b)
    }
    let response = r.send().await.map_err(|_| "NETWORK_OUTCOME_UNKNOWN")?;
    if !response.status().is_success() {
        return Err(format!("GOOGLE_HTTP_{}", response.status().as_u16()));
    }
    response
        .json()
        .await
        .map_err(|_| "GOOGLE_INVALID_RESPONSE".into())
}
pub fn entries(value: &Value) -> Option<Vec<Value>> {
    for v in [
        &value["response"]["inlinedResponses"],
        &value["output"]["inlinedResponses"],
        &value["dest"]["inlinedResponses"],
        &value["inlinedResponses"],
    ] {
        if let Some(a) = v.as_array() {
            return Some(a.clone());
        }
        if let Some(a) = v["inlinedResponses"].as_array() {
            return Some(a.clone());
        }
    }
    None
}
pub fn keyed(entries: &[Value], keys: &[String]) -> Res<Vec<(String, Value)>> {
    let mut seen = std::collections::HashSet::new();
    let mut out = vec![];
    for (i, entry) in entries.iter().enumerate() {
        let key = entry["metadata"]["key"]
            .as_str()
            .or(entry["key"].as_str())
            .map(str::to_owned)
            .or_else(|| {
                if entries.len() == keys.len() {
                    keys.get(i).cloned()
                } else {
                    None
                }
            })
            .ok_or("MISSING_RESULT_KEY")?;
        if !keys.contains(&key) || !seen.insert(key.clone()) {
            return Err("INVALID_RESULT_KEY".into());
        }
        out.push((key, entry.clone()));
    }
    Ok(out)
}
async fn outputs(value: &Value) -> Res<Vec<Value>> {
    if let Some(e) = entries(value) {
        return Ok(e);
    }
    let f = value["response"]["responsesFile"]
        .as_str()
        .or(value["output"]["responsesFile"].as_str())
        .or(value["dest"]["fileName"].as_str())
        .ok_or("BATCH_RESULTS_MISSING")?;
    resource(f, "files/")?;
    let key = crate::credential()?
        .get_password()
        .map_err(|_| "GOOGLE_KEY_REQUIRED")?;
    let r = reqwest::Client::new()
        .get(format!(
            "https://generativelanguage.googleapis.com/download/v1beta/{f}:download?alt=media"
        ))
        .header("x-goog-api-key", key)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|_| "RESULT_DOWNLOAD_FAILED")?;
    if !r.status().is_success() {
        return Err(format!("RESULT_DOWNLOAD_HTTP_{}", r.status().as_u16()));
    }
    r.text()
        .await
        .map_err(|_| "INVALID_JSONL".to_string())?
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).map_err(|_| "INVALID_JSONL".into()))
        .collect()
}
fn receive(root: &Path, job: &mut Job, index: usize, values: Vec<Value>) -> Res<()> {
    let mapped = keyed(&values, &job.chunks[index].keys)?;
    for (key, entry) in mapped {
        let item = job
            .items
            .iter_mut()
            .find(|i| i.key == key)
            .ok_or("UNKNOWN_ITEM")?;
        if matches!(item.stage.as_str(), "done" | "generated") {
            continue;
        }
        let result = (|| -> Res<Value> {
            if !entry["error"].is_null() {
                return Err(format!("GENERATION_FAILED: {}", entry["error"]));
            }
            let r = if entry["response"].is_object() {
                &entry["response"]
            } else {
                &entry
            };
            let parts = r["candidates"][0]["content"]["parts"]
                .as_array()
                .ok_or("NO_IMAGE")?;
            let data = parts
                .iter()
                .filter(|p| p["thought"] != true)
                .find_map(|p| {
                    p["inlineData"]["data"]
                        .as_str()
                        .or(p["inline_data"]["data"].as_str())
                })
                .ok_or("NO_IMAGE")?;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(data)
                .map_err(|_| "INVALID_IMAGE")?;
            let file = dir(root, &job.id)?.join(format!("{key}-generated.png"));
            fs::write(&file, bytes).map_err(|e| e.to_string())?;
            core::decode(&file)?;
            Ok(r["usageMetadata"].clone())
        })();
        match result {
            Ok(usage) => {
                item.stage = "generated".into();
                item.usage = usage
            }
            Err(e) => {
                item.stage = "failed".into();
                item.error = Some(e)
            }
        }
    }
    for key in &job.chunks[index].keys {
        if let Some(item) = job.items.iter_mut().find(|i| i.key == *key) {
            if item.stage == "submitted" {
                item.stage = "failed".into();
                item.error = Some("BATCH_RESULT_MISSING".into())
            }
        }
    }
    job.chunks[index].state = "completed".into();
    save(root, job)
}
async fn reconcile(chunk: &Chunk) -> Res<Option<String>> {
    let mut token = String::new();
    for _ in 0..5 {
        let path = if token.is_empty() {
            "batches?pageSize=100".into()
        } else {
            format!("batches?pageSize=100&pageToken={}", token)
        };
        let v = google(reqwest::Method::GET, &path, None).await?;
        for op in v["operations"]
            .as_array()
            .or(v["batches"].as_array())
            .into_iter()
            .flatten()
        {
            let name = op["metadata"]["displayName"]
                .as_str()
                .or(op["displayName"].as_str());
            if name == Some(&chunk.display_name) {
                let n = op["name"].as_str().ok_or("BATCH_NAME_MISSING")?;
                resource(n, "batches/")?;
                return Ok(Some(n.into()));
            }
        }
        token = v["nextPageToken"].as_str().unwrap_or("").into();
        if token.is_empty() {
            break;
        }
    }
    Ok(None)
}
fn export_result(
    image: &image::DynamicImage,
    o: &Output,
    folder: &str,
    name: &str,
    key: &str,
) -> Res<String> {
    fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    let mut im = core::fit(image, o.width, o.height, o.fit == "pad", o.background);
    if o.format == "jpeg" {
        im = core::flatten(&im, o.background)
    }
    let preset = ExportPreset {
        format: o.format.clone(),
        width: o.width,
        height: o.height,
        fit: o.fit.clone(),
        quality: 92,
        max_bytes: o.max_bytes,
        output_dir: folder.into(),
        background: o.background,
    };
    let (bytes, _) = core::encode_target(&im, &preset)?;
    if o.max_bytes > 0 && bytes.len() as u64 > o.max_bytes {
        return Err(format!(
            "EXPORT_TARGET_UNREACHABLE: 最近 {} 字节",
            bytes.len()
        ));
    }
    let stem = Path::new(name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .chars()
        .map(|c| if "<>:\"/\\|?*".contains(c) { '_' } else { c })
        .take(70)
        .collect::<String>();
    for n in 0..100 {
        let suffix = if n == 0 {
            String::new()
        } else {
            format!("_{n}")
        };
        let p = Path::new(folder).join(format!("{stem}_{}{suffix}.{}", &key[..8], o.format));
        if p.exists() {
            if fs::read(&p).ok().as_deref() == Some(bytes.as_slice()) {
                return Ok(p.to_string_lossy().into());
            }
            continue;
        }
        let mut temp = tempfile::NamedTempFile::new_in(folder).map_err(|e| e.to_string())?;
        temp.write_all(&bytes).map_err(|e| e.to_string())?;
        temp.as_file().sync_all().map_err(|e| e.to_string())?;
        match temp.persist_noclobber(&p) {
            Ok(_) => {
                return Ok(p.to_string_lossy().into());
            }
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("OUTPUT_NAME_CONFLICT".into())
}
fn finish(s: &Shared, root: &Path, job: &mut Job) -> Res<()> {
    for i in 0..job.items.len() {
        if !matches!(job.items[i].stage.as_str(), "generated" | "ready") {
            continue;
        }
        if job.recipe.generation.is_some() && job.items[i].stage != "generated" {
            continue;
        }
        if dir(root, &job.id)?.join("cancel.request").exists() {
            break;
        }
        let result = (|| -> Res<(String, String)> {
            let key = &job.items[i].key;
            let kind = if job.recipe.generation.is_some() {
                "generated"
            } else {
                "prepared"
            };
            let im = core::decode(&dir(root, &job.id)?.join(format!("{key}-{kind}.png")))?;
            let im = apply_steps(im, &job.recipe.after, false, &mut job.items[i].warnings)?;
            let store = locked(s)?;
            let p = store.load(&job.project_id)?;
            let a = p
                .assets
                .iter()
                .find(|a| a.id == job.items[i].asset_id && !a.trashed)
                .ok_or("ASSET_NOT_FOUND")?;
            let unchanged = signature(a) == job.items[i].signature;
            let prior = a
                .versions
                .iter()
                .find(|v| {
                    v.generation.as_ref().is_some_and(|m| {
                        m["hybridJobId"] == job.id && m["itemKey"] == job.items[i].key
                    })
                })
                .map(|v| v.id.clone());
            let version = if let Some(id) = prior {
                id
            } else {
                let metadata = json!({"hybridJobId":job.id,"itemKey":job.items[i].key,"recipe":job.recipe,"usageMetadata":job.items[i].usage,"priceSnapshot":job.price,"batchFactor":if job.recipe.generation.as_ref().is_some_and(|g|g.mode=="realtime"||crate::image_provider::is_poe(&g.model)){1.0}else{0.5}});
                if job.recipe.generation.is_some() {
                    store
                        .save_generated(&job.project_id, &job.items[i].asset_id, &im, metadata)?
                        .1
                } else {
                    store
                        .save_local_version(&job.project_id, &job.items[i].asset_id, &im, metadata)?
                        .1
                }
            };
            if !unchanged {
                job.items[i]
                    .warnings
                    .push("等待期间当前编辑已改变，结果已另存版本并导出".into());
            }
            drop(store);
            let file = export_result(
                &im,
                &job.recipe.output,
                &job.output_dir,
                &job.items[i].name,
                &job.items[i].key,
            )?;
            Ok((version, file))
        })();
        match result {
            Ok((vid, file)) => {
                job.items[i].version_id = Some(vid);
                job.items[i].exported = Some(file);
                job.items[i].stage = "done".into()
            }
            Err(e) => {
                job.items[i].error = Some(e);
                job.items[i].stage = "failed".into()
            }
        }
        save(root, job)?;
    }
    Ok(())
}
async fn immediate_tick(s:&Shared,root:&Path,job:&mut Job)->Res<()>{
 let g=job.recipe.generation.clone().unwrap();
 for i in 0..job.items.len(){if job.items[i].stage=="generating"{job.items[i].stage="uncertain".into();job.items[i].error=Some("服务曾中断，生成结果待确认。不会自动再次扣费提交。".into());}}
 if dir(root,&job.id)?.join("cancel.request").exists(){job.state="cancelled".into();return save(root,job)}
 if let Some(i)=job.items.iter().position(|i|i.stage=="ready"){
  job.items[i].stage="generating".into();job.state="generating".into();save(root,job)?;
  let input=fs::read(dir(root,&job.id)?.join(format!("{}-prepared.png",job.items[i].key))).map_err(|e|e.to_string())?;
  match crate::image_provider::generate(&g.model,&g.prompt,Some(input),&g.resolution,&g.ratio).await{
   Ok((im,v))=>{im.save(dir(root,&job.id)?.join(format!("{}-generated.png",job.items[i].key))).map_err(|e|e.to_string())?;job.items[i].usage=v.get("usageMetadata").cloned().unwrap_or(Value::Null);job.items[i].stage="generated".into();},
   Err(e)=>{job.items[i].stage=if e.starts_with("NETWORK_OUTCOME_UNKNOWN"){"uncertain"}else{"failed"}.into();job.items[i].error=Some(e);}
  }save(root,job)?;
 }
 finish(s,root,job)?;
 job.state=if job.items.iter().any(|i|i.stage=="ready"){"queued"}else if job.items.iter().any(|i|i.stage=="uncertain"){"attention"}else if job.items.iter().all(|i|i.stage=="done"){"completed"}else if job.items.iter().any(|i|i.stage=="done"){"partial"}else{"failed"}.into();job.updated_at=now();save(root,job)
}
pub async fn tick(s: &Shared, root: &Path, job: &mut Job) -> Res<()> {
    if terminal(&job.state) {
        return Ok(());
    }
    let mut cancelling = dir(root, &job.id)?.join("cancel.request").exists();
    if !cancelling && matches!(job.state.as_str(), "queued" | "preparing") {
        prepare(s, root, job)?;
    }
    cancelling = dir(root, &job.id)?.join("cancel.request").exists();
    if job.recipe.generation.as_ref().is_some_and(|g|g.mode=="realtime"||crate::image_provider::is_poe(&g.model)){return immediate_tick(s,root,job).await;}
    if job.recipe.generation.is_some() {
        for index in 0..job.chunks.len() {
            if matches!(
                job.chunks[index].state.as_str(),
                "completed" | "failed" | "cancelled"
            ) {
                continue;
            }
            if now().saturating_sub(job.chunks[index].last_poll) < 30 {
                continue;
            }
            job.chunks[index].last_poll = now();
            save(root, job)?;
            if job.chunks[index].remote_name.is_none() {
                if let Some(name) = reconcile(&job.chunks[index]).await? {
                    job.chunks[index].remote_name = Some(name);
                    job.chunks[index].state = "waiting".into();
                    job.error = None;
                    save(root, job)?;
                } else {
                    job.state = "attention".into();
                    job.error =
                        Some("提交结果未能确认；正在按唯一任务名查询，不会重复扣费提交".into());
                    continue;
                }
            }
            let name = job.chunks[index].remote_name.clone().unwrap();
            resource(&name, "batches/")?;
            if cancelling {
                google(reqwest::Method::POST, &format!("{name}:cancel"), None).await?;
            }
            let v = google(reqwest::Method::GET, &name, None).await?;
            let state = v["metadata"]["state"]
                .as_str()
                .or(v["state"].as_str())
                .unwrap_or("");
            if state == "JOB_STATE_SUCCEEDED" || (v["done"] == true && v["error"].is_null()) {
                let values = outputs(&v).await?;
                receive(root, job, index, values)?;
            } else if [
                "JOB_STATE_FAILED",
                "JOB_STATE_EXPIRED",
                "JOB_STATE_CANCELLED",
            ]
            .contains(&state)
                || !v["error"].is_null()
            {
                job.chunks[index].state = if state == "JOB_STATE_CANCELLED" {
                    "cancelled"
                } else {
                    "failed"
                }
                .into();
                for item in &mut job.items {
                    if job.chunks[index].keys.contains(&item.key) {
                        item.stage = "failed".into();
                        item.error = Some(format!("{state}: {}", v["error"]));
                    }
                }
                save(root, job)?;
            }
        }
        if job
            .chunks
            .iter()
            .any(|c| c.remote_name.is_none() && c.state == "submitting")
        {
            job.state = "attention".into();
            save(root, job)?;
            return Ok(());
        }
        if !cancelling {
            let mut keys = vec![];
            let mut requests = vec![];
            let mut bytes = 0;
            for item in &job.items {
                if item.stage != "ready" || job.chunks.iter().any(|c| c.keys.contains(&item.key)) {
                    continue;
                }
                let r = request(root, job, &item.key)?;
                let n = r.to_string().len();
                if n > 18_000_000 {
                    return Err("BATCH_INPUT_TOO_LARGE".into());
                }
                if !keys.is_empty() && (bytes + n > 12_000_000 || keys.len() >= 8) {
                    break;
                }
                bytes += n;
                keys.push(item.key.clone());
                requests.push(r);
            }
            if !keys.is_empty() {
                let display = format!("gvpak-{}-{}", job.id, job.chunks.len());
                job.chunks.push(Chunk {
                    display_name: display.clone(),
                    keys: keys.clone(),
                    state: "submitting".into(),
                    remote_name: None,
                    last_poll: now(),
                });
                job.state = "submitting".into();
                for item in &mut job.items {
                    if keys.contains(&item.key) {
                        item.stage = "submitted".into()
                    }
                }
                save(root, job)?;
                let g = job.recipe.generation.as_ref().unwrap();
                let body = json!({"batch":{"displayName":display,"inputConfig":{"requests":{"requests":requests}}}});
                match google(
                    reqwest::Method::POST,
                    &format!("models/{}:batchGenerateContent", g.model),
                    Some(&body),
                )
                .await
                {
                    Ok(v) => {
                        let n = v["name"].as_str().ok_or("BATCH_NAME_MISSING")?;
                        resource(n, "batches/")?;
                        let c = job.chunks.last_mut().unwrap();
                        c.remote_name = Some(n.into());
                        c.state = "waiting".into();
                        job.state = "waiting".into();
                        job.error = None;
                    }
                    Err(e) if e.starts_with("GOOGLE_HTTP_4") => {
                        job.chunks.last_mut().unwrap().state = "failed".into();
                        for item in &mut job.items {
                            if keys.contains(&item.key) {
                                item.stage = "failed".into();
                                item.error = Some(e.clone());
                            }
                        }
                    }
                    Err(e) => {
                        job.state = "attention".into();
                        job.error = Some(format!("{e}；提交结果不明，自动查询而不重发"));
                    }
                }
                save(root, job)?;
            }
        }
    }
    if !cancelling {
        finish(s, root, job)?;
    }
    let remote_active = job
        .chunks
        .iter()
        .any(|c| !matches!(c.state.as_str(), "completed" | "failed" | "cancelled"));
    if cancelling && !remote_active {
        job.state = "cancelled".into();
    } else if job
        .items
        .iter()
        .all(|i| matches!(i.stage.as_str(), "done" | "failed"))
    {
        let done = job.items.iter().filter(|i| i.stage == "done").count();
        job.state = if done == job.items.len() {
            "completed"
        } else if done > 0 {
            "partial"
        } else {
            "failed"
        }
        .into();
    } else if remote_active && job.state != "attention" {
        job.state = "waiting".into();
    }
    job.updated_at = now();
    if let Some(price) = &job.price {
        let mut low = 0.;
        let mut high = 0.;
        let mut measured = 0;
        for item in &job.items {
            if let Some(bounds) = models::cost_bounds(price, &item.usage) {
                low += bounds["minimumUsd"].as_f64().unwrap_or(0.) * 0.5;
                high += bounds["maximumUsd"].as_f64().unwrap_or(0.) * 0.5;
                measured += 1;
            }
        }
        if measured > 0 {
            job.cost_bounds = Some(
                json!({"minimumUsd":low,"maximumUsd":high,"measuredImages":measured,"note":"仅已返回用量按 Batch 费率折算，非账单"}),
            );
        }
    }
    save(root, job)
}
pub fn spawn(app: crate::transport::AppHandle, s: Shared) { spawn_queued(app,s,None); }
pub fn spawn_queued(app: crate::transport::AppHandle, s: Shared, queue: Option<std::sync::Arc<tokio::sync::Semaphore>>) {
    std::thread::spawn(move || {
        loop {
            let root = match locked(&s) {
                Ok(store) => store.root.clone(),
                Err(_) => break,
            };
            if let Ok(mut jobs) = list(&root) {
                for job in jobs.iter_mut().filter(|j| !terminal(&j.state)) {
                    let _permit = queue.as_ref().map(|q| crate::transport::runtime::block_on(q.clone().acquire_owned()).expect("queue open"));
                    let previous = job.state.clone();
                    if let Err(e) = crate::transport::runtime::block_on(tick(&s, &root, job)) {
                        job.error = Some(e);
                        job.updated_at = now();
                        let _ = save(&root, job);
                    }
                    let _ = app.emit("hybrid-progress", json!({"id":job.id,"state":job.state}));
                    if !terminal(&previous) && terminal(&job.state) {
                        let _ = app.emit("hybrid-completed", job.clone());

                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completed_cloud_result_resumes_without_another_submission() {
        let (tmp, s, pid, ids, recipe) = setup();
        let mut j = start(
            &s,
            pid,
            ids,
            "receive".into(),
            recipe,
            tmp.path().to_string_lossy().into(),
            false,
            uuid::Uuid::new_v4().to_string(),
        )
        .unwrap();
        let root = locked(&s).unwrap().root.clone();
        j.recipe.generation = Some(Generation {
            mode: String::new(),
            prompt: "test".into(),
            model: "gemini-nano-banana-2.1".into(),
            resolution: "1K".into(),
            ratio: "1:1".into(),
        });
        j.price = Some(models::load(&root).0.models[0].clone());
        j.state = "waiting".into();
        j.items[0].stage = "submitted".into();
        let key = j.items[0].key.clone();
        j.chunks.push(Chunk {
            display_name: "test".into(),
            keys: vec![key.clone()],
            state: "waiting".into(),
            remote_name: Some("batches/test".into()),
            last_poll: now(),
        });
        let generated = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            40,
            30,
            image::Rgb([10, 120, 220]),
        ));
        let data = base64::engine::general_purpose::STANDARD.encode(core::png(&generated).unwrap());
        receive(&root,&mut j,0,vec![json!({"metadata":{"key":key},"response":{"candidates":[{"content":{"parts":[{"inlineData":{"mimeType":"image/png","data":data}}]}}],"usageMetadata":{"promptTokenCount":100,"candidatesTokenCount":1120,"candidatesTokensDetails":[{"modality":"IMAGE","tokenCount":1120}]}}})]).unwrap();
        let mut restored = list(&root).unwrap().pop().unwrap();
        crate::transport::runtime::block_on(tick(&s, &root, &mut restored)).unwrap();
        assert_eq!(restored.state, "completed");
        assert!(restored.items[0].exported.is_some());
        assert!(
            (restored.cost_bounds.as_ref().unwrap()["maximumUsd"]
                .as_f64()
                .unwrap()
                - 0.016875)
                .abs()
                < 1e-9
        );
        assert_eq!(
            locked(&s)
                .unwrap()
                .load(&restored.project_id)
                .unwrap()
                .assets[0]
                .versions
                .len(),
            1
        );
    }
    fn setup() -> (tempfile::TempDir, Shared, String, Vec<String>, Recipe) {
        let tmp = tempfile::tempdir().unwrap();
        let store = core::Store::open(tmp.path().join("data"), "test").unwrap();
        let p = store.create("test".into()).unwrap();
        let file = tmp.path().join("source.png");
        image::RgbaImage::from_pixel(40, 30, image::Rgba([120, 80, 40, 255]))
            .save(&file)
            .unwrap();
        let p = store
            .import(
                &p.id,
                vec![file.to_string_lossy().into()],
                &AtomicBool::new(false),
                |_, _| {},
            )
            .unwrap()
            .project;
        let ids = vec![p.assets[0].id.clone()];
        let recipe = Recipe {
            summary: "local".into(),
            before: vec![Step::Rotate { degrees: 90 }],
            generation: None,
            after: vec![],
            output: Output {
                format: "webp".into(),
                width: 100,
                height: 100,
                fit: "pad".into(),
                max_bytes: 800000,
                background: [255; 3],
            },
            clarification: None,
            warnings: vec![],
        };
        let s = std::sync::Arc::new(crate::AppState {
            store: std::sync::Mutex::new(store),
            cancel: AtomicBool::new(false),
            local_job: std::sync::Mutex::new(()),
            preview_epoch: std::sync::atomic::AtomicU64::new(0),
        });
        (tmp, s, p.id, ids, recipe)
    }
    #[test]
    fn persistent_local_job_and_idempotent_start() {
        let (tmp, s, pid, ids, recipe) = setup();
        let id = uuid::Uuid::new_v4().to_string();
        let out = tmp.path().to_string_lossy().to_string();
        let job = start(
            &s,
            pid.clone(),
            ids.clone(),
            "rotate".into(),
            recipe.clone(),
            out.clone(),
            false,
            id.clone(),
        )
        .unwrap();
        assert_eq!(
            start(&s, pid, ids, "rotate".into(), recipe, out, false, id)
                .unwrap()
                .id,
            job.id
        );
        let root = locked(&s).unwrap().root.clone();
        let mut restored = list(&root).unwrap().pop().unwrap();
        crate::transport::runtime::block_on(tick(&s, &root, &mut restored)).unwrap();
        assert_eq!(restored.state, "completed");
        assert!(Path::new(restored.items[0].exported.as_ref().unwrap()).exists());
        let project = locked(&s).unwrap().load(&restored.project_id).unwrap();
        assert_eq!(project.assets[0].versions.len(), 1);
        assert!(project.assets[0].generated_base.is_none());
        crate::transport::runtime::block_on(tick(&s, &root, &mut restored)).unwrap();
        assert_eq!(
            locked(&s)
                .unwrap()
                .load(&restored.project_id)
                .unwrap()
                .assets[0]
                .versions
                .len(),
            1
        );
    }
    #[test]
    fn queued_cancellation_never_creates_versions() {
        let (tmp, s, pid, ids, recipe) = setup();
        let mut j = start(
            &s,
            pid,
            ids,
            "cancel".into(),
            recipe,
            tmp.path().to_string_lossy().into(),
            false,
            uuid::Uuid::new_v4().to_string(),
        )
        .unwrap();
        let root = locked(&s).unwrap().root.clone();
        request_cancel(&root, &j.id).unwrap();
        crate::transport::runtime::block_on(tick(&s, &root, &mut j)).unwrap();
        assert_eq!(j.state, "cancelled");
        assert!(
            locked(&s).unwrap().load(&j.project_id).unwrap().assets[0]
                .versions
                .is_empty()
        );
    }
    #[test]
    fn unknown_submission_is_not_resubmitted() {
        let (tmp, s, pid, ids, recipe) = setup();
        let mut j = start(
            &s,
            pid,
            ids,
            "uncertain".into(),
            recipe,
            tmp.path().to_string_lossy().into(),
            false,
            uuid::Uuid::new_v4().to_string(),
        )
        .unwrap();
        let root = locked(&s).unwrap().root.clone();
        j.recipe.generation = Some(Generation {
            mode: String::new(),
            prompt: "test".into(),
            model: "gemini-nano-banana-2.1".into(),
            resolution: "1K".into(),
            ratio: "1:1".into(),
        });
        j.state = "submitting".into();
        j.items[0].stage = "submitted".into();
        j.chunks.push(Chunk {
            display_name: "unique".into(),
            keys: vec![j.items[0].key.clone()],
            state: "submitting".into(),
            remote_name: None,
            last_poll: now(),
        });
        crate::transport::runtime::block_on(tick(&s, &root, &mut j)).unwrap();
        assert_eq!(j.state, "attention");
        assert_eq!(j.chunks.len(), 1);
    }
    #[test]
    fn keyed_results_do_not_depend_on_order() {
        let keys = vec!["a".into(), "b".into()];
        let rows = vec![
            json!({"metadata":{"key":"b"},"response":{"ok":2}}),
            json!({"metadata":{"key":"a"},"response":{"ok":1}}),
        ];
        let mapped = keyed(&rows, &keys).unwrap();
        assert_eq!(mapped[0].0, "b");
        assert!(keyed(&[rows[0].clone(), rows[0].clone()], &keys).is_err());
        assert!(keyed(&[json!({"response":{}})], &keys).is_err());
    }
    #[test]
    fn reads_nested_official_response() {
        let v = json!({"response":{"inlinedResponses":{"inlinedResponses":[{"metadata":{"key":"a"},"response":{}}]}}});
        assert_eq!(entries(&v).unwrap().len(), 1);
        assert!(resource("batches/abc-123", "batches/").is_ok());
        assert!(resource("batches/../files/key", "batches/").is_err());
    }
    #[test]
    fn exports_are_recoverable_and_bounded() {
        let tmp = tempfile::tempdir().unwrap();
        let im = image::DynamicImage::new_rgb8(30, 20);
        let o = Output {
            format: "webp".into(),
            width: 1000,
            height: 1000,
            fit: "pad".into(),
            max_bytes: 800000,
            background: [255; 3],
        };
        let key = uuid::Uuid::new_v4().to_string();
        let p = export_result(&im, &o, tmp.path().to_str().unwrap(), "中文.png", &key).unwrap();
        assert_eq!(
            p,
            export_result(&im, &o, tmp.path().to_str().unwrap(), "中文.png", &key).unwrap()
        );
        let image = core::decode(Path::new(&p)).unwrap();
        assert_eq!((image.width(), image.height()), (1000, 1000));
        assert!(fs::metadata(p).unwrap().len() <= 800000);
    }
}
