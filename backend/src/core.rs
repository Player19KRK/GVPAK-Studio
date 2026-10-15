use image::{DynamicImage, ImageDecoder, ImageReader, Rgba, RgbaImage, imageops::FilterType};
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
        Ok(v)
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
        let mut list = Vec::new();
        for entry in fs::read_dir(self.root.join("projects")).map_err(|e| e.to_string())? {
            let e = entry.map_err(|e| e.to_string())?;
            if e.file_type().map_err(|e| e.to_string())?.is_dir() {
                let name = e.file_name().to_string_lossy().to_string();
                if valid_id(&name).is_ok() {
                    list.push(self.load(&name)?);
                }
            }
        }
        list.sort_by_key(|p| std::cmp::Reverse(p.updated_at));
        Ok(list)
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
                .any(|a| a.id == aid && a.versions.iter().any(|v| v.id == vid && v.kind == "local"))
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
        for aid in ids {
            valid_id(&aid)?;
            if let Some(a) = p.assets.iter_mut().find(|a| a.id == aid) {
                if a.trashed == restore {
                    let old = self.dir(pid)?.join(&a.original);
                    let filename = old.file_name().ok_or("INVALID_PATH")?.to_string_lossy();
                    let new = format!("{}/{filename}", if restore { "originals" } else { "trash" });
                    fs::rename(&old, self.dir(pid)?.join(&new)).map_err(|e| e.to_string())?;
                    a.original = new;
                    a.trashed = !restore;
                }
            }
        }
        p.updated_at = now();
        self.save(&p)?;
        Ok(p)
    }
    pub fn export(
        &self,
        pid: &str,
        ids: Vec<String>,
        preset: ExportPreset,
        cancel: &AtomicBool,
        progress: impl Fn(usize, usize),
    ) -> Res<Vec<ExportItem>> {
        if !["png", "jpeg", "webp"].contains(&preset.format.as_str())
            || !["crop", "pad"].contains(&preset.fit.as_str())
            || preset.width == 0
            || preset.height == 0
            || preset.width > 16000
            || preset.height > 16000
            || preset.width as u64 * preset.height as u64 > 120_000_000
            || preset.quality == 0
            || preset.quality > 100
        {
            return Err(err("INVALID_EXPORT", "导出尺寸、格式或适配方式无效"));
        }
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
                let im = render(self.source(&p, aid)?, &a.edit, None)?;
                let mut im = fit(
                    &im,
                    preset.width,
                    preset.height,
                    preset.fit == "pad",
                    preset.background,
                );
                if preset.format == "jpeg" {
                    im = flatten(&im, preset.background)
                }
                let (data, q) = encode_target(&im, &preset)?;
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
    if depth > 32 || files.len() >= 5000 {
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
    image::imageops::overlay(
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
