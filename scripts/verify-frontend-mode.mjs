import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
const {chromium}=await import(pathToFileURL(path.resolve(process.argv[2]||process.env.PLAYWRIGHT_MODULE)).href);
const root=path.resolve('output/playwright/frontend-build-mock'),requests=[];
const server=http.createServer(async(req,res)=>{
 try{
  const url=new URL(req.url,'http://localhost');if(url.pathname.startsWith('/api/'))requests.push(url.pathname);
  const file=path.resolve(root,'.'+(url.pathname==='/'?'/index.html':decodeURIComponent(url.pathname)));
  if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}
  const bytes=await fs.readFile(file);res.setHeader('Content-Type',({'.html':'text/html','.js':'text/javascript','.css':'text/css','.woff':'font/woff','.woff2':'font/woff2'})[path.extname(file)]||'application/octet-stream');res.end(bytes);
 }catch{res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await chromium.launch({headless:true,...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE?{executablePath:process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE}:{})});
const report={status:'running',errors:[],requests};
try{
 const page=await browser.newPage();page.on('pageerror',error=>report.errors.push(error.message));
 await page.goto(`http://127.0.0.1:${server.address().port}/`);await page.getByRole('heading',{name:'今天要处理什么？'}).waitFor();
 assert.deepEqual(requests,[]);assert.deepEqual(report.errors,[]);report.status='passed';console.log('Environment-selected mock build boots without API requests.');
}catch(error){report.status='failed';report.error=error.stack;throw error;}
finally{await fs.mkdir('output/playwright/frontend-data',{recursive:true});await fs.writeFile('output/playwright/frontend-data/mode-report.json',JSON.stringify(report,null,2));await browser.close();server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
