import {createWorkerClient} from './worker-client.ts';
const client = createWorkerClient(() => new Worker(new URL('../workers/thumb.worker.ts', import.meta.url), { type: 'module' }));
export const thumbnailBlob = (blob: Blob, hash: string, size = 160) => client.call<{ blob: Blob; width: number; height: number; cacheHit: boolean }>({ blob, hash, size });
export const disposeThumbnailWorker = () => client.close();
