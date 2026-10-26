import test from 'node:test';
import assert from 'node:assert/strict';
import {flushSync} from 'svelte';
import {harness} from './studio-test-harness.mjs';

test('generate click submits directly after saving a dirty draft',async t=>{
 const h=await harness(t),ui=await h.generate();
 h.state.change({exposure:.75});flushSync();
 await ui.generate();
 const request=h.calls.find(c=>c.command==='generate');
 assert.ok(request);assert.equal(request.args.approved,true);
 assert.equal(h.state.studio.error,'');
});
test('latest prompt can generate directly without a separate approval state',async t=>{
 const h=await harness(t),ui=await h.generate();
 ui.setPrompt('Different request');flushSync();await ui.generate();
 const request=h.calls.find(c=>c.command==='generate');
 assert.equal(request.args.prompt,'Different request');assert.equal(request.args.approved,true);
});

test('new workspaces display CNY by default',async t=>{
 const h=await harness(t);assert.equal(h.state.studio.currency,'CNY');
});
test('refresh reloads a changed image thumbnail but reuses unaffected images',async t=>{
 const h=await harness(t);await h.state.thumbnails();
 const before={...h.state.studio.thumbs};
 const updated=structuredClone(h.project);updated.assets[0].generatedBase='local-result';updated.assets[0].edit.exposure=.5;
 h.setBackend(updated);await h.state.refresh();await h.state.thumbnails();
 assert.notEqual(h.state.studio.thumbs['0'],before['0']);
 assert.equal(h.state.studio.thumbs['1'],before['1']);
 assert.equal(h.calls.filter(c=>c.command==='thumbnail').length,3);
});


test('text generation omits the current reference and leaves dirty edits untouched',async t=>{
 const h=await harness(t),ui=await h.generate();h.state.change({exposure:.75});ui.setSource('text');flushSync();await ui.generate();
 const request=h.calls.find(c=>c.command==='generate');assert.equal(request.args.assetId,'');assert.equal(h.calls.filter(c=>c.command==='edit_commit').length,0);assert.equal(h.state.studio.error,'');
});
test('pasting plain text is left to the prompt field',async t=>{
 const h=await harness(t),ui=await h.generate();h.state.studio.mode='generate';let prevented=false;
 await ui.pasteReference({clipboardData:{files:[]},preventDefault(){prevented=true;}});assert.equal(prevented,false);
});
