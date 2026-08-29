import process from 'node:process';
import { readFile, writeFile } from 'node:fs/promises';

const origin = process.env.VERIFY_ORIGIN || 'https://mtd-quarterly-ready.sociobot.in';
const value = process.env.DURABILITY_PROBE_VALUE;
const mode = process.argv[2];
const stateFile = process.env.DURABILITY_PROBE_STATE_FILE || '/tmp/quarterly-ready-durability-probe.json';
if (!value || !['seed', 'check'].includes(mode)) throw new Error('Use seed or check and set DURABILITY_PROBE_VALUE.');

function documentFor(index) {
  const workspace = `00000000-0000-4000-8000-${String(index + 10).padStart(12, '0')}`;
  const marker = `Durability probe ${value} concurrent workspace ${index}`;
  return { workspace, marker, document: {
  schemaVersion: 1,
  businessName: 'Durable storage release probe',
  quarterLabel: '6 April to 5 July 2026',
  quarterStart: '2026-04-06',
  quarterEnd: '2026-07-05',
  figuresReviewed: false,
  packDownloaded: false,
  markedReady: false,
  updatedAt: new Date().toISOString(),
  transactions: [{ id: `durability-probe-${index}`, date: '2026-04-06', description: marker, amountPence: index + 1, kind: 'income', category: 'Sales' }],
  }};
}

async function readWorkspace(probe, suffix) {
  const response = await fetch(`${origin}/api/workspace`, {
    headers: { 'x-workspace-id': probe.workspace, 'x-forwarded-for': `2001:db8:210::${suffix}` },
  });
  if (!response.ok) throw new Error(`durability workspace read returned ${response.status}`);
  const result = await response.json();
  if (result.document?.transactions?.[0]?.description !== probe.marker) {
    throw new Error(`durability workspace ${probe.workspace} was not restored`);
  }
}

if (mode === 'seed') {
  const probes = Array.from({ length: 10 }, (_, index) => documentFor(index));
  const saves = await Promise.all(probes.map((probe, index) => fetch(`${origin}/api/workspace`, {
    method: 'PUT',
    headers: { 'content-type': 'application/json', 'x-workspace-id': probe.workspace, 'x-forwarded-for': `203.0.113.${index + 1}` },
    body: JSON.stringify({ document: probe.document }),
  })));
  for (const response of saves) if (!response.ok) throw new Error(`concurrent durability seed returned ${response.status}`);
  await Promise.all(probes.map((probe, index) => readWorkspace(probe, index + 1)));
  const entitlementResponse = await fetch(`${origin}/api/qa/entitlement`, { headers: { 'x-forwarded-for': '203.0.113.211' } });
  if (!entitlementResponse.ok) throw new Error(`durability entitlement returned ${entitlementResponse.status}`);
  const entitlement = await entitlementResponse.json();
  const shareResponse = await fetch(`${origin}/api/share`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'x-workspace-id': probes[0].workspace,
      'x-sociobot-license': entitlement.token,
      'x-forwarded-for': '203.0.113.212',
    },
    body: JSON.stringify({ document: entitlement.document }),
  });
  if (!shareResponse.ok) throw new Error(`durability share seed returned ${shareResponse.status}`);
  const share = await shareResponse.json();
  await writeFile(stateFile, JSON.stringify({ probes, share_token: share.token, share_business: entitlement.document.businessName }), { mode: 0o600 });
}

const state = JSON.parse(await readFile(stateFile, 'utf8'));
await Promise.all(state.probes.flatMap((probe, index) => [
  readWorkspace(probe, index + 1),
  readWorkspace(probe, index + 101),
  readWorkspace(probe, index + 201),
]));
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
console.log(JSON.stringify({ mode, concurrent_workspaces: state.probes.length, durable_workspace: true, encrypted_accountant_link: true, routed_reads: 60, value }));
