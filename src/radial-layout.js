const boxes={root:{x:-112,y:-112,w:224,h:224},color:{x:-144,y:-144,w:288,h:288},brush:{x:-34,y:-34,w:208,h:146},eraser:{x:-174,y:-34,w:208,h:122}};
const clamp=(v,min,max)=>Math.max(min,Math.min(max,v));
export function radialPlacement(page,anchor,stage,viewport){
 const left=Math.max(0,stage.left),top=Math.max(0,stage.top);
 const right=Math.min(viewport.width,stage.left+stage.width),bottom=Math.min(viewport.height,stage.top+stage.height);
 const bounds={left:left+8,top:top+8,width:Math.max(1,right-left-16),height:Math.max(1,bottom-top-16)};
 const rootScale=Math.min(1,bounds.width/224,bounds.height/224);
 const horizontalMargin=Math.max(112*rootScale,Math.min(174,bounds.width/2));
 const root={x:clamp(anchor.x,bounds.left+horizontalMargin,bounds.left+bounds.width-horizontalMargin),y:clamp(anchor.y,bounds.top+112*rootScale,bounds.top+bounds.height-112*rootScale)};
 const box=boxes[page]||boxes.root,scale=Math.min(1,bounds.width/box.w,bounds.height/box.h);
 const x=clamp(root.x,bounds.left-box.x*scale,bounds.left+bounds.width-(box.x+box.w)*scale);
 const y=clamp(root.y,bounds.top-box.y*scale,bounds.top+bounds.height-(box.y+box.h)*scale);
 return {left:x+box.x*scale,top:y+box.y*scale,width:box.w*scale,height:box.h*scale,viewBox:`${box.x} ${box.y} ${box.w} ${box.h}`,scale,origin:{x,y},bounds};
}
