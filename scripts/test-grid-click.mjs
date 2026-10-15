import test from 'node:test';
import assert from 'node:assert/strict';
import {isEditDoubleClick} from '../src/grid-click.js';
const first={id:'a',time:1000,detail:1,x:20,y:20};
const second={...first,time:1150,detail:2};
test('deliberate tight double click opens',()=>assert.equal(isEditDoubleClick(first,second),true));
test('selection cancellation outside 200ms never opens even if OS calls it double click',()=>{for(const gap of [201,250,350,500])assert.equal(isEditDoubleClick(first,{...second,time:1000+gap}),false);});
test('threshold boundary is explicit',()=>assert.equal(isEditDoubleClick(first,{...second,time:1200}),true));
test('different image, movement, keyboard and triple click do not open',()=>{for(const patch of [{id:'b'},{x:30},{detail:0},{detail:3}])assert.equal(isEditDoubleClick(first,{...second,...patch}),false);assert.equal(isEditDoubleClick(null,second),false);});
