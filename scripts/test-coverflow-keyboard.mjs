import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createQuickSelection} from '../src/coverflow-selection.js';

// Exercise the component's actual capture handler, including its routing guards.
const source=fs.readFileSync(new URL('../src/components/Coverflow.svelte',import.meta.url),'utf8');
const handler=source.slice(source.indexOf('function keyDown('),source.indexOf('function keyUp('));
function setup(){
 let current='b',available=true;const selected=new Set(),visits=[];
 const quick=createQuickSelection({getIds:()=>['a','b','c','d'],getCurrent:()=>current,isSelected:id=>selected.has(id),setSelected:(ids,on)=>ids.forEach(id=>on?selected.add(id):selected.delete(id)),navigate:id=>{current=id;visits.push(id);}});
 const keyDown=new Function('quick','available','release',handler+';return keyDown;')(quick,()=>available,()=>quick.release());
 function send(code,options={}){const event={code,key:code,repeat:false,prevented:false,stopped:false,preventDefault(){this.prevented=true;},stopPropagation(){this.stopped=true;},...options};keyDown(event);return event;}
 return {quick,selected,visits,send,unavailable(){available=false;}};
}
test('Space plus arrow auto-repeat is consumed once; ordinary arrows remain global navigation',()=>{
 const s=setup();assert.equal(s.send('ArrowRight').stopped,false);
 s.send('Space');const first=s.send('ArrowRight'),repeat=s.send('ArrowRight',{repeat:true});
 assert.ok(first.prevented&&first.stopped&&repeat.prevented&&repeat.stopped);
 assert.deepEqual(s.visits,['c','d']);assert.deepEqual([...s.selected],['b','c','d']);
 assert.equal(s.send('ArrowRight',{repeat:true}).stopped,true);assert.equal(s.visits.length,2);
 s.quick.release();assert.equal(s.send('ArrowLeft').stopped,false);
});
test('typing/busy/dialog guards end arrow selection and modifier shortcuts are not consumed',()=>{
 const s=setup();s.send('Space');assert.equal(s.send('ArrowRight',{ctrlKey:true}).stopped,false);
 assert.equal(s.send('ArrowRight',{isComposing:true}).stopped,false);
 s.unavailable();assert.equal(s.send('ArrowRight').stopped,false);assert.equal(s.quick.held,false);assert.deepEqual(s.visits,[]);
});
