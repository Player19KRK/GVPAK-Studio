export class PreviewCache {
 constructor({maxEntries=12,maxBlobBytes=32*1024*1024,maxDecodedBytes=128*1024*1024,makeURL=blob=>URL.createObjectURL(blob),dropURL=url=>URL.revokeObjectURL(url)}={}){this.entries=new Map();this.maxEntries=maxEntries;this.maxBlobBytes=maxBlobBytes;this.maxDecodedBytes=maxDecodedBytes;this.makeURL=makeURL;this.dropURL=dropURL;this.hits=0;this.misses=0;this.evictions=0;}
 get(key){const entry=this.entries.get(key);if(!entry){this.misses++;return null;}this.hits++;this.entries.delete(key);this.entries.set(key,entry);return entry;}
 peek(key){return this.entries.get(key)||null;}
 put(key,blob,bitmap){const existing=this.entries.get(key);if(existing){bitmap.close?.();return existing;}const entry={key,blob,bitmap,url:this.makeURL(blob),width:bitmap.width,height:bitmap.height,refs:0};this.entries.set(key,entry);this.trim(key);return entry;}
 retain(entry){if(entry)entry.refs++;return entry;}
 release(entry){if(entry){entry.refs=Math.max(0,entry.refs-1);this.trim();}}
 trim(protectedKey){let stats=this.stats();for(const [key,e] of this.entries){if(stats.entries<=this.maxEntries&&stats.blobBytes<=this.maxBlobBytes&&stats.decodedBytes<=this.maxDecodedBytes)break;if(key===protectedKey||e.refs)continue;this.entries.delete(key);this.dropURL(e.url);e.bitmap.close?.();this.evictions++;stats=this.stats();}}
 stats(){const items=[...this.entries.values()];return {entries:items.length,blobBytes:items.reduce((n,e)=>n+e.blob.size,0),decodedBytes:items.reduce((n,e)=>n+e.width*e.height*4,0),pinned:items.filter(e=>e.refs).length,hits:this.hits,misses:this.misses,evictions:this.evictions};}
 clear(){for(const e of this.entries.values()){this.dropURL(e.url);e.bitmap.close?.();}this.entries.clear();}
}
export function sourceKey(root,pid,a){return JSON.stringify([root,pid,a.id,a.sha256||'',a.generatedBase||'',a.width,a.height]);}
export function normalizeEdit(e){return {adjustmentOpacity:Math.fround(e.adjustmentOpacity??1),adjustmentEnabled:e.adjustmentEnabled!==false,exposure:Math.fround(e.exposure||0),contrast:Math.fround(e.contrast??1),saturation:Math.fround(e.saturation??1),temperature:Math.fround(e.temperature||0),rotation:e.rotation||0,flipH:!!e.flipH,flipV:!!e.flipV,crop:e.crop?e.crop.map(Math.fround):null};}
export function previewKey(root,pid,a,edit){return sourceKey(root,pid,a)+'|preview|'+JSON.stringify(normalizeEdit(edit));}
export function originalKey(root,pid,a){return JSON.stringify([root,pid,a.id,a.sha256||'','original']);}
export function outputGeometry(width,height,edit,max=1600){const c=edit.crop;let x=0,y=0,w=width,h=height;if(c){x=Math.round(c[0]*width);y=Math.round(c[1]*height);w=Math.min(Math.max(1,Math.round(c[2]*width)),width-x);h=Math.min(Math.max(1,Math.round(c[3]*height)),height-y);}const crop=[x/width,y/height,w/width,h/height];if(edit.rotation%180)[w,h]=[h,w];const scale=Math.min(1,max/Math.max(w,h));return {width:Math.max(1,Math.round(w*scale)),height:Math.max(1,Math.round(h*scale)),crop};}
