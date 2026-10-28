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

test('manual steps submit directly without a sample or application-confirmation request',async t=>{const h=await harness(t),ui=await h.assistant();await ui.apply();assert.equal(h.calls.filter(c=>c.command==='local_plan_submit').length,1);assert.equal(h.calls.filter(c=>['assistant_preview','assistant_apply'].includes(c.command)).length,0);assert.equal(h.state.studio.modal,'');});

test('a later draft remains intact while the current explicit plan is submitted',async t=>{const h=await harness(t),ui=await h.assistant();h.state.change({exposure:.75});flushSync();await ui.apply();assert.equal(h.calls.find(c=>c.command==='local_plan_submit').args.plan.steps[0].degrees,90);assert.equal(h.state.studio.draft.exposure,.75);assert.equal(h.state.studio.drafts['0'].exposure,.75);});

test('drafts on every selected image remain intact until authoritative execution results arrive',async t=>{const h=await harness(t),ui=await h.assistant();h.state.studio.drafts['1']={...h.initialEdit,contrast:1.4};flushSync();await ui.apply();const request=h.calls.find(c=>c.command==='local_plan_submit');assert.deepEqual(request.args.context.resolvedTarget.ids,['0','1']);assert.equal(h.state.studio.drafts['1'].contrast,1.4);});

test('selected-image submission leaves the current unselected draft untouched',async t=>{const h=await harness(t),ui=await h.assistant();h.state.studio.selected=['1'];h.state.change({exposure:.9});flushSync();await ui.apply();assert.deepEqual(h.calls.find(c=>c.command==='local_plan_submit').args.context.resolvedTarget.ids,['1']);assert.equal(h.state.studio.draft.exposure,.9);assert.equal(h.state.studio.drafts['0'].exposure,.9);});

test('the submitted plan is detached from subsequent step edits',async t=>{const h=await harness(t),ui=await h.assistant();const applying=ui.apply();ui.getPlan().steps[0].degrees=180;flushSync();await applying;assert.equal(h.calls.find(c=>c.command==='local_plan_submit').args.plan.steps[0].degrees,90);assert.equal(ui.getPlan().steps[0].degrees,180);});

test('export parameters submit with the plan and never wait on a native output dialog',async t=>{const h=await harness(t),ui=await h.assistant();ui.setPlan({summary:'导出',steps:[],warnings:[],clarification:null,output:{format:'webp',width:100,height:100,fit:'pad',maxBytes:10000,background:[255,255,255]}});flushSync();await ui.apply();assert.equal(h.calls.find(c=>c.command==='local_plan_submit').args.plan.output.maxBytes,10000);assert.equal(h.calls.filter(c=>c.command==='pick_output').length,0);assert.equal(h.pickerOpen(),false);});

test('a forced thumbnail refresh still fetches new bytes and shares overlapping work',async t=>{
 const h=await harness(t,3);
 await h.state.thumbnails();
 const before=h.state.studio.thumbs['0'];
 await Promise.all([h.state.thumbnails(h.state.current(),true,['0']),h.state.thumbnails(h.state.current(),false,['0'])]);
 assert.notEqual(h.state.studio.thumbs['0'],before);
 assert.equal(h.calls.filter(c=>c.command==='thumbnail').length,4);
});
