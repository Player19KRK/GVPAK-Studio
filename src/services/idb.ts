export async function withStore<T>(database: string, storeName: string, mode: IDBTransactionMode, operation: (store: IDBObjectStore) => IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    const open = indexedDB.open(database, 1);
    open.onupgradeneeded = () => open.result.createObjectStore(storeName);
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {
      const db = open.result, tx = db.transaction(storeName, mode);let value: T;
      const request = operation(tx.objectStore(storeName));request.onsuccess = () => value = request.result;
      tx.oncomplete = () => { db.close();resolve(value); };
      tx.onabort = tx.onerror = () => { db.close();reject(tx.error || Error('本地存储不可用。')); };
    };
  });
}
