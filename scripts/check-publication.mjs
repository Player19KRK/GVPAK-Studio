import {execFileSync} from 'node:child_process';
import path from 'node:path';
import fs from 'node:fs/promises';
const git=args=>execFileSync('git',args,{maxBuffer:64*1024*1024});
const files=git(['diff','--cached','--name-only','--diff-filter=ACMR','-z']).toString().split('\0').filter(Boolean);
const rejected=[];
const blocked=/(?:^|\/)(?:\.tools|node_modules|output|target|team-private|studio-data|给Claude审阅的代码|Claude核心代码TXT)(?:\/|$)|(?:^|\/)\.env(?:\.|$)(?!example$)|\.(?:key|pem|crt|cer|pfx|p12|pdb|sqlite3?|db|log)$/i;
const secrets=[['Google API key',/AIza[\w-]{35}/g],['GitHub credential',/\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{50,})\b/g],['provider credential',/\bsk-(?:proj-|svcacct-)?[A-Za-z0-9_-]{20,}\b/g],['private key',/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/g],['personal home path',/[A-Za-z]:[\\/]+Users[\\/]+(?!Public\b|Default\b|runneradmin\b)[^\\/\s"']+/gi]];
for(const file of files){
 if(blocked.test(file)){rejected.push({file,reason:'private runtime or generated file'});continue;}
 const bytes=git(['show',':'+file]);
 for(const [reason,pattern]of secrets){pattern.lastIndex=0;if(pattern.test(bytes.toString('utf8')))rejected.push({file,reason});}
}
const indexed=new Set(git(['ls-files','-z']).toString().split('\0').filter(Boolean));
if(indexed.has('dist/index.html')){
 const index=git(['show',':dist/index.html']).toString();
 for(const match of index.matchAll(/(?:src|href)="(\/assets\/[^"?]+)/g))if(!indexed.has('dist'+match[1]))rejected.push({file:'dist/index.html',reason:'missing staged build reference'});
 const currentRoot='.tools/new-product-trial/runtime/dist';
 if(await fs.access(currentRoot).then(()=>true,()=>false)){
  const current=new Set();async function walk(dir){for(const entry of await fs.readdir(dir,{withFileTypes:true})){const file=path.join(dir,entry.name);if(entry.isDirectory())await walk(file);else current.add('dist/'+path.relative(currentRoot,file).split(path.sep).join('/'));}}await walk(currentRoot);
  for(const file of indexed)if(file.startsWith('dist/')&&!current.has(file))rejected.push({file,reason:'obsolete build artifact'});
  for(const file of current)if(!indexed.has(file))rejected.push({file,reason:'current build file omitted'});
 }
}
if(rejected.length){console.error(JSON.stringify({status:'failed',files:files.length,rejected},null,2));process.exitCode=1;}else console.log(JSON.stringify({status:'passed',stagedFilesScanned:files.length,checks:['private runtime and output paths excluded','staged blob credential patterns checked','current build references complete']},null,2));
