from pathlib import Path
p=Path('src/components/Export.svelte');s=p.read_text(encoding='utf-8-sig').replace('notify,referencePreview}', 'notify,referencePreview,current}')
s=s.replace("let errors=$derived", "let leadId=$derived(scope().includes(studio.aid)?studio.aid:scope()[0]);let lead=$derived(current()?.assets.find(a=>a.id===leadId));\n let errors=$derived")
s=s.replace('src={referencePreview()}','src={leadId===studio.aid?referencePreview():studio.thumbs[leadId]}').replace('{asset()?.name}', '{lead?.name}')
p.write_text(s,encoding='utf-8')
p=Path('src/components/Assistant.svelte');s=p.read_text(encoding='utf-8-sig').replace('src={studio.thumbs[studio.aid]} alt="Selected sample"','src={studio.thumbs[scope()[0]]} alt="Selected sample"');s=s.replace('<h3>Sample</h3>', '<h3>Sample</h3><p class="sample-name">{current()?.assets.find(a=>a.id===scope()[0])?.name}</p>');p.write_text(s,encoding='utf-8')
p=Path('src/components/Coverflow.svelte');s=p.read_text(encoding='utf-8-sig').replace('class="cf-card" style=', 'class="cf-card" data-side={i>index?\'right\':i<index?\'left\':\'center\'} style=');p.write_text(s,encoding='utf-8')
p=Path('src/App.svelte');s=p.read_text(encoding='utf-8-sig').replace("studio.mode=value;studio.view='canvas';radial=null;", "studio.mode=value;studio.settingsReturn='';studio.view='canvas';radial=null;")
s=s.replace("{isDirty()?studio.draftStatus:''}<b>EDGE</b>","{isDirty()?studio.draftStatus:''}{#if isDirty()}<button class=\"clear-selection\" title=\"Return to the saved version\" onclick={()=>change({...asset().edit})}>Revert draft</button>{/if}<b>EDGE</b>")
s=s.replace("const tools=['adjust','crop','transform','compare'];const i=tools.indexOf(studio.tool);tool(tools[(i+(e.key.toLowerCase()==='q'?3:1))%4]);", "const tools=['adjust','crop','transform'];const i=tools.indexOf(studio.tool);tool(tools[(i+(e.key.toLowerCase()==='q'?tools.length-1:1))%tools.length]);")
import re
s=re.sub(r"e\.key==='\\+'", "e.code==='Backslash'", s)
p.write_text(s,encoding='utf-8')
