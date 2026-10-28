import fs from 'node:fs';import {compile,compileModule} from 'svelte/compiler';
const files=directory=>fs.readdirSync(directory,{withFileTypes:true}).flatMap(entry=>entry.isDirectory()?files(`${directory}/${entry.name}`):[`${directory}/${entry.name}`]);
for(const file of files('src').filter(x=>x.endsWith('.svelte'))){try{const result=compile(fs.readFileSync(file,'utf8'),{filename:file});console.log(file,'OK',result.warnings.map(x=>x.code).join(','));}catch(e){console.log(file,e.message,e.frame);process.exitCode=1;}}
for(const file of files('src').filter(x=>x.endsWith('.svelte.js'))){try{compileModule(fs.readFileSync(file,'utf8'),{filename:file});console.log(file,'OK')}catch(e){console.log(e.message);process.exitCode=1;}}
