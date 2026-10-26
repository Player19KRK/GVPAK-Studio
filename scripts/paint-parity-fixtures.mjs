import fs from 'node:fs';
import {renderPaint} from '../src/paint.js';
import {normalizeEdit,outputGeometry} from '../src/preview-cache.js';
const strokes=[{points:[[.18,.31],[.43,.52],[.8,.71]],color:[227,42,50],size:37,hardness:.37,opacity:.48,erase:false},{points:[[.45,.2],[.5,.78]],color:[20,110,210],size:29,hardness:.7,opacity:.6,erase:false},{points:[[.48,.4],[.55,.5]],color:[0,0,0],size:19,hardness:.3,opacity:1,erase:true}];
const fixtures=[{}, {crop:[.1,.2,.7,.6],rotation:90,flipH:true}, {rotation:270,flipV:true}].map(p=>{const edit=normalizeEdit({...p,strokes}),g=outputGeometry(400,300,edit);return {edit,width:g.width,height:g.height,rgba:Buffer.from(renderPaint(g.width,g.height,edit.strokes,edit,400,300)).toString('base64')};});
fs.writeFileSync('backend/target/paint-parity-fixtures.json',JSON.stringify(fixtures));
