import fs from 'node:fs/promises';import assert from 'node:assert/strict';
const before=JSON.parse(await fs.readFile('output/playwright/theme-before/receipt.json','utf8')),a=JSON.parse(await fs.readFile('output/playwright/theme-after/receipt.json','utf8'));
assert.equal(a.rows.length,32);assert.deepEqual(a.errors,[]);assert.deepEqual(a.unexpected,[]);
for(const r of a.rows){assert.equal(r.horizontalOverflow,false,`${r.view}/${r.width}/${r.colorScheme}`);assert.equal(r.header.background,r.colorScheme==='dark'?'rgb(33, 41, 35)':'rgb(255, 255, 255)');if(r.workflow)assert.equal(r.workflow.color,r.colorScheme==='dark'?'rgb(232, 237, 231)':'rgb(37, 46, 40)');if(r.view==='editor')assert(r.inspector);}
for(const scheme of ['light','dark'])assert.deepEqual(a.rows.find(r=>r.view==='editor'&&r.width===1280&&r.colorScheme===scheme).cf,before.rows.find(r=>r.view==='editor'&&r.colorScheme===scheme).cf);
assert.deepEqual(a.themeSwitch.map(v=>v.background),['rgb(255, 255, 255)','rgb(33, 41, 35)','rgb(255, 255, 255)']);assert.equal(a.interaction.keyboardFocus.visible,true);assert.equal(a.interaction.keyboardFocus.width,'2px');assert.equal(a.interaction.keyboardFocus.transition,'0s');assert.notEqual(a.interaction.hoverColor,a.interaction.activeColor);
console.log('PASS: 32 layouts, live system-theme switch, keyboard focus, reduced motion, hover/active, frozen coverflow colors');
