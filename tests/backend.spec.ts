import { expect, test } from '@playwright/test';

test('health reports the build identity', async ({ request }) => {
  const response = await request.get('/health');
  expect(response.status()).toBe(200);
  expect(await response.json()).toMatchObject({ status: 'ok', build_sha: 'dev' });
});

test('workspace endpoints save and return an encrypted document', async ({ request }) => {
  const id = '9735ee38-13fe-4a21-985b-96a32a720cef';
  const headers = { 'x-workspace-id': id, 'x-forwarded-for': '203.0.113.20' };
  expect((await request.put('/api/workspace', { headers, data: { document: { transactions: [{ description: 'Test lesson' }] } } })).status()).toBe(200);
  const result = await (await request.get('/api/workspace', { headers })).json();
  expect(result.document.transactions[0].description).toBe('Test lesson');
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

test('rate limiting returns 429 with Retry-After', async ({ request }) => {
  const responses = await Promise.all(Array.from({ length: 48 }, () => request.get('/api/workspace', { headers: { 'x-forwarded-for': '203.0.113.99' } })));
  const limited = responses.find(response => response.status() === 429);
  expect(limited).toBeTruthy();
  expect(limited?.headers()['retry-after']).toBe('1');
});

test('static files never consume the API rate allowance', async ({ request }) => {
  const responses = await Promise.all(Array.from({ length: 48 }, () => request.get('/favicon.svg', { headers: { 'x-forwarded-for': '203.0.113.97' } })));
  expect(responses.every(response => response.status() === 200)).toBe(true);
});

test('write endpoints use the stricter allowance', async ({ request }) => {
  const responses = await Promise.all(Array.from({ length: 16 }, () => request.post('/api/page-view', { headers: { 'x-forwarded-for': '203.0.113.98' } })));
  const limited = responses.find(response => response.status() === 429);
  expect(limited).toBeTruthy();
  expect(limited?.headers()['retry-after']).toBe('1');
});
