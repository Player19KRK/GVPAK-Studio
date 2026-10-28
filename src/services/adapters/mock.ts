import {createMockWorkspace} from './mock-workspace.ts';
import {eventBus} from '../realtime.ts';
import {normalizeProjects} from '../normalize.ts';
import type {EventListener} from '../realtime.ts';
import type {SelectionContext} from '../../lib/target.ts';
export function createMockAdapter(seed: { projects?: any[]; session?: any; tasks?: any[]; allowTaskSimulation?: boolean } = {}) {
  const projects = normalizeProjects(structuredClone(seed.projects || [])), tasks = structuredClone(seed.tasks || []), bus = eventBus();
  const workspace=createMockWorkspace(projects);
  async function execute(request: { context: SelectionContext; [key: string]: any }) {
    if(seed.allowTaskSimulation===false)throw Error('模拟模式不执行图像处理或生成结果。');
    const text=request.operations?.[0]?.kind==='generate'&&request.operations[0].source==='text';
    if (!request.context.resolvedTarget.ids.length&&!text) throw Error('请先选择图片。');
    const task = { id: crypto.randomUUID(), projectId:request.projectId||null, type: 'mock', accountId: 'mock', targetCount: text?1:request.context.resolvedTarget.ids.length, state: 'running', done: 0, total: text?1:request.context.resolvedTarget.ids.length, queuePosition: null, startedAt: Date.now(), mock: true };
    tasks.push(task);queueMicrotask(() => bus.emit({ type: 'task.progress', payload: { ...task } }));
    return { task: { ...task }, mock: true };
  }
  return {
    ...workspace,mode: 'mock' as const,
    async output(_taskId:string,_assetId:string):Promise<Blob> {throw Error('模拟任务不返回假图片。');},

    async credentialStatus(_model:string) {return false;},
    async readSession() { return structuredClone(seed.session || { team: false }); },
    async readStatus() { return structuredClone(seed.session || {team:false,isHost:true,isLocal:true,quota:null}); },
    async bootstrap() { return structuredClone({ projects, dataRoot: 'mock', warnings: [], catalog: workspace.catalog, tasks, jobs: [] }); },
    async metadata(input:any){const project=projects.find(project=>project.id===input.projectId);if(!project)throw Error('项目不存在');const candidate=structuredClone(project);for(const entry of input.entries){const asset=candidate.assets.find(asset=>asset.id===entry.assetId);if(!asset)throw Error('素材不存在');if(entry.folder!==undefined){asset.folderId=entry.folder;asset.relativePath=(entry.folder?entry.folder+'/':'')+asset.name;}if(entry.tags)asset.tags=[...entry.tags];}Object.assign(project,candidate);for(const entry of input.entries){const asset=project.assets.find(asset=>asset.id===entry.assetId);bus.emit({type:'asset.updated',payload:{assetId:asset.id,patch:structuredClone(asset),projectId:project.id}});}return {project:structuredClone(project)};},
    async listTasks() { return structuredClone(tasks); },
    subscribe(listener: EventListener) { return bus.subscribe(listener); },
    execute, async chat(input:any){if(seed.allowTaskSimulation===false)throw Error('模拟模式不调用DeepSeek或生成图片。');return execute(input);},
    async stopTask(input: { id: string }) { const task = tasks.find((item: any) => item.id === input.id); if (!task) throw Error('任务不存在。'); task.state = 'cancelled'; bus.emit({ type: 'task.done', payload: { ...task } }); },
    // Test/dev progression is explicit. No mock image pixels or paid output are claimed.
    advanceTask(id: string, patch: Record<string, unknown>) {
      const task = tasks.find((item: any) => item.id === id); if (!task) throw Error('任务不存在。');
      Object.assign(task, patch);bus.emit({ type: ['done', 'partial', 'cancelled'].includes(task.state) ? 'task.done' : task.state === 'failed' ? 'task.failed' : 'task.progress', payload: { ...task } });
    },
  };
}
