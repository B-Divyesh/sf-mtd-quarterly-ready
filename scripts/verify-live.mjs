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
assert(typeof health.hmrc_integration_mode === 'string', '/health omitted the HMRC integration mode');
if (process.env.REQUIRE_APPROVED_HMRC === '1') {
  assert(health.hmrc_integration_configured === true, 'production has no approved HMRC integration configured');
  assert(health.hmrc_integration_mode === 'approved_provider', `production HMRC mode is ${health.hmrc_integration_mode}, expected approved_provider`);
  assert(health.hmrc_taxpayer_consent_required === true, 'production approved HMRC integration has no taxpayer-consent flow');
  assert(typeof health.hmrc_provider_name === 'string' && health.hmrc_provider_name.trim().length > 0, 'production approved HMRC integration did not identify its provider');
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
let taxpayerConsentFlow = 'not_required';
if (process.env.REQUIRE_APPROVED_HMRC === '1') {
  const consentHeaders = { 'x-workspace-id': workspaceId, 'x-forwarded-for': '203.0.113.240' };
  const beforeConsent = await response('/api/hmrc/consent', { headers: consentHeaders });
  assert(beforeConsent.status === 200, `taxpayer consent status returned ${beforeConsent.status}`);
  assert((await beforeConsent.json()).consented === false, 'fresh live verifier workspace unexpectedly has taxpayer consent');
  const consentStart = await response('/api/hmrc/consent', { method: 'POST', headers: consentHeaders });
  assert(consentStart.status === 200, `taxpayer consent start returned ${consentStart.status}`);
  const consent = await consentStart.json();
  const authorizationUrl = new URL(consent.authorization_url || '');
  assert(authorizationUrl.protocol === 'https:', 'taxpayer consent did not return an HTTPS provider authorization URL');
  assert(authorizationUrl.searchParams.get('response_type') === 'code', 'taxpayer consent did not use OAuth authorization-code flow');
  assert(authorizationUrl.searchParams.get('state'), 'taxpayer consent did not return a one-time OAuth state');
  taxpayerConsentFlow = 'oauth-authorize-url-issued';
}
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
assert(['fixture_only_no_filing', 'sandbox_accepted_no_filing'].includes(fixtureSubmissionBody.status), 'safe fixture submission was not explicitly non-filing');
assert(fixtureSubmissionBody.files_with_hmrc === false, 'safe submission did not prove that it files nothing with HMRC');

const rateLimitEvidence = JSON.parse(execFileSync(process.execPath, ['scripts/verify-rate-limit.mjs'], {
  cwd: new URL('..', import.meta.url),
  encoding: 'utf8',
  env: { ...process.env, VERIFY_ORIGIN: origin },
}).trim());
assert(rateLimitEvidence.status === 'ok', 'stable-connection rate-limit verification failed');

if (process.env.VERIFY_AZURE_TOPOLOGY === '1') {
  execFileSync('bash', ['scripts/verify-azure-topology.sh'], { stdio: 'inherit' });
}

console.log(JSON.stringify({ origin, build_sha: health.build_sha, checkout: ['monthly', 'annual'], durable_workspace: true, hmrc_integration_configured: health.hmrc_integration_configured, hmrc_integration_mode: health.hmrc_integration_mode, taxpayer_consent_flow: taxpayerConsentFlow, safe_paid_fixture: 'non-charging/non-filing', read_limit: rateLimitEvidence.read.allowance, write_limit: rateLimitEvidence.write.allowance, stable_rate_limit_connection: true, topology_verified: process.env.VERIFY_AZURE_TOPOLOGY === '1', status: 'ok' }));
