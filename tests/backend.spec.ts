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

test('rate limiting returns 429 with Retry-After', async ({ request }) => {
  const responses = await Promise.all(Array.from({ length: 48 }, () => request.get('/missing-asset.txt', { headers: { 'x-forwarded-for': '203.0.113.99' } })));
  const limited = responses.find(response => response.status() === 429);
  expect(limited).toBeTruthy();
  expect(limited?.headers()['retry-after']).toBe('1');
});
