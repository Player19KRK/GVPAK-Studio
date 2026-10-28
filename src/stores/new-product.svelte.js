import {getNewProductApi} from '../services/new-product.ts';
import {chooseProductDirectory,restoreProductOutput,rememberProductOutput,writeProductResult,productZip,downloadProductBlob} from '../services/new-product-files.ts';
import {createSelectionContext} from '../lib/target.ts';
import {ratioLabel} from '../lib/ratio.ts';
import {DEMO_FACTS,defaultProductOptions,compileProductManifest,validateAnalysis,assignmentOrder,buildProductPrompt} from '../lib/new-product.ts';
export function createNewProductStore(api=getNewProductApi()){
  const state=$state({phase:'idle',error:'',brief:'',facts:DEMO_FACTS,options:defaultProductOptions(),assets:[],projectId:null,analysis:null,answers:{},manifest:null,assignmentMode:'auto',activeKind:'white',assignments:{},manualRefs:{},confirmation:null,capabilities:null,capabilityState:'idle',capabilityError:'',jobs:[],activeJobId:null,outputs:{},outputName:'',savingError:'',stopRequested:false});
  let autoRun=true,nonce=crypto.randomUUID(),directory=null,timer=null,off=()=>{},alive=true,refreshing=false,refreshAgain=false;
  const blobs=new Map(),saving=new Set();
  const prefKey=()=>`gvpak-new-product:${window.__gvpakTeamUser?.id||'host'}`;
  function persist(){try{localStorage.setItem(prefKey(),JSON.stringify({brief:state.brief,facts:state.facts,options:state.options,projectId:state.projectId,activeJobId:state.activeJobId,assignmentMode:state.assignmentMode,assignments:state.assignments}));}catch{}}
  function context(){return createSelectionContext({page:'library',mentions:[],selected:state.assets.map(asset=>asset.id),editorAssetId:null,filteredIds:state.assets.map(asset=>asset.id),filter:{folder:'',tag:'',status:'',q:'',sort:''},names:Object.fromEntries(state.assets.map(asset=>[asset.id,asset.name]))});}
  function forgetRequest(){nonce=crypto.randomUUID();state.confirmation=null;state.error='';persist();}
  function questions(){return state.manifest?.questions||[];}
  function rebuild(){
    forgetRequest();if(!state.analysis){state.manifest=null;return false;}
    try{state.manifest=compileProductManifest({analysis:state.analysis,options:state.options,brief:state.brief,facts:state.facts,answers:state.answers,names:Object.fromEntries(state.assets.map(asset=>[asset.id,asset.name])),manualRefs:state.manualRefs,assignments:state.assignmentMode==='manual'?state.assignments:undefined});state.phase=state.manifest.pending.length?'needs_input':'ready';return !state.manifest.pending.length;}
    catch(error){state.error=error.message;state.manifest=null;state.phase='ready';return false;}
  }
  async function importFiles(files,label='图片套图'){
    state.phase='importing';state.error='';
    try{const result=await api.importFiles(files,label,state.projectId);if(!alive)return;for(const asset of state.assets)if(asset.previewUrl)URL.revokeObjectURL(asset.previewUrl);state.assets=result.assets;state.projectId=result.projectId;state.analysis=null;state.manifest=null;state.answers={};state.assignments={};state.manualRefs={};state.activeJobId=null;state.stopRequested=false;state.phase='idle';forgetRequest();}
    catch(error){if(error.name!=='AbortError'){state.error=error.message;state.phase='failed';}}
  }
  async function chooseSource(){try{const source=await chooseProductDirectory();directory=source.directory;state.outputName=directory.name;await rememberProductOutput(directory);await importFiles(source.files,source.label);}catch(error){if(error.name!=='AbortError')state.error=error.message;}}
  async function chooseOutput(){try{const picker=window.showDirectoryPicker;if(!picker)throw Error('请使用Chrome或Edge选择保存目录。');directory=await picker.call(window,{mode:'readwrite'});state.outputName=directory.name;state.savingError='';await rememberProductOutput(directory);await saveAvailable();}catch(error){if(error.name!=='AbortError')state.savingError=error.message;}}
  function effectiveAnswers(){const answers={...state.answers};if(state.options.contents!=='ask')for(const question of state.analysis.questions)if(question.kind==='contents')answers[question.id]={choice:state.options.contents};return answers;}
  async function analyze(approvedAnalysis=false){
    state.phase='analyzing';state.error='';
    try{const result=await api.analyze({projectId:state.projectId,context:context(),brief:state.brief,productFacts:state.facts,appearance:state.options.appearance,approvedAnalysis});if(!alive)return;
      if(result.confirmation){state.confirmation=result.confirmation;state.phase='awaiting_confirm';return;}
      state.analysis=validateAnalysis(result,state.assets.map(asset=>asset.id));state.answers={};if(rebuild()&&autoRun)await startGeneration();
    }catch(error){state.error=error.message;state.phase='failed';}
  }
  async function oneClick(generate=true){autoRun=generate;if(generate&&['done','partial','failed','cancelled','attention'].includes(state.phase)&&activeJob())forgetRequest();if(!state.assets.length){state.error='请先选择素材文件夹或拖入原图。';return;}if(!state.analysis)await analyze();else if((state.manifest?!state.manifest.pending.length:rebuild())&&generate)await startGeneration();}
  async function continueQuestions(generate=autoRun){autoRun=generate;for(const question of state.manifest?.questions||[]){if(question.kind==='exploded'&&state.answers[question.id]?.choice==='visible'&&!state.manualRefs[`${question.productId}:exploded`]?.length){state.error='请先为爆炸图指定参考图。';return;}}if(rebuild()&&autoRun)await startGeneration();}
  async function startGeneration(approvedGeneration=false){
    if(!state.manifest?.tasks?.length||state.manifest.pending?.length)return;
    state.phase='starting';state.error='';
    try{const result=await api.start({projectId:state.projectId,context:context(),analysisId:state.analysis.analysisId,answers:effectiveAnswers(),manifest:state.manifest,mode:state.options.mode,concurrency:state.options.concurrency,resolution:state.options.resolution,ratio:ratioLabel(state.options.ratio),orientation:state.options.ratio.orientation,clientRequestId:nonce,approvedGeneration});
      if(!alive)return;if(result.confirmation){state.confirmation=result.confirmation;state.phase='awaiting_confirm';return;}
      state.confirmation=null;state.activeJobId=result.job.id;state.jobs=[result.job,...state.jobs.filter(job=>job.id!==result.job.id)];state.phase='running';state.stopRequested=false;persist();await refreshJobs();
    }catch(error){state.error=error.message;state.phase='failed';}
  }
  async function confirm(){const stage=state.confirmation?.stage;state.confirmation=null;if(stage==='analysis')await analyze(true);else await startGeneration(true);}
  function cancelConfirmation(){state.confirmation=null;state.phase=state.analysis?'ready':'idle';autoRun=false;}
  function enableManual(){if(state.assignmentMode==='auto'&&state.manifest?.tasks?.length){const values={};for(const task of state.manifest.tasks){const id=task.referenceIds[0];values[id]=[...new Set([...(values[id]||[]),task.kind])];}state.assignments=values;}state.assignmentMode='manual';}
  function toggleAssignment(id){enableManual();const values=state.assignments[id]||[];state.assignments[id]=values.includes(state.activeKind)?values.filter(kind=>kind!==state.activeKind):assignmentOrder.filter(kind=>kind===state.activeKind||values.includes(kind));rebuild();}
  function touchManifest(){forgetRequest();state.phase='ready';}
  function setTaskReferences(task,ids){if(!ids.length||ids.length>13){state.error='每条任务需要1至13张原图。';return;}const product=state.analysis.products.find(product=>product.id===task.productId);task.referenceIds=ids;task.prompt=buildProductPrompt({kind:task.kind,title:task.title,product,references:ids.map(id=>({id,name:state.assets.find(asset=>asset.id===id)?.name||id,role:state.analysis.items.find(item=>item.assetId===id)?.role||'product evidence'})),brief:state.brief,facts:state.facts,appearance:state.options.appearance,answers:state.answers,questions:state.manifest.questions,options:state.options});touchManifest();}
  function setAnswer(id,choice,text){state.answers[id]={choice,text:text??state.answers[id]?.text??''};const question=state.manifest?.questions.find(question=>question.id===id);if(question?.kind==='exploded'&&choice==='visible'&&!state.manualRefs[`${question.productId}:exploded`])state.manualRefs[`${question.productId}:exploded`]=state.assets.filter(asset=>state.assignments[asset.id]?.includes('exploded')||state.analysis.products.find(product=>product.id===question.productId)?.explodedIds.includes(asset.id)).map(asset=>asset.id);forgetRequest();}
  function setQuestionReferences(productId,ids){state.manualRefs[`${productId}:exploded`]=ids;forgetRequest();}
  function resetAnalysis(){state.analysis=null;state.manifest=null;state.answers={};state.phase='idle';forgetRequest();}
  function activeJob(){return state.jobs.find(job=>job.id===state.activeJobId);}
  async function loadOutput(job,task){
    const key=`${job.id}:${task.id}`;if(blobs.has(key)||saving.has(key))return;saving.add(key);
    try{const blob=await api.result(job.id,task.id);if(!alive)return;blobs.set(key,blob);state.outputs[key]={url:URL.createObjectURL(blob),name:task.result.name,saved:false,error:''};if(directory){const name=await writeProductResult(directory,job.id,task.result,blob);if(alive)state.outputs[key]={...state.outputs[key],saved:true,savedName:name};}}
    catch(error){state.savingError=error.message;}finally{saving.delete(key);}
  }
  async function saveAvailable(){const job=activeJob();if(!job)return;for(const task of job.tasks.filter(task=>task.state==='done'&&task.result)){const key=`${job.id}:${task.id}`;if(blobs.has(key)&&directory&&!state.outputs[key]?.saved){try{const name=await writeProductResult(directory,job.id,task.result,blobs.get(key));state.outputs[key]={...state.outputs[key],saved:true,savedName:name};}catch(error){state.savingError=error.message;}}else await loadOutput(job,task);}}
  async function refreshJobs(){
    if(refreshing){refreshAgain=true;return;}refreshing=true;
    try{do{refreshAgain=false;const result=await api.jobs();if(!alive)return;state.jobs=result.jobs;if(!state.activeJobId&&result.jobs.length)state.activeJobId=result.jobs[0].id;
      const job=activeJob();if(job){if(['done','partial','failed','cancelled','attention'].includes(job.status)&&!['analyzing','needs_input','awaiting_confirm','ready','starting'].includes(state.phase))state.phase=job.status;else if(['running','waiting','queued','submitting','cancelling'].includes(job.status)&&!['analyzing','needs_input','awaiting_confirm','ready','starting'].includes(state.phase))state.phase='running';
      state.stopRequested=state.stopRequested||!!job.stopRequested;for(const task of job.tasks.filter(task=>task.state==='done'&&task.result))void loadOutput(job,task);persist();}
    }while(refreshAgain&&alive);}catch(error){if(alive)state.error=error.message;}finally{refreshing=false;}
  }
  async function cancel(){const job=activeJob();if(!job)return;try{await api.cancel(job.id);state.stopRequested=true;await refreshJobs();}catch(error){state.error=error.message;}}
  async function downloadAll(){const job=activeJob();if(!job)return;state.savingError='';try{const files=[];for(const task of job.tasks.filter(task=>task.state==='done'&&task.result)){const key=`${job.id}:${task.id}`,blob=blobs.get(key)||await api.result(job.id,task.id);files.push({name:task.result.name,blob});}files.push({name:'任务单.json',blob:new Blob([JSON.stringify(job,null,2)],{type:'application/json'})});downloadProductBlob(await productZip(files),`新品套图-${job.id.slice(0,8)}.zip`);}catch(error){state.savingError=error.message;}}
  function exportManifest(){if(state.manifest)downloadProductBlob(new Blob([JSON.stringify({brief:state.brief,facts:state.facts,options:state.options,answers:state.answers,manifest:state.manifest},null,2)],{type:'application/json'}),'新品任务单.json');}
  async function reloadCapabilities(){
    if(state.capabilityState==='loading')return;state.capabilityState='loading';state.capabilityError='';state.capabilities=null;
    try{const value=await api.capabilities();if(!alive)return;state.capabilities=value;state.capabilityState='ready';}
    catch(error){if(alive){state.capabilityState='error';state.capabilityError=error.message;}}
  }
  async function begin(){
    try{const saved=JSON.parse(localStorage.getItem(prefKey())||'null');if(saved){state.brief=saved.brief||'';state.facts=saved.facts||DEMO_FACTS;state.options={...defaultProductOptions(),...saved.options};state.projectId=saved.projectId||null;state.activeJobId=saved.activeJobId||null;state.assignmentMode=saved.assignmentMode||'auto';state.assignments=saved.assignments||{};}}catch{}
    try{await reloadCapabilities();if(!alive)return;directory=await restoreProductOutput();if(directory)state.outputName=directory.name;await refreshJobs();}catch(error){state.error=error.message;}
    if(!alive)return;off=api.subscribe(()=>{void refreshJobs();});timer=setInterval(()=>{if(activeJob()&&!['done','partial','failed','cancelled','attention'].includes(activeJob().status))void refreshJobs();},5000);
  }
  async function saveKey(key){await api.saveKey(key);await reloadCapabilities();state.error='';}
  function dispose(){alive=false;clearInterval(timer);off();for(const asset of state.assets)if(asset.previewUrl)URL.revokeObjectURL(asset.previewUrl);for(const output of Object.values(state.outputs))URL.revokeObjectURL(output.url);}
  return{state,begin,reloadCapabilities,dispose,chooseSource,chooseOutput,importFiles,oneClick,analyze,rebuild,continueQuestions,confirm,cancelConfirmation,toggleAssignment,enableManual,touchManifest,setTaskReferences,setAnswer,setQuestionReferences,resetAnalysis,activeJob,refreshJobs,cancel,downloadAll,exportManifest,saveKey,persist,saveAvailable};
}
