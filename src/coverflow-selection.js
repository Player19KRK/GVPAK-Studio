// Selection intent is latched once per Space press and remains unchanged until release.
export function createQuickSelection({getIds,getCurrent,isSelected,setSelected,navigate,onHeld=()=>{}}){
 let held=false,selecting=true,accumulator=0;
 const apply=ids=>setSelected(ids,selecting);
 function step(direction){
  if(!held||(direction!==-1&&direction!==1))return;
  accumulator=0;
  const ids=getIds(),from=ids.indexOf(getCurrent());if(from<0)return;
  const to=Math.max(0,Math.min(ids.length-1,from+direction));
  if(to!==from){apply([ids[to]]);navigate(ids[to]);}
 }
 return {
  get held(){return held;},
  get selecting(){return selecting;},
  press(){if(held)return;const ids=getIds(),id=getCurrent();if(!ids.includes(id))return;held=true;selecting=!isSelected(id);accumulator=0;apply([id]);onHeld(true,selecting);},
  release(){held=false;accumulator=0;onHeld(false,selecting);},
  step,
  wheel(delta){if(!held||!Number.isFinite(delta)||delta===0)return;if(accumulator&&Math.sign(accumulator)!==Math.sign(delta))accumulator=0;accumulator+=delta;if(Math.abs(accumulator)<=35)return;step(Math.sign(accumulator));}
 };
}
