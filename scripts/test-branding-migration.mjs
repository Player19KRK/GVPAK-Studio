import test from 'node:test';
import assert from 'node:assert/strict';
import {readLocalSetting} from '../src/branding-migration.js';

test('brand migration preserves browser drafts and keeps newer settings',()=>{
 const values=new Map([['edge-draft',JSON.stringify({exposure:.5})]]);
 const storage={getItem:k=>values.get(k)??null,setItem:(k,v)=>values.set(k,v),removeItem:k=>values.delete(k)};
 assert.equal(JSON.parse(readLocalSetting(storage,'draft')).exposure,.5);
 assert.equal(values.has('edge-draft'),false);
 assert.equal(values.has('gvpak-studio-draft'),true);
 values.set('edge-draft','older');
 assert.equal(JSON.parse(readLocalSetting(storage,'draft')).exposure,.5);
 assert.equal(readLocalSetting(storage,'missing'),null);
});
