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
  expect(outgoing.every(url => new URL(url).origin === new URL(page.url()).origin)).toBe(true);
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

test('@claim:free-quarter-persistence @claim:quarter-record-separation @regression:current-and-future-quarters remain separate across reloads', async ({ page, request }) => {
  await page.goto('/records');
  const selector = page.getByLabel('Working quarter');
  const currentStart = await selector.inputValue();
  const currentDate = await page.locator('#add-form input[name="date"]').getAttribute('min');
  await page.getByRole('button', { name: 'Add a transaction' }).click();
  await page.locator('#add-form input[name="date"]').fill(currentDate!);
  await page.locator('#add-form input[name="description"]').fill('Current-quarter lesson');
  await page.locator('#add-form input[name="amount"]').fill('75.00');
  await page.locator('#add-form select[name="category"]').selectOption('Sales');
  const currentSave = page.waitForResponse(response => new URL(response.url()).pathname === '/api/workspace' && response.request().method() === 'PUT');
  await page.getByRole('button', { name: 'Save transaction' }).click();
  const currentWorkspaceId = await (await currentSave).request().headerValue('x-workspace-id');
  await page.reload();
  await expect(page.getByText('Current-quarter lesson', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Create next quarter' }).click();
  const nextStart = await page.getByLabel('Working quarter').inputValue();
  expect(nextStart).not.toBe(currentStart);
  await expect(page.getByRole('heading', { level: 3, name: 'No transactions in this quarter' })).toBeVisible();
  await page.getByRole('button', { name: 'Add the first transaction' }).click();
  const nextDate = await page.locator('#add-form input[name="date"]').getAttribute('min');
  await page.locator('#add-form input[name="date"]').fill(nextDate!);
  await page.locator('#add-form input[name="description"]').fill('Next-quarter lesson');
  await page.locator('#add-form input[name="amount"]').fill('85.00');
  await page.locator('#add-form select[name="category"]').selectOption('Sales');
  const nextSave = page.waitForResponse(response => new URL(response.url()).pathname === '/api/workspace' && response.request().method() === 'PUT');
  await page.getByRole('button', { name: 'Save transaction' }).click();
  const nextWorkspaceId = await (await nextSave).request().headerValue('x-workspace-id');
  expect(currentWorkspaceId).toBeTruthy();
  expect(nextWorkspaceId).toBeTruthy();
  expect(nextWorkspaceId).not.toBe(currentWorkspaceId);
  const headers = (workspaceId: string) => ({ 'x-workspace-id': workspaceId, 'x-forwarded-for': '203.0.113.77' });
  const [currentRemote, nextRemote] = await Promise.all([
    request.get('/api/workspace', { headers: headers(currentWorkspaceId!) }),
    request.get('/api/workspace', { headers: headers(nextWorkspaceId!) }),
  ]);
  expect((await currentRemote.json()).document.transactions.map((transaction: { description: string }) => transaction.description)).toContain('Current-quarter lesson');
  expect((await nextRemote.json()).document.transactions.map((transaction: { description: string }) => transaction.description)).toContain('Next-quarter lesson');
  await page.getByLabel('Working quarter').selectOption(currentStart);
  await expect(page.getByText('Current-quarter lesson', { exact: true })).toBeVisible();
  const keys = await page.evaluate(() => Object.keys(localStorage));
  expect(keys).toContain(`quarterly-ready:document:${currentStart}`);
  expect(keys).toContain(`quarterly-ready:document:${nextStart}`);
});

test('@regression:receipt-small attaches a receipt to an existing expense', async ({ page }) => {
  await page.goto('/demo');
  const row = page.locator('tr', { hasText: 'Whiteboard markers' });
  await row.locator('[data-receipt]').setInputFiles({ name: 'markers-receipt.pdf', mimeType: 'application/pdf', buffer: Buffer.from('%PDF-1.4 sample receipt') });
  await expect(page.getByText('Receipt attached.')).toBeVisible();
  await expect(page.locator('tr', { hasText: 'Whiteboard markers' }).getByText('Receipt · markers-receipt.pdf')).toBeVisible();
  const stored = await page.evaluate(() => localStorage.getItem('demo:quarterly-ready:document'));
  expect(stored).not.toContain('data:application/pdf;base64');
  const receiptCount = await page.evaluate(async () => new Promise<number>((resolve, reject) => {
    const open = indexedDB.open('quarterly-ready-receipts-v1');
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {
      const count = open.result.transaction('receipts').objectStore('receipts').count();
      count.onerror = () => reject(count.error);
      count.onsuccess = () => resolve(count.result);
    };
  }));
  expect(receiptCount).toBe(1);
});

test('@claim:receipt-capture @regression:receipt-quota stores three valid near-limit receipts outside localStorage', async ({ page }) => {
  const browserErrors: string[] = [];
  page.on('console', message => { if (message.type() === 'error') browserErrors.push(message.text()); });
  page.on('pageerror', error => browserErrors.push(error.message));
  await page.goto('/demo');
  await page.evaluate(() => {
    const original = JSON.parse(localStorage.getItem('demo:quarterly-ready:document') || '{}');
    original.transactions = [1, 2, 3].map(index => ({
      id: `quota-expense-${index}`,
      date: '2026-04-10',
      description: `Near-limit receipt ${index}`,
      amountPence: 1000,
      kind: 'expense',
      category: 'Office costs',
    }));
    localStorage.setItem('demo:quarterly-ready:document', JSON.stringify(original));
  });
  await page.reload();
  const receipt = Buffer.alloc(1_400_000, 0x25);
  for (let index = 1; index <= 3; index += 1) {
    await page.locator('tr', { hasText: `Near-limit receipt ${index}` }).locator('[data-receipt]').setInputFiles({
      name: `near-limit-${index}.pdf`, mimeType: 'application/pdf', buffer: receipt,
    });
    await expect(page.locator('tr', { hasText: `Near-limit receipt ${index}` }).getByText(`Receipt · near-limit-${index}.pdf`)).toBeVisible();
  }
  const stored = await page.evaluate(() => localStorage.getItem('demo:quarterly-ready:document') || '');
  expect(stored).not.toContain('data:application/pdf;base64');
  const receiptSizes = await page.evaluate(async () => new Promise<number[]>((resolve, reject) => {
    const open = indexedDB.open('quarterly-ready-receipts-v1');
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {
      const records = open.result.transaction('receipts').objectStore('receipts').getAll();
      records.onerror = () => reject(records.error);
      records.onsuccess = () => resolve(records.result.map(record => record.size));
    };
  }));
  expect(receiptSizes).toEqual([1_400_000, 1_400_000, 1_400_000]);
  expect(browserErrors).toEqual([]);
});

test('@claim:receipt-locality keeps receipt contents in browser IndexedDB and out of the server document', async ({ page }) => {
  const workspaceBodies: string[] = [];
  page.on('request', request => {
    if (new URL(request.url()).pathname === '/api/workspace' && request.method() === 'PUT') workspaceBodies.push(request.postData() || '');
  });
  await page.goto('/records');
  const row = page.locator('tr', { hasText: 'Next receipt stays local' });
  await page.getByRole('button', { name: 'Add a transaction' }).click();
  const date = await page.locator('#add-form input[name="date"]').getAttribute('min');
  await page.locator('#add-form input[name="date"]').fill(date!);
  await page.locator('#add-form input[name="description"]').fill('Next receipt stays local');
  await page.locator('#add-form input[name="amount"]').fill('12.34');
  await page.locator('#add-form select[name="kind"]').selectOption('expense');
  await page.locator('#add-form select[name="category"]').selectOption('Office costs');
  const privateReceipt = 'private-receipt-body-must-not-leave-browser';
  const save = page.waitForResponse(response => new URL(response.url()).pathname === '/api/workspace' && response.request().method() === 'PUT');
  await page.locator('#add-form input[name="receipt"]').setInputFiles({
    name: 'private-proof.pdf', mimeType: 'application/pdf', buffer: Buffer.from(privateReceipt),
  });
  await page.getByRole('button', { name: 'Save transaction' }).click();
  await save;
  await expect(row.getByText('Receipt · private-proof.pdf')).toBeVisible();
  expect(workspaceBodies).not.toEqual([]);
  expect(workspaceBodies.join('\n')).not.toContain(privateReceipt);
  expect(workspaceBodies.join('\n')).not.toContain('data:application/pdf;base64');
  const storage = await page.evaluate(async () => new Promise<{ local: string; stored: boolean }>((resolve, reject) => {
    const local = localStorage.getItem('quarterly-ready:document:' + localStorage.getItem('quarterly-ready:active-quarter')) || '';
    const open = indexedDB.open('quarterly-ready-receipts-v1');
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {
      const store = open.result.transaction('receipts').objectStore('receipts');
      const records = store.getAll();
      records.onerror = () => reject(records.error);
      records.onsuccess = () => resolve({ local, stored: records.result.some(record => record.name === 'private-proof.pdf' && record.blob instanceof Blob) });
    };
  }));
  expect(storage.local).not.toContain(privateReceipt);
  expect(storage.local).not.toContain('data:application/pdf;base64');
  expect(storage.stored).toBe(true);
});

test('@regression:receipt-quota-error keeps the transaction unchanged and announces recovery', async ({ page }) => {
  const browserErrors: string[] = [];
  page.on('pageerror', error => browserErrors.push(error.message));
  await page.addInitScript(() => {
    const originalPut = IDBObjectStore.prototype.put;
    IDBObjectStore.prototype.put = function (...args: Parameters<IDBObjectStore['put']>) {
      if (this.name === 'receipts') throw new DOMException('Synthetic storage boundary', 'QuotaExceededError');
      return originalPut.apply(this, args);
    };
  });
  await page.goto('/demo');
  const row = page.locator('tr', { hasText: 'Whiteboard markers' });
  await row.locator('[data-receipt]').setInputFiles({
    name: 'quota-boundary.pdf', mimeType: 'application/pdf', buffer: Buffer.from('%PDF-1.4 quota boundary'),
  });
  await expect(page.getByText(/browser does not have enough space/)).toBeVisible();
  await expect(page.locator('tr', { hasText: 'Whiteboard markers' }).locator('[data-receipt]')).toBeVisible();
  await expect(page.getByText('Receipt · quota-boundary.pdf')).toHaveCount(0);
  expect(browserErrors).toEqual([]);
});

test('@regression:legacy-receipt-data migrates from localStorage to IndexedDB', async ({ page }) => {
  await page.goto('/demo');
  await page.evaluate(() => {
    const document = JSON.parse(localStorage.getItem('demo:quarterly-ready:document') || '{}');
    document.transactions[0].receiptName = 'older-receipt.pdf';
    document.transactions[0].receiptData = 'data:application/pdf;base64,JVBERi0xLjQ=';
    localStorage.setItem('demo:quarterly-ready:document', JSON.stringify(document));
  });
  await page.reload();
  await expect.poll(() => page.evaluate(() => localStorage.getItem('demo:quarterly-ready:document') || ''))
    .not.toContain('receiptData');
  const names = await page.evaluate(async () => new Promise<string[]>((resolve, reject) => {
    const open = indexedDB.open('quarterly-ready-receipts-v1');
    open.onerror = () => reject(open.error);
    open.onsuccess = () => {
      const records = open.result.transaction('receipts').objectStore('receipts').getAll();
      records.onerror = () => reject(records.error);
      records.onsuccess = () => resolve(records.result.map(record => record.name));
    };
  }));
  expect(names).toContain('older-receipt.pdf');
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
  const update = await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.ready;
    await registration.update();
    return { active: Boolean(registration.active), controlled: Boolean(navigator.serviceWorker.controller) };
  });
  expect(update).toEqual({ active: true, controlled: true });
  await context.setOffline(true);
  await page.reload({ waitUntil: 'domcontentloaded' });
  await expect(page.getByText('Offline — browser copy active')).toBeVisible();
  await expect(page.getByText('Maya Patel Tutoring')).toBeVisible();
});

test('@claim:paid-tier @regression:paid-tier-checkout-navigation uses Sociobot subscription checkout and keeps CSV free', async ({ page }) => {
  const checkoutRequests: { url: string; method: string }[] = [];
  const context = page.context();
  await context.route('https://api.sociobot.in/api/v1/products/*/checkout', async route => {
    checkoutRequests.push({ url: route.request().url(), method: route.request().method() });
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ checkout_url: 'https://checkout.dodopayments.com/session/test-session' }) });
  });
  await context.route('https://checkout.dodopayments.com/**', route => route.fulfill({ contentType: 'text/html', body: '<title>Test checkout</title>' }));
  await page.goto('/');
  const origin = new URL(page.url()).origin;
  await Promise.all([
    page.waitForURL('https://checkout.dodopayments.com/session/test-session'),
    page.getByRole('button', { name: 'Choose monthly' }).click(),
  ]);
  await expect.poll(() => checkoutRequests.length).toBe(1);
  expect(checkoutRequests[0]).toEqual({ url: 'https://api.sociobot.in/api/v1/products/mtd-quarterly-ready/checkout', method: 'POST' });

  // A checkout intentionally replaces the document. Use fresh browser pages
  // for the annual and free-tier assertions rather than racing a forced
  // navigation back to the app while the cross-origin checkout is committing.
  const annualPage = await context.newPage();
  await annualPage.goto(`${origin}/`);
  await Promise.all([
    annualPage.waitForURL('https://checkout.dodopayments.com/session/test-session'),
    annualPage.getByRole('button', { name: /Choose annual/ }).click(),
  ]);
  await expect.poll(() => checkoutRequests.length).toBe(2);
  expect(checkoutRequests[1]).toEqual({ url: 'https://api.sociobot.in/api/v1/products/mtd-quarterly-ready-annual/checkout', method: 'POST' });

  const recordsPage = await context.newPage();
  await recordsPage.goto(`${origin}/records`);
  await expect(recordsPage.getByRole('button', { name: 'Download accountant CSV' })).toBeEnabled();
  await recordsPage.getByRole('button', { name: 'Make accountant link' }).click();
  await expect(recordsPage.getByText('A live accountant link needs an active Sociobot subscription. The CSV remains free.')).toBeVisible();
  const registration = readFileSync(new URL('../.factory/billing.md', import.meta.url), 'utf8');
  expect(registration).toContain('`monthly` | GBP 1,200 pence | monthly');
  expect(registration).toContain('`annual` | GBP 9,900 pence | yearly');
  expect(registration).not.toMatch(/(?:price|product)_[A-Za-z0-9]{8,}/);
  await annualPage.close();
  await recordsPage.close();
});

test('@regression:checkout-transient-503 retries the selected Sociobot checkout before showing an error', async ({ page }) => {
  let attempts = 0;
  await page.route('https://api.sociobot.in/api/v1/products/mtd-quarterly-ready/checkout', async route => {
    attempts += 1;
    if (attempts < 3) {
      await route.fulfill({ status: 503, contentType: 'text/html', body: 'Temporarily unavailable' });
      return;
    }
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ checkout_url: 'https://checkout.dodopayments.com/session/recovered-session' }) });
  });
  await page.route('https://checkout.dodopayments.com/**', route => route.fulfill({ contentType: 'text/html', body: '<title>Recovered checkout</title>' }));

  await page.goto('/');
  await Promise.all([
    page.waitForURL('https://checkout.dodopayments.com/session/recovered-session'),
    page.getByRole('button', { name: 'Choose monthly' }).click(),
  ]);
  expect(attempts).toBe(3);
});
