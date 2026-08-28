import { emptyDocument, SAMPLE_DOCUMENT } from './sample';
import type { QuarterDocument } from './types';

const DEMO_KEY = 'demo:quarterly-ready:document';
const REAL_KEY = 'quarterly-ready:document';
const WORKSPACE_KEY = 'quarterly-ready:workspace-id';

function clone<T>(value: T): T { return JSON.parse(JSON.stringify(value)) as T; }

export function loadDocument(demo: boolean): QuarterDocument {
  const key = demo ? DEMO_KEY : REAL_KEY;
  const saved = localStorage.getItem(key);
  if (saved) {
    try { return JSON.parse(saved) as QuarterDocument; } catch { localStorage.removeItem(key); }
  }
  const document = demo ? clone(SAMPLE_DOCUMENT) : emptyDocument();
  localStorage.setItem(key, JSON.stringify(document));
  return document;
}

export function saveDocument(document: QuarterDocument, demo: boolean): void {
  document.updatedAt = new Date().toISOString();
  localStorage.setItem(demo ? DEMO_KEY : REAL_KEY, JSON.stringify(document));
  if (!demo && navigator.onLine) void saveRemote(document);
}

export function resetDemo(): QuarterDocument {
  localStorage.removeItem(DEMO_KEY);
  return loadDocument(true);
}

export function leaveDemo(): void { localStorage.removeItem(DEMO_KEY); }

export function workspaceId(): string {
  let id = localStorage.getItem(WORKSPACE_KEY);
  if (!id) { id = crypto.randomUUID(); localStorage.setItem(WORKSPACE_KEY, id); }
  return id;
}

export async function loadRemote(): Promise<QuarterDocument | null> {
  const response = await fetch('/api/workspace', { headers: { 'x-workspace-id': workspaceId() } });
  if (response.status === 404) return null;
  if (!response.ok) throw new Error('Saved records could not be loaded. Your browser copy is still available.');
  const result = await response.json() as { document: QuarterDocument };
  localStorage.setItem(REAL_KEY, JSON.stringify(result.document));
  return result.document;
}

async function saveRemote(document: QuarterDocument): Promise<void> {
  try {
    const response = await fetch('/api/workspace', {
      method: 'PUT', headers: { 'content-type': 'application/json', 'x-workspace-id': workspaceId() },
      body: JSON.stringify({ document })
    });
    if (!response.ok) window.dispatchEvent(new CustomEvent('save-error'));
  } catch { window.dispatchEvent(new CustomEvent('save-error')); }
}

export async function createShare(document: QuarterDocument): Promise<string> {
  const response = await fetch('/api/share', {
    method: 'POST', headers: { 'content-type': 'application/json', 'x-workspace-id': workspaceId() },
    body: JSON.stringify({ document })
  });
  if (!response.ok) throw new Error('The accountant link was not created. Check your connection and try again.');
  const result = await response.json() as { token: string };
  return `${location.origin}/share/${result.token}`;
}

export async function loadShare(token: string): Promise<QuarterDocument> {
  if (token === 'demo') return clone(SAMPLE_DOCUMENT);
  const response = await fetch(`/api/share/${encodeURIComponent(token)}`);
  const result = await response.json() as { document?: QuarterDocument; error?: string };
  if (!response.ok || !result.document) throw new Error(result.error || 'This accountant pack could not be opened.');
  return result.document;
}

export const storageKeys = { demo: DEMO_KEY, real: REAL_KEY };
