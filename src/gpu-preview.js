import {outputGeometry} from './preview-cache';
const vertex=`#version 300 es
precision highp float;
out vec2 uv;
void main(){vec2 p=vec2(gl_VertexID==1?3.0:-1.0,gl_VertexID==2?3.0:-1.0);gl_Position=vec4(p,0.,1.);uv=vec2((p.x+1.)*.5,1.-(p.y+1.)*.5);}`;
const fragment=`#version 300 es
precision highp float;
in vec2 uv;out vec4 result;
uniform sampler2D source;uniform vec4 crop;uniform vec4 color;uniform float strength;uniform int rotation;uniform vec2 flip;
void main(){vec2 p=uv;if(flip.x>.5)p.x=1.-p.x;if(flip.y>.5)p.y=1.-p.y;if(rotation==90)p=vec2(p.y,1.-p.x);else if(rotation==180)p=1.-p;else if(rotation==270)p=vec2(1.-p.y,p.x);p=crop.xy+p*crop.zw;vec4 pixel=texture(source,p);vec3 c=pixel.rgb;float l=dot(c,vec3(.2126,.7152,.0722));vec3 v=((vec3(l)+(c-vec3(l))*color.z)*exp2(color.x)-.5)*color.y+.5;v+=vec3(color.w*.08,0.,-color.w*.08);vec3 outColor=floor(mix(c,clamp(v,0.,1.),strength)*255.+.5)/255.;result=vec4(outColor,pixel.a);}`;
export function createGpuPreview(canvas,onLost=()=>{}){
 const gl=canvas.getContext('webgl2',{alpha:true,premultipliedAlpha:false,antialias:false,depth:false,stencil:false,powerPreference:'high-performance',preserveDrawingBuffer:false});if(!gl)return null;
 function shader(type,code){const s=gl.createShader(type);gl.shaderSource(s,code);gl.compileShader(s);if(!gl.getShaderParameter(s,gl.COMPILE_STATUS)){const reason=gl.getShaderInfoLog(s);gl.deleteShader(s);throw Error(reason);}return s;}
 let program,texture,vs,fs;try{vs=shader(gl.VERTEX_SHADER,vertex);fs=shader(gl.FRAGMENT_SHADER,fragment);program=gl.createProgram();gl.attachShader(program,vs);gl.attachShader(program,fs);gl.linkProgram(program);if(!gl.getProgramParameter(program,gl.LINK_STATUS))throw Error(gl.getProgramInfoLog(program));texture=gl.createTexture();}catch{gl.deleteProgram(program);gl.deleteShader(vs);gl.deleteShader(fs);gl.getExtension('WEBGL_lose_context')?.loseContext();return null;}
 const uniforms=Object.fromEntries(['source','crop','color','strength','rotation','flip'].map(k=>[k,gl.getUniformLocation(program,k)]));let uploaded=null,lost=false,disposed=false;let draws=0,lastDrawMs=0;
 const extension=gl.getExtension('WEBGL_debug_renderer_info');const renderer=extension?gl.getParameter(extension.UNMASKED_RENDERER_WEBGL):gl.getParameter(gl.RENDERER);
 const contextLost=e=>{e.preventDefault();lost=true;onLost();};canvas.addEventListener('webglcontextlost',contextLost);
 function draw(bitmap,edit,sourceWidth=bitmap.width,sourceHeight=bitmap.height){if(lost||disposed)return false;const start=performance.now();const geometry=outputGeometry(sourceWidth,sourceHeight,edit);if(canvas.width!==geometry.width||canvas.height!==geometry.height){canvas.width=geometry.width;canvas.height=geometry.height;}gl.viewport(0,0,canvas.width,canvas.height);gl.useProgram(program);gl.activeTexture(gl.TEXTURE0);gl.bindTexture(gl.TEXTURE_2D,texture);
 if(uploaded!==bitmap){gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL,false);gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL,false);gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL,gl.NONE);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,gl.RGBA,gl.UNSIGNED_BYTE,bitmap);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.CLAMP_TO_EDGE);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.CLAMP_TO_EDGE);uploaded=bitmap;}
 gl.uniform1i(uniforms.source,0);gl.uniform4fv(uniforms.crop,geometry.crop);gl.uniform4f(uniforms.color,edit.exposure||0,edit.contrast??1,edit.saturation??1,edit.temperature||0);gl.uniform1f(uniforms.strength,edit.adjustmentEnabled===false?0:edit.adjustmentOpacity??1);gl.uniform1i(uniforms.rotation,edit.rotation||0);gl.uniform2f(uniforms.flip,edit.flipH?1:0,edit.flipV?1:0);gl.drawArrays(gl.TRIANGLES,0,3);draws++;lastDrawMs=performance.now()-start;return true;}
 return {draw,renderer,stats:()=>({draws,lastDrawMs,renderer}),pixels(){const p=new Uint8Array(canvas.width*canvas.height*4);gl.readPixels(0,0,canvas.width,canvas.height,gl.RGBA,gl.UNSIGNED_BYTE,p);return p;},dispose(){disposed=true;canvas.removeEventListener('webglcontextlost',contextLost);gl.deleteTexture(texture);gl.deleteProgram(program);gl.deleteShader(vs);gl.deleteShader(fs);}};
}
