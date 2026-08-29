import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

const validDocument = {
  schemaVersion: 1, businessName: 'Maya Patel Tutoring', quarterLabel: '6 April to 5 July 2026',
  quarterStart: '2026-04-06', quarterEnd: '2026-07-05', figuresReviewed: false, packDownloaded: false,
  markedReady: false, updatedAt: '2026-06-28T10:30:00.000Z',
  transactions: [{ id: 'test-lesson', date: '2026-04-09', description: 'Test lesson', amountPence: 4500, kind: 'income', category: 'Sales' }],
};

test('health reports the build identity', async ({ request }) => {
  const response = await request.get('/health');
  expect(response.status()).toBe(200);
  expect(await response.json()).toMatchObject({
    status: 'ok',
    build_sha: process.env.EXPECTED_BUILD_SHA || 'dev',
    safe_qa_fixtures: true,
  });
});

test('@regression:deployed-safe-qa-runtime is enabled and observable through health', async ({ request }) => {
  const health = await (await request.get('/health')).json();
  expect(health.safe_qa_fixtures).toBe(true);
  const fixture = await request.get('/api/qa/entitlement', {
    headers: { 'x-forwarded-for': '203.0.113.16' },
  });
  expect(fixture.status()).toBe(200);
  expect(await fixture.json()).toMatchObject({ charges: false, files_with_hmrc: false });
});

test('workspace endpoints save and return an encrypted document', async ({ request }) => {
  const id = '9735ee38-13fe-4a21-985b-96a32a720cef';
  const headers = { 'x-workspace-id': id, 'x-forwarded-for': '203.0.113.20' };
  expect((await request.put('/api/workspace', { headers, data: { document: validDocument } })).status()).toBe(200);
  const result = await (await request.get('/api/workspace', { headers })).json();
  expect(result.document.transactions[0].description).toBe('Test lesson');
});

test('@regression:concurrent-workspace-saves-are-readable-before-the-success-response-is-trusted', async ({ request }) => {
  const probes = await Promise.all(Array.from({ length: 10 }, async (_, index) => {
    const id = crypto.randomUUID();
    const description = `Concurrent durable workspace ${index} ${id}`;
    const headers = { 'x-workspace-id': id, 'x-forwarded-for': `198.51.100.${index + 1}` };
    const document = {
      ...validDocument,
      updatedAt: new Date().toISOString(),
      transactions: [{ ...validDocument.transactions[0], id, description, amountPence: index + 1 }],
    };
    const saved = await request.put('/api/workspace', { headers, data: { document } });
    const restored = await request.get('/api/workspace', { headers });
    return { id, description, saved, restored };
  }));

  for (const probe of probes) {
    expect(probe.saved.status(), `save for ${probe.id}`).toBe(200);
    expect(probe.restored.status(), `read for ${probe.id}`).toBe(200);
    expect((await probe.restored.json()).document.transactions[0].description, `document for ${probe.id}`).toBe(probe.description);
  }
});

test('@regression:workspace-rejects-malformed-transaction-objects before persistence', async ({ request }) => {
  const headers = { 'x-workspace-id': 'd735ee38-13fe-4a21-985b-96a32a720cef', 'x-forwarded-for': '203.0.113.22' };
  const malformed = [
    { ...validDocument.transactions[0], date: '2026-02-30' },
    { ...validDocument.transactions[0], description: '' },
    { ...validDocument.transactions[0], amountPence: 0 },
    { ...validDocument.transactions[0], kind: 'transfer' },
    { ...validDocument.transactions[0], category: 'Uncategorised' },
    { ...validDocument.transactions[0], date: '2026-07-06' },
    { ...validDocument.transactions[0], receiptData: 'data:text/plain;base64,SGVsbG8=' },
  ];
  for (const transaction of malformed) {
    const response = await request.put('/api/workspace', { headers, data: { document: { ...validDocument, transactions: [transaction] } } });
    expect(response.status()).toBe(422);
  }
  expect((await request.get('/api/workspace', { headers })).status()).toBe(200);
  expect(await (await request.get('/api/workspace', { headers: { ...headers, 'x-forwarded-for': '203.0.113.23' } })).json()).toEqual({ document: null });
});

test('@regression:workspace-rejects-invalid-quarter-boundaries atomically', async ({ request }) => {
  const headers = { 'x-workspace-id': 'f735ee38-13fe-4a21-985b-96a32a720cef', 'x-forwarded-for': '203.0.113.24' };
  for (const document of [
    { ...validDocument, quarterStart: '2026-02-30' },
    { ...validDocument, quarterEnd: '2026-07-06' },
  ]) {
    expect((await request.put('/api/workspace', { headers, data: { document } })).status()).toBe(422);
  }
  expect(await (await request.get('/api/workspace', { headers })).json()).toEqual({ document: null });
});

test('@regression:empty-workspace returns a successful empty document', async ({ request }) => {
  const response = await request.get('/api/workspace', {
    headers: {
      'x-workspace-id': '8735ee38-13fe-4a21-985b-96a32a720cef',
      'x-forwarded-for': '203.0.113.21',
    },
  });
  expect(response.status()).toBe(200);
  expect(await response.json()).toEqual({ document: null });
});

test('@claim:server-licence-gate @regression:unauthenticated-share cannot create a live accountant link without a server-verified Sociobot subscription', async ({ request }) => {
  const response = await request.post('/api/share', {
    headers: { 'x-workspace-id': '15aa583d-84cf-43f1-8438-354ddbfd6358', 'x-forwarded-for': '203.0.113.10' },
    data: { document: { transactions: [] } }
  });
  expect(response.status()).toBe(402);
  expect(await response.json()).toEqual({ error: 'An active Sociobot subscription is required for live accountant links and HMRC submissions.' });
});

test('@regression:submission-needs-human-review refuses an unreviewed submission before contacting any integration', async ({ request }) => {
  const response = await request.post('/api/hmrc/submit', {
    headers: { 'x-workspace-id': '25aa583d-84cf-43f1-8438-354ddbfd6358', 'x-forwarded-for': '203.0.113.12' },
    data: { document: { transactions: [] }, review_confirmed: false }
  });
  expect(response.status()).toBe(422);
  expect(await response.json()).toEqual({ error: 'Confirm that you reviewed the totals before submitting to HMRC.' });
});

test('@regression:safe-paid-fixture proves share and submission paths without charging or filing', async ({ request }) => {
  const fixtureResponse = await request.get('/api/qa/entitlement', { headers: { 'x-forwarded-for': '203.0.113.13' } });
  expect(fixtureResponse.status()).toBe(200);
  const fixture = await fixtureResponse.json();
  expect(fixture).toMatchObject({ charges: false, files_with_hmrc: false });
  const headers = {
    'x-workspace-id': '35aa583d-84cf-43f1-8438-354ddbfd6358',
    'x-sociobot-license': fixture.token,
    'x-forwarded-for': '203.0.113.14',
  };
  const share = await request.post('/api/share', { headers, data: { document: fixture.document } });
  expect(share.status()).toBe(201);
  const submission = await request.post('/api/hmrc/submit', {
    headers: { ...headers, 'x-forwarded-for': '203.0.113.15' },
    data: { document: fixture.document, review_confirmed: true },
  });
  expect(submission.status()).toBe(200);
  expect(await submission.json()).toMatchObject({
    status: 'fixture_only_no_filing',
    files_with_hmrc: false,
  });
});

test('@regression:shared-read-limit allows 40 reads across routes then returns 429 with Retry-After', async ({ request }) => {
  if (process.env.VERIFY_ORIGIN) {
    const result = JSON.parse(execFileSync(process.execPath, ['scripts/verify-rate-limit.mjs', '--kind', 'read'], {
      cwd: process.cwd(),
      encoding: 'utf8',
      env: process.env,
    }).trim());
    expect(result).toMatchObject({ status: 'ok', read: { allowance: 40, first_limited_request: 41, stable_keep_alive_connection: true, paced_beyond_previous_one_second_window: true } });
    expect(Number.parseInt(result.read.retry_after, 10)).toBeGreaterThan(0);
    return;
  }
  const headers = { 'x-forwarded-for': '203.0.113.99' };
  const responses = [];
  for (let index = 0; index < 41; index += 1) {
    responses.push(index % 2 === 0
      ? await request.get('/api/workspace', { headers })
      : await request.get('/api/share/not-a-token', { headers }));
    if (index < 40) await new Promise(resolve => setTimeout(resolve, 30));
  }
  expect(responses.slice(0, 40).every(response => response.status() !== 429)).toBe(true);
  expect(responses[40].status()).toBe(429);
  expect(Number.parseInt(responses[40].headers()['retry-after'], 10)).toBeGreaterThan(0);
});

test('static files never consume the API rate allowance', async ({ request }) => {
  const responses = await Promise.all(Array.from({ length: 48 }, () => request.get('/favicon.svg', { headers: { 'x-forwarded-for': '203.0.113.97' } })));
  expect(responses.every(response => response.status() === 200)).toBe(true);
});

test('@regression:shared-write-limit allows 12 writes then returns 429 with Retry-After', async ({ request }) => {
  if (process.env.VERIFY_ORIGIN) {
    const result = JSON.parse(execFileSync(process.execPath, ['scripts/verify-rate-limit.mjs', '--kind', 'write'], {
      cwd: process.cwd(),
      encoding: 'utf8',
      env: process.env,
    }).trim());
    expect(result).toMatchObject({ status: 'ok', write: { allowance: 12, first_limited_request: 13, stable_keep_alive_connection: true, paced_beyond_previous_one_second_window: true } });
    expect(Number.parseInt(result.write.retry_after, 10)).toBeGreaterThan(0);
    return;
  }
  const headers = { 'x-forwarded-for': '203.0.113.98' };
  const responses = [];
  for (let index = 0; index < 13; index += 1) {
    responses.push(await request.post('/api/page-view', { headers }));
    if (index < 12) await new Promise(resolve => setTimeout(resolve, 100));
  }
  expect(responses.slice(0, 12).every(response => response.status() === 204)).toBe(true);
  expect(responses[12].status()).toBe(429);
  expect(Number.parseInt(responses[12].headers()['retry-after'], 10)).toBeGreaterThan(0);
});

test('@regression:OAuth callback shares the stricter write quota', async ({ request }) => {
  const headers = { 'x-forwarded-for': '203.0.113.96' };
  for (let index = 0; index < 12; index += 1) {
    expect((await request.post('/api/page-view', { headers })).status()).toBe(204);
  }
  const callback = await request.get('/api/hmrc/consent/callback?state=missing', { headers });
  expect(callback.status()).toBe(429);
  expect(Number.parseInt(callback.headers()['retry-after'], 10)).toBeGreaterThan(0);
});

test('@claim:api-rate-limit enforces both paced burst allowances on a stable client connection', async () => {
  const result = JSON.parse(execFileSync(process.execPath, ['scripts/verify-rate-limit.mjs'], {
    cwd: process.cwd(),
    encoding: 'utf8',
    env: { ...process.env, VERIFY_ORIGIN: process.env.VERIFY_ORIGIN || 'http://127.0.0.1:4173' },
  }).trim());
  expect(result).toMatchObject({
    status: 'ok',
    read: { allowance: 40, first_limited_request: 41, paced_beyond_previous_one_second_window: true },
    write: { allowance: 12, first_limited_request: 13, paced_beyond_previous_one_second_window: true },
  });
  expect(Number.parseInt(result.read.retry_after, 10)).toBeGreaterThan(0);
  expect(Number.parseInt(result.write.retry_after, 10)).toBeGreaterThan(0);
});

test('@regression:anonymous-page-view-fallback separates browser sessions while retaining each session limit', async ({ request }) => {
  test.skip(Boolean(process.env.VERIFY_ORIGIN), 'The public ingress supplies X-Forwarded-For, so the direct-origin fallback header is intentionally not used.');
  const firstBrowser = '0d0bde02-f4f3-46db-9b6d-08f10f1b48c1';
  const secondBrowser = '4185d12f-873b-4882-bdb6-c302cb694ef1';
  for (let index = 0; index < 12; index += 1) {
    expect((await request.post('/api/page-view', { headers: { 'x-quarterly-ready-client': firstBrowser } })).status()).toBe(204);
  }
  expect((await request.post('/api/page-view', { headers: { 'x-quarterly-ready-client': firstBrowser } })).status()).toBe(429);
  expect((await request.post('/api/page-view', { headers: { 'x-quarterly-ready-client': secondBrowser } })).status()).toBe(204);
});

test('@regression:unknown-route returns the designed page with a genuine 404', async ({ request }) => {
  const response = await request.get('/not-a-quarterly-ready-route');
  expect(response.status()).toBe(404);
  expect(await response.text()).toContain('This page is not on the panel');
});

test('@regression:response-policy protects HTML and service-worker responses', async ({ request }) => {
  const page = await request.get('/');
  expect(page.status()).toBe(200);
  expect(page.headers()).toMatchObject({
    'x-content-type-options': 'nosniff',
    'referrer-policy': 'strict-origin-when-cross-origin',
    'permissions-policy': 'camera=(), microphone=(), geolocation=()',
    'cache-control': 'no-cache',
  });
  expect(page.headers()['content-security-policy']).toContain("frame-ancestors 'none'");
  expect(page.headers()['content-security-policy']).toContain("connect-src 'self' https://api.sociobot.in");

  const serviceWorker = await request.get('/sw.js');
  expect(serviceWorker.status()).toBe(200);
  expect(serviceWorker.headers()['cache-control']).toBe('no-cache');
});
