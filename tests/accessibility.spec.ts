import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

for (const path of ['/', '/demo', '/privacy', '/terms']) {
  test(`accessibility baseline ${path}`, async ({ page }) => {
    await page.goto(path);
    await expect(page.locator('main')).toHaveCount(1);
    await expect(page.locator('h1')).toHaveCount(1);
    expect(await page.locator('html').getAttribute('lang')).toBe('en-GB');
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations.filter(item => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
  });
}

test('mobile layout does not overflow at 390px', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/demo');
  const width = await page.evaluate(() => ({ scroll: document.documentElement.scrollWidth, client: document.documentElement.clientWidth }));
  expect(width.scroll).toBeLessThanOrEqual(width.client);
});

test('@regression:mobile-navigation-and-footer-targets are at least 44 by 44 CSS pixels', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  const targets = page.locator('.wordmark, .site-header nav a, .site-footer a');
  for (let index = 0; index < await targets.count(); index += 1) {
    const target = targets.nth(index);
    if (!(await target.isVisible())) continue;
    const box = await target.boundingBox();
    expect(box?.width, await target.getAttribute('href') || `target ${index}`).toBeGreaterThanOrEqual(44);
    expect(box?.height, await target.getAttribute('href') || `target ${index}`).toBeGreaterThanOrEqual(44);
  }
});

test('@regression:mobile-review-control is at least 44 CSS pixels high', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/demo');
  const box = await page.getByLabel('I checked these figures').locator('..').boundingBox();
  expect(box?.height).toBeGreaterThanOrEqual(44);
});

test('@regression:route-metadata follows SPA navigation', async ({ page }) => {
  await page.goto('/demo');
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', 'https://mtd-quarterly-ready.sociobot.in/demo');
  await expect(page.locator('meta[property="og:url"]')).toHaveAttribute('content', 'https://mtd-quarterly-ready.sociobot.in/demo');
  await expect(page.locator('meta[property="og:title"]')).toHaveAttribute('content', 'Demo — Quarterly Ready');
});

test('@regression:cold-records-load has no console or failed-response errors', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  page.on('pageerror', error => errors.push(error.message));
  page.on('response', response => { if (response.status() >= 400) errors.push(`${response.status()} ${response.url()}`); });
  const workspaceResponse = page.waitForResponse(response => response.url().endsWith('/api/workspace'));
  await page.goto('/records');
  await expect(page.getByRole('heading', { level: 3, name: 'No transactions in this quarter' })).toBeVisible();
  expect((await workspaceResponse).status()).toBe(200);
  expect(errors).toEqual([]);
});

test('all internal links return successful pages', async ({ page, request }) => {
  await page.goto('/');
  const links = await page.locator('a[href]').evaluateAll(items => [...new Set(items.map(item => (item as HTMLAnchorElement).href).filter(href => new URL(href).origin === location.origin))]);
  for (const link of links) expect((await request.get(link)).status(), link).toBeLessThan(400);
});

test('keyboard path opens the demo without console errors', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('/');
  await page.getByRole('link', { name: 'Try it with sample data' }).focus();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/demo$/);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Check this quarter');
  expect(errors).toEqual([]);
});

test('submission review dialog is keyboard-operable and has no serious Axe findings', async ({ page }) => {
  await page.goto('/records');
  await page.evaluate(() => localStorage.setItem('quarterly-ready:document', JSON.stringify({
    schemaVersion: 1, businessName: 'Maya Patel Tutoring', quarterLabel: '6 April to 5 July 2026',
    quarterStart: '2026-04-06', quarterEnd: '2026-07-05', figuresReviewed: true, markedReady: true,
    packDownloaded: true, updatedAt: new Date().toISOString(),
    transactions: [{ id: 'income-1', date: '2026-04-09', description: 'Lesson', amountPence: 4500, kind: 'income', category: 'Sales' }]
  })));
  await page.reload();
  await page.getByRole('button', { name: 'Review and submit to HMRC' }).click();
  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();
  await expect(page.getByLabel('I reviewed these totals and want to submit this quarter.')).toBeFocused();
  await page.keyboard.press('Space');
  await expect(page.getByRole('button', { name: 'Submit through approved integration' })).toBeEnabled();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter(item => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
  await page.keyboard.press('Escape');
  await expect(dialog).toBeHidden();
});
