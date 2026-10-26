use image::{DynamicImage, ImageDecoder, ImageReader, Rgba, RgbaImage, imageops::FilterType};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
pub type Res<T> = Result<T, String>;
pub fn err(code: &str, e: impl std::fmt::Display) -> String {
    format!("{code}: {e}")
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn valid_id(value: &str) -> Res<()> {
    uuid::Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| err("INVALID_ID", "无效项目或图片 ID"))
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    #[serde(default)]
    pub strokes: Vec<crate::paint::Stroke>,
    #[serde(default = "default_opacity")]
    pub adjustment_opacity: f32,
    #[serde(default = "default_enabled")]
    pub adjustment_enabled: bool,
    pub exposure: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub temperature: f32,
    pub rotation: u32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub crop: Option<[f32; 4]>,
}
fn default_opacity() -> f32 {
    1.0
}
fn default_enabled() -> bool {
    true
}
impl Default for Edit {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            adjustment_opacity: 1.,
            adjustment_enabled: true,
            exposure: 0.,
            contrast: 1.,
            saturation: 1.,
            temperature: 0.,
            rotation: 0,
            flip_h: false,
            flip_v: false,
            crop: None,
        }
    }
}
impl Edit {
    fn validate(&self) -> Res<()> {
        crate::paint::validate(&self.strokes)?;
        if !(0.0..=1.0).contains(&self.adjustment_opacity)
            || !self.exposure.is_finite()
            || !(-3.0..=3.).contains(&self.exposure)
            || !(0.0..=3.).contains(&self.contrast)
            || !(0.0..=3.).contains(&self.saturation)
            || !(-1.0..=1.).contains(&self.temperature)
            || ![0, 90, 180, 270].contains(&self.rotation)
        {
            return Err(err("INVALID_EDIT", "调整参数超出范围"));
        }
        if let Some([x, y, w, h]) = self.crop {
            if [x, y, w, h].iter().any(|v| !v.is_finite())
                || x < 0.
                || y < 0.
                || w <= 0.
                || h <= 0.
                || x + w > 1.001
                || y + h > 1.001
            {
                return Err(err("INVALID_CROP", "裁切范围无效"));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    pub file: String,
    pub kind: String,
    pub created_at: u64,
    pub edit: Edit,
    #[serde(default)]
    pub generation: Option<serde_json::Value>,
    #[serde(default)]
    pub base_version: Option<String>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub source_path: String,
    pub original: String,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub trashed: bool,
    pub edit: Edit,
    pub versions: Vec<Version>,
    pub exported: bool,
    #[serde(default)]
    pub generated_base: Option<String>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub assets: Vec<Asset>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub project: Project,
    pub imported: usize,
    pub skipped: Vec<String>,
    pub errors: Vec<String>,
    pub cancelled: bool,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreset {
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub fit: String,
    pub quality: u8,
    pub max_bytes: u64,
    pub output_dir: String,
    pub background: [u8; 3],
}
impl ExportPreset {
    fn validate(&self) -> Res<()> {
        if !["png", "jpeg", "webp"].contains(&self.format.as_str())
            || !["crop", "pad"].contains(&self.fit.as_str())
            || self.width == 0 || self.height == 0 || self.width > 16000 || self.height > 16000
            || self.width as u64 * self.height as u64 > 120_000_000
            || self.quality == 0 || self.quality > 100 {
            return Err(err("INVALID_EXPORT", "导出尺寸、格式或适配方式无效"));
        }
        Ok(())
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportItem {
    pub asset_id: String,
    pub name: String,
    pub path: Option<String>,
    pub bytes: usize,
    pub status: String,
    pub quality: u8,
}
pub struct Store {
    pub root: PathBuf,
    pub profile: String,
    _lock: fs::File,
}
#[derive(Serialize, Deserialize)]
struct TrashMove {
    asset_id: String,
    from: String,
    to: String,
}
impl Store {
    pub fn open(root: PathBuf, profile: &str) -> Res<Self> {
        if !["dev", "test"].contains(&profile) {
            return Err(err("PROFILE_MISMATCH", "当前构建仅允许隔离 dev/test 数据"));
        }
        if root
            .ancestors()
            .skip(1)
            .any(|p| p.join(".gvpak-profile").exists())
        {
            return Err(err("NESTED_DATA_ROOT", "数据目录不能嵌套在另一个环境中"));
        }
        if !root.is_absolute() {
            return Err(err("INVALID_ROOT", "数据目录必须为绝对路径"));
        }
        fs::create_dir_all(&root).map_err(|e| err("PATH_NOT_WRITABLE", e))?;
        let mark = root.join(".gvpak-profile");
        if mark.exists() {
            if fs::read_to_string(&mark).map_err(|e| e.to_string())? != profile {
                return Err(err("PROFILE_MISMATCH", "数据目录属于另一个环境"));
            }
        } else {
            if fs::read_dir(&root)
                .map_err(|e| e.to_string())?
                .next()
                .is_some()
            {
                return Err(err("ROOT_NOT_EMPTY", "请选择空目录或已有 GVPAK 数据目录"));
            }
            fs::write(&mark, profile).map_err(|e| e.to_string())?;
        }
        let lock = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join(".session.lock"))
            .map_err(|e| e.to_string())?;
        lock.try_lock().map_err(|_| {
            err(
                "WORKSPACE_IN_USE",
                "该数据目录已在另一个 GVPAK STUDIO 窗口中打开",
            )
        })?;
        fs::create_dir_all(root.join("projects")).map_err(|e| e.to_string())?;
        Ok(Self {
            root,
            profile: profile.into(),
            _lock: lock,
        })
    }
    fn dir(&self, p: &str) -> Res<PathBuf> {
        valid_id(p)?;
        Ok(self.root.join("projects").join(p))
    }
    pub fn load(&self, p: &str) -> Res<Project> {
        let path = self.dir(p)?.join("project.json");
        let v: Project =
            serde_json::from_slice(&fs::read(path).map_err(|e| err("PROJECT_NOT_FOUND", e))?)
                .map_err(|e| err("PROJECT_CORRUPT", e))?;
        if v.id != p {
            return Err(err("PROJECT_CORRUPT", "项目 ID 不一致"));
        }
        self.recover_trash(&v)?;
        Ok(v)
    }
    fn save_trash_transaction(&self, pid: &str, moves: &[TrashMove]) -> Res<()> {
        let dir = self.dir(pid)?;
        let mut file = tempfile::NamedTempFile::new_in(&dir).map_err(|e| e.to_string())?;
        file.write_all(&serde_json::to_vec(moves).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        file.persist(dir.join("trash-transaction.json")).map_err(|e| e.to_string())?;
        Ok(())
    }
    fn recover_trash(&self, project: &Project) -> Res<()> {
        let dir = self.dir(&project.id)?;
        let journal = dir.join("trash-transaction.json");
        let bytes = match fs::read(&journal) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(err("TRASH_RECOVERY_PENDING", e)),
        };
        let moves: Vec<TrashMove> = serde_json::from_slice(&bytes)
            .map_err(|e| err("TRASH_RECOVERY_PENDING", e))?;
        // The atomically saved project index decides whether to keep or roll back the moves.
        // Keep the journal if recovery is blocked, so reopening can safely retry it.
        for step in moves.iter().rev() {
            valid_id(&step.asset_id)?;
            for relative in [&step.from, &step.to] {
                let parts: Vec<_> = Path::new(relative).components().collect();
                if parts.len() != 2
                    || !matches!(parts[0], std::path::Component::Normal(p) if p == "originals" || p == "trash")
                    || !matches!(parts[1], std::path::Component::Normal(_))
                {
                    return Err("TRASH_RECOVERY_PENDING: invalid journal path".into());
                }
            }
            if Path::new(&step.from).file_name() != Path::new(&step.to).file_name() || step.from == step.to {
                return Err("TRASH_RECOVERY_PENDING: invalid move".into());
            }
            let asset = project.assets.iter().find(|a| a.id == step.asset_id)
                .ok_or("TRASH_RECOVERY_PENDING: asset missing from index")?;
            if asset.original == step.to { continue; } // Index commit already succeeded.
            if asset.original != step.from {
                return Err("TRASH_RECOVERY_PENDING: index does not match journal".into());
            }
            let from = dir.join(&step.from);
            let to = dir.join(&step.to);
            if !to.try_exists().map_err(|e| err("TRASH_RECOVERY_PENDING", e))? { continue; }
            if from.try_exists().map_err(|e| err("TRASH_RECOVERY_PENDING", e))? {
                return Err("TRASH_RECOVERY_PENDING: both file locations exist".into());
            }
            fs::rename(&to, &from).map_err(|e| err("TRASH_RECOVERY_PENDING", e))?;
        }
        fs::remove_file(journal).map_err(|e| err("TRASH_RECOVERY_PENDING", e))
    }
    fn save(&self, p: &Project) -> Res<()> {
        let dir = self.dir(&p.id)?;
        let mut f =
            tempfile::NamedTempFile::new_in(&dir).map_err(|e| err("PATH_NOT_WRITABLE", e))?;
        f.write_all(&serde_json::to_vec_pretty(p).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        f.as_file().sync_all().map_err(|e| e.to_string())?;
        f.persist(dir.join("project.json"))
            .map_err(|e| err("PATH_NOT_WRITABLE", e))?;
        Ok(())
    }
    pub fn list(&self) -> Res<Vec<Project>> {
        Ok(self.list_with_warnings().0)
    }
    pub fn list_with_warnings(&self) -> (Vec<Project>, Vec<String>) {
        let mut list = Vec::new();
        let mut warnings = Vec::new();
        let directory = self.root.join("projects");
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(e) => return (list, vec![format!("{}: {e}", directory.display())]),
        };
        for entry in entries {
            let e = match entry { Ok(e) => e, Err(e) => { warnings.push(e.to_string()); continue; } };
            if e.path().is_dir() {
                let name = e.file_name().to_string_lossy().to_string();
                if valid_id(&name).is_ok() {
                    match self.load(&name) {
                        Ok(project) => list.push(project),
                        Err(error) => warnings.push(format!("{}: {error}", e.path().join("project.json").display())),
                    }
                }
            }
        }
        list.sort_by_key(|p| std::cmp::Reverse(p.updated_at));
        (list, warnings)
    }
    pub fn create(&self, name: String) -> Res<Project> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 120 {
            return Err(err("INVALID_NAME", "项目名称需为 1–120 个字符"));
        }
        let p = Project {
            id: id(),
            name: name.into(),
            created_at: now(),
            updated_at: now(),
            assets: vec![],
        };
        let dir = self.dir(&p.id)?;
        for sub in ["originals", "versions", "thumbs", "trash"] {
            fs::create_dir_all(dir.join(sub)).map_err(|e| e.to_string())?;
        }
        self.save(&p)?;
        Ok(p)
    }
    pub fn rename(&self, pid: &str, name: String) -> Res<Project> {
        if name.trim().is_empty() || name.chars().count() > 120 {
            return Err(err("INVALID_NAME", "项目名称需为 1–120 个字符"));
        }
        let mut p = self.load(pid)?;
        p.name = name.trim().into();
        p.updated_at = now();
        self.save(&p)?;
        Ok(p)
    }
    fn asset_path(&self, p: &Project, a: &Asset) -> Res<PathBuf> {
        let rel = Path::new(&a.original);
        if rel.is_absolute()
            || rel
                .components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(err("INVALID_PATH", "素材路径越界"));
        }
        Ok(self.dir(&p.id)?.join(rel))
    }
    pub fn import(
        &self,
        pid: &str,
        paths: Vec<String>,
        cancel: &AtomicBool,
        progress: impl Fn(usize, usize),
    ) -> Res<ImportReport> {
        let mut p = self.load(pid)?;
        let mut files = vec![];
        let mut errors = vec![];
        for path in paths {
            collect(&PathBuf::from(path), &mut files, &mut errors, 0)
        }
        let total = files.len();
        let mut skipped = vec![];
        let mut imported = 0;
        for (i, path) in files.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let attempt = (|| -> Res<()> {
                if fs::metadata(path).map_err(|e| e.to_string())?.len() > 512_000_000 {
                    return Err(err("IMAGE_TOO_LARGE", "文件超过 512 MB"));
                }
                let data = fs::read(path).map_err(|e| err("READ_FAILED", e))?;
                let sha = format!("{:x}", Sha256::digest(&data));
                if p.assets.iter().any(|a| a.sha256 == sha) {
                    skipped.push(path.to_string_lossy().into());
                    return Ok(());
                }
                let im = decode(path)?;
                let aid = id();
                let ext = path
                    .extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("png")
                    .to_lowercase();
                let rel = format!("originals/{aid}.{ext}");
                let dest = self.dir(pid)?.join(&rel);
                fs::write(&dest, &data).map_err(|e| err("PATH_NOT_WRITABLE", e))?;
                let mut perm = fs::metadata(&dest)
                    .map_err(|e| e.to_string())?
                    .permissions();
                perm.set_readonly(true);
                fs::set_permissions(&dest, perm).map_err(|e| e.to_string())?;
                im.thumbnail(360, 260)
                    .save(self.dir(pid)?.join(format!("thumbs/{aid}.png")))
                    .map_err(|e| e.to_string())?;
                p.assets.push(Asset {
                    id: aid,
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into(),
                    source_path: path.to_string_lossy().into(),
                    original: rel,
                    sha256: sha,
                    width: im.width(),
                    height: im.height(),
                    bytes: data.len() as u64,
                    trashed: false,
                    edit: Edit::default(),
                    versions: vec![],
                    exported: false,
                    generated_base: None,
                });
                p.updated_at = now();
                self.save(&p)?;
                imported += 1;
                Ok(())
            })();
            if let Err(e) = attempt {
                errors.push(format!("{}: {e}", path.display()))
            }
            progress(i + 1, total);
        }
        Ok(ImportReport {
            project: p,
            imported,
            skipped,
            errors,
            cancelled: cancel.load(Ordering::Relaxed),
        })
    }
    pub(crate) fn source(&self, p: &Project, aid: &str) -> Res<DynamicImage> {
        let a = p
            .assets
            .iter()
            .find(|a| a.id == aid && !a.trashed)
            .ok_or_else(|| err("ASSET_NOT_FOUND", "图片不存在"))?;
        let path = self.asset_path(p, a)?;
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        if format!("{:x}", Sha256::digest(&bytes)) != a.sha256 {
            return Err(err("FILE_CORRUPT", "原图哈希校验失败"));
        }
        if let Some(vid) = &a.generated_base {
            return self.version_image(p, aid, vid);
        }
        decode(&path)
    }
    fn version_image(&self, p: &Project, aid: &str, vid: &str) -> Res<DynamicImage> {
        valid_id(vid)?;
        let a = p
            .assets
            .iter()
            .find(|a| a.id == aid && !a.trashed)
            .ok_or("ASSET_NOT_FOUND")?;
        let v = a
            .versions
            .iter()
            .find(|v| v.id == vid && ["generated", "local"].contains(&v.kind.as_str()))
            .ok_or("VERSION_NOT_FOUND")?;
        let expected = format!("versions/{vid}.png");
        if v.file != expected {
            return Err("INVALID_VERSION_PATH".into());
        }
        decode(&self.dir(&p.id)?.join(expected))
    }
    pub fn original_preview(&self, pid: &str, aid: &str) -> Res<Vec<u8>> {
        let p = self.load(pid)?;
        let a = p
            .assets
            .iter()
            .find(|a| a.id == aid && !a.trashed)
            .ok_or("ASSET_NOT_FOUND")?;
        let path = self.asset_path(&p, a)?;
        if format!(
            "{:x}",
            Sha256::digest(fs::read(&path).map_err(|e| e.to_string())?)
        ) != a.sha256
        {
            return Err("FILE_CORRUPT".into());
        }
        png(&decode(&path)?.thumbnail(1600, 1600))
    }
    pub fn generated_preview(&self, pid: &str, aid: &str, vid: &str) -> Res<Vec<u8>> {
        png(&self
            .version_image(&self.load(pid)?, aid, vid)?
            .thumbnail(1600, 1600))
    }
    pub fn generation_metadata(&self, pid: &str, aid: &str, vid: &str) -> Res<serde_json::Value> {
        let p = self.load(pid)?;
        let a = p
            .assets
            .iter()
            .find(|a| a.id == aid && !a.trashed)
            .ok_or("ASSET_NOT_FOUND")?;
        a.versions
            .iter()
            .find(|v| v.id == vid && v.kind == "generated")
            .and_then(|v| v.generation.clone())
            .ok_or_else(|| "GENERATION_METADATA_MISSING".into())
    }
    pub fn save_generated(
        &self,
        pid: &str,
        aid: &str,
        image: &DynamicImage,
        metadata: serde_json::Value,
    ) -> Res<(Project, String)> {
        self.save_pixels(pid, aid, image, metadata, "generated")
    }
    pub fn save_processed(
        &self,
        pid: &str,
        aid: &str,
        image: &DynamicImage,
        metadata: serde_json::Value,
    ) -> Res<Project> {
        let (_, vid) = self.save_pixels(pid, aid, image, metadata, "local")?;
        self.adopt_generated(pid, aid, &vid)
    }
    pub(crate) fn save_local_version(
        &self,
        pid: &str,
        aid: &str,
        image: &DynamicImage,
        metadata: serde_json::Value,
    ) -> Res<(Project, String)> {
        self.save_pixels(pid, aid, image, metadata, "local")
    }
    fn save_pixels(
        &self,
        pid: &str,
        aid: &str,
        image: &DynamicImage,
        metadata: serde_json::Value,
        kind: &str,
    ) -> Res<(Project, String)> {
        let mut p = self.load(pid)?;
        let a = p
            .assets
            .iter_mut()
            .find(|a| a.id == aid && !a.trashed)
            .ok_or("ASSET_NOT_FOUND")?;
        let vid = id();
        let file = format!("versions/{vid}.png");
        image
            .save(self.dir(pid)?.join(&file))
            .map_err(|e| err("PATH_NOT_WRITABLE", e))?;
        a.versions.push(Version {
            id: vid.clone(),
            file,
            kind: kind.into(),
            created_at: now(),
            edit: Edit::default(),
            generation: Some(metadata),
            base_version: None,
        });
        p.updated_at = now();
        self.save(&p)?;
        Ok((p, vid))
    }
    pub fn adopt_generated(&self, pid: &str, aid: &str, vid: &str) -> Res<Project> {
        let mut p = self.load(pid)?;
        let image = self.version_image(&p, aid, vid)?;
        image
            .thumbnail(360, 260)
            .save(self.dir(pid)?.join(format!("thumbs/{aid}.png")))
            .map_err(|e| e.to_string())?;
        let a = p
            .assets
            .iter_mut()
            .find(|a| a.id == aid)
            .ok_or("ASSET_NOT_FOUND")?;
        a.generated_base = Some(vid.into());
        a.edit = Edit::default();
        a.width = image.width();
        a.height = image.height();
        a.exported = false;
        p.updated_at = now();
        self.save(&p)?;
        Ok(p)
    }
    pub fn restore_version(&self, pid: &str, aid: &str, vid: Option<&str>) -> Res<Project> {
        if let Some(vid) = vid {
            if self
                .load(pid)?
                .assets
                .iter()
                .any(|a| a.id == aid && a.versions.iter().any(|v| v.id == vid && matches!(v.kind.as_str(), "local" | "generated")))
            {
                return self.adopt_generated(pid, aid, vid);
            }
        }
        let mut p = self.load(pid)?;
        let a = p
            .assets
            .iter_mut()
            .find(|a| a.id == aid && !a.trashed)
            .ok_or("ASSET_NOT_FOUND")?;
        if let Some(vid) = vid {
            let v = a
                .versions
                .iter()
                .find(|v| v.id == vid && v.kind == "edited")
                .ok_or("VERSION_NOT_FOUND")?
                .clone();
            a.edit = v.edit;
            a.generated_base = v.base_version;
        } else {
            a.edit = Edit::default();
            a.generated_base = None;
        }
        let edit = a.edit.clone();
        let image = self.source(&p, aid)?;
        let a = p.assets.iter_mut().find(|a| a.id == aid).unwrap();
        a.width = image.width();
        a.height = image.height();
        a.exported = false;
        render(image, &edit, Some(360))?
            .save(self.dir(pid)?.join(format!("thumbs/{aid}.png")))
            .map_err(|e| e.to_string())?;
        p.updated_at = now();
        self.save(&p)?;
        Ok(p)
    }
    pub fn preview_source(&self, pid: &str, aid: &str) -> Res<Vec<u8>> {
        let p = self.load(pid)?;
        let source = self.source(&p, aid)?;
        if source.width() > 1600 || source.height() > 1600 {
            png(&source.resize(1600, 1600, FilterType::Triangle))
        } else { png(&source) }
    }
    pub fn preview(&self, pid: &str, aid: &str, edit: Edit) -> Res<Vec<u8>> {
        let p = self.load(pid)?;
        let im = self.source(&p, aid)?;
        let im = render(im, &edit, Some(1600))?;
        png(&im)
    }
    pub fn thumb(&self, pid: &str, aid: &str) -> Res<Vec<u8>> {
        valid_id(aid)?;
        fs::read(self.dir(pid)?.join(format!("thumbs/{aid}.png"))).map_err(|e| e.to_string())
    }
    pub fn commit(&self, pid: &str, aid: &str, edit: Edit) -> Res<Project> {
        let mut p = self.load(pid)?;
        let image = render(self.source(&p, aid)?, &edit, None)?;
        let vid = id();
        let file = format!("versions/{vid}.png");
        image
            .save(self.dir(pid)?.join(&file))
            .map_err(|e| err("PATH_NOT_WRITABLE", e))?;
        image
            .thumbnail(360, 260)
            .save(self.dir(pid)?.join(format!("thumbs/{aid}.png")))
            .map_err(|e| e.to_string())?;
        let a = p.assets.iter_mut().find(|a| a.id == aid).unwrap();
        a.edit = edit.clone();
        a.versions.push(Version {
            id: vid,
            file,
            kind: "edited".into(),
            created_at: now(),
            edit,
            generation: None,
            base_version: a.generated_base.clone(),
        });
        p.updated_at = now();
        self.save(&p)?;
        Ok(p)
    }
    pub fn trash(&self, pid: &str, ids: Vec<String>, restore: bool) -> Res<Project> {
        let mut p = self.load(pid)?;
        let dir = self.dir(pid)?;
        let mut moves: Vec<TrashMove> = Vec::new();
        for aid in ids {
            valid_id(&aid)?;
            if moves.iter().any(|step| step.asset_id == aid) { continue; }
            if let Some(a) = p.assets.iter().find(|a| a.id == aid) {
                if a.trashed == restore {
                    let old = dir.join(&a.original);
                    let filename = old.file_name().ok_or("INVALID_PATH")?.to_string_lossy();
                    let new = format!("{}/{filename}", if restore { "originals" } else { "trash" });
                    // Never overwrite an existing destination, including on Unix.
                    if dir.join(&new).try_exists().map_err(|e| e.to_string())? {
                        return Err("TRASH_DESTINATION_EXISTS".into());
                    }
                    moves.push(TrashMove { asset_id: aid, from: a.original.clone(), to: new });
                }
            }
        }
        if moves.is_empty() { return Ok(p); }
        self.save_trash_transaction(pid, &moves)?;
        let result = (|| -> Res<()> {
            for step in &moves {
                fs::rename(dir.join(&step.from), dir.join(&step.to)).map_err(|e| e.to_string())?;
                let a = p.assets.iter_mut().find(|a| a.id == step.asset_id).ok_or("ASSET_NOT_FOUND")?;
                a.original = step.to.clone();
                a.trashed = !restore;
            }
            p.updated_at = now();
            self.save(&p)
        })();
        // load() reconciles files against the index on success, failure and after a restart.
        let recovered = self.load(pid);
        match result {
            Ok(()) => recovered,
            Err(error) => match recovered {
                Ok(_) => Err(error),
                Err(recovery) => Err(format!("{error}; {recovery}")),
            },
        }
    }
    pub fn export_preview(&self, pid: &str, aid: &str, edit: &Edit, preset: &ExportPreset) -> Res<serde_json::Value> {
        preset.validate()?;
        let p = self.load(pid)?;
        let source = self.source(&p, aid)?;
        let before = png(&render(source.clone(), edit, Some(1600))?)?;
        let (data, quality) = export_pixels(source, edit, preset)?;
        let status = if preset.max_bytes > 0 && data.len() as u64 > preset.max_bytes { "EXPORT_TARGET_UNREACHABLE" } else { "ok" };
        Ok(serde_json::json!({"data":base64::engine::general_purpose::STANDARD.encode(&data),
            "before":base64::engine::general_purpose::STANDARD.encode(before),
            "mime":format!("image/{}",preset.format),"bytes":data.len(),"quality":quality,
            "width":preset.width,"height":preset.height,"status":status}))
    }
    pub fn export(
        &self,
        pid: &str,
        ids: Vec<String>,
        preset: ExportPreset,
        cancel: &AtomicBool,
        progress: impl Fn(usize, usize),
    ) -> Res<Vec<ExportItem>> {
        preset.validate()?;
        let dir = PathBuf::from(&preset.output_dir);
        if !dir.is_absolute() || !dir.is_dir() {
            return Err(err("PATH_NOT_WRITABLE", "请选择已存在的输出目录"));
        }
        let mut p = self.load(pid)?;
        let mut report = vec![];
        let total = ids.len();
        for (i, aid) in ids.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let a = p
                .assets
                .iter()
                .find(|a| a.id == *aid && !a.trashed)
                .ok_or("ASSET_NOT_FOUND")?
                .clone();
            let attempt = (|| -> Res<(Option<String>, usize, u8, String)> {
                let (data, q) = export_pixels(self.source(&p, aid)?, &a.edit, &preset)?;
                if preset.max_bytes > 0 && data.len() as u64 > preset.max_bytes {
                    return Ok((None, data.len(), q, "EXPORT_TARGET_UNREACHABLE".into()));
                }
                let stem = Path::new(&a.name)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .chars()
                    .map(|c| if "<>:\"/\\|?*".contains(c) { '_' } else { c })
                    .take(100)
                    .collect::<String>();
                let ext = if preset.format == "jpeg" {
                    "jpg"
                } else {
                    &preset.format
                };
                let mut n = 0;
                loop {
                    let suffix = if n == 0 {
                        String::new()
                    } else {
                        format!("_{n}")
                    };
                    let path = dir.join(format!("{stem}_GVPAK{suffix}.{ext}"));
                    match fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&path)
                    {
                        Ok(mut f) => {
                            if let Err(e) = f.write_all(&data) {
                                let _ = fs::remove_file(&path);
                                return Err(err("PATH_NOT_WRITABLE", e));
                            }
                            f.sync_all().map_err(|e| e.to_string())?;
                            return Ok((
                                Some(path.to_string_lossy().into()),
                                data.len(),
                                q,
                                "ok".into(),
                            ));
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                            n += 1;
                            if n > 9999 {
                                return Err(err("NAME_CONFLICT", "重名文件过多"));
                            }
                        }
                        Err(e) => return Err(err("PATH_NOT_WRITABLE", e)),
                    }
                }
            })();
            let (path, bytes, q, status) = match attempt {
                Ok(v) => v,
                Err(e) => (None, 0, 0, e),
            };
            if status == "ok" {
                if let Some(item) = p.assets.iter_mut().find(|x| x.id == *aid) {
                    item.exported = true;
                }
                self.save(&p)?;
            }
            report.push(ExportItem {
                asset_id: aid.clone(),
                name: a.name,
                path,
                bytes,
                status,
                quality: q,
            });
            progress(i + 1, total);
        }
        Ok(report)
    }
}
fn collect(path: &Path, files: &mut Vec<PathBuf>, errors: &mut Vec<String>, depth: usize) {
    if depth > 32 {
        errors.push("IMPORT_LIMIT: 单次最多 5000 个文件、32 层目录".into());
        return;
    }
    match fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_symlink() => {
            errors.push(format!("SYMLINK_SKIPPED: {}", path.display()))
        }
        Ok(m) if m.is_dir() => match fs::read_dir(path) {
            Ok(entries) => {
                for e in entries.flatten() {
                    collect(&e.path(), files, errors, depth + 1)
                }
            }
            Err(e) => errors.push(e.to_string()),
        },
        Ok(_) => files.push(path.into()),
        Err(e) => errors.push(e.to_string()),
    }
}
pub fn decode(path: &Path) -> Res<DynamicImage> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
        return Err(err("UNSUPPORTED_FORMAT", "支持 PNG、JPEG、WebP"));
    }
    let dims = ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_dimensions()
        .map_err(|e| err("FILE_CORRUPT", e))?;
    if dims.0 > 16000 || dims.1 > 16000 || dims.0 as u64 * dims.1 as u64 > 120_000_000 {
        return Err(err("IMAGE_TOO_LARGE", "最大边长 16000px、总像素 1.2 亿"));
    }
    let mut reader = ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut lim = image::Limits::default();
    lim.max_alloc = Some(600_000_000);
    reader.limits(lim);
    let mut decoder = reader.into_decoder().map_err(|e| err("FILE_CORRUPT", e))?;
    let orientation = decoder.orientation().map_err(|e| e.to_string())?;
    let mut im = DynamicImage::from_decoder(decoder).map_err(|e| err("FILE_CORRUPT", e))?;
    im.apply_orientation(orientation);
    Ok(im)
}
pub fn render(mut im: DynamicImage, e: &Edit, max: Option<u32>) -> Res<DynamicImage> {
    e.validate()?;
    let source_size = (im.width(), im.height());
    if let Some([x, y, w, h]) = e.crop {
        let iw = im.width();
        let ih = im.height();
        let x = (x * iw as f32).round() as u32;
        let y = (y * ih as f32).round() as u32;
        let w = ((w * iw as f32).round() as u32)
            .max(1)
            .min(iw.saturating_sub(x));
        let h = ((h * ih as f32).round() as u32)
            .max(1)
            .min(ih.saturating_sub(y));
        if w == 0 || h == 0 {
            return Err("INVALID_CROP".into());
        }
        im = im.crop_imm(x, y, w, h)
    }
    im = match e.rotation {
        90 => im.rotate90(),
        180 => im.rotate180(),
        270 => im.rotate270(),
        _ => im,
    };
    if e.flip_h {
        im = im.fliph()
    }
    if e.flip_v {
        im = im.flipv()
    }
    if let Some(m) = max {
        if im.width() > m || im.height() > m {
            im = im.resize(m, m, FilterType::Triangle)
        }
    }
    let mut pixels = im.to_rgba8();
    let gain = 2f32.powf(e.exposure);
    for p in pixels.pixels_mut() {
        let c = [p[0] as f32 / 255., p[1] as f32 / 255., p[2] as f32 / 255.];
        let l = c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722;
        for i in 0..3 {
            let mut v = ((l + (c[i] - l) * e.saturation) * gain - 0.5) * e.contrast + 0.5;
            if i == 0 {
                v += e.temperature * 0.08
            } else if i == 2 {
                v -= e.temperature * 0.08
            }
            let strength = if e.adjustment_enabled {
                e.adjustment_opacity
            } else {
                0.
            };
            p[i] = ((c[i] + (v.clamp(0., 1.) - c[i]) * strength) * 255.).round() as u8;
        }
    }
    crate::paint::composite(&mut pixels, e, source_size);
    Ok(DynamicImage::ImageRgba8(pixels))
}
pub(crate) fn png(im: &DynamicImage) -> Res<Vec<u8>> {
    let mut buf = std::io::Cursor::new(Vec::new());
    im.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(buf.into_inner())
}
pub(crate) fn flatten(im: &DynamicImage, bg: [u8; 3]) -> DynamicImage {
    let mut canvas =
        RgbaImage::from_pixel(im.width(), im.height(), Rgba([bg[0], bg[1], bg[2], 255]));
    image::imageops::overlay(&mut canvas, im, 0, 0);
    DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(canvas).to_rgb8())
}
pub(crate) fn fit(im: &DynamicImage, w: u32, h: u32, pad: bool, bg: [u8; 3]) -> DynamicImage {
    if !pad {
        return im.resize_to_fill(w, h, FilterType::Lanczos3);
    }
    let small = im.resize(w, h, FilterType::Lanczos3);
    let mut canvas = RgbaImage::from_pixel(w, h, Rgba([bg[0], bg[1], bg[2], 255]));
    image::imageops::replace(
        &mut canvas,
        &small,
        ((w - small.width()) / 2) as i64,
        ((h - small.height()) / 2) as i64,
    );
    DynamicImage::ImageRgba8(canvas)
}
fn encode(im: &DynamicImage, format: &str, q: u8) -> Res<Vec<u8>> {
    match format {
        "png" => png(im),
        "webp" => Ok(
            webp::Encoder::from_rgba(&im.to_rgba8(), im.width(), im.height())
                .encode(q as f32)
                .to_vec(),
        ),
        "jpeg" => {
            let mut v = vec![];
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut v, q)
                .encode_image(&im.to_rgb8())
                .map_err(|e| e.to_string())?;
            Ok(v)
        }
        _ => Err("UNSUPPORTED_FORMAT".into()),
    }
}
fn export_pixels(source: DynamicImage, edit: &Edit, preset: &ExportPreset) -> Res<(Vec<u8>, u8)> {
    let im = render(source, edit, None)?;
    let mut im = fit(&im, preset.width, preset.height, preset.fit == "pad", preset.background);
    if preset.format == "jpeg" { im = flatten(&im, preset.background); }
    encode_target(&im, preset)
}
pub(crate) fn encode_target(im: &DynamicImage, p: &ExportPreset) -> Res<(Vec<u8>, u8)> {
    let initial = encode(im, &p.format, p.quality)?;
    if p.format == "png" || p.max_bytes == 0 || initial.len() as u64 <= p.max_bytes {
        return Ok((initial, p.quality));
    }
    let mut closest = (initial, p.quality);
    let mut best = None;
    let (mut lo, mut hi) = (1, p.quality.saturating_sub(1));
    while lo <= hi {
        let q = lo + (hi - lo) / 2;
        let bytes = encode(im, &p.format, q)?;
        if (bytes.len() as u64) <= p.max_bytes {
            best = Some((bytes, q));
            lo = q + 1;
        } else {
            if bytes.len() < closest.0.len() {
                closest = (bytes, q)
            }
            if q == 1 {
                break;
            }
            hi = q - 1;
        }
    }
    Ok(best.unwrap_or(closest))
}
#[cfg(test)]
mod trash_regressions {
    use super::*;

    fn fixture() -> (tempfile::TempDir, Store, Project) {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().join("data"), "test").unwrap();
        let project = store.create("trash regression".into()).unwrap();
        let paths = (0..2).map(|i| {
            let path = tmp.path().join(format!("source-{i}.png"));
            RgbaImage::from_pixel(4, 4, Rgba([i * 100, 20, 30, 255])).save(&path).unwrap();
            path.to_string_lossy().into_owned()
        }).collect();
        let project = store.import(&project.id, paths, &AtomicBool::new(false), |_, _| {}).unwrap().project;
        (tmp, store, project)
    }

    #[test]
    fn second_move_failure_preserves_first_asset_on_trash_and_restore() {
        for restore in [false, true] {
            let (_tmp, store, mut project) = fixture();
            let ids: Vec<_> = project.assets.iter().map(|a| a.id.clone()).collect();
            if restore { project = store.trash(&project.id, ids.clone(), false).unwrap(); }
            let dir = store.dir(&project.id).unwrap();
            let second = dir.join(&project.assets[1].original);
            let held = dir.join("held.png");
            fs::rename(&second, &held).unwrap();
            assert!(store.trash(&project.id, ids.clone(), restore).is_err());
            assert!(dir.join(&project.assets[0].original).is_file(), "first file must still match the saved index");
            fs::rename(&held, &second).unwrap();
            let saved = store.load(&project.id).unwrap();
            assert_eq!(saved.assets[0].original, project.assets[0].original);
            assert_eq!(saved.assets[0].trashed, project.assets[0].trashed);
            let retried = store.trash(&project.id, ids, restore).unwrap();
            assert!(retried.assets.iter().all(|a| a.trashed == !restore && dir.join(&a.original).is_file()));
        }
    }

    #[cfg(windows)]
    #[test]
    fn index_save_failure_rolls_back_all_moves() {
        use std::os::windows::fs::OpenOptionsExt;
        let (_tmp, store, project) = fixture();
        let dir = store.dir(&project.id).unwrap();
        // Allow reads but deny replacement of project.json, as another Windows app can do.
        let index_lock = fs::OpenOptions::new().read(true).share_mode(1).open(dir.join("project.json")).unwrap();
        let ids = project.assets.iter().map(|a| a.id.clone()).collect();
        assert!(store.trash(&project.id, ids, false).is_err());
        assert!(project.assets.iter().all(|a| dir.join(&a.original).is_file()), "failed index commit must restore every original path");
        drop(index_lock);
        let saved = store.load(&project.id).unwrap();
        assert!(saved.assets.iter().all(|a| !a.trashed));
    }

    fn moves_for(project: &Project) -> Vec<TrashMove> {
        project.assets.iter().map(|a| TrashMove {
            asset_id: a.id.clone(),
            from: a.original.clone(),
            to: format!("trash/{}", Path::new(&a.original).file_name().unwrap().to_string_lossy()),
        }).collect()
    }

    #[test]
    fn reopen_rolls_back_an_interrupted_move_before_index_commit() {
        let (_tmp, store, project) = fixture();
        let root = store.root.clone();
        let dir = store.dir(&project.id).unwrap();
        let moves = moves_for(&project);
        let original = fs::read(dir.join(&moves[0].from)).unwrap();
        store.save_trash_transaction(&project.id, &moves).unwrap();
        fs::rename(dir.join(&moves[0].from), dir.join(&moves[0].to)).unwrap();
        drop(store); // Persist exactly the state left by interruption after the first move.
        let reopened = Store::open(root, "test").unwrap();
        let saved = reopened.load(&project.id).unwrap();
        assert!(saved.assets.iter().all(|a| !a.trashed && dir.join(&a.original).is_file()));
        assert_eq!(fs::read(dir.join(&moves[0].from)).unwrap(), original);
        assert!(!dir.join("trash-transaction.json").exists());
    }

    #[test]
    fn reopen_keeps_moves_when_index_commit_already_succeeded() {
        let (_tmp, store, mut project) = fixture();
        let root = store.root.clone();
        let dir = store.dir(&project.id).unwrap();
        let moves = moves_for(&project);
        store.save_trash_transaction(&project.id, &moves).unwrap();
        for (asset, step) in project.assets.iter_mut().zip(&moves) {
            fs::rename(dir.join(&step.from), dir.join(&step.to)).unwrap();
            asset.original = step.to.clone();
            asset.trashed = true;
        }
        store.save(&project).unwrap();
        drop(store); // Simulate interruption between the commit and journal cleanup.
        let reopened = Store::open(root, "test").unwrap();
        let saved = reopened.load(&project.id).unwrap();
        assert!(saved.assets.iter().all(|a| a.trashed && dir.join(&a.original).is_file()));
        assert!(!dir.join("trash-transaction.json").exists());
        let ids = saved.assets.iter().map(|a| a.id.clone()).collect();
        let restored = reopened.trash(&project.id, ids, true).unwrap();
        assert!(restored.assets.iter().all(|a| !a.trashed && dir.join(&a.original).is_file()));
    }

    #[cfg(windows)]
    #[test]
    fn blocked_rollback_keeps_journal_and_can_be_retried() {
        use std::os::windows::fs::OpenOptionsExt;
        let (_tmp, store, project) = fixture();
        let dir = store.dir(&project.id).unwrap();
        let moves = moves_for(&project);
        store.save_trash_transaction(&project.id, &moves).unwrap();
        fs::rename(dir.join(&moves[0].from), dir.join(&moves[0].to)).unwrap();
        let lock = fs::OpenOptions::new().read(true).share_mode(1).open(dir.join(&moves[0].to)).unwrap();
        assert!(store.load(&project.id).unwrap_err().contains("TRASH_RECOVERY_PENDING"));
        assert!(dir.join("trash-transaction.json").is_file());
        drop(lock);
        let saved = store.load(&project.id).unwrap();
        assert!(saved.assets.iter().all(|a| !a.trashed && dir.join(&a.original).is_file()));
        assert!(!dir.join("trash-transaction.json").exists());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_review_adoption_export_and_original_preservation() {
        let temp = tempfile::tempdir().unwrap();
        let s = Store::open(temp.path().join("data"), "test").unwrap();
        let source = temp.path().join("source.png");
        RgbaImage::from_pixel(40, 30, Rgba([255, 0, 0, 255]))
            .save(&source)
            .unwrap();
        let original = fs::read(&source).unwrap();
        let p = s.create("生成版本测试".into()).unwrap();
        let imported = s
            .import(
                &p.id,
                vec![source.to_string_lossy().into()],
                &AtomicBool::new(false),
                |_, _| {},
            )
            .unwrap();
        let aid = &imported.project.assets[0].id;
        let generated =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(40, 30, Rgba([0, 255, 0, 255])));
        let (pending, vid) = s
            .save_generated(
                &p.id,
                aid,
                &generated,
                serde_json::json!({"testFixture":true}),
            )
            .unwrap();
        assert_eq!(pending.assets.len(), 1);
        assert_eq!(pending.assets[0].versions[0].kind, "generated");
        assert_eq!(
            s.source(&pending, aid).unwrap().to_rgba8().get_pixel(0, 0)[0],
            255
        );
        assert!(s.adopt_generated(&p.id, aid, "../escape").is_err());
        let adopted = s.adopt_generated(&p.id, aid, &vid).unwrap();
        assert_eq!(
            adopted.assets[0].generated_base.as_deref(),
            Some(vid.as_str())
        );
        assert_eq!(
            s.source(&s.load(&p.id).unwrap(), aid)
                .unwrap()
                .to_rgba8()
                .get_pixel(0, 0)[1],
            255
        );
        let preview = image::load_from_memory(&s.original_preview(&p.id, aid).unwrap()).unwrap();
        assert_eq!(preview.to_rgba8().get_pixel(0, 0)[0], 255);
        let out = temp.path().join("output");
        fs::create_dir(&out).unwrap();
        let preset = ExportPreset {
            format: "png".into(),
            width: 40,
            height: 30,
            fit: "pad".into(),
            quality: 90,
            max_bytes: 800000,
            output_dir: out.to_string_lossy().into(),
            background: [255; 3],
        };
        let report = s
            .export(
                &p.id,
                vec![aid.clone()],
                preset,
                &AtomicBool::new(false),
                |_, _| {},
            )
            .unwrap();
        let exported = image::open(report[0].path.as_ref().unwrap()).unwrap();
        assert_eq!(exported.to_rgba8().get_pixel(0, 0), &Rgba([0, 255, 0, 255]));
        assert_eq!(fs::read(&source).unwrap(), original);
        assert_eq!(
            fs::read(s.asset_path(&adopted, &adopted.assets[0]).unwrap()).unwrap(),
            original
        );
        let edited = s
            .commit(
                &p.id,
                aid,
                Edit {
                    exposure: -1.,
                    ..Edit::default()
                },
            )
            .unwrap();
        let edited_id = edited.assets[0].versions.last().unwrap().id.clone();
        let restored_original = s.restore_version(&p.id, aid, None).unwrap();
        assert!(restored_original.assets[0].generated_base.is_none());
        assert_eq!(
            s.source(&restored_original, aid)
                .unwrap()
                .to_rgba8()
                .get_pixel(0, 0)[0],
            255
        );
        let restored_edit = s.restore_version(&p.id, aid, Some(&edited_id)).unwrap();
        assert_eq!(
            restored_edit.assets[0].generated_base.as_deref(),
            Some(vid.as_str())
        );
        assert_eq!(restored_edit.assets[0].edit.exposure, -1.);
    }
    #[test]
    fn real_lifecycle() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("data");
        let s = Store::open(root.clone(), "test").unwrap();
        assert!(Store::open(root, "dev").is_err());
        let source = temp.path().join("中文 图.png");
        RgbaImage::from_pixel(40, 30, Rgba([80, 100, 150, 77]))
            .save(&source)
            .unwrap();
        let original = fs::read(&source).unwrap();
        let p = s.create("测试".into()).unwrap();
        let cancel = AtomicBool::new(false);
        let report = s
            .import(
                &p.id,
                vec![
                    source.to_string_lossy().into(),
                    source.to_string_lossy().into(),
                ],
                &cancel,
                |_, _| {},
            )
            .unwrap();
        assert_eq!(report.imported, 1);
        assert_eq!(report.skipped.len(), 1);
        let a = &report.project.assets[0];
        assert!(
            fs::metadata(s.asset_path(&report.project, a).unwrap())
                .unwrap()
                .permissions()
                .readonly()
        );
        let edit = Edit {
            exposure: 1.,
            ..Edit::default()
        };
        let committed = s.commit(&p.id, &a.id, edit).unwrap();
        assert_eq!(committed.assets[0].versions.len(), 1);
        let removed = s.trash(&p.id, vec![a.id.clone()], false).unwrap();
        assert!(removed.assets[0].trashed);
        s.trash(&p.id, vec![a.id.clone()], true).unwrap();
        assert_eq!(fs::read(&source).unwrap(), original);
        let output = temp.path().join("export");
        fs::create_dir(&output).unwrap();
        let preset = ExportPreset {
            format: "webp".into(),
            width: 80,
            height: 60,
            fit: "crop".into(),
            quality: 90,
            max_bytes: 800000,
            output_dir: output.to_string_lossy().into(),
            background: [255; 3],
        };
        let x = s
            .export(
                &p.id,
                vec![a.id.clone()],
                preset.clone(),
                &cancel,
                |_, _| {},
            )
            .unwrap();
        let y = s
            .export(&p.id, vec![a.id.clone()], preset, &cancel, |_, _| {})
            .unwrap();
        assert_ne!(x[0].path, y[0].path);
        let decoded = image::open(x[0].path.as_ref().unwrap()).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (80, 60));
        assert_eq!(decoded.to_rgba8().get_pixel(10, 10)[3], 77);
        assert!(decoded.to_rgba8().get_pixel(10, 10)[0] > 140);
        let other = s.create("另一项目".into()).unwrap();
        let second = s
            .import(
                &other.id,
                vec![source.to_string_lossy().into()],
                &cancel,
                |_, _| {},
            )
            .unwrap();
        assert_ne!(
            s.asset_path(&report.project, a).unwrap(),
            s.asset_path(&second.project, &second.project.assets[0])
                .unwrap()
        );
    }
    #[test]
    fn adjustment_layer_bypass() {
        let source =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, Rgba([80, 100, 120, 77])));
        let e = Edit {
            exposure: 2.,
            adjustment_opacity: 0.,
            ..Edit::default()
        };
        assert_eq!(
            render(source.clone(), &e, None).unwrap().to_rgba8(),
            source.to_rgba8()
        );
        let e = Edit {
            exposure: 2.,
            adjustment_enabled: false,
            ..Edit::default()
        };
        assert_eq!(
            render(source.clone(), &e, None).unwrap().to_rgba8(),
            source.to_rgba8()
        );
    }
    #[test]
    fn prevents_two_writers() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("workspace");
        let first = Store::open(root.clone(), "test").unwrap();
        assert!(Store::open(root.clone(), "test").is_err());
        drop(first);
        assert!(Store::open(root, "test").is_ok());
    }
    #[test]
    fn rejects_invalid_input() {
        assert!(
            Edit {
                exposure: f32::NAN,
                ..Edit::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            Edit {
                crop: Some([0.9, 0., 0.5, 1.]),
                ..Edit::default()
            }
            .validate()
            .is_err()
        );
        assert!(valid_id("../../outside").is_err());
    }
}
