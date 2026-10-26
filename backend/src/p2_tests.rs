use base64::Engine;
use super::*;
use image::{DynamicImage, Rgba, RgbaImage};
use std::fs;

fn fixture() -> (tempfile::TempDir, Store, Project) {
    let tmp = tempfile::tempdir().unwrap();
    let store = Store::open(tmp.path().join("data"), "test").unwrap();
    let p = store.create("P2 fixture".into()).unwrap();
    let path = tmp.path().join("transparent.png");
    let mut image = RgbaImage::from_pixel(8, 4, Rgba([120, 30, 60, 255]));
    image.put_pixel(0, 0, Rgba([0, 0, 0, 0]));
    image.put_pixel(1, 0, Rgba([120, 30, 60, 100]));
    image.save(&path).unwrap();
    let p = store.import(&p.id, vec![path.to_string_lossy().into()], &AtomicBool::new(false), |_, _| {}).unwrap().project;
    (tmp, store, p)
}
fn preset(dir: &std::path::Path, format: &str) -> ExportPreset {
    ExportPreset { format: format.into(), width: 8, height: 8, fit: "pad".into(), quality: 92, max_bytes: 0, output_dir: dir.to_string_lossy().into(), background: [255;3] }
}

#[test]
fn p2_pad_preserves_source_alpha_in_png_and_webp_and_flattens_jpeg() {
    let (tmp, store, p) = fixture();
    for format in ["png", "webp", "jpeg"] {
        let result = store.export(&p.id, vec![p.assets[0].id.clone()], preset(tmp.path(), format), &AtomicBool::new(false), |_, _| {}).unwrap();
        let image = image::open(result[0].path.as_ref().unwrap()).unwrap().to_rgba8();
        assert_eq!(image.get_pixel(0, 0)[3], 255, "padding remains opaque");
        assert!(image.get_pixel(0, 0).0[..3].iter().all(|v| *v >= 245), "lossy encoding may slightly change white");
        if format != "jpeg" {
            assert_eq!(image.get_pixel(0, 2)[3], 0, "source transparency must survive pad");
            assert_eq!(image.get_pixel(1, 2)[3], 100);
        } else { assert!(image.pixels().all(|p| p[3] == 255)); }
    }
}

#[test]
fn p2_generated_history_restore_adopts_the_requested_pixels() {
    let (_tmp, store, p) = fixture();
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(6, 3, Rgba([0, 150, 80, 255])));
    let (_, vid) = store.save_generated(&p.id, &p.assets[0].id, &image, json!({"fixture":true})).unwrap();
    let restored = store.restore_version(&p.id, &p.assets[0].id, Some(&vid)).unwrap();
    assert_eq!(restored.assets[0].generated_base.as_deref(), Some(vid.as_str()));
    assert_eq!(store.source(&restored, &p.assets[0].id).unwrap().to_rgba8(), image.to_rgba8());
}

#[test]
fn p2_local_processing_reports_an_unreachable_export_as_partial() {
    let (tmp, store, p) = fixture();
    let plan = assistant::Plan { summary:"Export".into(),steps:vec![],warnings:vec![],clarification:None,
        output:Some(assistant::Output{format:"png".into(),width:8,height:8,fit:"pad".into(),max_bytes:1,background:[255;3]}) };
    let sample = assistant::preview(&store, &p.id, vec![p.assets[0].id.clone()], plan).unwrap();
    let result = assistant::apply(&store, sample["sampleId"].as_str().unwrap(), Some(tmp.path().to_string_lossy().into()), &AtomicBool::new(false), |_, _| {}).unwrap();
    assert_eq!(result["results"][0]["status"], "partial");
    assert!(result["results"][0]["error"].as_str().unwrap().contains("EXPORT_TARGET_UNREACHABLE"));
    assert_ne!(result["status"], "completed");
    assert!(result["results"][0]["exports"][0]["path"].is_null());
}

#[test]
fn p2_one_corrupt_project_does_not_hide_healthy_projects() {
    let (_tmp, store, p) = fixture();
    let broken = store.create("broken".into()).unwrap();
    fs::write(store.root.join("projects").join(&broken.id).join("project.json"), b"broken json").unwrap();
    let projects = store.list().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].id, p.id);
    let (_, warnings) = store.list_with_warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains(&broken.id));
    assert_eq!(fs::read(store.root.join("projects").join(&broken.id).join("project.json")).unwrap(), b"broken json");
}

#[test]
fn p2_preview_matches_export_bytes_for_every_format_and_fit_without_writes() {
    let (tmp, store, p) = fixture();
    let mut edit = Edit::default();edit.exposure = 0.5;edit.rotation = 90;
    let p = store.commit(&p.id, &p.assets[0].id, edit.clone()).unwrap();
    for format in ["png", "jpeg", "webp"] {
        for fit in ["crop", "pad"] {
            let mut preset = preset(tmp.path(), format);preset.fit = fit.into();
            let index = store.root.join("projects").join(&p.id).join("project.json");
            let before = fs::read(&index).unwrap();
            let mut preview_preset = preset.clone();preview_preset.output_dir = String::new();
            let preview = store.export_preview(&p.id, &p.assets[0].id, &edit, &preview_preset).unwrap();
            assert_eq!(fs::read(&index).unwrap(), before);
            let bytes = base64::engine::general_purpose::STANDARD.decode(preview["data"].as_str().unwrap()).unwrap();
            assert_eq!(preview["bytes"].as_u64().unwrap(), bytes.len() as u64);
            let decoded = image::load_from_memory(&bytes).unwrap();
            assert_eq!((decoded.width(),decoded.height()),(8,8));
            let exported = store.export(&p.id, vec![p.assets[0].id.clone()], preset, &AtomicBool::new(false), |_, _| {}).unwrap();
            assert_eq!(fs::read(exported[0].path.as_ref().unwrap()).unwrap(), bytes);
        }
    }
    let mut tiny = preset(tmp.path(), "png");tiny.max_bytes=1;
    assert_eq!(store.export_preview(&p.id,&p.assets[0].id,&edit,&tiny).unwrap()["status"], "EXPORT_TARGET_UNREACHABLE");
    tiny.width=0;
    assert!(store.export_preview(&p.id,&p.assets[0].id,&edit,&tiny).unwrap_err().contains("INVALID_EXPORT"));
}

#[test]
fn p2_second_local_job_cannot_clear_the_active_jobs_cancellation() {
    let (_tmp, store, _) = fixture();
    let shared = Arc::new(AppState {store:Mutex::new(store),cancel:AtomicBool::new(false),local_job:Mutex::new(()),preview_epoch:AtomicU64::new(0)});
    let first = begin_local_job(&shared).unwrap();
    cancel_job(State(&shared));
    let other = shared.clone();
    let rejected = std::thread::spawn(move || begin_local_job(&other).err()).join().unwrap().unwrap();
    assert!(rejected.contains("LOCAL_JOB_BUSY"));
    assert!(shared.cancel.load(Ordering::Relaxed));
    drop(first);
    let _next = begin_local_job(&shared).unwrap();
    assert!(!shared.cancel.load(Ordering::Relaxed));
}
