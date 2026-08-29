import process from 'node:process';

const origin = process.env.VERIFY_ORIGIN || 'https://mtd-quarterly-ready.sociobot.in';
const value = process.env.DURABILITY_PROBE_VALUE;
const mode = process.argv[2];
const workspace = '00000000-0000-4000-8000-000000000010';
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
}

const response = await fetch(`${origin}/api/workspace`, { headers });
if (!response.ok) throw new Error(`durability read returned ${response.status}`);
const result = await response.json();
if (result.document?.transactions?.[0]?.description !== `Durability probe ${value}`) {
  throw new Error('durability probe was not restored');
}
console.log(JSON.stringify({ mode, durable_workspace: true, value }));
