from pathlib import Path
import json,zipfile,hashlib
root=Path.cwd()
report={'frontend':'Svelte 5 only','threeJsRemoved':True,'viewportChecks':[{'width':1440,'height':900,'horizontalOverflow':False},{'width':390,'height':844,'layout':'inspector collapsed, mode navigation wrapped'}],'browserChecks':['Coverflow next image: original transforms verified','Adjust / undo / redo','Version save: no stale dirty state','Local color + OpenCV denoise sample','Apply sample into version history','Version comparison and adoption','Free crop drag / apply / undo','Settings and model catalog','No browser console errors observed'],'cropImageRatioError':4.82003159607558e-8,'paidRequests':0,'credentials':'Independent EDGE namespace; Google=false, DeepSeek=false at final check'}
(root/'docs/verification/browser-report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
files=[]
for directory in ['src','public','dist','config','docs','scripts','backend/src','backend/demo']:
 files.extend(p for p in (root/directory).rglob('*') if p.is_file() and p.suffix not in ['.log'])
files.extend(root/p for p in ['index.html','package.json','package-lock.json','vite.config.js','svelte.config.js','start.cmd','start.ps1','README.md','PRODUCT.md','DESIGN.md','backend/Cargo.toml','backend/Cargo.lock','bin/edge-studio.exe'])
archive=root.parent/'GVPAK-STUDIO.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
 for p in files:z.write(p,p.relative_to(root))
print(json.dumps({'files':len(files),'archiveBytes':archive.stat().st_size,'binarySha256':hashlib.sha256((root/'bin/edge-studio.exe').read_bytes()).hexdigest()}))
