from pathlib import Path
import shutil,re,json,zipfile
root=Path.cwd();source=Path(r'D:/AI/AI应用项目/GVPAK STUDIO 2')
with zipfile.ZipFile(root/'docs/pre-migration-demo.zip','w',zipfile.ZIP_DEFLATED) as z:
 for d in ['src','backend/src']:
  for p in (root/d).rglob('*'):
   if p.is_file():z.write(p,p.relative_to(root))
 for f in ['package.json','backend/Cargo.toml','start.ps1','README.md']:z.write(root/f,f)
for name in ['core.rs','assistant.rs','hybrid.rs','models.rs','opencv_worker.py']:
 shutil.copy2(source/'apps/desktop/src'/name,root/'backend/src'/name)
shutil.copy2(source/'config/models.json',root/'config/models.json')
shutil.copytree(source/'apps/web/public/demo',root/'backend/demo',dirs_exist_ok=True)
shutil.copytree(source/'packages/contracts',root/'docs/source-contracts',dirs_exist_ok=True)
shutil.copy2(source/'integrations/aistudio-coverflow/src/CoverFlow.tsx',root/'docs/CoverFlow-reference.tsx')
shutil.copy2(source/'integrations/aistudio-coverflow/src/coverflow.css',root/'docs/coverflow-reference.css')
s=(source/'apps/desktop/src/main.rs').read_text(encoding='utf-8-sig')
commands=[]
for m in re.finditer(r'#\[tauri::command\]\s*(async\s+)?fn\s+(\w+)\s*\((.*?)\)\s*(?:->\s*([^\{]+))?\{',s,re.S):
 async_,name,args,ret=m.groups();parts=[];start=0;depth=0
 for i,c in enumerate(args):
  if c=='<':depth+=1
  elif c=='>':depth-=1
  elif c==',' and depth==0:parts.append(args[start:i].strip());start=i+1
 parts.append(args[start:].strip());call=[];fields=[]
 for part in filter(None,parts):
  pname,typ=map(str.strip,part.split(':',1))
  if pname=='s':call.append('State(&s)')
  elif pname=='app':call.append('hub.clone()')
  else:
   camel=re.sub(r'_([a-z])',lambda m:m[1].upper(),pname);call.append(f'arg(&v,"{camel}")?');fields.append({'name':camel,'type':typ})
 expr=f'{name}('+','.join(call)+')'+('.await' if async_ else '')
 if ret:expr+='?'
 if ret and 'ipc::Response' in ret:body=f'Ok(binary({expr}))'
 else:body=f'Ok(axum::Json(serde_json::to_value({expr}).map_err(|e|e.to_string())?).into_response())'
 commands.append((name,fields,f'"{name}" => {body},'))
body=s[:s.index('fn main() {')]
body=body.replace('#[tauri::command]','').replace('use tauri::{Emitter, State};','use transport::{State, AppHandle};\nmod transport;\nuse axum::response::IntoResponse;')
body=body.replace('tauri::AppHandle','AppHandle').replace('tauri::async_runtime::','transport::runtime::').replace('tauri::ipc::Response::new','Vec::from').replace('tauri::ipc::Response','Vec<u8>')
body=body.replace('GVPAK-STUDIO-2-{}','EDGE-STUDIO-{}').replace('gvpak-settings-{}','edge-settings-{}')
init=s[s.index('    let root =',s.index('fn main() {')):s.index('    tauri::Builder::default()')]
init=init.replace('PathBuf::from("D:/AI/GVPAK-STUDIO2-DATA/dev")','project_root.join("backend/studio-data")')
init=init.replace('include_bytes!("../../web/public/demo/','include_bytes!("../demo/').replace('光与石 · 建筑研究','Architecture').replace('GVPAK STUDIO','EDGE')
(root/'backend/src/main.rs').write_text(body+'\n#[tokio::main]\nasync fn main() {\n    let executable=std::env::current_exe().expect("Executable path");\n    let project_root=executable.ancestors().find(|p|p.join("dist/index.html").exists()).map(|p|p.to_path_buf()).unwrap_or_else(||std::env::current_dir().unwrap());\n'+init+'\n    transport::serve(shared,project_root).await;\n}\n',encoding='utf-8')
(root/'docs/command-contract.json').write_text(json.dumps([{'command':n,'args':a} for n,a,_ in commands],ensure_ascii=False,indent=2),encoding='utf-8')
# Derive the complete dispatch from the original command signatures, with no feature stubs.
p=root/'backend/src/main.rs'
with p.open('a',encoding='utf-8') as f:
 f.write('\nfn arg<T:serde::de::DeserializeOwned>(v:&Value,key:&str)->Res<T>{serde_json::from_value(v.get(key).cloned().unwrap_or(Value::Null)).map_err(|e|format!("INVALID_ARGUMENT {key}: {e}"))}\nfn binary(bytes:Vec<u8>)->axum::response::Response{([(axum::http::header::CONTENT_TYPE,"application/octet-stream")],bytes).into_response()}\npub async fn dispatch(s:Shared,hub:AppHandle,name:String,v:Value)->Res<axum::response::Response>{match name.as_str(){\n'+ '\n'.join(c[2] for c in commands)+'\n_=>Err("UNKNOWN_COMMAND".into())}}\n')
for name in ['assistant.rs','hybrid.rs','models.rs']:
 p=root/'backend/src'/name;t=p.read_text(encoding='utf-8-sig');t=t.replace('tauri::async_runtime::','crate::transport::runtime::').replace('tauri::AppHandle','crate::transport::AppHandle').replace('use tauri::{Emitter, Manager};','')
 t=t.replace('GVPAK-STUDIO-2-{}','EDGE-STUDIO-{}').replace('../../../config/models.json','../../config/models.json')
 if name=='hybrid.rs':
  begin=t.index('                        if let Some(w) = app.get_webview_window');i=begin;level=0;started=False
  while i<len(t):
   if t[i]=='{':level+=1;started=True
   elif t[i]=='}':
    level-=1
    if started and level==0:i+=1;break
   i+=1
  t=t[:begin]+t[i:]
 p.write_text(t,encoding='utf-8')
# Original core is retained byte for byte; shell-only changes live in main/transport.
print('Migrated commands:',len(commands))
print(json.dumps([n for n,_,_ in commands]))
