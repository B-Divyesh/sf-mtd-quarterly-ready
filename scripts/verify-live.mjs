import process from 'node:process';

const origin = process.env.VERIFY_ORIGIN || 'https://mtd-quarterly-ready.sociobot.in';
const billing = 'https://api.sociobot.in/api/v1/products/mtd-quarterly-ready';

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
if (process.env.EXPECTED_BUILD_SHA) {
  assert(health.build_sha === process.env.EXPECTED_BUILD_SHA, `/health reported ${health.build_sha}, expected ${process.env.EXPECTED_BUILD_SHA}`);
}

for (const plan of ['monthly', 'annual']) {
  const checkout = await fetch(`${billing}/checkout?plan=${plan}`, { redirect: 'manual' });
  assert([301, 302, 303, 307, 308].includes(checkout.status), `${plan} checkout returned ${checkout.status}, expected a hosted-checkout redirect`);
  const location = checkout.headers.get('location');
  assert(location?.startsWith('https://'), `${plan} checkout did not return an HTTPS location`);
}

const unknown = await response(`/release-regression-${Date.now()}`);
assert(unknown.status === 404, `unknown route returned ${unknown.status}`);
assert((await unknown.text()).includes('This page is not on the panel'), 'unknown route did not return the designed recovery page');

const workspaceId = crypto.randomUUID();
const empty = await response('/api/workspace', { headers: { 'x-workspace-id': workspaceId, 'x-forwarded-for': '203.0.113.241' } });
assert(empty.status === 200, `empty workspace returned ${empty.status}`);
assert((await empty.json()).document === null, 'empty workspace did not return document: null');

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

console.log(JSON.stringify({ origin, build_sha: health.build_sha, checkout: ['monthly', 'annual'], read_limit: 40, write_limit: 12, status: 'ok' }));
