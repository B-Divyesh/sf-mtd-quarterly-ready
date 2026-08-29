import process from 'node:process';
import { readFile, writeFile } from 'node:fs/promises';

const origin = process.env.VERIFY_ORIGIN || 'https://mtd-quarterly-ready.sociobot.in';
const value = process.env.DURABILITY_PROBE_VALUE;
const mode = process.argv[2];
const workspace = '00000000-0000-4000-8000-000000000010';
const stateFile = process.env.DURABILITY_PROBE_STATE_FILE || '/tmp/quarterly-ready-durability-probe.json';
if (!value || !['seed', 'check'].includes(mode)) throw new Error('Use seed or check and set DURABILITY_PROBE_VALUE.');

const headers = { 'content-type': 'application/json', 'x-workspace-id': workspace, 'x-forwarded-for': '203.0.113.210' };
const document = {
  schemaVersion: 1,
  businessName: 'Durable storage release probe',
  quarterLabel: '6 April to 5 July 2026',
  quarterStart: '2026-04-06',
  quarterEnd: '2026-07-05',
  figuresReviewed: false,
  packDownloaded: false,
  markedReady: false,
  updatedAt: new Date().toISOString(),
  transactions: [{ id: 'durability-probe', date: '2026-04-06', description: `Durability probe ${value}`, amountPence: 100, kind: 'income', category: 'Sales' }],
};

if (mode === 'seed') {
  const response = await fetch(`${origin}/api/workspace`, { method: 'PUT', headers, body: JSON.stringify({ document }) });
  if (!response.ok) throw new Error(`durability seed returned ${response.status}`);
  const entitlementResponse = await fetch(`${origin}/api/qa/entitlement`, { headers: { 'x-forwarded-for': '203.0.113.211' } });
  if (!entitlementResponse.ok) throw new Error(`durability entitlement returned ${entitlementResponse.status}`);
  const entitlement = await entitlementResponse.json();
  const shareResponse = await fetch(`${origin}/api/share`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'x-workspace-id': workspace,
      'x-sociobot-license': entitlement.token,
      'x-forwarded-for': '203.0.113.212',
    },
    body: JSON.stringify({ document: entitlement.document }),
  });
  if (!shareResponse.ok) throw new Error(`durability share seed returned ${shareResponse.status}`);
  const share = await shareResponse.json();
  await writeFile(stateFile, JSON.stringify({ share_token: share.token, share_business: entitlement.document.businessName }), { mode: 0o600 });
}

const state = JSON.parse(await readFile(stateFile, 'utf8'));
const routedWorkspaceReads = await Promise.all(Array.from({ length: 30 }, (_, index) => fetch(`${origin}/api/workspace`, {
  headers: { ...headers, 'x-forwarded-for': `2001:db8:210::${index + 1}` },
})));
for (const response of routedWorkspaceReads) {
  if (!response.ok) throw new Error(`durability routed workspace read returned ${response.status}`);
  const result = await response.json();
  if (result.document?.transactions?.[0]?.description !== `Durability probe ${value}`) {
    throw new Error('durability probe was not restored on every routed request');
  }
}
const routedShareReads = await Promise.all(Array.from({ length: 30 }, (_, index) => fetch(`${origin}/api/share/${state.share_token}`, {
  headers: { 'x-forwarded-for': `2001:db8:211::${index + 1}` },
})));
for (const response of routedShareReads) {
  if (!response.ok) throw new Error(`durability routed accountant-link read returned ${response.status}`);
  const result = await response.json();
  if (result.document?.businessName !== state.share_business) {
    throw new Error('encrypted accountant link was not restored on every routed request');
  }
}
console.log(JSON.stringify({ mode, durable_workspace: true, encrypted_accountant_link: true, routed_reads: 60, value }));
