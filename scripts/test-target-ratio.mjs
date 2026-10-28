import test from 'node:test';
import assert from 'node:assert/strict';
import {resolveTarget,createSelectionContext,serializeSelectionContext,fileNameOnly} from '../src/lib/target.ts';
import {ratios,chooseRatio,defaultOrientation,ratioParts,ratioLabel,swapRatio,ratioOutputSize} from '../src/lib/ratio.ts';
const empty={mentions:[],selected:[],editorAssetId:null,filteredIds:[]};
test('target priority covers all four sources without changing input order or mutation',()=>{
 const input={mentions:['m2','m1','m2'],selected:['s2','s1'],editorAssetId:'e',filteredIds:['f2','f1']};const original=structuredClone(input);
 assert.deepEqual(resolveTarget(input),{source:'mention',ids:['m2','m1']});
 assert.deepEqual(resolveTarget({...input,mentions:[]}),{source:'selected',ids:['s2','s1']});
 assert.deepEqual(resolveTarget({...input,mentions:[],selected:[]}),{source:'editor',ids:['e']});
 assert.deepEqual(resolveTarget({...input,mentions:[],selected:[],editorAssetId:null}),{source:'filter',ids:['f2','f1']});
 assert.deepEqual(input,original);assert.deepEqual(resolveTarget(empty),{source:'filter',ids:[]});
});
test('filter targets over 50 retain every ID and never introduce a fourth confirmation gate',()=>{
 const ids=Array.from({length:80},(_,i)=>`asset-${i}`);const result=resolveTarget({...empty,filteredIds:ids});
 assert.deepEqual(result.ids,ids);assert.equal('needsConfirmation' in result,false);
});
test('SelectionContext strips paths and extra fields and excludes a stale editor ID off the editor page',()=>{
 const context=createSelectionContext({...empty,page:'library',editorAssetId:'stale',filteredIds:['a','b'],filter:{folder:'folder-id',tag:'',status:'',q:'',sort:''},names:{a:'C:\\private\\a.png',b:'/home/private/b.jpg'},sourcePath:'must not leak'});
 assert.equal(context.editorAssetId,null);assert.equal(context.resolvedTarget.source,'filter');assert.deepEqual(context.fileNames,['a.png','b.jpg']);
 const wire=serializeSelectionContext({...context,sourcePath:'secret',filter:{...context.filter,absolutePath:'secret'}});
 assert(!JSON.stringify(wire).includes('secret'));assert.deepEqual(Object.keys(wire),['page','mentions','selected','editorAssetId','filter','filteredCount','resolvedTarget','fileNames']);
 assert.equal(fileNameOnly('C:sample.png'),'sample.png');
});
for(const ratio of ratios)for(const orientation of ['landscape','portrait'])test(`${ratio} / ${orientation}: actual label and dimensions are within one pixel`,()=>{
 const state={ratio,orientation},[a,b]=ratioParts(ratio,orientation),size=ratioOutputSize(803,601,state);
 assert.equal(ratioLabel(state),`${a}:${b}`);assert.equal(Math.max(size.width,size.height),803);
 assert(Math.abs(size.width-size.height*a/b)<=1);assert(Math.abs(size.height-size.width*b/a)<=1);
 if(ratio!=='1:1')assert.equal(ratioLabel(swapRatio(state)),`${b}:${a}`);
 assert(Math.max(size.width,size.height)<=803);
});
test('choosing resets direction, the square toggle is inert, and explicit long edge is honored',()=>{
 assert.equal(defaultOrientation('4:5'),'portrait');assert.equal(defaultOrientation('16:9'),'landscape');
 assert.deepEqual(chooseRatio('16:9'),{ratio:'16:9',orientation:'landscape'});
 const square=chooseRatio('1:1');assert.deepEqual(swapRatio(square),square);
 assert.deepEqual(ratioOutputSize(100,80,chooseRatio('4:5'),1000),{width:800,height:1000});
 assert.deepEqual(ratioOutputSize(100,80,chooseRatio('4:5')),{width:80,height:100});
 assert.throws(()=>ratioOutputSize(0,80,square));assert.throws(()=>ratioOutputSize(100,80,square,NaN));assert.throws(()=>ratioParts('2:1'));
});
