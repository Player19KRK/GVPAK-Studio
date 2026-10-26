import test from 'node:test';import assert from 'node:assert/strict';import {PreviewCache,previewKey,originalKey,normalizeEdit,outputGeometry} from '../src/preview-cache.js';
function cache(options={}){let next=0;const dropped=[];const c=new PreviewCache({...options,makeURL:()=>`url-${++next}`,dropURL:u=>dropped.push(u)});const bitmap=(w=10,h=10)=>({width:w,height:h,closed:false,close(){this.closed=true;}});return {c,dropped,bitmap};}
test('LRU evicts and closes old previews while retaining displayed entries',()=>{const {c,dropped,bitmap}=cache({maxEntries:2});const a=c.put('a',{size:10},bitmap()),b=c.put('b',{size:10},bitmap());c.retain(a);c.put('c',{size:10},bitmap());assert.equal(c.peek('b'),null);assert.equal(b.bitmap.closed,true);assert.equal(c.peek('a'),a);assert.deepEqual(dropped,['url-2']);c.release(a);});
test('encoded and decoded limits stay bounded across 40 images',()=>{const {c,bitmap}=cache({maxEntries:12,maxBlobBytes:100,maxDecodedBytes:1000});for(let i=0;i<40;i++)c.put(String(i),{size:20},bitmap());assert.ok(c.stats().blobBytes<=100);assert.ok(c.stats().decodedBytes<=1000);assert.ok(c.stats().entries<=12);assert.equal(c.stats().evictions,38);});
test('current pins cannot be revoked; releasing enables trim',()=>{const {c,bitmap}=cache({maxEntries:1});const a=c.put('a',{size:1},bitmap());c.retain(a);const b=c.put('b',{size:1},bitmap());assert.equal(a.bitmap.closed,false);c.retain(b);c.release(a);assert.equal(a.bitmap.closed,true);assert.equal(c.peek('b'),b);});
test('cache identity separates source, adopted version, directory and edit',()=>{const a={id:'a',sha256:'hash',width:100,height:100};const e={exposure:0,contrast:1,saturation:1};const key=previewKey('root','p',a,e);assert.notEqual(key,previewKey('other','p',a,e));assert.notEqual(key,previewKey('root','p',{...a,generatedBase:'v'},e));assert.notEqual(key,previewKey('root','p',a,{...e,exposure:.5}));assert.equal(key,previewKey('root','p',a,{saturation:1,contrast:1,exposure:0}));assert.equal(previewKey('root','p',a,{...e,contrast:1.2}),previewKey('root','p',a,{...e,contrast:Math.fround(1.2)}));assert.equal(originalKey('root','p',a),originalKey('root','p',{...a,generatedBase:'v'}));});
test('GPU geometry preserves crop then rotation sizing',()=>{const g=outputGeometry(1000,800,{crop:[.1,.2,.5,.5],rotation:90});assert.equal(g.width,400);assert.equal(g.height,500);assert.deepEqual(g.crop,[.1,.2,.5,.5]);});

test('duplicate inserts keep the displayed bitmap alive and refresh LRU order',()=>{
 const {c,bitmap,dropped}=cache({maxEntries:2});
 const original=bitmap(),a=c.put('a',{size:10},original);
 const b=c.put('b',{size:10},bitmap());
 assert.equal(c.put('a',{size:10},original),a);
 assert.equal(original.closed,false);
 const redundant=bitmap();
 assert.equal(c.put('a',{size:10},redundant),a);
 assert.equal(redundant.closed,true);
 c.put('c',{size:10},bitmap());
 assert.equal(c.peek('a'),a);
 assert.equal(c.peek('b'),null);
 assert.equal(b.bitmap.closed,true);
 assert.deepEqual(dropped,['url-2']);
});

test('URL allocation failure releases the decoded bitmap without adding an entry',()=>{
 const {c,bitmap}=cache();
 c.makeURL=()=>{throw Error('URL allocation failed');};
 const decoded=bitmap();
 assert.throws(()=>c.put('a',{size:10},decoded),/URL allocation failed/);
 assert.equal(decoded.closed,true);
 assert.equal(c.stats().entries,0);
});

test('releasing a pin evicts multiple entries while preserving the other displayed image',()=>{
 const {c,bitmap}=cache({maxEntries:100,maxBlobBytes:10000,maxDecodedBytes:100000});
 const entries=Array.from({length:20},(_,i)=>c.retain(c.put(String(i),{size:10},bitmap())));
 c.maxEntries=1;
 for(const entry of entries.slice(0,-1))c.release(entry);
 assert.equal(c.stats().entries,1);
 assert.equal(c.stats().blobBytes,10);
 assert.equal(c.stats().decodedBytes,400);
 assert.equal(c.stats().evictions,19);
 assert.equal(entries.at(-1).bitmap.closed,false);
 assert.ok(entries.slice(0,-1).every(entry=>entry.bitmap.closed));
});
