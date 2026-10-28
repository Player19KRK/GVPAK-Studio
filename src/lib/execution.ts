export type ExecState='resolving'|'planning'|'awaiting_confirm'|'running'|'done'|'partial'|'failed'|'cancelled';
const transitions:Record<ExecState,ExecState[]>={resolving:['planning','awaiting_confirm','running','failed','cancelled'],planning:['awaiting_confirm','running','failed','cancelled'],awaiting_confirm:['planning','running','cancelled','failed'],running:['done','partial','failed','cancelled'],done:[],partial:[],failed:[],cancelled:[]};
export const terminalExecStates=new Set<ExecState>(['done','partial','failed','cancelled']);
export function transitionExec(row:any,next:ExecState,patch:Record<string,unknown>={}) {if(row.state!==next&&!transitions[row.state as ExecState]?.includes(next))throw Error(`执行状态不能从 ${row.state} 转到 ${next}`);return {...row,...patch,state:next};}
export function executionGate(input:{kind?:string;count?:number;estimatedCny?:number|null;limitCny?:number;quota?:{used:number;limit:number}|null}) {
 if(input.quota&&input.quota.limit>0&&1-input.quota.used/input.quota.limit<.05)return 'quota';
 if(input.kind==='describe'&&(input.count||0)>20)return 'analysis';
 if(input.kind==='generate'&&((input.count||0)>4||(input.estimatedCny!=null&&input.estimatedCny>(input.limitCny??2))))return 'generation';
 return null;
}
export async function replaceExecution(previous:any,undo:(id:string)=>Promise<unknown>,execute:()=>Promise<unknown>) {if(previous?.batchId)await undo(previous.batchId);return execute();}
