from pathlib import Path
p=Path('backend/src/core.rs');s=p.read_text(encoding='utf-8-sig');needle='    pub fn preview(&self, pid: &str, aid: &str, edit: Edit) -> Res<Vec<u8>> {'
new='''    pub fn preview_source(&self, pid: &str, aid: &str) -> Res<Vec<u8>> {
        let p = self.load(pid)?;
        let source = self.source(&p, aid)?;
        if source.width() > 1600 || source.height() > 1600 {
            png(&source.resize(1600, 1600, FilterType::Triangle))
        } else { png(&source) }
    }
'''
assert needle in s;s=s.replace(needle,new+needle);p.write_text(s,encoding='utf-8')
p=Path('backend/src/main.rs');s=p.read_text(encoding='utf-8-sig');needle='fn thumbnail(s: State<Shared>'
idx=s.index(needle);s=s[:idx]+'''async fn preview_source(s: State<'_, Shared>, project_id: String, asset_id: String) -> Res<Vec<u8>> {
    let s = s.inner().clone();
    transport::runtime::spawn_blocking(move || locked(&s)?.preview_source(&project_id, &asset_id)).await.map_err(|e| e.to_string())?
}
'''+s[idx:]
s=s.replace('"thumbnail" =>', '"preview_source" => Ok(binary(preview_source(State(&s),arg(&v,"projectId")?,arg(&v,"assetId")?).await?)),\n"thumbnail" =>');p.write_text(s,encoding='utf-8')
p=Path('src/studio.svelte.js');s=p.read_text(encoding='utf-8-sig');s=s.replace("import {api,imageUrl", "import {disposePreviews} from './preview-loader';\nimport {api,imageUrl",1);s=s.replace("previewBusy:false,", "previewBusy:false,previewSignature:'',previewEngine:'checking',gpuPreviewEnabled:stored('gpu-preview-enabled',true),previewDiagnostics:{},")
s=s.replace("studio.previewAssetId===studio.aid&&studio.preview?", "studio.previewAssetId===studio.aid&&studio.previewSignature===JSON.stringify(studio.draft)&&studio.preview?")
s=s.replace('export function dispose(){cacheDrafts();','export function dispose(){cacheDrafts();disposePreviews();')
p.write_text(s,encoding='utf-8')
