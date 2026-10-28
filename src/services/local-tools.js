import {api,clone} from '../api.js';
import {studio,save,refresh,clearDrafts,asset,run} from '../studio.svelte';
export async function autoDeskewSubject(){
  const pid=studio.pid,id=studio.aid;if(!id||studio.busy)return;
  let completed=false;
  await run('正在识别主体倾斜并自动摆正',async()=>{
    const status=await api('assistant_tools');if(!status.available)throw Error('本地OpenCV不可用，请重新用一键启动.cmd启动完整应用。');
    await save([id]);
    const plan={summary:'主体自动摆正',steps:[{op:'deskew',max_degrees:45}],warnings:[],clarification:null,output:null};
    const sample=await api('assistant_preview',{projectId:pid,ids:[id],plan});
    const result=await api('assistant_apply',{sampleId:sample.sampleId,outputDir:null});
    const failure=(result.results||result.items||[]).find(item=>item.error);if(failure)throw Error(failure.error);
    await refresh();clearDrafts([id]);if(studio.aid===id&&asset())studio.draft=clone(asset().edit);completed=true;
  });
  if(!completed)throw Error(studio.error||'主体无法可靠识别，请使用清晰的单主体图片。');
  return true;
}
