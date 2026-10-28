import test from 'node:test';import assert from 'node:assert/strict';import {homeContext} from '../src/lib/home.ts';
const input={assets:[{id:'a',name:'C:/source/a.png',folderId:'新品'},{id:'b',name:'b.png',folderId:'其他'}],mentions:[],selected:[],filter:{folder:'',tag:'',status:'',q:'',sort:''}};
test('home explicit folder request narrows filtered target and never sends paths in filenames',()=>{const context=homeContext('把 新品/ 的图做白底',input);assert.deepEqual(context.resolvedTarget,{source:'filter',ids:['a']});assert.deepEqual(context.fileNames,['a.png']);assert.equal(context.page,'home');});
test('home respects explicit mentions and selected target priority',()=>{assert.deepEqual(homeContext('新品/',{...input,selected:['b']}).resolvedTarget,{source:'selected',ids:['b']});assert.deepEqual(homeContext('新品/',{...input,selected:['b'],mentions:['a']}).resolvedTarget,{source:'mention',ids:['a']});});

test('explicit targets remain visible when a previous folder filter hides them',()=>{const context=homeContext('/白底',{...input,mentions:['a'],filter:{...input.filter,folder:'其他'}});assert.equal(context.filter.folder,'');assert.deepEqual(context.resolvedTarget.ids,['a']);});
