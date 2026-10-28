export function createWorkerClient(factory: () => Worker) {
  let worker: Worker | null = null, serial = 0;
  const pending = new Map<number, { resolve: (value: any) => void; reject: (error: Error) => void }>();
  function close(message = '图片后台处理已停止。') {
    worker?.terminate();worker = null;
    for (const call of pending.values()) call.reject(Error(message));pending.clear();
  }
  return {
    call<T>(payload: Record<string, unknown>): Promise<T> {
      if (!worker) {
        worker = factory();
        worker.onmessage = ({ data }) => {
          const call = pending.get(data.id);if (!call) return;pending.delete(data.id);
          if (data.error) call.reject(Error(data.error));else call.resolve(data.result);
        };
        worker.onerror = () => close('图片后台处理失败，请重试。');
      }
      const id = ++serial;
      return new Promise((resolve, reject) => { pending.set(id, { resolve, reject });try { worker!.postMessage({ ...payload, id }); } catch (error) { pending.delete(id);reject(error); } });
    }, close,
  };
}
