import test from 'node:test';
import assert from 'node:assert/strict';
import {harness} from './studio-test-harness.mjs';
test('first draft undo retains a reactive redo entry and redo restores the edit',async t=>{
 const {state}=await harness(t);
 state.change({exposure:.5});assert.equal(state.studio.history['0'].length,1);
 state.undo();assert.equal(state.studio.draft.exposure,0);assert.equal(state.studio.redo['0'].length,1);
 state.redo();assert.equal(state.studio.draft.exposure,.5);assert.equal(state.studio.history['0'].length,1);
});
