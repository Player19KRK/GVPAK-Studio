export class HttpError extends Error {
  status: number;
  constructor(message: string, status: number) { super(message); this.name = 'HttpError'; this.status = status; }
}
export async function fetchResponse(url: string, options: RequestInit = {}, fetcher: typeof fetch = fetch) {
 if((import.meta as ImportMeta & {env?:Record<string,string>}).env?.VITE_FRONTEND_DATA_MODE==='mock')throw Error('模拟模式不访问真实后台。');
 return fetcher(url,{credentials:'same-origin',...options});
}
export async function request(url: string, options: RequestInit = {}, fetcher: typeof fetch = fetch) {
  const response = await fetchResponse(url, options, fetcher);
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: response.statusText }));
    throw new HttpError(error.error || '请求失败，请重试。', response.status);
  }
  return response;
}
export async function invoke(name: string, args: unknown = {}, fetcher: typeof fetch = fetch) {
  if((import.meta as ImportMeta & {env?:Record<string,string>}).env?.VITE_FRONTEND_DATA_MODE==='mock')return (await import('./api.ts')).getApi().legacy(name,args);
  const response = await request(`/api/invoke/${encodeURIComponent(name)}`, {
    method: 'POST', headers: { 'Content-Type': 'application/json', 'X-GVPAK-IPC': '1' }, body: JSON.stringify(args),
  }, fetcher);
  return response.headers.get('content-type')?.includes('octet-stream') ? response.blob() : response.json();
}
