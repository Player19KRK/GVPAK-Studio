globalThis.onmessage = async ({ data }) => {
  try {
    const digest = await crypto.subtle.digest('SHA-256', await data.blob.arrayBuffer());
    const result = [...new Uint8Array(digest)].map(value => value.toString(16).padStart(2, '0')).join('');
    globalThis.postMessage({ id: data.id, result });
  } catch { globalThis.postMessage({ id: data.id, error: '文件摘要计算失败，请重试。' }); }
};
