import { expect, test } from '@playwright/test';

const validDocument = {
  schemaVersion: 1, businessName: 'Maya Patel Tutoring', quarterLabel: '6 April to 5 July 2026',
  quarterStart: '2026-04-06', quarterEnd: '2026-07-05', figuresReviewed: false, packDownloaded: false,
  markedReady: false, updatedAt: '2026-06-28T10:30:00.000Z',
  transactions: [{ id: 'test-lesson', date: '2026-04-09', description: 'Test lesson', amountPence: 4500, kind: 'income', category: 'Sales' }],
};

test('health reports the build identity', async ({ request }) => {
  const response = await request.get('/health');
  expect(response.status()).toBe(200);
  expect(await response.json()).toMatchObject({ status: 'ok', build_sha: 'dev' });
});

test('workspace endpoints save and return an encrypted document', async ({ request }) => {
  const id = '9735ee38-13fe-4a21-985b-96a32a720cef';
  const headers = { 'x-workspace-id': id, 'x-forwarded-for': '203.0.113.20' };
  expect((await request.put('/api/workspace', { headers, data: { document: validDocument } })).status()).toBe(200);
  const result = await (await request.get('/api/workspace', { headers })).json();
  expect(result.document.transactions[0].description).toBe('Test lesson');
});

test('@regression:workspace-rejects-malformed-transaction-objects before persistence', async ({ request }) => {
  const headers = { 'x-workspace-id': 'd735ee38-13fe-4a21-985b-96a32a720cef', 'x-forwarded-for': '203.0.113.22' };
  const malformed = [
    { ...validDocument.transactions[0], date: '2026-02-30' },
    { ...validDocument.transactions[0], description: '' },
    { ...validDocument.transactions[0], amountPence: 0 },
    { ...validDocument.transactions[0], kind: 'transfer' },
    { ...validDocument.transactions[0], category: 'Uncategorised' },
    { ...validDocument.transactions[0], receiptData: 'data:text/plain;base64,SGVsbG8=' },
  ];
  for (const transaction of malformed) {
    const response = await request.put('/api/workspace', { headers, data: { document: { ...validDocument, transactions: [transaction] } } });
    expect(response.status()).toBe(422);
  }
  expect((await request.get('/api/workspace', { headers })).status()).toBe(200);
  expect(await (await request.get('/api/workspace', { headers: { ...headers, 'x-forwarded-for': '203.0.113.23' } })).json()).toEqual({ document: null });
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

test('@regression:shared-read-limit allows 40 reads across routes then returns 429 with Retry-After', async ({ request }) => {
  const headers = { 'x-forwarded-for': '203.0.113.99' };
  const responses = [];
  for (let index = 0; index < 41; index += 1) {
    responses.push(index % 2 === 0
      ? await request.get('/api/workspace', { headers })
      : await request.get('/api/share/not-a-token', { headers }));
  }
  expect(responses.slice(0, 40).every(response => response.status() !== 429)).toBe(true);
  expect(responses[40].status()).toBe(429);
  expect(responses[40].headers()['retry-after']).toBe('1');
});

test('static files never consume the API rate allowance', async ({ request }) => {
  const responses = await Promise.all(Array.from({ length: 48 }, () => request.get('/favicon.svg', { headers: { 'x-forwarded-for': '203.0.113.97' } })));
  expect(responses.every(response => response.status() === 200)).toBe(true);
});

test('@regression:shared-write-limit allows 12 writes then returns 429 with Retry-After', async ({ request }) => {
  const headers = { 'x-forwarded-for': '203.0.113.98' };
  const responses = [];
  for (let index = 0; index < 13; index += 1) responses.push(await request.post('/api/page-view', { headers }));
  expect(responses.slice(0, 12).every(response => response.status() === 204)).toBe(true);
  expect(responses[12].status()).toBe(429);
  expect(responses[12].headers()['retry-after']).toBe('1');
});

test('@regression:unknown-route returns the designed page with a genuine 404', async ({ request }) => {
  const response = await request.get('/not-a-quarterly-ready-route');
  expect(response.status()).toBe(404);
  expect(await response.text()).toContain('This page is not on the panel');
});
