import test from 'node:test';import assert from 'node:assert/strict';
import {defaultProductOptions,compileProductManifest,collectQuestions,isConfidentDuplicate,validateAnalysis,DEMO_FACTS,assignmentOrder} from '../src/lib/new-product.ts';
import {productZip} from '../src/services/new-product-files.ts';
function fixture(){return{analysisId:'analysis',items:[
 {assetId:'a',productId:'p1',role:'assembled',view:'正面',quality:.95,contents:'unknown',duplicateOf:null,duplicateConfidence:0,productView:true},
 {assetId:'b',productId:'p1',role:'refill',view:'内托',quality:.85,contents:'empty',duplicateOf:null,duplicateConfidence:0,productView:true},
 {assetId:'c',productId:'p1',role:'parts',view:'拆解',quality:.9,contents:'unknown',duplicateOf:null,duplicateConfidence:0,productView:false},
 ],products:[{id:'p1',name:'可替换香体膏',identity:['塑料壳体','湿压纸浆内托','旋转推出'],sceneIds:['a','b'],humanIds:['a'],explodedIds:['c','a','b'],explodedReady:true,sceneDirections:['浅木台面','石材与柔光']}],questions:[]};}
const names={a:'正面.JPG',b:'内托.JPG',c:'拆解.JPG'};
const input=(patch={})=>({analysis:fixture(),options:defaultProductOptions(),brief:'干净简约高级',facts:DEMO_FACTS,answers:{},names,...patch});
test('empty refill blocks submission until a content question is answered',()=>{
 const result=compileProductManifest(input());assert.equal(result.tasks.length,0);assert.equal(result.pending[0].kind,'contents');
 const ready=compileProductManifest(input({answers:{'contents-p1':{choice:'add',text:'浅米色膏体，略高于纸浆口沿'}}}));
 assert.equal(ready.pending.length,0);assert.equal(ready.tasks.length,7);assert(ready.tasks.every(task=>task.prompt.includes('wet-pressed')&&task.prompt.includes('浅米色膏体')));
 assert(ready.tasks.some(task=>task.kind==='human'&&task.prompt.includes('adult model')));assert(ready.tasks.some(task=>task.kind==='scene'&&task.prompt.includes('Sony A7R III')));
});
test('keeping an insert empty cannot silently produce balm application images',()=>{
 const options=defaultProductOptions();options.contents='empty';const result=compileProductManifest(input({options}));
 assert(result.tasks.every(task=>task.prompt.includes('insert remains empty')));assert(result.tasks.find(task=>task.kind==='human').prompt.includes('rather than an empty package touching skin'));
});
test('same view/state is required for semantic dedup and manual choice can retain duplicates',()=>{
 const analysis=fixture();const copy={...analysis.items[0],assetId:'d',duplicateOf:'a',duplicateConfidence:.99};analysis.items.push(copy);names.d='重复正面.JPG';
 assert.equal(isConfidentDuplicate(copy,analysis),true);copy.view='侧面';assert.equal(isConfidentDuplicate(copy,analysis),false);copy.view='正面';copy.contents='filled';assert.equal(isConfidentDuplicate(copy,analysis),false);copy.contents='unknown';
 const options=defaultProductOptions();options.contents='empty';options.scene=options.human=options.exploded=0;
 assert.equal(compileProductManifest(input({analysis,options})).tasks.length,2);
 const manual=compileProductManifest(input({analysis,options,assignments:{a:['white'],d:['white']}}));assert.equal(manual.tasks.length,2);assert.equal(manual.tasks[1].referenceIds[0],'d');
});
test('manual grid gives each selected source the requested kinds with white inside the fixed order',()=>{
 const options=defaultProductOptions();options.contents='empty';const result=compileProductManifest(input({options,assignments:{a:['white','scene','human'],c:['exploded']}}));
 assert.deepEqual(assignmentOrder,['white','scene','human','exploded']);assert.equal(result.tasks.length,4);
 for(const kind of ['white','scene','human'])assert.equal(result.tasks.find(task=>task.kind===kind).referenceIds[0],'a');
 assert.equal(result.tasks.find(task=>task.kind==='exploded').referenceIds[0],'c');
 options.copies.scene=2;assert.equal(compileProductManifest(input({options,assignments:{a:['scene']}})).tasks.length,2);
 assert.throws(()=>compileProductManifest(input({options,assignments:{}})),/指定/);
});
test('redesign has one shared white anchor and never unlocks unselected components',()=>{
 const options=defaultProductOptions();options.contents='empty';options.appearance={mode:'customize',targets:['outer_shell'],instruction:'Only the outer shell becomes matte blue.'};
 const result=compileProductManifest(input({options,assignments:{a:['scene'],c:['exploded']}}));const anchors=result.tasks.filter(task=>task.anchor);
 assert.equal(anchors.length,1);assert.equal(anchors[0].kind,'white');assert(result.tasks.filter(task=>!task.anchor).every(task=>task.dependsOn[0]===anchors[0].id));
 assert(result.tasks.every(task=>task.prompt.includes('Lock every unselected component')&&task.prompt.includes('twist-up mechanism')));
 options.appearance.targets=[];assert.throws(()=>compileProductManifest(input({options})),/改款/);
});
test('insufficient exploded evidence asks once, and skipping it preserves the rest of the set',()=>{
 const analysis=fixture();analysis.products[0].explodedReady=false;const options=defaultProductOptions();options.contents='empty';
 assert.equal(collectQuestions(analysis,options).filter(question=>question.kind==='exploded').length,1);
 const result=compileProductManifest(input({analysis,options,answers:{'exploded-p1':{choice:'skip'}}}));assert(!result.tasks.some(task=>task.kind==='exploded'));assert.equal(result.tasks.length,6);
});
test('foreign references and missing images are rejected before generation',()=>{
 const analysis=fixture();validateAnalysis(analysis,['a','b','c']);analysis.products[0].sceneIds=['foreign'];assert.throws(()=>validateAnalysis(analysis,['a','b','c']),/参考图/);
 assert.throws(()=>validateAnalysis(fixture(),['a','b','c','d']),/部分素材/);
});
test('task prompts only contain filename labels and multiple product groups stay isolated',()=>{
 const analysis=fixture(),options=defaultProductOptions();options.contents='empty';analysis.products.push({...analysis.products[0],id:'p2',name:'第二款',sceneIds:['z'],humanIds:['z'],explodedIds:['z']});analysis.items.push({...analysis.items[0],assetId:'z',productId:'p2'});
 const result=compileProductManifest(input({analysis,options,names:{...names,a:'C:\\private\\front.JPG',z:'second.JPG'}}));
 assert(result.tasks.every(task=>!task.prompt.includes('C:\\private')));assert(result.tasks.filter(task=>task.productId==='p2').every(task=>task.referenceIds.every(id=>id==='z')));
});
test('download ZIP has UTF-8 names, correct CRC and one stored copy of each file',async()=>{
 const bytes=Buffer.from(await(await productZip([{name:'C:\\private\\图片.txt',blob:new Blob(['abc'])}])).arrayBuffer());
 assert.equal(bytes.readUInt32LE(0),0x04034b50);assert.equal(bytes.readUInt16LE(6),0x800);assert.equal(bytes.readUInt32LE(14),0x352441c2);assert.equal(bytes.readUInt32LE(18),3);
 const length=bytes.readUInt16LE(26);assert.equal(bytes.subarray(30,30+length).toString(),'图片.txt');assert.equal(bytes.readUInt32LE(bytes.length-22),0x06054b50);assert.equal(bytes.readUInt16LE(bytes.length-14),1);
});

test('insufficient exploded evidence remains blocking when the model labels its question optional',()=>{const analysis=fixture();analysis.products[0].explodedReady=false;analysis.questions=[{id:'model-exploded',productId:'p1',kind:'exploded',question:'是否需要爆炸图？',options:[{id:'visible',label:'只见部件'},{id:'skip',label:'跳过'}],assetIds:['a'],affectedKinds:['exploded'],required:false}];const options=defaultProductOptions();options.contents='empty';const result=compileProductManifest(input({analysis,options}));assert.equal(result.pending.length,1);assert.equal(result.pending[0].id,'model-exploded');});

test('shell redesign uses the complete product as anchor even when manual white shots only select the refill',()=>{const options=defaultProductOptions();options.contents='empty';options.appearance={mode:'customize',targets:['outer_shell'],instruction:'仅外壳改成雾蓝，纸浆内托不改'};const result=compileProductManifest(input({options,assignments:{b:['white'],a:['scene']}}));const anchor=result.tasks.find(task=>task.anchor);assert.equal(anchor.referenceIds[0],'a');assert(result.tasks.some(task=>!task.anchor&&task.kind==='white'&&task.referenceIds[0]==='b'));});
