use crate::core::{self, Edit, ExportPreset, Project, Res, Store};
use base64::Engine;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Step {
    AutoCenter {
        padding: f32,
    },
    Deskew {
        max_degrees: f32,
    },
    Adjust {
        exposure: f32,
        contrast: f32,
        saturation: f32,
        temperature: f32,
    },
    Rotate {
        degrees: u32,
    },
    Flip {
        axis: String,
    },
    Crop {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    Resize {
        width: u32,
        height: u32,
        fit: String,
        background: [u8; 3],
    },
    Flatten {
        background: [u8; 3],
    },
    Blur {
        sigma: f32,
    },
    Sharpen {
        sigma: f32,
        threshold: i32,
    },
    Denoise {
        strength: f32,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Output {
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub fit: String,
    pub max_bytes: u64,
    pub background: [u8; 3],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Plan {
    pub summary: String,
    pub steps: Vec<Step>,
    pub warnings: Vec<String>,
    pub clarification: Option<String>,
    pub output: Option<Output>,
}
fn size(w: u32, h: u32) -> bool {
    w > 0 && h > 0 && w <= 16000 && h <= 16000 && w as u64 * h as u64 <= 120_000_000
}
impl Plan {
    pub fn validate(&self) -> Res<()> {
        if self.summary.len() > 4000 || self.steps.len() > 20 || self.warnings.len() > 20 {
            return Err("INVALID_PLAN: 步骤或说明过长".into());
        }
        for (i, s) in self.steps.iter().enumerate() {
            let ok = match s {
                Step::AutoCenter { padding } => (0.0..=0.35).contains(padding),
                Step::Deskew { max_degrees } => (1.0..=45.).contains(max_degrees),
                Step::Adjust {
                    exposure,
                    contrast,
                    saturation,
                    temperature,
                } => {
                    (-3.0..=3.).contains(exposure)
                        && (0.0..=3.).contains(contrast)
                        && (0.0..=3.).contains(saturation)
                        && (-1.0..=1.).contains(temperature)
                }
                Step::Rotate { degrees } => [0, 90, 180, 270].contains(degrees),
                Step::Flip { axis } => ["horizontal", "vertical"].contains(&axis.as_str()),
                Step::Crop {
                    x,
                    y,
                    width,
                    height,
                } => {
                    [*x, *y, *width, *height].iter().all(|v| v.is_finite())
                        && *x >= 0.
                        && *y >= 0.
                        && *width > 0.
                        && *height > 0.
                        && x + width <= 1.
                        && y + height <= 1.
                }
                Step::Resize {
                    width, height, fit, ..
                } => size(*width, *height) && ["crop", "pad"].contains(&fit.as_str()),
                Step::Flatten { .. } => true,
                Step::Blur { sigma } => (0.1..=10.).contains(sigma),
                Step::Sharpen { sigma, threshold } => {
                    (0.1..=5.).contains(sigma) && (0..=100).contains(threshold)
                }
                Step::Denoise { strength } => (1.0..=20.).contains(strength),
            };
            if !ok {
                return Err(format!("INVALID_PLAN: 第 {} 步参数超出范围", i + 1));
            }
        }
        if let Some(o) = &self.output {
            if !size(o.width, o.height)
                || !["webp", "png", "jpeg"].contains(&o.format.as_str())
                || !["crop", "pad"].contains(&o.fit.as_str())
                || o.max_bytes > 1_000_000_000
            {
                return Err("INVALID_PLAN: 导出参数无效".into());
            }
        }
        Ok(())
    }
    fn executable(&self) -> Res<()> {
        self.validate()?;
        if self
            .clarification
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty())
        {
            return Err("PLAN_NEEDS_CLARIFICATION: 请先补充指令".into());
        }
        if self.steps.is_empty() && self.output.is_none() {
            return Err("EMPTY_PLAN: 没有可执行步骤".into());
        }
        Ok(())
    }
}
fn python() -> Command {
    let executable = std::env::var_os("GVPAK_PYTHON").unwrap_or("python".into());
    let mut c = Command::new(executable);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x08000000);
    }
    c.args(["-I", "-c", include_str!("opencv_worker.py")]);
    c
}
pub fn opencv_status() -> Value {
    match python().arg("status").output() {
        Ok(o) if o.status.success() => {
            json!({"available":true,"version":String::from_utf8_lossy(&o.stdout).trim()})
        }
        _ => {
            json!({"available":false,"message":"OpenCV 不可用；其余本地操作仍可执行。可通过 GVPAK_PYTHON 指定已安装 cv2 的 Python。"})
        }
    }
}
fn denoise(im: DynamicImage, strength: f32) -> Res<DynamicImage> {
    opencv(im, "denoise", strength)
}
pub fn opencv(im: DynamicImage, mode: &str, value: f32) -> Res<DynamicImage> {
    if im.width() as u64 * im.height() as u64 > 16_000_000 {
        return Err("OPENCV_IMAGE_TOO_LARGE: 本地视觉处理前请缩小到 1600 万像素以内".into());
    }
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let input = dir.path().join("input.png");
    let output = dir.path().join("output.png");
    im.save(&input).map_err(|e| e.to_string())?;
    let result = python()
        .arg(&input)
        .arg(&output)
        .arg(mode)
        .arg(value.to_string())
        .output()
        .map_err(|_| "OPENCV_UNAVAILABLE: Python/OpenCV 不可用")?;
    if !result.status.success() {
        return Err(format!(
            "OPENCV_FAILED: 本地 {mode} 未能可靠完成；请检查 OpenCV 环境、图片大小或主体轮廓"
        ));
    }
    image::open(output).map_err(|_| "OPENCV_INVALID_OUTPUT".into())
}
pub fn process(mut im: DynamicImage, plan: &Plan, cancel: &AtomicBool) -> Res<DynamicImage> {
    plan.executable()?;
    for step in &plan.steps {
        if cancel.load(Ordering::Relaxed) {
            return Err("CANCELLED".into());
        }
        im = match step {
            Step::AutoCenter { padding } => opencv(im, "center", *padding)?,
            Step::Deskew { max_degrees } => opencv(im, "deskew", *max_degrees)?,
            Step::Adjust {
                exposure,
                contrast,
                saturation,
                temperature,
            } => core::render(
                im,
                &Edit {
                    exposure: *exposure,
                    contrast: *contrast,
                    saturation: *saturation,
                    temperature: *temperature,
                    ..Edit::default()
                },
                None,
            )?,
            Step::Rotate { degrees } => core::render(
                im,
                &Edit {
                    rotation: *degrees,
                    ..Edit::default()
                },
                None,
            )?,
            Step::Flip { axis } => {
                if axis == "horizontal" {
                    im.fliph()
                } else {
                    im.flipv()
                }
            }
            Step::Crop {
                x,
                y,
                width,
                height,
            } => core::render(
                im,
                &Edit {
                    crop: Some([*x, *y, *width, *height]),
                    ..Edit::default()
                },
                None,
            )?,
            Step::Resize {
                width,
                height,
                fit,
                background,
            } => core::fit(&im, *width, *height, fit == "pad", *background),
            Step::Flatten { background } => core::flatten(&im, *background),
            Step::Blur { sigma } => im.blur(*sigma),
            Step::Sharpen { sigma, threshold } => im.unsharpen(*sigma, *threshold),
            Step::Denoise { strength } => denoise(im, *strength)?,
        };
    }
    Ok(im)
}
fn fingerprint(p: &Project, aid: &str) -> Res<String> {
    let a = p
        .assets
        .iter()
        .find(|a| a.id == aid && !a.trashed)
        .ok_or("ASSET_NOT_FOUND")?;
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(a).map_err(|e| e.to_string())?)
    ))
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sample {
    id: String,
    project_id: String,
    ids: Vec<String>,
    fingerprints: BTreeMap<String, String>,
    plan: Plan,
    status: String,
}
fn sample_path(root: &Path, id: &str) -> Res<std::path::PathBuf> {
    uuid::Uuid::parse_str(id).map_err(|_| "INVALID_SAMPLE_ID")?;
    Ok(root.join("assistant-samples").join(format!("{id}.json")))
}
pub fn preview(store: &Store, pid: &str, ids: Vec<String>, plan: Plan) -> Res<Value> {
    plan.executable()?;
    if ids.is_empty() {
        return Err("INVALID_SCOPE: 请选择 1–500 张图片".into());
    }
    let p = store.load(pid)?;
    let mut fingerprints = BTreeMap::new();
    for aid in &ids {
        if fingerprints
            .insert(aid.clone(), fingerprint(&p, aid)?)
            .is_some()
        {
            return Err("INVALID_SCOPE: 图片重复".into());
        }
    }
    let first = &ids[0];
    let a = p
        .assets
        .iter()
        .find(|a| a.id == *first)
        .ok_or("ASSET_NOT_FOUND")?;
    let before = core::render(store.source(&p, first)?, &a.edit, None)?;
    let after = process(before.clone(), &plan, &AtomicBool::new(false))?;
    let display = if let Some(o) = &plan.output {
        let fitted = core::fit(&after, o.width, o.height, o.fit == "pad", o.background);
        if o.format == "jpeg" {
            core::flatten(&fitted, o.background)
        } else {
            fitted
        }
    } else {
        after
    };
    let id = uuid::Uuid::new_v4().to_string();
    let path = sample_path(&store.root, &id)?;
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let sample = Sample {
        id: id.clone(),
        project_id: pid.into(),
        ids,
        fingerprints,
        plan,
        status: "ready".into(),
    };
    fs::write(
        path,
        serde_json::to_vec_pretty(&sample).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(
        json!({"sampleId":id,"width":display.width(),"height":display.height(),"before":base64::engine::general_purpose::STANDARD.encode(core::png(&before.thumbnail(1200,1200))?),"after":base64::engine::general_purpose::STANDARD.encode(core::png(&display.thumbnail(1200,1200))?)}),
    )
}
pub fn apply(
    store: &Store,
    sample_id: &str,
    output_dir: Option<String>,
    cancel: &AtomicBool,
    mut progress: impl FnMut(usize, usize),
) -> Res<Value> {
    let path = sample_path(&store.root, sample_id)?;
    let mut sample: Sample =
        serde_json::from_slice(&fs::read(&path).map_err(|_| "SAMPLE_NOT_FOUND")?)
            .map_err(|_| "INVALID_SAMPLE")?;
    if sample.id != sample_id || sample.status != "ready" {
        return Err("SAMPLE_ALREADY_USED: 请重新试一张后再应用".into());
    }
    sample.plan.executable()?;
    if sample.plan.output.is_some()
        && !output_dir
            .as_ref()
            .is_some_and(|p| Path::new(p).is_absolute() && Path::new(p).is_dir())
    {
        return Err("OUTPUT_REQUIRED: 请选择导出目录".into());
    }
    let current = store.load(&sample.project_id)?;
    for aid in &sample.ids {
        if Some(&fingerprint(&current, aid)?) != sample.fingerprints.get(aid) {
            return Err("SAMPLE_STALE: 图片发生变化，请重新试一张".into());
        }
    }
    sample.status = "running".into();
    fs::write(
        &path,
        serde_json::to_vec_pretty(&sample).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut results = vec![];
    for (index, aid) in sample.ids.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let attempt = (|| -> Res<Value> {
            let p = store.load(&sample.project_id)?;
            let a = p
                .assets
                .iter()
                .find(|a| a.id == *aid && !a.trashed)
                .ok_or("ASSET_NOT_FOUND")?;
            let im = core::render(store.source(&p, aid)?, &a.edit, None)?;
            let im = process(im, &sample.plan, cancel)?;
            if cancel.load(Ordering::Relaxed) {
                return Err("CANCELLED".into());
            }
            store.save_processed(
                &sample.project_id,
                aid,
                &im,
                json!({"engine":"local-rust-opencv","sampleId":sample_id,"plan":sample.plan}),
            )?;
            let exported = if let Some(o) = &sample.plan.output {
                store.export(
                    &sample.project_id,
                    vec![aid.clone()],
                    ExportPreset {
                        format: o.format.clone(),
                        width: o.width,
                        height: o.height,
                        fit: o.fit.clone(),
                        quality: 92,
                        max_bytes: o.max_bytes,
                        output_dir: output_dir.clone().unwrap(),
                        background: o.background,
                    },
                    cancel,
                    |_, _| {},
                )?
            } else {
                vec![]
            };
            let export_cancelled = sample.plan.output.is_some() && exported.is_empty() && cancel.load(Ordering::Relaxed);
            let failures: Vec<_> = exported.iter().filter(|item| item.status != "ok").map(|item| item.status.clone()).collect();
            let status = if export_cancelled { "cancelled" } else if failures.is_empty() { "ok" } else { "partial" };
            let error = if export_cancelled { Some("CANCELLED: export was not processed".to_string()) }
                else if failures.is_empty() { None } else { Some(failures.join("; ")) };
            Ok(json!({"assetId":aid,"status":status,"processingStatus":"ok","error":error,"exports":exported}))
        })();
        results.push(match attempt {
            Ok(v) => v,
            Err(e) => json!({"assetId":aid,"status":"failed","error":e}),
        });
        progress(index + 1, sample.ids.len());
    }
    sample.status = if cancel.load(Ordering::Relaxed) {
        "cancelled"
    } else if results.iter().any(|item| item["status"] != "ok") {
        if results.iter().any(|item| item["status"] == "ok" || item["status"] == "partial") { "partial" } else { "failed" }
    } else {
        "completed"
    }
    .into();
    fs::write(
        path,
        serde_json::to_vec_pretty(&sample).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let report = json!({"project":store.load(&sample.project_id)?,"results":results,"status":sample.status,"total":sample.ids.len(),"processed":results.len()});
    fs::write(
        store
            .root
            .join("assistant-samples")
            .join(format!("{sample_id}-report.json")),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(report)
}
pub fn presets(store: &Store) -> Res<BTreeMap<String, Plan>> {
    let path = store.root.join("assistant-presets.json");
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|_| "INVALID_PRESETS".into())
}
pub fn save_preset(store: &Store, name: String, plan: Plan) -> Res<()> {
    plan.executable()?;
    if name.trim().is_empty() || name.chars().count() > 60 {
        return Err("INVALID_PRESET_NAME".into());
    }
    let mut p = presets(store)?;
    p.insert(name.trim().into(), plan);
    fs::write(
        store.root.join("assistant-presets.json"),
        serde_json::to_vec_pretty(&p).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
pub fn credential() -> Res<keyring::Entry> {
    crate::branding::credential("deepseek-official")
}

const PROMPT: &str = r#"You are the GVPAK local image tool planner. Return a json object only: {"summary":"中文摘要","steps":[],"warnings":[],"clarification":null,"output":null}.
Allowed steps (only these exact keys):
{"op":"adjust","exposure":0.0,"contrast":1.0,"saturation":1.0,"temperature":0.0} exposure [-3,3] EV, contrast/saturation [0,3], temperature [-1,1].
{"op":"rotate","degrees":90} degrees 0/90/180/270 clockwise.
{"op":"flip","axis":"horizontal"} horizontal/vertical.
{"op":"crop","x":0.0,"y":0.0,"width":1.0,"height":1.0} normalized rect wholly inside image.
{"op":"resize","width":1000,"height":1000,"fit":"pad","background":[255,255,255]} fit crop/pad. Ask clarification if target ratio differs and user did not specify crop/pad. Never stretch.
{"op":"flatten","background":[255,255,255]} fills existing transparency ONLY; cannot remove a photographed background.
{"op":"blur","sigma":1.0} sigma 0.1..10.
{"op":"sharpen","sigma":1.0,"threshold":3} sigma 0.1..5, threshold 0..100.
{"op":"denoise","strength":3.0} strength 1..20, local OpenCV; <=16 million pixels, resize first if needed.
Optional output: {"format":"webp","width":1000,"height":1000,"fit":"pad","maxBytes":500000,"background":[255,255,255]}. Formats webp/png/jpeg. KB = 1000 bytes. No paths. Output directory chosen by user locally. Use output when format/compression/export requested. Dimensions <=16000 each and <=120 million pixels.
If request needs semantic masking, background removal on opaque photos, object changes, relighting or generation: do not simulate with flatten/blur; set clarification in Chinese explaining supported local options or explicit Google generate mode, with no automatic cloud image calls. Compositing/layers/paste-image operations are NOT available; never offer them as implemented alternatives. You only see instructions and dimensions, never claim to have inspected pixels. Preserve product shape/logo/colors unless explicitly asked to change. At most 20 steps; execute in order. Unknown/ambiguous requests require clarification; no partial unsupported operation disguised as done. Steps apply after existing edits. Reuse these steps for the whole selection. Never emit code, shell commands, URLs or file access operations. All user-facing strings in clear, concise Chinese."#;
pub async fn plan(instruction: String, context: Value) -> Res<Value> {
    let result = query(PROMPT, instruction, context).await?;
    let parsed: Plan =
        serde_json::from_value(result["plan"].clone()).map_err(|e| format!("INVALID_PLAN: {e}"))?;
    parsed.validate()?;
    Ok(result)
}
pub async fn query(system: &str, instruction: String, context: Value) -> Res<Value> {
    if instruction.trim().is_empty() || instruction.len() > 6000 {
        return Err("INVALID_INSTRUCTION: 请输入不超过 6000 字节的指令".into());
    }
    let key = credential()?
        .get_password()
        .map_err(|_| "DEEPSEEK_KEY_REQUIRED: 请在设置中配置 DeepSeek 密钥")?;
    let response=reqwest::Client::new().post("https://api.deepseek.com/chat/completions").bearer_auth(key)
        .json(&json!({"model":"deepseek-flash","thinking":{"type":"disabled"},"temperature":0,"max_tokens":1600,"response_format":{"type":"json_object"},"messages":[{"role":"system","content":system},{"role":"user","content":json!({"instruction":instruction,"context":context}).to_string()}]}))
        .timeout(std::time::Duration::from_secs(60)).send().await.map_err(|_|"DEEPSEEK_NETWORK_ERROR: 请求未取得结果，未自动重试")?;
    if !response.status().is_success() {
        return Err(format!(
            "DEEPSEEK_API_ERROR: HTTP {}；未自动重试",
            response.status().as_u16()
        ));
    }
    let v: Value = response
        .json()
        .await
        .map_err(|_| "DEEPSEEK_INVALID_RESPONSE")?;
    if v["choices"][0]["finish_reason"] != "stop" {
        return Err("DEEPSEEK_INCOMPLETE_PLAN: 返回未完成，未执行任何步骤".into());
    }
    let content = v["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("DEEPSEEK_EMPTY_PLAN")?;
    let plan: Value = serde_json::from_str(content).map_err(|e| format!("INVALID_PLAN: {e}"))?;
    let usage = &v["usage"];
    let cost=usage["prompt_tokens"].as_u64().zip(usage["completion_tokens"].as_u64()).map(|(input,output)| {
        let hits=usage["prompt_cache_hit_tokens"].as_u64().unwrap_or(0).min(input);
        let low=((input-hits) as f64*0.15+hits as f64*0.003+output as f64*0.6)/1e6;
        json!({"minimumUsd":low,"maximumUsd":low*2.,"note":"按官方峰谷价格区间估算，非账单","source":"https://api-docs.deepseek.com/quick_start/pricing/","checkedAt":"2026-10-12"})
    });
    Ok(
        json!({"plan":plan,"usage":usage,"cost":cost,"model":v["model"],"route":"text-only planning; local pixels"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    fn plan() -> Plan {
        Plan {
            summary: "白底与旋转".into(),
            steps: vec![
                Step::Rotate { degrees: 90 },
                Step::Resize {
                    width: 100,
                    height: 100,
                    fit: "pad".into(),
                    background: [255; 3],
                },
            ],
            warnings: vec![],
            clarification: None,
            output: Some(Output {
                format: "webp".into(),
                width: 100,
                height: 100,
                fit: "pad".into(),
                max_bytes: 500000,
                background: [255; 3],
            }),
        }
    }
    #[test]
    fn rejects_arbitrary_tools_and_invalid_geometry() {
        assert!(serde_json::from_str::<Plan>(r#"{"summary":"x","steps":[{"op":"shell","command":"calc"}],"warnings":[],"clarification":null,"output":null}"#).is_err());
        let mut p = plan();
        p.steps = vec![Step::Crop {
            x: 0.9,
            y: 0.,
            width: 0.2,
            height: 1.,
        }];
        assert!(p.validate().is_err());
        p.steps = vec![Step::Resize {
            width: 16000,
            height: 16000,
            fit: "pad".into(),
            background: [255; 3],
        }];
        assert!(p.validate().is_err());
        p = plan();
        p.clarification = Some("请选择裁切还是补边".into());
        assert!(process(DynamicImage::new_rgba8(10, 10), &p, &AtomicBool::new(false)).is_err());
    }
    #[test]
    fn sample_gate_batch_export_originals_and_reuse() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().join("data"), "test").unwrap();
        let source = tmp.path().join("中文 source.png");
        RgbaImage::from_pixel(80, 40, Rgba([40, 100, 180, 180]))
            .save(&source)
            .unwrap();
        let original = fs::read(&source).unwrap();
        let p = store.create("pipeline".into()).unwrap();
        let imported = store
            .import(
                &p.id,
                vec![source.to_string_lossy().into()],
                &AtomicBool::new(false),
                |_, _| {},
            )
            .unwrap();
        let aid = imported.project.assets[0].id.clone();
        let sample = preview(&store, &p.id, vec![aid.clone()], plan()).unwrap();
        assert_eq!(store.load(&p.id).unwrap().assets[0].versions.len(), 0);
        let dir = tmp.path().join("export");
        fs::create_dir(&dir).unwrap();
        let token = sample["sampleId"].as_str().unwrap();
        let done = apply(
            &store,
            token,
            Some(dir.to_string_lossy().into()),
            &AtomicBool::new(false),
            |_, _| {},
        )
        .unwrap();
        assert_eq!(done["results"][0]["status"], "ok");
        assert_eq!(done["project"]["assets"][0]["versions"][0]["kind"], "local");
        let result =
            image::open(done["results"][0]["exports"][0]["path"].as_str().unwrap()).unwrap();
        assert_eq!((result.width(), result.height()), (100, 100));
        assert_eq!(fs::read(source).unwrap(), original);
        assert!(
            apply(
                &store,
                token,
                Some(dir.to_string_lossy().into()),
                &AtomicBool::new(false),
                |_, _| {}
            )
            .unwrap_err()
            .contains("ALREADY_USED")
        );
        let sample = preview(&store, &p.id, vec![aid.clone()], plan()).unwrap();
        store
            .commit(
                &p.id,
                &aid,
                Edit {
                    exposure: 0.5,
                    ..Edit::default()
                },
            )
            .unwrap();
        assert!(
            apply(
                &store,
                sample["sampleId"].as_str().unwrap(),
                Some(dir.to_string_lossy().into()),
                &AtomicBool::new(false),
                |_, _| {}
            )
            .unwrap_err()
            .contains("STALE")
        );
        save_preset(&store, "Test".into(), plan()).unwrap();
        assert_eq!(presets(&store).unwrap()["Test"].steps.len(), 2);
    }
    #[test]
    fn cancellation_does_not_save_new_version() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().join("data"), "test").unwrap();
        let p = store.create("cancel".into()).unwrap();
        let src = tmp.path().join("source.png");
        RgbaImage::from_pixel(20, 10, Rgba([1, 2, 3, 255]))
            .save(&src)
            .unwrap();
        let p = store
            .import(
                &p.id,
                vec![src.to_string_lossy().into()],
                &AtomicBool::new(false),
                |_, _| {},
            )
            .unwrap()
            .project;
        let mut plan = plan();
        plan.output = None;
        let s = preview(&store, &p.id, vec![p.assets[0].id.clone()], plan).unwrap();
        let report = apply(
            &store,
            s["sampleId"].as_str().unwrap(),
            None,
            &AtomicBool::new(true),
            |_, _| {},
        )
        .unwrap();
        assert_eq!(report["status"], "cancelled");
        assert_eq!(report["processed"], 0);
        assert!(store.load(&p.id).unwrap().assets[0].versions.is_empty());
    }
    #[test]
    fn per_image_failure_and_cancel_after_first() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().join("data"), "test").unwrap();
        let p = store.create("batch".into()).unwrap();
        let mut paths = vec![];
        for n in 0..3 {
            let path = tmp.path().join(format!("source-{n}.png"));
            RgbaImage::from_pixel(24, 16, Rgba([n * 40, 20, 30, 255]))
                .save(&path)
                .unwrap();
            paths.push(path.to_string_lossy().into());
        }
        let p = store
            .import(&p.id, paths, &AtomicBool::new(false), |_, _| {})
            .unwrap()
            .project;
        let ids = p.assets.iter().map(|a| a.id.clone()).collect::<Vec<_>>();
        let mut plan = plan();
        plan.output = None;
        let sample = preview(&store, &p.id, ids.clone(), plan.clone()).unwrap();
        let corrupt = store
            .root
            .join("projects")
            .join(&p.id)
            .join(&p.assets[1].original);
        let mut permissions = fs::metadata(&corrupt).unwrap().permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&corrupt, permissions).unwrap();
        fs::write(corrupt, b"broken").unwrap();
        let done = apply(
            &store,
            sample["sampleId"].as_str().unwrap(),
            None,
            &AtomicBool::new(false),
            |_, _| {},
        )
        .unwrap();
        assert_eq!(done["results"][0]["status"], "ok");
        assert_eq!(done["results"][1]["status"], "failed");
        assert_eq!(done["results"][2]["status"], "ok");
        let sample = preview(&store, &p.id, vec![ids[0].clone(), ids[2].clone()], plan).unwrap();
        let cancel = AtomicBool::new(false);
        let before = store.load(&p.id).unwrap().assets[2].versions.len();
        let done = apply(
            &store,
            sample["sampleId"].as_str().unwrap(),
            None,
            &cancel,
            |_, _| cancel.store(true, Ordering::Relaxed),
        )
        .unwrap();
        assert_eq!(done["processed"], 1);
        assert_eq!(store.load(&p.id).unwrap().assets[2].versions.len(), before);
    }
}

