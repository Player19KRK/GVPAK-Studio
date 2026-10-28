export type RealtimeEvent = { type: string; payload: any };
export type EventListener = (event: RealtimeEvent) => void;
export function eventBus() {
  const listeners = new Set<EventListener>();
  return {
    subscribe(listener: EventListener) { listeners.add(listener); return () => listeners.delete(listener); },
    emit(event: RealtimeEvent) { for (const listener of listeners) listener(event); },
    close() { listeners.clear(); },
  };
}
export const appEvents = eventBus();
export function normalizeEvent(message: { event?: string; type?: string; payload?: any }): RealtimeEvent | null {
  const type = message.event || message.type;
  if (!type) return null;
  if (type === 'job-progress') return { type: 'legacy.progress', payload: message.payload };
  if (type === 'team-queue') return { type: 'legacy.queue', payload: message.payload };
  return { type, payload: message.payload };
}
export function connectSse(listener: EventListener, onError: (error: Error) => void = () => {}, sourceFactory = () => new EventSource('/api/events')) {
  const source = sourceFactory();
  source.onmessage = event => {
    try { const value = normalizeEvent(JSON.parse(event.data)); if (value) listener(value); }
    catch { onError(Error('任务更新格式无效，请重新连接。')); }
  };
  source.onerror = () => onError(Error('实时连接中断，正在重新连接。'));
  source.onopen = () => listener({ type: 'connection.open', payload: null });
  return () => source.close();
}
