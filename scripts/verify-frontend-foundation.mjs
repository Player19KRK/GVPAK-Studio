import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
import {deflateSync,crc32} from 'node:zlib';

// Use an already installed browser runtime; this script does not install packages.
const runtime=process.argv[2]||process.env.PLAYWRIGHT_MODULE;
if(!runtime)throw Error('Pass the existing playwright package entry as the first argument.');
const {chromium}=await import(pathToFileURL(path.resolve(runtime)).href);
const base=process.env.FRONTEND_QA_URL||'http://127.0.0.1:5173';
const output='output/playwright/frontend-foundation';
await fs.mkdir(output,{recursive:true});
await fs.writeFile('output/playwright/frontend-foundation.html',`<!doctype html><html lang="zh-CN"><meta charset="utf-8"><div id="app"></div><script type="module">import {mount} from 'svelte';import UI from '/scripts/fixtures/FoundationUI.svelte';import '/src/style.css';import '/src/styles/tokens.css';import '/src/styles/base.css';mount(UI,{target:document.getElementById('app')});</script></html>`);
const edit={adjustmentOpacity:1,adjustmentEnabled:true,exposure:0,contrast:1,saturation:1,temperature:0,rotation:0,flipH:false,flipV:false,crop:null,strokes:[]};
const projects=[{id:'qa-a',name:'验收项目 A',assets:['image-a','image-b'].map((id,i)=>({id,name:`验收图片 ${i+1}.png`,width:1,height:1,bytes:68,sha256:id,trashed:false,edit,versions:[]}))},{id:'qa-b',name:'验收项目 B',assets:[]}];
function chunk(type,data){const bytes=Buffer.concat([Buffer.from(type),data]);const length=Buffer.alloc(4),crc=Buffer.alloc(4);length.writeUInt32BE(data.length);crc.writeUInt32BE(crc32(bytes));return Buffer.concat([length,bytes,crc]);}
const header=Buffer.alloc(13);header.writeUInt32BE(1,0);header.writeUInt32BE(1,4);header[8]=8;header[9]=6;
const png=Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',header),chunk('IDAT',deflateSync(Buffer.from([0,232,235,230,255]))),chunk('IEND',Buffer.alloc(0))]);
let mode='normal';
const commands=[];
async function fixtures(context,team=false){
  await context.route('**/api/**',async route=>{
    const url=new URL(route.request().url());
    const json=(value,status=200)=>route.fulfill({status,contentType:'application/json',body:JSON.stringify(value)});
    if(url.pathname==='/api/team/session')return json(team?{team:true,user:{id:'qa-team',username:'同事'}}:{team:false});
    if(url.pathname==='/api/events')return route.fulfill({contentType:'text/event-stream',body:': test stream\n\n'});
    const command=url.pathname.split('/').at(-1);commands.push(command);
    if(command==='bootstrap')return mode==='error'?json({error:'服务暂时不可用，请重试。'},503):json({projects,dataRoot:'qa-fixture',warnings:[]});
    if(command==='model_catalog')return json({catalog:{models:[]}});
    if(command==='hybrid_jobs')return json({jobs:[],notices:[]});
    if(['thumbnail','preview_source','preview','original_preview'].includes(command))return route.fulfill({contentType:'application/octet-stream',body:png});
    throw Error(`Unexpected API request during foundation QA: ${command}`);
  });
}
const browser=await chromium.launch({headless:true,...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE?{executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE}:{})});
const report={checks:[],screenshots:[],pageErrors:[],metrics:[]};
try{
  const context=await browser.newContext({viewport:{width:1280,height:900},colorScheme:'light'});await fixtures(context);
  const page=await context.newPage();page.on('pageerror',error=>report.pageErrors.push(error.message));
  await page.goto(base);await page.getByRole('heading',{name:'今天要处理什么？'}).waitFor();
  const light=await page.locator('.foundation-shell').evaluate(node=>getComputedStyle(node).backgroundColor);
  await page.emulateMedia({colorScheme:'dark'});
  const dark=await page.locator('.foundation-shell').evaluate(node=>getComputedStyle(node).backgroundColor);
  assert.notEqual(light,dark);report.checks.push('OS color scheme changes immediately without reload');
  for(const width of [1280,1920]){
    await page.setViewportSize({width,height:900});
    for(const colorScheme of ['light','dark']){
      await page.emulateMedia({colorScheme});
      for(const [name,url] of [['home','/'],['library','/library'],['editor','/editor/image-a'],['settings','/settings']]){
        await page.goto(base+url);await page.locator('.foundation-shell').waitFor();
        if(name==='editor')await page.locator('.cf-root').waitFor();else await page.getByRole('heading').first().waitFor();
        const metrics=await page.evaluate(()=>({viewport:innerWidth,scroll:document.documentElement.scrollWidth,header:document.querySelector('.shell-header').getBoundingClientRect().height,chat:document.querySelector('.shell-chat')?.getBoundingClientRect().width||0}));
        assert.equal(metrics.scroll,metrics.viewport,`${name} overflow at ${width} / ${colorScheme}`);assert.equal(metrics.header,48);
        if(['library','editor'].includes(name))assert.equal(metrics.chat,400);
        const file=`${output}/${name}-${width}-${colorScheme}.png`;await page.screenshot({path:file});report.screenshots.push(file);report.metrics.push({name,width,colorScheme,...metrics});
      }
    }
  }
  report.checks.push('Four route views at 1280/1920 and light/dark: no document overflow; header 48px; default chat 400px');
  await page.goto(base+'/library?folder=folder-a&tag=礼盒&status=done&q=包装&sort=name');
  await page.getByRole('separator',{name:'对话栏宽度'}).focus();await page.keyboard.press('ArrowLeft');
  await page.getByRole('separator',{name:'文件夹树宽度'}).focus();await page.keyboard.press('ArrowRight');
  await page.getByRole('button',{name:'收起对话栏'}).click();
  await page.getByLabel('当前项目').selectOption('qa-b');
  await page.reload();await page.getByLabel('当前项目').waitFor();
  await page.waitForFunction(()=>document.querySelector('[aria-label="当前项目"]').value==='qa-b');
  assert.equal(await page.locator('.shell-chat').count(),0);
  await page.getByRole('button',{name:'展开对话栏'}).first().click();
  assert.equal(await page.getByRole('separator',{name:'对话栏宽度'}).getAttribute('aria-valuenow'),'410');
  assert.equal(await page.getByRole('separator',{name:'文件夹树宽度'}).getAttribute('aria-valuenow'),'230');
  const filterUrl=page.url();await page.getByRole('link',{name:'首页',exact:true}).click();await page.getByRole('link',{name:'素材库',exact:true}).click();assert.equal(page.url(),filterUrl);
  await page.goBack();assert.equal(new URL(page.url()).pathname,'/');
  await page.goForward();assert.equal(page.url(),filterUrl);
  report.checks.push('Project, filters, chat width/collapse and tree width survive refresh; back/forward and nav preserve filters');
  await page.goto(base+'/editor/image-a');await page.locator('.cf-root').waitFor();
  await page.locator('.cf-next').click();await page.waitForURL('**/editor/image-b');
  await page.reload();await page.locator('.cf-current').waitFor();assert.match(await page.locator('.cf-current').getAttribute('aria-label'),/2/);
  assert.equal(await page.getByRole('button',{name:'历史版本',exact:true}).isDisabled(),true);
  await page.locator('.cf-project').click();await page.waitForURL('**/library');
  report.checks.push('Editor deep link and current image navigation remain synchronized');
  await page.goto(base+'/unknown');await page.getByRole('heading',{name:'找不到这个页面'}).waitFor();
  mode='error';await page.goto(base);await page.getByRole('heading',{name:'工作区暂时无法打开'}).waitFor();
  mode='normal';await page.getByRole('button',{name:'重试',exact:true}).click();await page.getByRole('heading',{name:'今天要处理什么？'}).waitFor();
  report.checks.push('Unknown route and recoverable service error render inline without dialogs');
  await page.goto(base+'/output/playwright/frontend-foundation.html');await page.getByRole('button',{name:'操作 0',exact:true}).waitFor();
  await page.getByRole('button',{name:'操作 0',exact:true}).click();await page.getByRole('button',{name:'操作 1',exact:true}).waitFor();
  await page.getByRole('button',{name:'列表',exact:true}).click();assert.equal(await page.getByRole('button',{name:'列表',exact:true}).getAttribute('aria-pressed'),'true');
  await page.getByRole('separator',{name:'测试宽度'}).focus();await page.keyboard.press('End');assert.equal(await page.getByLabel('当前宽度').textContent(),'520');
  await page.keyboard.press('Home');assert.equal(await page.getByLabel('当前宽度').textContent(),'360');
  await page.getByRole('separator',{name:'测试宽度'}).dblclick();assert.equal(await page.getByLabel('当前宽度').textContent(),'400');
  assert.equal(await page.getByRole('button',{name:'设置',exact:true}).getAttribute('title'),'设置 · Ctrl+,');
  const controls=page.locator('button:not(:disabled),[role="separator"]');
  await page.keyboard.press('Tab');
  for(let i=0;i<await controls.count();i++){
    await controls.nth(i).focus();const outline=await controls.nth(i).evaluate(node=>{const s=getComputedStyle(node);return {width:s.outlineWidth,style:s.outlineStyle};});assert.equal(outline.width,'2px');assert.equal(outline.style,'solid');
  }
  await page.emulateMedia({colorScheme:'dark',reducedMotion:'reduce'});
  for(const animation of await page.locator('.ui-spinner svg').evaluateAll(nodes=>nodes.map(node=>getComputedStyle(node).animationName)))assert.equal(animation,'none');
  const gallery=`${output}/ui-components-dark.png`;await page.screenshot({path:gallery});report.screenshots.push(gallery);
  report.checks.push('Button/Segmented events, disabled/loading, Tooltip title, Resizer keyboard/reset, 2px focus-visible, reduced-motion');
  const remote=await browser.newContext();await fixtures(remote,true);const remotePage=await remote.newPage();
  await remotePage.goto(base+'/settings');await remotePage.waitForURL(base+'/');assert.equal(await remotePage.getByRole('button',{name:'设置',exact:true}).count(),0);
  report.checks.push('Team session cannot enter settings or see the settings control');
  assert.deepEqual(report.pageErrors,[]);report.commands=[...new Set(commands)];
  report.status='passed';console.log(JSON.stringify(report,null,2));
}catch(error){report.status='failed';report.error=error.stack;throw error;}
finally{await fs.writeFile(`${output}/report.json`,JSON.stringify(report,null,2));await browser.close();}
