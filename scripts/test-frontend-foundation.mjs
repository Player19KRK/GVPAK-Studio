import test from 'node:test';
import assert from 'node:assert/strict';
import {normalizePreferences,readPreferences,writePreferences} from '../src/lib/preferences.ts';
import {parseRoute,libraryUrl,editorUrl,canOpenSettings} from '../src/lib/router.ts';

test('routes preserve Unicode asset IDs, filter parameters, and unknown pages',()=>{
  assert.equal(parseRoute('/').page,'home');
  const url=editorUrl('图片 /#?');assert.equal(parseRoute(url).assetId,'图片 /#?');
  assert.equal(parseRoute('/editor/%xx').page,'not-found');
  assert.equal(parseRoute('/editor').page,'not-found');
  assert.equal(parseRoute('/other').page,'not-found');
  const filter={folder:'folder-1',tag:'礼盒',status:'done',q:'A & B',sort:'name'};
  assert.deepEqual(parseRoute(libraryUrl(filter)).filter,filter);
});
test('settings requires both a host session and a loopback browser origin',()=>{
  for(const host of ['127.0.0.1','localhost','[::1]'])assert.equal(canOpenSettings({team:false},host),true);
  for(const session of [{},{team:true},{team:false}])assert.equal(canOpenSettings(session,'192.168.1.2'),false);
  assert.equal(canOpenSettings({team:true},'localhost'),false);
});
test('preferences restore only known data and clamp finite dimensions',()=>{
  const result=normalizePreferences({projectId:'p2',chatWidth:800,treeWidth:12,chatCollapsed:true,filter:{q:'图片'},credential:'must not be persisted'});
  assert.equal(result.chatWidth,520);assert.equal(result.treeWidth,180);assert.equal(result.chatCollapsed,true);
  assert.equal(result.projectId,'p2');assert.equal(result.filter.q,'图片');assert(!('credential' in result));
  assert.equal(normalizePreferences({chatWidth:NaN,treeWidth:Infinity}).chatWidth,400);
});
test('corrupt or unavailable browser storage does not block startup',()=>{
  assert.equal(readPreferences({getItem:()=>'{broken'},'key').chatWidth,400);
  assert.equal(readPreferences({getItem(){throw Error('blocked');}},'key').treeWidth,220);
  assert.equal(writePreferences({setItem(){throw Error('full');}},'key',{}),false);
  let saved;assert.equal(writePreferences({setItem(k,v){saved=JSON.parse(v);}},'key',{chatCollapsed:true}),true);
  assert.equal(saved.chatCollapsed,true);
});
