from pathlib import Path
import json
p=Path('package-lock.json');v=json.loads(p.read_text(encoding='utf-8-sig'));v['version']='2.0.0';v['packages']['']['version']='2.0.0';p.write_text(json.dumps(v,indent=2),encoding='utf-8')
p=Path('start.ps1');s=p.read_text(encoding='utf-8-sig').replace("if($health.service -eq 'gvpak-studio')","if($health.service -eq 'gvpak-studio' -and $health.version -eq '2.0.0')");p.write_text(s,encoding='utf-8')
