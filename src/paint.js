import {outputGeometry} from './preview-cache.js';

export function sourcePoint([x,y], edit, width, height) {
 const {crop}=outputGeometry(width,height,edit);
 if(edit.flipH)x=1-x;if(edit.flipV)y=1-y;
 if(edit.rotation===90)[x,y]=[y,1-x];else if(edit.rotation===180)[x,y]=[1-x,1-y];else if(edit.rotation===270)[x,y]=[1-y,x];
 return [crop[0]+x*crop[2],crop[1]+y*crop[3]];
}
export function viewPoint([x,y],edit,width,height) {
 const {crop}=outputGeometry(width,height,edit);
 x=(x-crop[0])/crop[2];y=(y-crop[1])/crop[3];
 if(edit.rotation===90)[x,y]=[1-y,x];else if(edit.rotation===180)[x,y]=[1-x,1-y];else if(edit.rotation===270)[x,y]=[y,1-x];
 if(edit.flipH)x=1-x;if(edit.flipV)y=1-y;return [x,y];
}
export function strokeRadius(stroke,edit,sourceWidth,sourceHeight,width) {
 const {crop}=outputGeometry(sourceWidth,sourceHeight,edit);
 const span=edit.rotation%180?crop[3]*sourceHeight:crop[2]*sourceWidth;
 return Math.max(.5,stroke.size*.5*width/span);
}
// Maximum coverage per stroke prevents repeated pointer samples darkening one stroke.
// Keep this kernel and compositing arithmetic aligned with backend/src/paint.rs.
export function applyStroke(layer,width,height,stroke,edit,sourceWidth,sourceHeight) {
 const mask=new Uint8Array(width*height),r=strokeRadius(stroke,edit,sourceWidth,sourceHeight,width);
 const feather=Math.max(1,r*(1-stroke.hardness));let left=width,top=height,right=0,bottom=0;
 function dab(cx,cy) {
  const x0=Math.max(0,Math.floor(cx-r)),y0=Math.max(0,Math.floor(cy-r)),x1=Math.min(width,Math.ceil(cx+r)),y1=Math.min(height,Math.ceil(cy+r));
  left=Math.min(left,x0);top=Math.min(top,y0);right=Math.max(right,x1);bottom=Math.max(bottom,y1);
  for(let y=y0;y<y1;y++)for(let x=x0;x<x1;x++){
   const a=Math.round(Math.min(1,Math.max(0,(r-Math.hypot(x+.5-cx,y+.5-cy))/feather))*255),i=y*width+x;
   if(a>mask[i])mask[i]=a;
  }
 }
 let previous=null;
 for(const point of stroke.points){const v=viewPoint(point,edit,sourceWidth,sourceHeight),p=[v[0]*width,v[1]*height];
  if(previous){const count=Math.max(1,Math.ceil(Math.hypot(p[0]-previous[0],p[1]-previous[1])/Math.max(.5,r*.25)));
   for(let i=1;i<=count;i++){const t=i/count;dab(previous[0]+(p[0]-previous[0])*t,previous[1]+(p[1]-previous[1])*t);}
  }else dab(...p);previous=p;
 }
 const color=stroke.color;
 for(let y=top;y<bottom;y++)for(let x=left;x<right;x++){
  const pixel=y*width+x,k=pixel*4,a=mask[pixel]/255*stroke.opacity;if(!a)continue;
  const old=layer[k+3]/255;
  if(stroke.erase){layer[k+3]=Math.round(old*(1-a)*255);continue;}
  const alpha=a+old*(1-a);
  for(let c=0;c<3;c++)layer[k+c]=Math.round((color[c]*a+layer[k+c]*old*(1-a))/alpha);
  layer[k+3]=Math.round(alpha*255);
 }
 return layer;
}
export function renderPaint(width,height,strokes,edit,sourceWidth,sourceHeight) {
 const layer=new Uint8ClampedArray(width*height*4);
 for(const stroke of strokes)applyStroke(layer,width,height,stroke,edit,sourceWidth,sourceHeight);
 return layer;
}
export const polar=(r,a)=>[Math.cos(a*Math.PI/180)*r,Math.sin(a*Math.PI/180)*r];
export function arcPath(radius,start,end){const a=polar(radius,start),b=polar(radius,end);return `M ${a} A ${radius} ${radius} 0 ${end-start>180?1:0} 1 ${b}`;}
export function sectorPath(inner,outer,start,end){const a=polar(outer,start),b=polar(outer,end),c=polar(inner,end),d=polar(inner,start);return `M ${a} A ${outer} ${outer} 0 ${end-start>180?1:0} 1 ${b} L ${c} A ${inner} ${inner} 0 ${end-start>180?1:0} 0 ${d} Z`;}
export function paletteColor(index){
 const ring=Math.floor(index/24),column=index%24;
 if(column>=21){const v=[255,224,192,160,128,96,64,32,0][ring*3+column-21];return '#'+v.toString(16).padStart(2,'0').repeat(3);}
 const h=column/21*6,s=[.55,.78,.85][ring],l=[.8,.52,.29][ring],c=(1-Math.abs(2*l-1))*s,x=c*(1-Math.abs(h%2-1)),m=l-c/2;
 const channels=h<1?[c,x,0]:h<2?[x,c,0]:h<3?[0,c,x]:h<4?[0,x,c]:h<5?[x,0,c]:[c,0,x];
 return '#'+channels.map(v=>Math.round((v+m)*255).toString(16).padStart(2,'0')).join('');
}
