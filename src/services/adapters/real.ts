import {invoke, request} from '../http.ts';
import {connectSse} from '../realtime.ts';
import {normalizeProjects} from '../normalize.ts';
import type {EventListener} from '../realtime.ts';
import type {SelectionContext} from '../../lib/target.ts';
export class CapabilityUnavailable extends Error {
  capability: string;
  constructor(capability: string) { super(`后端尚未提供 ${capability}，请先确认接口。`); this.name = 'CapabilityUnavailable'; this.capability = capability; }
}
export function taskFromJob(job: any) {
  return {
    id: job.id, type: 'hybrid', accountId: job.accountId ?? null, targetCount: job.items?.length || 0,
    state: ({ completed: 'done', partial: 'partial', failed: 'failed', cancelled: 'cancelled', queued: 'queued' } as Record<string, string>)[job.state] || 'running',
    done: (job.items || []).filter((item: any) => item.stage === 'done').length, total: job.items?.length || 0,
    queuePosition: null, startedAt: job.startedAt ? job.startedAt * 1000 : null, raw: job,
  };
}
export function createRealAdapter(fetcher: typeof fetch = fetch) {
  const call = (name: string, args: unknown = {}) => invoke(name, args, fetcher);
  return {
    mode: 'real' as const,
    async readSession() { return (await request('/api/team/session', { cache: 'no-store' }, fetcher)).json(); },
    async readStatus() { return call('session_status'); },
    async bootstrap() {
      const [workspace, models, jobs] = await Promise.all([call('bootstrap'), call('model_catalog'), call('hybrid_jobs')]);
      return { ...workspace, projects: normalizeProjects(workspace.projects || []), catalog: models.catalog, modelWarning: models.warning || '', tasks: (jobs.jobs || []).map(taskFromJob), jobs: jobs.jobs || [] };
    },
    async metadata(input:unknown) { return call('assets_metadata',input); },
    async output(taskId:string,assetId:string):Promise<Blob> {return call('operation_result',{taskId,assetId});},
    async createProject(name:string) { return call('project_create',{name}); },
    async credentialStatus(model:string) {return call(model.startsWith('GPT-Image')?'poe_key_status':'credential_status');},
    async listTasks() { return ((await call('task_list')).tasks || []); },
    subscribe(listener: EventListener, onError?: (error: Error) => void) { return connectSse(listener, onError); },
    async execute(input: { context: SelectionContext; [key: string]: unknown }): Promise<any> { return call('operation_execute',input); },
    async chat(input: { context: SelectionContext; [key: string]: unknown }): Promise<any> { return call('chat_execute',input); },
    async stopTask(task: { id: string; type: string }) {
      return call('task_stop', { taskId: task.id });
    },
  };
}
