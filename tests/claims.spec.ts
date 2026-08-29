import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';

test('@claim:demo-isolation @claim:demo-access @claim:privacy-no-tracking keeps sample changes separate from real records', async ({ page, context }) => {
  const outgoing: string[] = [];
  page.on('request', request => outgoing.push(request.url()));
  await page.goto('/demo');
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  const row = page.locator('tr', { hasText: 'Bank transfer from J. Clarke' });
  await row.locator('select').selectOption('Sales');
  const keys = await page.evaluate(() => Object.keys(localStorage));
  expect(keys).toContain('demo:quarterly-ready:document');
  expect(keys).not.toContain('quarterly-ready:document');
  expect(outgoing.every(url => new URL(url).origin === 'http://127.0.0.1:4173')).toBe(true);
  expect(await context.cookies()).toEqual([]);
});

test('@claim:accountant-csv exports every sample transaction', async ({ page }) => {
  await page.goto('/demo');
  const download = await Promise.all([page.waitForEvent('download'), page.getByRole('button', { name: 'Download accountant CSV' }).click()]);
  const contents = await download[0].createReadStream().then(async stream => { let text = ''; for await (const chunk of stream) text += chunk.toString(); return text; });
  expect(contents).toContain('Maya Patel Tutoring');
  expect(contents).toContain('Bank transfer from J. Clarke');
  expect(contents).toContain('Whiteboard markers');
  expect(contents.split('\r\n')).toHaveLength(19);
});

test('@claim:quarter-review shows totals and resolves the outstanding category', async ({ page }) => {
  await page.goto('/demo');
  await expect(page.getByText('£260.00', { exact: true })).toBeVisible();
  await expect(page.getByText('£155.83', { exact: true })).toBeVisible();
  await expect(page.getByText('1 transaction needs a category')).toBeVisible();
  await page.locator('tr', { hasText: 'Bank transfer from J. Clarke' }).locator('select').selectOption('Sales');
  await expect(page.getByText('Every transaction has a category').first()).toBeVisible();
});

test('@claim:csv-import imports a bank CSV into the quarter', async ({ page }) => {
  await page.goto('/demo');
  await page.locator('#csv-input').setInputFiles({ name: 'bank.csv', mimeType: 'text/csv', buffer: Buffer.from('date,description,amount,type,category\n2026-07-01,Revision lesson,55.00,income,Sales') });
  await expect(page.getByText('1 transactions imported.')).toBeVisible();
  await expect(page.getByText('Revision lesson', { exact: true })).toBeVisible();
});

test('@regression:csv-invalid-rows-are-atomic rejects impossible, out-of-quarter, zero, and unknown-category rows without changing totals', async ({ page }) => {
  await page.goto('/demo');
  const originalRows = await page.locator('tbody tr').count();
  const originalIncome = await page.getByText('£260.00', { exact: true }).first().textContent();
  const cases = [
    ['impossible.csv', 'date,description,amount,type,category\n2026-04-10,Valid lesson,25,income,Sales\n2026-02-30,Impossible,20,income,Sales', 'Row 3: The date is not a real calendar date'],
    ['outside.csv', 'date,description,amount,type,category\n2026-07-06,Outside quarter,20,income,Sales', 'between 2026-04-06 and 2026-07-05'],
    ['zero.csv', 'date,description,amount,type,category\n2026-04-10,Zero,0,income,Sales', 'between £0.01 and £1,000,000'],
    ['category.csv', 'date,description,amount,type,category\n2026-04-10,Unknown,20,expense,Bananas', 'unknown category'],
  ];
  for (const [name, csv, message] of cases) {
    await page.locator('#csv-input').setInputFiles({ name, mimeType: 'text/csv', buffer: Buffer.from(csv) });
    await expect(page.getByText(new RegExp(message.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i'))).toBeVisible();
    expect(await page.locator('tbody tr').count()).toBe(originalRows);
    await expect(page.getByText(originalIncome || '', { exact: true }).first()).toBeVisible();
  }
});

test('@claim:free-quarter-persistence @regression:current-and-future-quarters remain separate across reloads', async ({ page }) => {
  await page.goto('/records');
  const selector = page.getByLabel('Working quarter');
  const currentStart = await selector.inputValue();
  const currentDate = await page.locator('#add-form input[name="date"]').getAttribute('min');
  await page.getByRole('button', { name: 'Add a transaction' }).click();
  await page.locator('#add-form input[name="date"]').fill(currentDate!);
  await page.locator('#add-form input[name="description"]').fill('Current-quarter lesson');
  await page.locator('#add-form input[name="amount"]').fill('75.00');
  await page.locator('#add-form select[name="category"]').selectOption('Sales');
  await page.getByRole('button', { name: 'Save transaction' }).click();
  await page.reload();
  await expect(page.getByText('Current-quarter lesson', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Create next quarter' }).click();
  const nextStart = await page.getByLabel('Working quarter').inputValue();
  expect(nextStart).not.toBe(currentStart);
  await expect(page.getByRole('heading', { level: 3, name: 'No transactions in this quarter' })).toBeVisible();
  await page.getByLabel('Working quarter').selectOption(currentStart);
  await expect(page.getByText('Current-quarter lesson', { exact: true })).toBeVisible();
  const keys = await page.evaluate(() => Object.keys(localStorage));
  expect(keys).toContain(`quarterly-ready:document:${currentStart}`);
  expect(keys).toContain(`quarterly-ready:document:${nextStart}`);
});

test('@claim:receipt-capture attaches a receipt to an existing expense', async ({ page }) => {
  await page.goto('/demo');
  const row = page.locator('tr', { hasText: 'Whiteboard markers' });
  await row.locator('[data-receipt]').setInputFiles({ name: 'markers-receipt.pdf', mimeType: 'application/pdf', buffer: Buffer.from('%PDF-1.4 sample receipt') });
  await expect(page.getByText('Receipt attached.')).toBeVisible();
  await expect(page.locator('tr', { hasText: 'Whiteboard markers' }).getByText('Receipt · markers-receipt.pdf')).toBeVisible();
  const stored = await page.evaluate(() => localStorage.getItem('demo:quarterly-ready:document'));
  expect(stored).toContain('data:application/pdf;base64');
});

test('@claim:hmrc-handoff creates reviewed period totals for recognised software', async ({ page }) => {
  await page.goto('/demo');
  await page.locator('tr', { hasText: 'Bank transfer from J. Clarke' }).locator('select').selectOption('Sales');
  await page.getByLabel('I checked these figures').check();
  const download = await Promise.all([page.waitForEvent('download'), page.getByRole('button', { name: 'Download HMRC handoff' }).click()]);
  const contents = await download[0].createReadStream().then(async stream => { let text = ''; for await (const chunk of stream) text += chunk.toString(); return text; });
  const handoff = JSON.parse(contents);
  expect(handoff).toMatchObject({ format: 'quarterly-ready-mtd-itsa-handoff-v1', periodStartDate: '2026-04-06', periodEndDate: '2026-07-05', reviewedByUser: true });
  expect(handoff.periodIncome.turnover).toBe(260);
});

test('@claim:accountant-link opens a read-only sample pack', async ({ page }) => {
  await page.goto('/demo');
  await page.getByRole('button', { name: 'Make accountant link' }).click();
  const link = page.locator('#output-result a');
  await expect(link).toHaveAttribute('href', /\/share\/demo$/);
  await link.click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Review this accountant pack');
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Reset demo' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Start for real' })).toBeVisible();
  await expect(page.getByText('Maya Patel Tutoring')).toBeVisible();
  await expect(page.getByRole('button', { name: /delete/i })).toHaveCount(0);
});

test('@claim:offline-browser-copy reloads the demo after the network is disabled', async ({ page, context }) => {
  await page.goto('/demo');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.getByText('Offline — browser copy active')).toBeVisible();
  await expect(page.getByText('Maya Patel Tutoring')).toBeVisible();
});

test('@claim:paid-tier uses Sociobot subscription checkout and keeps CSV free', async ({ page }) => {
  const checkoutRequests: { url: string; method: string }[] = [];
  await page.route('https://api.sociobot.in/api/v1/products/*/checkout', async route => {
    checkoutRequests.push({ url: route.request().url(), method: route.request().method() });
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ checkout_url: 'https://checkout.dodopayments.com/session/test-session' }) });
  });
  await page.route('https://checkout.dodopayments.com/**', route => route.fulfill({ contentType: 'text/html', body: '<title>Test checkout</title>' }));
  await page.goto('/');
  await page.getByRole('button', { name: 'Choose monthly' }).click();
  await expect.poll(() => checkoutRequests.length).toBe(1);
  expect(checkoutRequests[0]).toEqual({ url: 'https://api.sociobot.in/api/v1/products/mtd-quarterly-ready/checkout', method: 'POST' });
  await page.waitForURL('https://checkout.dodopayments.com/session/test-session');
  await page.goto('/');
  await page.getByRole('button', { name: /Choose annual/ }).click();
  await expect.poll(() => checkoutRequests.length).toBe(2);
  expect(checkoutRequests[1]).toEqual({ url: 'https://api.sociobot.in/api/v1/products/mtd-quarterly-ready-annual/checkout', method: 'POST' });
  await page.goto('/records');
  await expect(page.getByRole('button', { name: 'Download accountant CSV' })).toBeEnabled();
  await page.getByRole('button', { name: 'Make accountant link' }).click();
  await expect(page.getByText('A live accountant link needs an active Sociobot subscription. The CSV remains free.')).toBeVisible();
  const registration = readFileSync(new URL('../.factory/billing.md', import.meta.url), 'utf8');
  expect(registration).toContain('`monthly` | GBP 1,200 pence | monthly');
  expect(registration).toContain('`annual` | GBP 9,900 pence | yearly');
  expect(registration).not.toMatch(/(?:price|product)_[A-Za-z0-9]{8,}/);
});
