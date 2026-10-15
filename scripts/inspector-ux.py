from pathlib import Path
p=Path('src/studio.svelte.js');s=p.read_text(encoding='utf-8-sig').replace('inspector:true});',"inspector:true,inspectorTab:'adjust'});");p.write_text(s,encoding='utf-8')
p=Path('src/App.svelte');s=p.read_text(encoding='utf-8-sig').replace('PanelRightClose,Download','PanelRightClose,PanelRightOpen,Download')
s=s.replace("function tool(value){if(studio.busy)return;studio.tool=value;studio.mode='edit';studio.view='canvas';studio.inspector=true;}","""let panelVisible=$derived(studio.inspector&&studio.mode==='edit'&&studio.view==='canvas');
 function panelOpen(value){return panelVisible&&studio.tool===value&&studio.inspectorTab==='adjust';}
 function tool(value,toggle=false){
  if(studio.busy)return;
  if(toggle&&panelOpen(value)){studio.inspector=false;return;}
  studio.tool=value;studio.mode='edit';studio.view='canvas';studio.inspectorTab='adjust';studio.inspector=true;
 }
 function closeInspector(){studio.inspector=false;document.querySelector(`[data-tool="${studio.tool}"]`)?.focus();}
 function toggleInspector(){if(studio.busy||studio.mode!=='edit'||studio.view!=='canvas')return;studio.inspector=!studio.inspector;}
""")
for name,label in [('adjust','Adjust'),('crop','Crop'),('transform','Transform')]:
 old=f'''class:chosen={{studio.tool==='{name}'&&studio.mode==='edit'}} onclick={{()=>tool('{name}')}} title="{label}" aria-label="{label}"'''
 new=f'''data-tool="{name}" class:chosen={{panelOpen('{name}')}} onclick={{()=>tool('{name}',true)}} aria-controls="editor-inspector" aria-expanded={{panelOpen('{name}')}} title={{panelOpen('{name}')?'{label} · Collapse panel':'{label} · Open panel'}} aria-label="{label}"'''
 assert old in s,name;s=s.replace(old,new)
old='''<button onclick={()=>studio.inspector=!studio.inspector} title="Toggle inspector" aria-label="Toggle inspector"><PanelRightClose size={19}/></button>'''
new='''<button onclick={toggleInspector} disabled={studio.mode!=='edit'||studio.view!=='canvas'} title={panelVisible?'Hide inspector':'Show inspector'} aria-label="Toggle inspector" aria-controls="editor-inspector" aria-expanded={panelVisible}>{#if panelVisible}<PanelRightClose size={19}/>{:else}<PanelRightOpen size={19}/>{/if}</button>'''
assert old in s;s=s.replace(old,new).replace('<Inspector/>','<Inspector onclose={closeInspector}/>');p.write_text(s,encoding='utf-8')
p=Path('src/components/Inspector.svelte');s=p.read_text(encoding='utf-8-sig').replace('Clock,ArrowUpRight}', 'Clock,ArrowUpRight,X}')
s=s.replace(" let tab=$state('adjust');const controls=", " import {slide} from 'svelte/transition';import {cubicOut} from 'svelte/easing';\n let {onclose}=$props();const reducedMotion=window.matchMedia('(prefers-reduced-motion: reduce)').matches;\n const controls=")
import re
s=re.sub(r'\btab\b','studio.inspectorTab',s)
s=s.replace('<aside class="inspector" inert={!!studio.busy}>','<aside id="editor-inspector" aria-label="Image inspector" class="inspector" inert={!!studio.busy||!studio.inspector} transition:slide={{axis:\'x\',duration:reducedMotion?0:180,easing:cubicOut}}>')
s=s.replace("{studio.tool==='crop'?'Crop':studio.tool==='transform'?'Transform':'Inspector'}","{studio.inspectorTab==='history'?'History':studio.inspectorTab==='layers'?'Layers':studio.tool==='crop'?'Crop':studio.tool==='transform'?'Transform':'Adjustments'}")
s=s.replace('<h2>', '<h2>',1)
needle='<button aria-label="Reset edits"'
s=s.replace(needle,'<div class="inspector-title-actions"><button aria-label="Reset edits"',1)
s=s.replace('<RotateCcw size={15}/></button></div>','<RotateCcw size={15}/></button><button aria-label="Close inspector" title="Close inspector" onclick={onclose}><X size={17}/></button></div></div>',1)
p.write_text(s,encoding='utf-8')
p=Path('DESIGN.md');s=p.read_text(encoding='utf-8-sig');s+='\nInspector behavior: left Adjust/Crop/Transform buttons toggle their own visible parameter panel. A different tool or a different inspector tab opens the requested tool parameters. Keyboard and context-menu selection opens the panel without toggling. The general inspector toggle preserves the Layers/History tab. Closing preserves edits and returns focus to the tool. Disclosure controls expose expanded state; non-editor screens disable the inspector toggle. Use a brief 180 ms width transition, disabled for reduced motion.\n';p.write_text(s,encoding='utf-8')
