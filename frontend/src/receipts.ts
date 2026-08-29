import type { QuarterDocument } from './types';

const DATABASE = 'quarterly-ready-receipts-v1';
const STORE = 'receipts';

interface StoredReceipt {
  key: string;
  blob: Blob;
  name: string;
  type: string;
  size: number;
  updatedAt: string;
}

function database(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DATABASE, 1);
    request.onupgradeneeded = () => request.result.createObjectStore(STORE, { keyPath: 'key' });
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
    request.onblocked = () => reject(new Error('Receipt storage is open in another tab. Close it and try again.'));
  });
}

function receiptKey(demo: boolean, quarterStart: string, transactionId: string): string {
  return `${demo ? 'demo' : 'real'}:${quarterStart}:${transactionId}`;
}

function transactionDone(transaction: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
    transaction.onabort = () => reject(transaction.error || new DOMException('Receipt storage was cancelled.', 'AbortError'));
  });
}

export function receiptStorageMessage(error: unknown): string {
  const name = error instanceof DOMException ? error.name : '';
  if (name === 'QuotaExceededError' || name === 'UnknownError') {
    return 'This browser does not have enough space for that receipt. The transaction was not changed. Remove an older receipt or choose a smaller file, then try again.';
  }
  return error instanceof Error && error.message.includes('another tab')
    ? error.message
    : 'The receipt could not be saved in this browser. The transaction was not changed. Try again.';
}

export async function saveReceipt(file: Blob, name: string, document: QuarterDocument, demo: boolean, transactionId: string): Promise<void> {
  const db = await database();
  try {
    const transaction = db.transaction(STORE, 'readwrite', { durability: 'strict' });
    const record: StoredReceipt = {
      key: receiptKey(demo, document.quarterStart, transactionId),
      blob: file,
      name,
      type: file.type,
      size: file.size,
      updatedAt: new Date().toISOString(),
    };
    transaction.objectStore(STORE).put(record);
    await transactionDone(transaction);
  } catch (error) {
    throw new Error(receiptStorageMessage(error), { cause: error });
  } finally {
    db.close();
  }
}

export async function deleteReceipt(document: QuarterDocument, demo: boolean, transactionId: string): Promise<void> {
  const db = await database();
  try {
    const transaction = db.transaction(STORE, 'readwrite');
    transaction.objectStore(STORE).delete(receiptKey(demo, document.quarterStart, transactionId));
    await transactionDone(transaction);
  } finally {
    db.close();
  }
}

export async function clearDemoReceipts(): Promise<void> {
  const db = await database();
  try {
    const transaction = db.transaction(STORE, 'readwrite');
    const store = transaction.objectStore(STORE);
    const keys = await new Promise<IDBValidKey[]>((resolve, reject) => {
      const request = store.getAllKeys(IDBKeyRange.bound('demo:', 'demo:\uffff'));
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    for (const key of keys) store.delete(key);
    await transactionDone(transaction);
  } finally {
    db.close();
  }
}

function dataUrlBlob(value: string): Blob {
  const match = /^data:([^;,]+);base64,(.*)$/.exec(value);
  if (!match) throw new Error('The older receipt format is not supported. Attach the file again.');
  const decoded = atob(match[2]);
  const bytes = new Uint8Array(decoded.length);
  for (let index = 0; index < decoded.length; index += 1) bytes[index] = decoded.charCodeAt(index);
  return new Blob([bytes], { type: match[1] });
}

export async function migrateLegacyReceipts(document: QuarterDocument, demo: boolean): Promise<boolean> {
  const legacy = document.transactions.filter(transaction => transaction.receiptData);
  if (!legacy.length) return false;
  for (const transaction of legacy) {
    const blob = dataUrlBlob(transaction.receiptData!);
    await saveReceipt(blob, transaction.receiptName || 'receipt', document, demo, transaction.id);
  }
  for (const transaction of legacy) delete transaction.receiptData;
  return true;
}

export const receiptStorage = { database: DATABASE, store: STORE };
