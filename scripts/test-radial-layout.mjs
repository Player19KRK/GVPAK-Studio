import test from 'node:test';
import assert from 'node:assert/strict';
import {radialPlacement} from '../src/radial-layout.js';
const viewport={width:1280,height:650};
test('all menu levels remain inside a canvas shorter than 300px',()=>{
 const stage={left:48,top:86,width:952,height:244};
 for(const page of ['root','color','brush','eraser'])for(const anchor of [{x:49,y:87},{x:999,y:329},{x:700,y:200}]){
  const p=radialPlacement(page,anchor,stage,viewport);
  assert.ok(p.left>=stage.left+8-.001);assert.ok(p.top>=stage.top+8-.001);
  assert.ok(p.left+p.width<=stage.left+stage.width-8+.001);assert.ok(p.top+p.height<=stage.top+stage.height-8+.001);
 }
});
test('root is compact and controls retain their origin when there is space',()=>{
 const stage={left:48,top:86,width:1000,height:600},anchor={x:500,y:300};
 const root=radialPlacement('root',anchor,stage,{width:1440,height:960});assert.equal(root.width,224);
 for(const page of ['brush','eraser'])assert.deepEqual(radialPlacement(page,anchor,stage,{width:1440,height:960}).origin,root.origin);
 assert.equal(radialPlacement('color',anchor,stage,{width:1440,height:960}).width,288);
});
test('viewport-clipped canvas and narrow window do not expose offscreen controls',()=>{
 const stage={left:42,top:114,width:348,height:500};
 for(const page of ['root','color','brush','eraser']){
  const p=radialPlacement(page,{x:385,y:440},stage,{width:390,height:450});assert.ok(p.left+p.width<=382);assert.ok(p.top+p.height<=442);
 }
});

test('short canvas retains the selected tool position when entering parameters',()=>{
 const stage={left:48,top:86,width:1232,height:254};
 for(const anchor of [{x:922,y:213},{x:1260,y:320},{x:55,y:100}]){
  const root=radialPlacement('root',anchor,stage,viewport);
  for(const page of ['brush','eraser'])assert.deepEqual(radialPlacement(page,anchor,stage,viewport).origin,root.origin);
 }
});
