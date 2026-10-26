import test from 'node:test';
import assert from 'node:assert/strict';
import {flushSync} from 'svelte';
import {harness} from './studio-test-harness.mjs';
const wait=()=>new Promise(resolve=>setTimeout(resolve,0));

test('project library loads ten thumbnails once even with an overlapping caller',async t=>{
 const h=await harness(t,10);h.state.studio.view='projects';await h.watchProjects();
 await h.state.thumbnails(h.state.current());
 await new Promise(resolve=>setTimeout(resolve,100));
 assert.equal(Object.keys(h.state.studio.thumbs).length,10);
 assert.equal(h.calls.filter(c=>c.command==='thumbnail').length,10);
});

test('local sample is invalidated by a later draft; Apply preserves that draft',async t=>{
 const h=await harness(t),ui=await h.assistant();
 await ui.preview();flushSync();assert.ok(ui.getSample());
 h.state.change({exposure:.75});flushSync();
 assert.equal(ui.getSample(),null);
 await ui.apply();
 assert.equal(h.calls.filter(c=>c.command==='assistant_apply').length,0);
 assert.equal(h.state.studio.draft.exposure,.75);
 assert.equal(h.state.studio.drafts['0'].exposure,.75);
});

test('sample covers edits to every selected image, not just the visible image',async t=>{
 const h=await harness(t),ui=await h.assistant();
 await ui.preview();flushSync();assert.ok(ui.getSample());
 h.state.studio.drafts['1']={...h.initialEdit,contrast:1.4};flushSync();
 assert.equal(ui.getSample(),null);
 await ui.apply();
 assert.equal(h.calls.filter(c=>c.command==='assistant_apply').length,0);
 assert.equal(h.state.studio.drafts['1'].contrast,1.4);
});

test('unchanged sample can still be applied',async t=>{
 const h=await harness(t),ui=await h.assistant();
 await ui.preview();flushSync();await ui.apply();
 assert.equal(h.calls.filter(c=>c.command==='assistant_apply').length,1);
 assert.equal(h.state.studio.error,'');
});

test('applying selected images leaves the current unselected image draft intact',async t=>{
 const h=await harness(t),ui=await h.assistant();
 h.state.studio.selected=['1'];h.state.change({exposure:.9});flushSync();
 await ui.preview();flushSync();await ui.apply();
 assert.equal(h.calls.filter(c=>c.command==='assistant_apply').length,1);
 assert.equal(h.state.studio.draft.exposure,.9);
 assert.equal(h.state.studio.drafts['0'].exposure,.9);
});

test('Apply rejects a changed draft before the invalidation effect has flushed',async t=>{
 const h=await harness(t),ui=await h.assistant();
 await ui.preview();flushSync();
 h.state.change({exposure:.5});
 await ui.apply();
 assert.equal(h.calls.filter(c=>c.command==='assistant_apply').length,0);
 assert.equal(h.state.studio.draft.exposure,.5);
 assert.match(h.state.studio.error,/SAMPLE_STALE/);
});

test('Apply rechecks the sample after the output-folder dialog returns',async t=>{
 const h=await harness(t),ui=await h.assistant();
 ui.setPlan({summary:'Export',steps:[{op:'rotate',degrees:90}],warnings:[],clarification:null,output:{format:'webp',width:100,height:100,fit:'pad',maxBytes:10000,background:[255,255,255]}});
 flushSync();await ui.preview();flushSync();
 const applying=ui.apply();
 for(let i=0;i<100&&!h.pickerOpen();i++)await wait();
 assert.ok(h.pickerOpen());
 // A late state update can arrive while the native dialog is awaiting a response.
 h.state.studio.draft={...h.initialEdit,exposure:.8};flushSync();
 h.releasePicker();await applying;
 assert.equal(h.calls.filter(c=>c.command==='assistant_apply').length,0);
 assert.equal(h.state.studio.draft.exposure,.8);
});

test('a forced thumbnail refresh still fetches new bytes and shares overlapping work',async t=>{
 const h=await harness(t,3);
 await h.state.thumbnails();
 const before=h.state.studio.thumbs['0'];
 await Promise.all([h.state.thumbnails(h.state.current(),true,['0']),h.state.thumbnails(h.state.current(),false,['0'])]);
 assert.notEqual(h.state.studio.thumbs['0'],before);
 assert.equal(h.calls.filter(c=>c.command==='thumbnail').length,4);
});
