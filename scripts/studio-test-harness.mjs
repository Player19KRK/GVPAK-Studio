import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {compileModule} from 'svelte/compiler';
import {flushSync} from 'svelte';
import {effect_root} from 'svelte/internal/client';

let serial=0;
const source=path=>fs.readFileSync(new URL(path,import.meta.url),'utf8');
const file=path=>new URL(path,import.meta.url).href;
const moduleUrl=code=>'data:text/javascript;base64,'+Buffer.from(code).toString('base64');
function compile(code){return moduleUrl(compileModule(code,{filename:'regression.svelte.js',generate:'client'}).js.code.replace(/['"](svelte(?:\/internal\/client)?)['"]/g,(_,name)=>JSON.stringify(import.meta.resolve(name)))+`\n// isolated harness ${serial++}`);}
const configurationUrl=compile(source('../src/lib/configuration.svelte.js'));
const wait=()=>new Promise(resolve=>setTimeout(resolve,0));

export async function harness(t,count=2){
 const savedFetch=globalThis.fetch,savedStorage=globalThis.localStorage;
 const storage=new Map();
 globalThis.localStorage={getItem:k=>storage.get(k)??null,setItem:(k,v)=>storage.set(k,v)};
 const calls=[];
 let releasePicker=null;
 const initialEdit={adjustmentOpacity:1,adjustmentEnabled:true,exposure:0,contrast:1,saturation:1,temperature:0,rotation:0,flipH:false,flipV:false,crop:null};
 const project={id:'p',assets:Array.from({length:count},(_,i)=>({id:String(i),name:`image-${i}`,trashed:false,edit:{...initialEdit},versions:[]}))};
 let backend=structuredClone(project);
 globalThis.fetch=async(url,options)=>{
  const command=url.split('/').at(-1),args=JSON.parse(options.body);
  calls.push({command,args});
  await wait();
  if(command==='thumbnail')return new Response(new Blob(['image']),{headers:{'Content-Type':'application/octet-stream'}});
  if(command==='pick_output')return new Response(JSON.stringify(await new Promise(resolve=>releasePicker=resolve)));
  let result;
  if(command==='assistant_preview')result={sampleId:'sample',before:'',after:''};
  else if(command==='assistant_apply')result={results:args.sampleId?[{assetId:'0',status:'ok'}]:[]};
  else if(command==='credential_status')result=true;
  else if(command==='single_generation_submit'||command==='local_plan_submit')result={task:{id:'fixture-task',state:'done'}};
  else if(command==='generate'){if(!args.approved)return new Response(JSON.stringify({error:'COST_APPROVAL_REQUIRED'}),{status:400});backend.assets[0].versions.push({id:'generated-fixture',kind:'generated'});result={project:backend,versionId:'generated-fixture'};}
  else if(command==='bootstrap')result={projects:[backend],dataRoot:'test-root'};
  else if(command==='edit_commit'){backend.assets.find(a=>a.id===args.assetId).edit=args.edit;result=backend;}
  else throw Error(`Unexpected API: ${command}`);
  return new Response(JSON.stringify(result),{headers:{'Content-Type':'application/json'}});
 };
 const thumbnailCacheFixture=moduleUrl('export {thumbnailInput,thumbnailSourceKey} from '+JSON.stringify(file('../src/services/thumbnail-cache.ts'))+';export const loadThumbnailSource=async(source,fetcher)=>({blob:await fetcher()});');
 const stateSource=source('../src/studio.svelte.js')
  .replace("'./services/thumbnail-cache.ts'",JSON.stringify(thumbnailCacheFixture))
  .replace("'./lib/task-ownership.js'",JSON.stringify(file('../src/lib/task-ownership.js')))
  .replace("'./services/realtime.ts'",JSON.stringify(file('../src/services/realtime.ts')))
  .replace("'./preview-cache'",JSON.stringify(file('../src/preview-cache.js')))
  .replace("'./preview-loader'",JSON.stringify(moduleUrl('export function disposePreviews(){}')))
  .replace("'./api'",JSON.stringify(file('../src/api.js')));
 const stateUrl=compile(stateSource),state=await import(stateUrl);
 Object.assign(state.studio,{projects:[project],pid:'p',aid:'0',dataRoot:'test-root',draft:{...initialEdit},selected:project.assets.map(a=>a.id)});
 const disposers=[];
 t.after(async()=>{for(const stop of disposers)stop();await state.thumbnails();state.dispose();globalThis.fetch=savedFetch;globalThis.localStorage=savedStorage;});
 return {state,stateModuleUrl:stateUrl,calls,project,initialEdit,setBackend(value){backend=structuredClone(value);},pickerOpen:()=>!!releasePicker,releasePicker:()=>releasePicker?.('test-output'),
  async generate(){
   let script=source('../src/components/Generate.svelte').split('<script>')[1].split('</script>')[0];
   script=script.replace(/import\s+[\s\S]*?\s+from\s+['"][^'"]+['"];?/g,'');
   const driver=await import(compile(`import {studio,asset,run,updateProject,save,notify,referencePreview,goSettings,importImages,dropImages,current,selectProject,select} from ${JSON.stringify(stateUrl)};
    import {api,clone} from ${JSON.stringify(file('../src/api.js'))};
    import {createConfiguration} from ${JSON.stringify(configurationUrl)};
    import {normalizeEdit} from ${JSON.stringify(file('../src/preview-cache.js'))};
    import {ratios,chooseRatio,swapRatio,ratioLabel,defaultOrientation} from ${JSON.stringify(file('../src/lib/ratio.ts'))};
    const estimate=()=>0,money=String,isTeam=false,tasks={byId:{}},executionPrefs={status:'idle'},selectVersion=async()=>{},navigate=()=>{},editorUrl=id=>'/editor/'+id;
    const submitSingleGeneration=input=>api('single_generation_submit',{input}),singleGenerationConfigured=async()=>true;
    export function mount(){${script}
     return {generate,pasteReference,setSource(value){sourceMode=value;},getPrompt(){return prompt;},configure(){prompt='Fixture edit';modelId='fixture';},setPrompt(value){prompt=value;}};
    }`));
   state.studio.catalog={models:[{id:'fixture',resolutions:['1K'],ratios:['1:1']}]};
   let ui;disposers.push(effect_root(()=>ui=driver.mount()));flushSync();ui.configure();flushSync();return ui;
  },
  async watchProjects(){
   const effect=source('../src/App.svelte').split('\n').find(line=>line.includes("if(studio.view==='projects')for"));
   const watcher=await import(compile(`import {studio,thumbnails} from ${JSON.stringify(stateUrl)}; export function watch(){${effect}}`));
   disposers.push(effect_root(()=>watcher.watch()));flushSync();
  },
  async assistant(){
   let script=source('../src/components/Assistant.svelte').split('<script>')[1].split('</script>')[0];
   script=script.replace(/import\s+[\s\S]*?\s+from\s+['"][^'"]+['"];?/g,'');
   const driver=await import(compile(`import {studio,current,scope,run,save,refresh,notify,jobs as reloadJobs,asset,clearDrafts,goSettings,importImages,editFor} from ${JSON.stringify(stateUrl)};
    import {api,clone,rgb,hex,stored,persist,bytes} from ${JSON.stringify(file('../src/api.js'))};
    import {createConfiguration} from ${JSON.stringify(configurationUrl)};
    import {createSelectionContext} from ${JSON.stringify(file('../src/lib/target.ts'))};
    const onMount=()=>{},tasks={byId:{}},getAssistantApi=()=>({configured:async()=>false,presets:async()=>({}),savePreset:async()=>{},plan:async()=>({})});
    const selection={get context(){return this.snapshot()},snapshot:()=>createSelectionContext({page:'editor',mentions:[],selected:studio.selected,editorAssetId:studio.aid,filteredIds:studio.projects[0].assets.map(a=>a.id),filter:{folder:'',tag:'',status:'',q:'',sort:''},names:Object.fromEntries(studio.projects[0].assets.map(a=>[a.id,a.name]))})};
    const executeLocalPlan=(plan,context)=>api('local_plan_submit',{plan,context}),executePipeline=async()=>({});
    export function mount(){${script}
     return {apply,setPlan(value){plan=value;},getPlan(){return plan;}};
    }`));
   let ui;disposers.push(effect_root(()=>ui=driver.mount()));flushSync();
   ui.setPlan({summary:'Rotate',steps:[{op:'rotate',degrees:90}],warnings:[],clarification:null,output:null});flushSync();
   return ui;
  }
 };
}

