import { emptyDocument, SAMPLE_DOCUMENT } from './sample';
import { currentUkQuarter, quarterFromStart } from './quarters';
import { validateTransaction } from './records';
import { clearDemoReceipts } from './receipts';
import type { QuarterDocument } from './types';

const DEMO_KEY = 'demo:quarterly-ready:document';
const LEGACY_REAL_KEY = 'quarterly-ready:document';
const REAL_PREFIX = 'quarterly-ready:document:';
const ACTIVE_QUARTER_KEY = 'quarterly-ready:active-quarter';
const WORKSPACE_PREFIX = 'quarterly-ready:workspace-id:';

function clone<T>(value: T): T { return JSON.parse(JSON.stringify(value)) as T; }

export function loadDocument(demo: boolean): QuarterDocument {
  if (!demo) migrateLegacyDocument();
  const activeStart = demo ? SAMPLE_DOCUMENT.quarterStart : activeQuarterStart();
  const key = demo ? DEMO_KEY : `${REAL_PREFIX}${activeStart}`;
  const saved = localStorage.getItem(key);
  if (saved) {
    try {
      const document = JSON.parse(saved) as QuarterDocument;
      const period = quarterFromStart(document.quarterStart);
      if (!period || period.end !== document.quarterEnd || !Array.isArray(document.transactions)) throw new Error('Invalid quarter');
      for (const transaction of document.transactions) validateTransaction(transaction, document.quarterStart, document.quarterEnd);
      return document;
    } catch { localStorage.removeItem(key); }
  }
  const period = quarterFromStart(activeStart) || currentUkQuarter();
  const document = demo ? clone(SAMPLE_DOCUMENT) : emptyDocument(period);
  localStorage.setItem(key, JSON.stringify(document));
  return document;
}

export function saveDocument(document: QuarterDocument, demo: boolean): void {
  document.updatedAt = new Date().toISOString();
  const browserDocument = clone(document);
  for (const transaction of browserDocument.transactions) delete transaction.receiptData;
  try {
    localStorage.setItem(demo ? DEMO_KEY : `${REAL_PREFIX}${document.quarterStart}`, JSON.stringify(browserDocument));
    if (!demo) localStorage.setItem(ACTIVE_QUARTER_KEY, document.quarterStart);
  } catch (error) {
    if (error instanceof DOMException && error.name === 'QuotaExceededError') {
      throw new Error('This browser could not save the record because its local storage is full. The change was not saved. Export a CSV, clear space, and try again.', { cause: error });
    }
    throw error;
  }
  if (!demo && navigator.onLine) void saveRemote(document);
}

export function resetDemo(): QuarterDocument {
  localStorage.removeItem(DEMO_KEY);
  void clearDemoReceipts();
  return loadDocument(true);
}

export function leaveDemo(): void { localStorage.removeItem(DEMO_KEY); void clearDemoReceipts(); }

export function selectQuarter(quarterStart: string): QuarterDocument {
  if (!quarterFromStart(quarterStart)) throw new Error('Choose a standard UK quarter.');
  localStorage.setItem(ACTIVE_QUARTER_KEY, quarterStart);
  return loadDocument(false);
}

export function activeQuarterStart(): string {
  const saved = localStorage.getItem(ACTIVE_QUARTER_KEY);
  return saved && quarterFromStart(saved) ? saved : currentUkQuarter().start;
}

function migrateLegacyDocument(): void {
  const saved = localStorage.getItem(LEGACY_REAL_KEY);
  if (!saved) return;
  try {
    const document = JSON.parse(saved) as QuarterDocument;
    if (quarterFromStart(document.quarterStart)) {
      localStorage.setItem(`${REAL_PREFIX}${document.quarterStart}`, saved);
      localStorage.setItem(ACTIVE_QUARTER_KEY, document.quarterStart);
    }
  } finally { localStorage.removeItem(LEGACY_REAL_KEY); }
}

export function workspaceId(quarterStart = activeQuarterStart()): string {
  const key = `${WORKSPACE_PREFIX}${quarterStart}`;
  let id = localStorage.getItem(key);
  if (!id) { id = crypto.randomUUID(); localStorage.setItem(key, id); }
  return id;
}

export async function loadRemote(): Promise<QuarterDocument | null> {
  const quarterStart = activeQuarterStart();
  const response = await fetch('/api/workspace', { headers: { 'x-workspace-id': workspaceId(quarterStart) } });
  if (!response.ok) throw new Error('Saved records could not be loaded. Your browser copy is still available.');
  const result = await response.json() as { document: QuarterDocument | null };
  if (!result.document) return null;
  localStorage.setItem(`${REAL_PREFIX}${result.document.quarterStart}`, JSON.stringify(result.document));
  return result.document;
}

async function saveRemote(document: QuarterDocument): Promise<void> {
  try {
    const serverDocument = clone(document);
    for (const transaction of serverDocument.transactions) delete transaction.receiptData;
    const response = await fetch('/api/workspace', {
      method: 'PUT', headers: { 'content-type': 'application/json', 'x-workspace-id': workspaceId(document.quarterStart) },
      body: JSON.stringify({ document: serverDocument })
    });
    if (!response.ok) window.dispatchEvent(new CustomEvent('save-error'));
  } catch { window.dispatchEvent(new CustomEvent('save-error')); }
}

function liveHeaders(document: QuarterDocument): HeadersInit {
  const licence = localStorage.getItem('sb_license:mtd-quarterly-ready');
  return {
    'content-type': 'application/json',
    'x-workspace-id': workspaceId(document.quarterStart),
    ...(licence ? { 'x-sociobot-license': licence } : {})
  };
}

export async function createShare(document: QuarterDocument): Promise<string> {
  const sharedDocument = clone(document);
  for (const transaction of sharedDocument.transactions) delete transaction.receiptData;
  const response = await fetch('/api/share', {
    method: 'POST', headers: liveHeaders(document),
    body: JSON.stringify({ document: sharedDocument })
  });
  const result = await response.json().catch(() => ({})) as { token?: string; error?: string };
  if (!response.ok || !result.token) throw new Error(result.error || 'The accountant link was not created. Check your connection and try again.');
  return `${location.origin}/share/${result.token}`;
}

export async function submitToHmrc(document: QuarterDocument): Promise<{ reference: string; status: string; filesWithHmrc: boolean }> {
  const submittedDocument = clone(document);
  for (const transaction of submittedDocument.transactions) delete transaction.receiptData;
  const response = await fetch('/api/hmrc/submit', {
    method: 'POST', headers: liveHeaders(document),
    body: JSON.stringify({ document: submittedDocument, review_confirmed: true })
  });
  const result = await response.json().catch(() => ({})) as { submission_id?: string; status?: string; files_with_hmrc?: boolean; error?: string };
  if (!response.ok || !result.submission_id) throw new Error(result.error || 'The HMRC submission could not be completed. No submission was made.');
  return { reference: result.submission_id, status: result.status || 'accepted', filesWithHmrc: result.files_with_hmrc === true };
}

export async function loadShare(token: string): Promise<QuarterDocument> {
  if (token === 'demo') return clone(SAMPLE_DOCUMENT);
  const response = await fetch(`/api/share/${encodeURIComponent(token)}`);
  const result = await response.json() as { document?: QuarterDocument; error?: string };
  if (!response.ok || !result.document) throw new Error(result.error || 'This accountant pack could not be opened.');
  return result.document;
}

export const storageKeys = { demo: DEMO_KEY, realPrefix: REAL_PREFIX, activeQuarter: ACTIVE_QUARTER_KEY };
