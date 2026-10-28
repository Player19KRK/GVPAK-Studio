import {createWorkerClient} from './worker-client.ts';
const client = createWorkerClient(() => new Worker(new URL('../workers/hash.worker.ts', import.meta.url), { type: 'module' }));
export const hashBlob = (blob: Blob) => client.call<string>({ blob });
export const disposeHashWorker = () => client.close();
