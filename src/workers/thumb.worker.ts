import {withStore} from '../services/idb.ts';
globalThis.onmessage = async ({ data }) => {
  let bitmap: ImageBitmap | undefined;
  try {
    const { hash, blob, size } = data;
    if (!/^[a-f0-9]{64}$/i.test(hash) || ![120, 160, 200].includes(size)) throw Error('缩略图参数无效。');
    const key = `${hash}:${size}`;
    const cached = await withStore<any>('gvpak-thumbnails', 'images', 'readonly', store => store.get(key)).catch(() => null);
    if (cached) { globalThis.postMessage({ id: data.id, result: { ...cached, cacheHit: true } });return; }
    bitmap = await createImageBitmap(blob);
    const scale = Math.min(1, size / Math.max(bitmap.width, bitmap.height));
    const width = Math.max(1, Math.round(bitmap.width * scale)), height = Math.max(1, Math.round(bitmap.height * scale));
    const canvas = new OffscreenCanvas(width, height), context = canvas.getContext('2d');
    if (!context) throw Error('无法生成缩略图。');
    context.drawImage(bitmap, 0, 0, width, height);
    const result = { blob: await canvas.convertToBlob({ type: 'image/webp', quality: .85 }), width, height };
    await withStore('gvpak-thumbnails', 'images', 'readwrite', store => store.put(result, key)).catch(() => {});
    globalThis.postMessage({ id: data.id, result: { ...result, cacheHit: false } });
  } catch (error) { globalThis.postMessage({ id: data.id, error: error instanceof Error ? error.message : '缩略图生成失败，请重试。' }); }
  finally { bitmap?.close(); }
};
