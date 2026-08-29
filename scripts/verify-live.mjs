import process from 'node:process';
import { execFileSync } from 'node:child_process';

const origin = process.env.VERIFY_ORIGIN || 'https://mtd-quarterly-ready.sociobot.in';
const billing = 'https://api.sociobot.in/api/v1/products';

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function response(path, init) {
  return fetch(new URL(path, origin), init);
}

const healthResponse = await response('/health');
assert(healthResponse.status === 200, `/health returned ${healthResponse.status}`);
const health = await healthResponse.json();
assert(health.status === 'ok', '/health did not report ok');
assert(health.safe_qa_fixtures === true, '/health reports SAFE_QA_FIXTURES is not enabled');
assert(typeof health.hmrc_integration_configured === 'boolean', '/health omitted the HMRC integration capability');
if (process.env.REQUIRE_APPROVED_HMRC === '1') {
  assert(health.hmrc_integration_configured === true, 'production has no approved HMRC integration configured');
}
if (process.env.EXPECTED_BUILD_SHA) {
  assert(health.build_sha === process.env.EXPECTED_BUILD_SHA, `/health reported ${health.build_sha}, expected ${process.env.EXPECTED_BUILD_SHA}`);
}

for (const [plan, slug] of [['monthly', 'mtd-quarterly-ready'], ['annual', 'mtd-quarterly-ready-annual']]) {
  const checkout = await fetch(`${billing}/${slug}/checkout`, { method: 'POST', headers: { accept: 'application/json' } });
  assert(checkout.status === 200, `${plan} checkout returned ${checkout.status}, expected a checkout URL`);
  const body = await checkout.json();
  const url = new URL(body.checkout_url || '');
  assert(url.protocol === 'https:' && url.hostname === 'checkout.dodopayments.com', `${plan} checkout did not return a Dodo HTTPS URL`);
}

const unknown = await response(`/release-regression-${Date.now()}`);
assert(unknown.status === 404, `unknown route returned ${unknown.status}`);
assert((await unknown.text()).includes('This page is not on the panel'), 'unknown route did not return the designed recovery page');

const workspaceId = crypto.randomUUID();
const empty = await response('/api/workspace', { headers: { 'x-workspace-id': workspaceId, 'x-forwarded-for': '203.0.113.241' } });
assert(empty.status === 200, `empty workspace returned ${empty.status}`);
assert((await empty.json()).document === null, 'empty workspace did not return document: null');
const durableDocument = {
  schemaVersion: 1, businessName: 'Live durability probe', quarterLabel: '6 April to 5 July 2026',
  quarterStart: '2026-04-06', quarterEnd: '2026-07-05', figuresReviewed: false, packDownloaded: false,
  markedReady: false, updatedAt: new Date().toISOString(),
  transactions: [{ id: workspaceId, date: '2026-04-09', description: `Live durability probe ${workspaceId}`, amountPence: 100, kind: 'income', category: 'Sales' }],
};
const saved = await response('/api/workspace', {
  method: 'PUT',
  headers: { 'content-type': 'application/json', 'x-workspace-id': workspaceId, 'x-forwarded-for': '203.0.113.241' },
  body: JSON.stringify({ document: durableDocument }),
});
assert(saved.status === 200, `workspace save returned ${saved.status}`);
const restored = await response('/api/workspace', { headers: { 'x-workspace-id': workspaceId, 'x-forwarded-for': '203.0.113.241' } });
assert(restored.status === 200, `saved workspace read returned ${restored.status}`);
assert((await restored.json()).document?.transactions?.[0]?.description === durableDocument.transactions[0].description, 'saved workspace was not restored');
const malformed = await response('/api/workspace', {
  method: 'PUT',
  headers: { 'content-type': 'application/json', 'x-workspace-id': crypto.randomUUID(), 'x-forwarded-for': '203.0.113.244' },
  body: JSON.stringify({ document: { ...durableDocument, transactions: [{ ...durableDocument.transactions[0], date: 'not-a-date' }] } }),
});
assert(malformed.status === 422, `malformed workspace transaction returned ${malformed.status}, expected 422`);

for (const [name, document] of [
  ['impossible quarter', { ...durableDocument, quarterStart: '2026-02-30' }],
  ['mismatched quarter', { ...durableDocument, quarterEnd: '2026-07-06' }],
  ['out-of-quarter row', { ...durableDocument, transactions: [{ ...durableDocument.transactions[0], date: '2026-07-06' }] }],
  ['zero row', { ...durableDocument, transactions: [{ ...durableDocument.transactions[0], amountPence: 0 }] }],
  ['unknown category row', { ...durableDocument, transactions: [{ ...durableDocument.transactions[0], category: 'Bananas' }] }],
]) {
  const invalid = await response('/api/workspace', {
    method: 'PUT',
    headers: { 'content-type': 'application/json', 'x-workspace-id': crypto.randomUUID(), 'x-forwarded-for': `203.0.113.${245 + name.length}` },
    body: JSON.stringify({ document }),
  });
  assert(invalid.status === 422, `${name} returned ${invalid.status}, expected 422`);
}

const fixture = await response('/api/qa/entitlement', { headers: { 'x-forwarded-for': '203.0.113.230' } });
assert(fixture.status === 200, `safe entitlement fixture returned ${fixture.status}`);
const safe = await fixture.json();
assert(safe.charges === false && safe.files_with_hmrc === false, 'safe fixture did not declare its non-charging, non-filing policy');
const fixtureHeaders = { 'content-type': 'application/json', 'x-workspace-id': crypto.randomUUID(), 'x-sociobot-license': safe.token, 'x-forwarded-for': '203.0.113.231' };
const fixtureShare = await response('/api/share', { method: 'POST', headers: fixtureHeaders, body: JSON.stringify({ document: safe.document }) });
assert(fixtureShare.status === 201, `safe fixture accountant link returned ${fixtureShare.status}`);
const fixtureSubmission = await response('/api/hmrc/submit', { method: 'POST', headers: { ...fixtureHeaders, 'x-forwarded-for': '203.0.113.232' }, body: JSON.stringify({ document: safe.document, review_confirmed: true }) });
assert(fixtureSubmission.status === 200, `safe fixture submission returned ${fixtureSubmission.status}`);
const fixtureSubmissionBody = await fixtureSubmission.json();
assert(fixtureSubmissionBody.status === 'fixture_only_no_filing' && fixtureSubmissionBody.submission_id.startsWith('safe-fixture-no-filing-'), 'safe fixture submission was not explicitly non-filing');

async function assertLimit(kind, allowance, clientIp) {
  const requests = Array.from({ length: allowance + 8 }, (_, index) => {
    const headers = { 'x-forwarded-for': clientIp };
    return kind === 'read'
      ? response(index % 2 ? '/api/share/not-a-token' : '/api/workspace', { headers })
      : response('/api/page-view', { method: 'POST', headers });
  });
  const results = await Promise.all(requests);
  const accepted = results.filter(item => item.status !== 429);
  const limited = results.filter(item => item.status === 429);
  assert(accepted.length === allowance, `${kind} limit accepted ${accepted.length}, expected ${allowance}`);
  assert(limited.length === 8, `${kind} limit returned ${limited.length} limited responses, expected 8`);
  assert(limited.every(item => item.headers.has('retry-after')), `${kind} 429 response omitted Retry-After`);
}

await assertLimit('read', 40, '203.0.113.242');
await assertLimit('write', 12, '203.0.113.243');

if (process.env.VERIFY_AZURE_TOPOLOGY === '1') {
  execFileSync('bash', ['scripts/verify-azure-topology.sh'], { stdio: 'inherit' });
}

console.log(JSON.stringify({ origin, build_sha: health.build_sha, checkout: ['monthly', 'annual'], durable_workspace: true, hmrc_integration_configured: health.hmrc_integration_configured, safe_paid_fixture: 'non-charging/non-filing', read_limit: 40, write_limit: 12, topology_verified: process.env.VERIFY_AZURE_TOPOLOGY === '1', status: 'ok' }));
