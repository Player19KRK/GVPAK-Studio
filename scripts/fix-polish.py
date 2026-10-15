from pathlib import Path
p=Path('src/components/Inspector.svelte');s=p.read_text(encoding='utf-8-sig').replace('value={studio.draft[c.key]} onchange=', 'value={Number(studio.draft[c.key].toFixed(2))} onchange=');p.write_text(s,encoding='utf-8')
for name,css in [('Assistant','workflow-panel assistant-panel'),('Generate','workflow-panel generation-panel'),('Export','workflow-panel export-panel'),('Library','library')]:
 p=Path('src/components')/(name+'.svelte');s=p.read_text(encoding='utf-8-sig').replace('<div class="'+css+'">','<div class="'+css+'" inert={!!studio.busy}>');p.write_text(s,encoding='utf-8')
p=Path('src/App.svelte');s=p.read_text(encoding='utf-8-sig').replace('<header class="app-header">','<header class="app-header" inert={!!studio.busy}>').replace('<nav class="toolrail"','<nav inert={!!studio.busy} class="toolrail"');p.write_text(s,encoding='utf-8')
p=Path('backend/src/transport.rs');s=p.read_text(encoding='utf-8-sig').replace('let path=temp.path().join(format!("{i}-{file}"));','let dir=temp.path().join(i.to_string());std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;let path=dir.join(file.as_ref());');p.write_text(s,encoding='utf-8')
p=Path('backend/src/main.rs');s=p.read_text(encoding='utf-8-sig').replace('pub async fn dispatch(','pub(crate) async fn dispatch(');p.write_text(s,encoding='utf-8')
