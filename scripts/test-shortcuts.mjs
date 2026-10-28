import test from 'node:test';
import assert from 'node:assert/strict';
import {createShortcutManager,shortcutKey,shortcutBindings,coverflowConflictTable,validateBindings} from '../src/lib/shortcuts.ts';
function key(value,options={}){
 const codes={' ':'Space','\\':'Backslash',r:'KeyR',R:'KeyR',z:'KeyZ',k:'KeyK',j:'KeyJ'};
 return {key:value,code:codes[value]||(/^\d$/.test(value)?'Digit'+value:value),ctrlKey:false,metaKey:false,altKey:false,shiftKey:false,repeat:false,defaultPrevented:false,isComposing:false,keyCode:0,preventDefault(){this.defaultPrevented=true;},stopPropagation(){this.stopped=true;},...options};
}
function setup(context={}){
 const calls=[],errors=[];let current={page:'editor',...context};
 const actions=Object.fromEntries(shortcutBindings.map(binding=>[binding.action,(event,phase)=>calls.push({action:binding.action,phase,code:event.code})]));
 const manager=createShortcutManager({getContext:()=>current,actions,onError:error=>errors.push(error)});
 return {manager,calls,errors,setContext(value){current={...current,...value};}};
}
test('conflict table matches every frozen key and documents the six nonconflicting editor keys',()=>{
 validateBindings(shortcutBindings);
 const keys=coverflowConflictTable.map(row=>row.key);assert.equal(keys.length,new Set(keys).size);
 for(const value of ['Space','ArrowLeft','ArrowRight','Escape','ArrowUp','ArrowDown','Home','End','Enter'])assert(keys.includes(value));
 for(const value of ['1','2','3','4','5','r'])assert.equal(coverflowConflictTable.find(row=>row.key===value).policy,'no-original-conflict');
 assert.throws(()=>validateBindings([{key:'r',action:'a'},{key:'r',action:'b'}]),/冲突/);
});
test('text focus blocks all page shortcuts and leaves native text undo/select-all untouched',()=>{
 const s=setup({inputFocused:true,page:'library'});
 for(const value of ['1','2','3','4','5','r','R','Delete',' ','ArrowLeft','ArrowRight','Enter'])assert.equal(s.manager.handleKeyDown(key(value)),null);
 for(const event of [key('z',{ctrlKey:true}),key('z',{ctrlKey:true,shiftKey:true}),key('a',{ctrlKey:true})]){assert.equal(s.manager.handleKeyDown(event),null);assert.equal(event.defaultPrevented,false);}
 for(const event of [key('k',{ctrlKey:true}),key('j',{ctrlKey:true}),key('Escape')])assert(s.manager.handleKeyDown(event));
 assert.deepEqual(s.calls.map(call=>call.action),['composer.focus','chat.toggle','task.escape']);
});
test('IME, AltGraph, consumed events and unexpected modifiers never trigger commands',()=>{
 const s=setup();
 for(const event of [key('r',{isComposing:true}),key('1',{keyCode:229}),key('r',{altKey:true}),key('r',{ctrlKey:true}),key('r',{getModifierState:()=>true}),key('r',{defaultPrevented:true})])assert.equal(s.manager.handleKeyDown(event),null);
 assert.deepEqual(s.calls,[]);assert.equal(shortcutKey(key('K',{ctrlKey:true})),'Mod+k');assert.equal(shortcutKey(key('z',{metaKey:true,shiftKey:true})),'Mod+Shift+z');
});
test('coverflow owns Space on hover/focus while unconsumed ordinary arrows call the original host once',()=>{
 const s=setup({coverflowFocused:true,nativeControl:true});
 assert.equal(s.manager.handleKeyDown(key(' ')),null);assert.equal(s.manager.handleKeyDown(key('Enter')),null);
 s.manager.handleKeyDown(key('ArrowRight'));assert.equal(s.calls.at(-1).action,'editor.next');assert.equal(s.calls.length,1);
 assert.equal(s.manager.handleKeyDown(key('ArrowRight',{defaultPrevented:true})),null);assert.equal(s.calls.length,1);
 s.manager.handleKeyDown(key('1'));s.manager.handleKeyDown(key('r'));assert.deepEqual(s.calls.slice(-2).map(call=>call.action),['editor.ratio.1','editor.ratio.swap']);
 s.setContext({coverflowFocused:false,nativeControl:false,coverflowHovered:true});assert.equal(s.manager.handleKeyDown(key(' ')),null);
 s.manager.handleKeyDown(key('Escape'));assert.equal(s.calls.at(-1).action,'task.escape');
});
test('separator/native button keys are not stolen by page handlers',()=>{
 const s=setup({separatorFocused:true,nativeControl:true});
 for(const value of ['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','Home','End',' ','Enter'])assert.equal(s.manager.handleKeyDown(key(value)),null);
 assert.deepEqual(s.calls,[]);
 s.setContext({page:'library',separatorFocused:false});assert.equal(s.manager.handleKeyDown(key('Enter')),null);
});
test('scope stack is LIFO, exclusive, removable out of order and returns to global Escape',()=>{
 const s=setup(),hits=[];
 const first=s.manager.pushScope('slash',[{key:'Escape',action:()=>hits.push('slash')}]);
 const second=s.manager.pushScope('mention',[{key:'Escape',action:()=>hits.push('mention')}]);
 assert.deepEqual(s.manager.scopes,['slash','mention']);s.manager.handleKeyDown(key('Escape'));assert.deepEqual(hits,['mention']);
 assert.equal(s.manager.handleKeyDown(key('1')),null);first();assert.deepEqual(s.manager.scopes,['mention']);
 second();s.manager.handleKeyDown(key('Escape'));assert.equal(s.calls.at(-1).action,'task.escape');
});
test('page binding wins over global and input Ctrl+K always remains the global focus command',()=>{
 const calls=[];const s=createShortcutManager({getContext:()=>({page:'editor'}),actions:{page:()=>calls.push('page'),global:()=>calls.push('global')}},[{key:'x',action:'global',scope:'global'},{key:'x',action:'page',scope:'editor'}]);
 s.handleKeyDown(key('x'));assert.deepEqual(calls,['page']);
 const text=setup({inputFocused:true});text.manager.pushScope('picker',[{key:'Mod+k',action:()=>calls.push('wrong')}]);text.manager.handleKeyDown(key('k',{ctrlKey:true}));assert.equal(text.calls[0].action,'composer.focus');assert(!calls.includes('wrong'));
});
test('long press fires once and releases despite context/focus changes',()=>{
 const s=setup();const down=key(' ');s.manager.handleKeyDown(down);s.manager.handleKeyDown(key(' ',{repeat:true}));
 assert.equal(s.manager.heldCount,1);assert.equal(s.calls.length,1);s.setContext({inputFocused:true});s.manager.handleKeyUp(key(' '));
 assert.deepEqual(s.calls.map(call=>call.phase),['press','release']);assert.equal(s.manager.heldCount,0);
 s.setContext({inputFocused:false});s.manager.handleKeyDown(key(' '));s.manager.releaseAll();s.manager.releaseAll();assert.equal(s.calls.length,4);
});
test('scope pushes/disposal release held actions and digit auto-repeat cannot submit repeatedly',()=>{
 const s=setup();s.manager.handleKeyDown(key(' '));s.manager.pushScope('picker',[]);assert.equal(s.calls.at(-1).phase,'release');
 s.manager.dispose();s.manager.handleKeyDown(key('1'));s.manager.handleKeyDown(key('1',{repeat:true}));assert.equal(s.calls.filter(call=>call.action==='editor.ratio.1').length,1);
 s.manager.handleKeyDown(key(' '));s.manager.dispose();assert.equal(s.manager.heldCount,0);
});
test('explicit input navigation scopes support pickers without enabling destructive single keys',()=>{
 const s=setup({inputFocused:true}),calls=[];
 s.manager.pushScope('mentions',[
  {key:'ArrowDown',action:()=>calls.push('next'),input:true},
  {key:'Enter',action:()=>calls.push('insert'),input:true},
  {key:'Delete',action:()=>calls.push('wrong'),input:true},
 ]);
 s.manager.handleKeyDown(key('ArrowDown'));s.manager.handleKeyDown(key('Enter'));s.manager.handleKeyDown(key('Delete'));
 assert.deepEqual(calls,['next','insert']);
});
test('busy state preserves only stop/focus/chat and rejected actions report errors',async()=>{
 const s=setup({busy:true});assert.equal(s.manager.handleKeyDown(key('1')),null);assert.equal(s.manager.handleKeyDown(key('z',{ctrlKey:true})),null);s.manager.handleKeyDown(key('Escape'));assert.equal(s.calls.length,1);
 const errors=[],manager=createShortcutManager({getContext:()=>({page:'editor'}),actions:{'editor.ratio.swap':async()=>{throw Error('unavailable');}},onError:error=>errors.push(error)});
 manager.handleKeyDown(key('r'));await new Promise(resolve=>setTimeout(resolve,0));assert.equal(errors[0].message,'unavailable');
});
