import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
const port=Number(process.env.GVPAK_NEW_PRODUCT_UI_PORT||8797),backend=Number(process.env.GVPAK_NEW_PRODUCT_BACKEND_PORT||8799);
const root=path.resolve(process.argv[2]||'.tools/new-product-trial/runtime/dist');
const allowedHost=new Set([`127.0.0.1:${port}`,`localhost:${port}`]),origins=new Set([`http://127.0.0.1:${port}`,`http://localhost:${port}`]);
const types={'.html':'text/html; charset=utf-8','.js':'text/javascript; charset=utf-8','.css':'text/css; charset=utf-8','.json':'application/json','.png':'image/png','.jpg':'image/jpeg','.webp':'image/webp','.woff':'font/woff','.woff2':'font/woff2'};
http.createServer(async(req,res)=>{
  if(!allowedHost.has(req.headers.host)||req.headers.origin&&!origins.has(req.headers.origin)){res.writeHead(403).end();return;}
  const url=new URL(req.url,`http://127.0.0.1:${port}`);
  if(url.pathname==='/__new-product-health'){res.setHeader('Content-Type','application/json');res.end(JSON.stringify({service:'gvpak-new-product-trial',backendPort:backend}));return;}
  if(url.pathname.startsWith('/api/')){
    if(process.env.GVPAK_TRIAL_NO_BACKEND==='1'){res.writeHead(503,{'Content-Type':'application/json'}).end(JSON.stringify({error:'Static mock preview has no backend'}));return;}
    if(!['GET','HEAD'].includes(req.method)&&req.headers['x-gvpak-ipc']!=='1'){res.writeHead(403).end();return;}
    // The local gateway validates the real browser origin above. The unchanged owner
    // transport accepts its original loopback origin; no main-service allowlist changes.
    const upstream=http.request({hostname:'127.0.0.1',port:backend,path:req.url,method:req.method,headers:{...req.headers,host:`127.0.0.1:${backend}`,origin:'http://127.0.0.1:8787'}},response=>{res.writeHead(response.statusCode,response.headers);response.pipe(res);});
    upstream.on('error',()=>{if(!res.headersSent)res.writeHead(502,{'Content-Type':'application/json'});res.end(JSON.stringify({error:'新品试用服务尚未就绪，请稍后重试。'}));});req.on('aborted',()=>upstream.destroy());req.pipe(upstream);return;
  }
  if(!['GET','HEAD'].includes(req.method)){res.writeHead(405).end();return;}
  try{
    const page=['/','/generate','/library','/settings'].includes(url.pathname)||/^\/editor\/[^/]+$/.test(url.pathname);
    const file=path.resolve(root,page?'index.html':'.'+decodeURIComponent(url.pathname));
    if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}
    const bytes=await fs.readFile(file);res.setHeader('Content-Type',types[path.extname(file)]||'application/octet-stream');res.setHeader('Cache-Control','no-store');res.setHeader('X-Content-Type-Options','nosniff');res.end(req.method==='HEAD'?undefined:bytes);
  }catch{res.writeHead(404).end();}
}).listen(port,'127.0.0.1',()=>console.log(`New-product trial ready at http://127.0.0.1:${port}/generate`));
