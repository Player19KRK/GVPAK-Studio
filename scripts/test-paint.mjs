import test from 'node:test';
import assert from 'node:assert/strict';
import {applyStroke,renderPaint,sourcePoint,viewPoint,paletteColor} from '../src/paint.js';
import {normalizeEdit,previewKey} from '../src/preview-cache.js';
import {harness} from './studio-test-harness.mjs';
const plain={crop:null,rotation:0,flipH:false,flipV:false};
const stroke={points:[[.25,.5],[.5,.5],[.75,.5]],color:[227,42,50],size:12,hardness:1,opacity:.5,erase:false};
test('one stroke has uniform opacity even where samples overlap; eraser only changes paint',()=>{
 const layer=renderPaint(40,40,[stroke],plain,40,40);assert.equal(layer[(20*40+20)*4+3],128);
 applyStroke(layer,40,40,{...stroke,erase:true,opacity:1},plain,40,40);assert.equal(layer[(20*40+20)*4+3],0);
});
test('source coordinates roundtrip through every orientation and crop',()=>{
 for(const rotation of [0,90,180,270])for(const flipH of [false,true])for(const flipV of [false,true]){
  const e={crop:[.1,.2,.5,.6],rotation,flipH,flipV};const p=[.31,.72],roundtrip=viewPoint(sourcePoint(p,e,1000,800),e,1000,800);roundtrip.forEach((v,i)=>assert.ok(Math.abs(v-p[i])<1e-10));
 }
});
test('stroke state changes cache keys and empty legacy edits normalize identically',()=>{
 const a={id:'a'};assert.notEqual(previewKey('r','p',a,plain),previewKey('r','p',a,{...plain,strokes:[stroke]}));
 assert.deepEqual(normalizeEdit(plain),normalizeEdit({...plain,strokes:[]}));
});
test('palette colors are valid and cover dark, light and neutral values',()=>{
 const colors=Array.from({length:72},(_,i)=>paletteColor(i));assert.ok(colors.every(c=>/^#[0-9a-f]{6}$/.test(c)));assert.ok(colors.includes('#ffffff'));assert.ok(colors.includes('#000000'));assert.equal(new Set(colors).size,72);
});
test('each stroke is one undo step and save sends real paint data',async t=>{
 const {state,calls}=await harness(t);state.change({strokes:[stroke]});assert.equal(state.studio.history['0'].length,1);state.undo();assert.equal(state.studio.draft.strokes?.length||0,0);state.redo();assert.equal(state.studio.draft.strokes.length,1);await state.save(['0']);assert.equal(calls.find(c=>c.command==='edit_commit').args.edit.strokes.length,1);assert.equal(state.isDirty(),false);
});
