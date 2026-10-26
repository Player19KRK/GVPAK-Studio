from pathlib import Path
import json,zipfile,hashlib
root=Path.cwd()
# Package existing artifacts and evidence verbatim. Packaging does not run checks or create acceptance reports.
files=[]
for directory in ['src','public','dist','config','docs','scripts','backend/src','backend/demo']:
 files.extend(p for p in (root/directory).rglob('*') if p.is_file() and p.suffix not in ['.log'])
files.extend(root/p for p in ['index.html','package.json','package-lock.json','vite.config.js','svelte.config.js','start.cmd','start.ps1','一键启动.cmd','开启局域网访问.cmd','enable-team-lan.ps1','README.md','PRODUCT.md','DESIGN.md','backend/Cargo.toml','backend/Cargo.lock','bin/gvpak-studio.exe'])
archive=root.parent/'GVPAK-STUDIO.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
 for p in files:z.write(p,p.relative_to(root))
print(json.dumps({'files':len(files),'archiveBytes':archive.stat().st_size,'binarySha256':hashlib.sha256((root/'bin/gvpak-studio.exe').read_bytes()).hexdigest()}))
