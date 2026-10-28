import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
import {deflateSync,crc32} from 'node:zlib';
const {chromium}=await import(pathToFileURL(path.resolve(process.argv[2]||process.env.PLAYWRIGHT_MODULE)).href);
const browser=await chromium.launch({headless:true,...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE?{executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE}:{})});
const output='output/playwright/frontend-shortcuts';await fs.mkdir(output,{recursive:true});
const report={checks:[],pageErrors:[],writes:[]};
const edit={adjustmentOpacity:1,adjustmentEnabled:true,exposure:0,contrast:1,saturation:1,temperature:0,rotation:0,flipH:false,flipV:false,crop:null,strokes:[]};
const projects=[{id:'keys',name:'快捷键验收',assets:['a','b','c'].map(id=>({id,name:`${id}.png`,width:1,height:1,bytes:70,sha256:id,trashed:false,edit,versions:[]}))}];
function chunk(type,data){const bytes=Buffer.concat([Buffer.from(type),data]),length=Buffer.alloc(4),crc=Buffer.alloc(4);length.writeUInt32BE(data.length);crc.writeUInt32BE(crc32(bytes));return Buffer.concat([length,bytes,crc]);}
const header=Buffer.alloc(13);header.writeUInt32BE(1,0);header.writeUInt32BE(1,4);header[8]=8;header[9]=6;
const png=Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',header),chunk('IDAT',deflateSync(Buffer.from([0,232,235,230,255]))),chunk('IEND',Buffer.alloc(0))]);
let jobState=null;
try{
 const context=await browser.newContext({viewport:{width:1280,height:900}});
 await context.route('**/api/**',async route=>{
  const url=new URL(route.request().url()),json=value=>route.fulfill({contentType:'application/json',body:JSON.stringify(value)});
  if(url.pathname==='/api/team/session')return json({team:false});
  if(url.pathname==='/api/events')return route.fulfill({contentType:'text/event-stream',body:': test\n\n'});
  const command=url.pathname.split('/').at(-1);
  if(command==='bootstrap')return json({projects,dataRoot:'keys-qa',warnings:[]});
  if(command==='model_catalog')return json({catalog:{models:[]}});
  if(command==='hybrid_jobs')return json({jobs:jobState?[{id:'qa-job',state:jobState,items:[{stage:'pending'}]}]:[],notices:[]});
  if(['thumbnail','preview_source','preview','original_preview'].includes(command))return route.fulfill({contentType:'application/octet-stream',body:png});
  report.writes.push({command,args:route.request().postDataJSON()});
  if(command==='hybrid_cancel'){jobState='cancelled';return json(true);}
  throw Error(`Unexpected request: ${command}`);
 });
 const page=await context.newPage();page.on('pageerror',error=>report.pageErrors.push(error.message));
 const base=process.env.FRONTEND_QA_URL||'http://127.0.0.1:5173';
 await page.goto(base+'/editor/a');await page.locator('.cf-root').waitFor();
 await page.evaluate(async()=>{
  const load=file=>{const canonical=file.endsWith('.svelte')?file+'.js':file;const url=performance.getEntriesByType('resource').find(entry=>new URL(entry.name).pathname===canonical)?.name||canonical;return import(url);};
  const [{studio,select,change},{selection},{editor},{assets},{tasks,setTasks},{getApi},{shortcutScope},{navigate}]=await Promise.all([load('/src/studio.svelte'),load('/src/stores/selection.svelte'),load('/src/stores/editor.svelte'),load('/src/stores/assets.svelte'),load('/src/stores/tasks.svelte'),load('/src/services/api.ts'),load('/src/lib/shortcuts.ts'),load('/src/app/shell.svelte')]);
  window.__keysQA={studio,select,change,selection,editor,assets,tasks,setTasks,getApi,shortcutScope,navigate};
 });
 await page.mouse.move(10,10);await page.keyboard.press('Control+k');await page.getByRole('textbox',{name:'处理要求'}).waitFor();
 assert.equal(await page.getByRole('textbox',{name:'处理要求'}).evaluate(node=>node===document.activeElement),true);
 await page.keyboard.type('12345R');await page.keyboard.press('Delete');await page.keyboard.press('ArrowLeft');await page.keyboard.press('ArrowRight');
 const inputState=await page.evaluate(()=>({aid:window.__keysQA.studio.aid,selected:[...window.__keysQA.selection.selectedIds],ratio:{...window.__keysQA.editor.ratio},compare:window.__keysQA.studio.compare,error:window.__keysQA.studio.error}));
 assert.equal(inputState.aid,'a');assert.deepEqual(inputState.selected,[]);assert.equal(inputState.ratio.ratio,'1:1');assert.equal(inputState.compare,false);assert.equal(inputState.error,'');
 await page.keyboard.press('Control+j');assert.equal(await page.locator('#shell-chat').count(),0);await page.keyboard.press('Control+k');
 assert.equal(await page.getByRole('textbox',{name:'处理要求'}).evaluate(node=>node===document.activeElement),true);assert.equal(await page.getByRole('textbox',{name:'处理要求'}).inputValue(),'12345R');
 report.checks.push('Ctrl+K focuses/reopens composer; Ctrl+J toggles once and preserves draft; textarea 1–5/R/Delete/arrows have no app side effects');
 await page.evaluate(()=>{
  const fixtures=document.createElement('div');fixtures.id='key-input-fixtures';fixtures.innerHTML='<input aria-label="test-input"><select aria-label="test-select"><option>A</option></select><div contenteditable="true" aria-label="test-editable"><span>text</span></div><div role="textbox" tabindex="0" aria-label="test-role">text</div><div id="shadow-input"></div>';
  document.querySelector('.shell-composer').append(fixtures);const shadow=fixtures.querySelector('#shadow-input').attachShadow({mode:'open'});shadow.innerHTML='<input aria-label="test-shadow">';
 });
 for(const label of ['test-input','test-select','test-editable','test-role','test-shadow']){
  await page.getByLabel(label).focus();for(const key of ['1','2','3','4','5','r','Delete','Space','ArrowLeft','ArrowRight'])await page.keyboard.press(key);
  assert.equal(await page.evaluate(()=>window.__keysQA.studio.error),'');assert.equal(await page.evaluate(()=>window.__keysQA.studio.aid),'a');assert.equal(await page.evaluate(()=>window.__keysQA.studio.compare),false);
 }
 await page.evaluate(()=>document.querySelector('#key-input-fixtures').remove());report.checks.push('input/select/contenteditable/role=textbox and shadow inputs suppress page shortcuts');
 await page.mouse.move(10,10);await page.locator('.canvas-stage').focus();await page.keyboard.press('2');await page.waitForFunction(()=>window.__keysQA.studio.error.includes('operation_execute'));
 await page.evaluate(()=>{window.__keysQA.studio.error='';window.__keysQA.editor.ratio={ratio:'16:9',orientation:'landscape'};});await page.locator('.cf-current .cf-image-button').focus();await page.keyboard.press('r');await page.waitForFunction(()=>window.__keysQA.studio.error.includes('operation_execute'));
 await page.evaluate(()=>{window.__keysQA.studio.error='';window.__keysQA.editor.ratio={ratio:'1:1',orientation:'landscape'};});report.checks.push('Editor ratio keys route to the services layer; nonconflicting R works under coverflow focus; missing real capability stays an explicit error');
 await page.mouse.move(10,10);await page.locator('.canvas-stage').focus();
 await page.keyboard.down('Space');assert.equal(await page.evaluate(()=>window.__keysQA.studio.compare),true);await page.keyboard.down('Space');await page.keyboard.down('Backslash');
 await page.keyboard.up('Space');assert.equal(await page.evaluate(()=>window.__keysQA.studio.compare),true);await page.keyboard.up('Backslash');assert.equal(await page.evaluate(()=>window.__keysQA.studio.compare),false);
 await page.keyboard.down('Space');await page.evaluate(()=>window.dispatchEvent(new Event('blur')));assert.equal(await page.evaluate(()=>window.__keysQA.studio.compare),false);await page.keyboard.up('Space');
 await page.locator('.canvas-stage').focus();await page.keyboard.down('Space');await page.keyboard.press('Control+k');assert.equal(await page.evaluate(()=>window.__keysQA.studio.compare),false);await page.keyboard.up('Space');
 report.checks.push('Space/Backslash comparison handles repeat, overlapping holds, release, blur and input focus without getting stuck');
 await page.mouse.move(10,10);await page.locator('.cf-current .cf-image-button').focus();await page.keyboard.press('Space');
 assert.equal(await page.evaluate(()=>window.__keysQA.studio.compare),false);assert.deepEqual(await page.evaluate(()=>[...window.__keysQA.selection.selectedIds]),[]);
 await page.locator('.cf-root').hover();await page.keyboard.down('Space');assert.deepEqual(await page.evaluate(()=>[...window.__keysQA.selection.selectedIds]),['a']);
 await page.keyboard.down('ArrowRight');assert.equal(await page.evaluate(()=>window.__keysQA.studio.aid),'b');await page.keyboard.down('ArrowRight');assert.equal(await page.evaluate(()=>window.__keysQA.studio.aid),'c');await page.keyboard.up('ArrowRight');
 await page.keyboard.press('ArrowLeft');assert.equal(await page.evaluate(()=>window.__keysQA.studio.aid),'b');assert.deepEqual(await page.evaluate(()=>[...window.__keysQA.selection.selectedIds]),['a','b','c']);assert.equal(await page.evaluate(()=>window.__keysQA.studio.compare),false);
 await page.keyboard.up('Space');await page.keyboard.press('ArrowLeft');assert.equal(await page.evaluate(()=>window.__keysQA.studio.aid),'a');
 await page.keyboard.down('Space');await page.keyboard.press('Escape');await page.keyboard.up('Space');assert.equal(await page.locator('.cf-root').evaluate(node=>node.classList.contains('quick-selecting')),false);
 report.checks.push('Original coverflow focus/hover semantics, Space selection, held repeated arrows/reversal, ordinary arrows and Escape remain intact with no double step');
 await page.mouse.move(10,10);const separator=page.getByRole('separator',{name:'图片浏览区高度'});await separator.focus();const height=Number(await separator.getAttribute('aria-valuenow'));await page.keyboard.press('ArrowUp');assert.equal(Number(await separator.getAttribute('aria-valuenow')),height+10);await page.keyboard.press('Home');assert.equal(Number(await separator.getAttribute('aria-valuenow')),100);
 await page.getByRole('separator',{name:'对话栏宽度'}).focus();const width=Number(await page.getByRole('separator',{name:'对话栏宽度'}).getAttribute('aria-valuenow'));await page.keyboard.press('ArrowLeft');assert.equal(Number(await page.getByRole('separator',{name:'对话栏宽度'}).getAttribute('aria-valuenow')),width+10);
 report.checks.push('Frozen height separator and centrally registered new width separator each handle their native keys once');
 await page.evaluate(()=>{const q=window.__keysQA;q.scopeHits=[];q.scopeNode=document.createElement('div');document.body.append(q.scopeNode);q.scope1=q.shortcutScope(q.scopeNode,{name:'first',bindings:[{key:'Escape',action:()=>q.scopeHits.push('first')}]});q.scope2=q.shortcutScope(q.scopeNode,{name:'second',bindings:[{key:'Escape',action:()=>q.scopeHits.push('second')}]});});
 await page.keyboard.press('Escape');assert.deepEqual(await page.evaluate(()=>window.__keysQA.scopeHits),['second']);await page.evaluate(()=>window.__keysQA.scope2.destroy());await page.keyboard.press('Escape');assert.deepEqual(await page.evaluate(()=>window.__keysQA.scopeHits),['second','first']);await page.evaluate(()=>{window.__keysQA.scope1.destroy();window.__keysQA.scopeNode.remove();});
 report.checks.push('Real DOM scope stack routes Escape to the most recently mounted scope');
 await page.evaluate(()=>window.__keysQA.navigate('/library'));await page.getByRole('separator',{name:'文件夹树宽度'}).waitFor();
 await page.evaluate(()=>{window.__keysQA.assets.setFilter({q:'b'});document.querySelector('.shell-main').tabIndex=-1;document.querySelector('.shell-main').focus();});
 await page.keyboard.press('Control+a');assert.deepEqual(await page.evaluate(()=>[...window.__keysQA.selection.selectedIds]),['b']);
 await page.keyboard.press('Control+k');await page.keyboard.press('Delete');assert.equal(await page.evaluate(()=>window.__keysQA.studio.error),'');assert.deepEqual(await page.evaluate(()=>[...window.__keysQA.selection.selectedIds]),['b']);
 await page.evaluate(()=>document.querySelector('.shell-main').focus());await page.keyboard.press('Delete');await page.waitForFunction(()=>window.__keysQA.studio.error.includes('operation_execute'));await page.evaluate(()=>window.__keysQA.studio.error='');
 await page.keyboard.press('Enter');await page.waitForURL('**/editor/b');report.checks.push('Library Ctrl+A selects the current filter only; Enter opens the selected asset and updates URL');
 await page.keyboard.press('Control+k');await page.evaluate(()=>{const q=window.__keysQA;q.change({exposure:.1});});
 await page.keyboard.press('Control+z');await page.keyboard.press('Control+Shift+z');assert.equal(await page.evaluate(()=>window.__keysQA.studio.history.b.length),1);
 await page.mouse.move(10,10);await page.locator('.canvas-stage').focus();await page.keyboard.press('Control+z');assert.equal(await page.evaluate(()=>window.__keysQA.studio.history.b.length),0);
 await page.keyboard.press('Control+Shift+z');assert.equal(await page.evaluate(()=>window.__keysQA.studio.history.b.length),1);report.checks.push('Native text undo/redo remains isolated; existing editor draft undo/redo is still available outside inputs');
 jobState='preparing';await page.evaluate(async()=>{const q=window.__keysQA;q.setTasks(await q.getApi().listTasks());});
 await page.keyboard.press('Control+k');await page.keyboard.press('Escape');await page.waitForFunction(()=>window.__keysQA.tasks.byId['qa-job']?.state==='cancelled');assert.deepEqual(report.writes.map(item=>item.command),['hybrid_cancel']);assert.equal(report.writes[0].args.jobId,'qa-job');report.checks.push('Escape cancels the known active task by its ID through the existing service adapter even with input focus');
 assert.deepEqual(report.pageErrors,[]);report.status='passed';console.log(JSON.stringify(report,null,2));
}catch(error){report.status='failed';report.error=error.stack;throw error;}
finally{await fs.writeFile(`${output}/report.json`,JSON.stringify(report,null,2));await browser.close();}
