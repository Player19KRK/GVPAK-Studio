import {ratios,chooseRatio,swapRatio,ratioLabel} from './ratio.ts';
function ratioStateFromLabel(label:string){for(const ratio of ratios){const state=chooseRatio(ratio);if(ratioLabel(state)===label)return state;const reversed=swapRatio(state);if(ratioLabel(reversed)===label)return reversed;}throw Error('比例仅支持 1:1 / 3:2 / 4:3 / 4:5 / 16:9 及横竖切换');}
export const slashCommands=[{name:'白底',kind:'white'},{name:'抠图',kind:'cutout'},{name:'居中',kind:'center'},{name:'比例',kind:'fit_ratio'},{name:'导出',kind:'export'},{name:'撤销',kind:'undo'}];
export function parseSlash(text:string) {
 const value=text.trim();if(value==='撤销刚才的'||value==='撤销刚才')return {name:'撤销',operations:[{kind:'undo'}]};
 if(!value.startsWith('/'))return null;
 const match=value.match(/^\/(白底|抠图|居中|比例|导出|撤销)(?:\s+(.*))?$/);if(!match)throw Error('未识别的命令。可用 /白底 /抠图 /居中 /比例 /导出 /撤销');
 const command=slashCommands.find(command=>command.name===match[1])!,parameter=(match[2]||'').trim();
 if(command.kind==='fit_ratio'){if(!parameter)throw Error('请填写比例，例如 /比例 16:9 竖');const parts=parameter.split(/\s+/),ratio=ratioStateFromLabel(parts[0]);if(parts.slice(1).some(value=>!['横','竖'].includes(value))||parts.length>2)throw Error('方向请使用 横 或 竖');if(parts[1]&&ratio.ratio!=='1:1')ratio.orientation=parts[1]==='竖'?'portrait':'landscape';return {name:command.name,operations:[{kind:command.kind,...ratio}]};}
 if(parameter)throw Error(`/${command.name} 不需要额外参数`);return {name:command.name,operations:[{kind:command.kind}]};
}
