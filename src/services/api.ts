import {invoke} from './http.ts';
import {createRealAdapter} from './adapters/real.ts';
import {createMockAdapter} from './adapters/mock.ts';
import {serializeSelectionContext} from '../lib/target.ts';
import type {SelectionContext} from '../lib/target.ts';
export function createApi(mode: string = 'real', options: { fetcher?: typeof fetch; seed?: Parameters<typeof createMockAdapter>[0] } = {}) {
  if (mode !== 'real' && mode !== 'mock') throw Error('VITE_FRONTEND_DATA_MODE 只能为 real 或 mock。');
  const adapter = mode === 'mock' ? createMockAdapter(options.seed) : createRealAdapter(options.fetcher);
  return {
    ...adapter,
    legacy(name:string,args:any={}){return adapter.mode==='mock'?adapter.legacy(name,args):invoke(name,args,options.fetcher);},
    execute(input: { context: SelectionContext; [key: string]: unknown }) { return adapter.execute({ projectId: input.projectId, operations: input.operations, clientRequestId: input.clientRequestId, confirmToken: input.confirmToken, context: serializeSelectionContext(input.context) }); },
    chat(input: { context: SelectionContext; [key: string]: unknown }) { return adapter.chat({ projectId: input.projectId, instruction: input.instruction,...(input.pipeline?{pipeline:true,execution:input.execution,model:input.model}:{}), clientRequestId: input.clientRequestId, confirmToken: input.confirmToken, context: serializeSelectionContext(input.context) }); },
  };
}
let instance: ReturnType<typeof createApi> | undefined;
export function getApi() {
  const env = (import.meta as ImportMeta & { env?: Record<string, string> }).env;
  return instance ||= createApi(env?.VITE_FRONTEND_DATA_MODE || 'real',{seed:{allowTaskSimulation:false}});
}
