import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

function submissionLabels(mode: string) {
  return mode === 'hmrc_sandbox_no_filing'
    ? {
      reviewAction: 'Review in HMRC sandbox',
      reviewConfirmation: 'I reviewed these totals and want to run the sandbox check.',
      confirmAction: 'Run HMRC sandbox check',
    }
    : {
      reviewAction: 'Review and submit to HMRC',
      reviewConfirmation: 'I reviewed these totals and want to submit this quarter.',
      confirmAction: 'Submit through approved integration',
    };
}

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

test('@regression:verify-url-helper checks title, language, landmark, image text, and browser errors', () => {
  const origin = process.env.VERIFY_ORIGIN || 'http://127.0.0.1:4173';
  const output = execFileSync('bash', ['scripts/verify-url.sh', `${origin}/demo`], {
    cwd: process.cwd(),
    encoding: 'utf8',
    timeout: 20_000,
  });
  expect(JSON.parse(output)).toMatchObject({
    language: 'en-GB',
    mainCount: 1,
    headingCount: 1,
    console_errors: 0,
    status: 'ok',
  });
});

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

test('@regression:two-hundred-percent-text keeps the mobile first action usable without horizontal overflow', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await page.evaluate(() => { document.documentElement.style.fontSize = '200%'; });
  const width = await page.evaluate(() => ({ scroll: document.documentElement.scrollWidth, client: document.documentElement.clientWidth }));
  expect(width.scroll).toBeLessThanOrEqual(width.client);
  await expect(page.getByRole('link', { name: 'Try it with sample data' })).toBeVisible();
});

test('@regression:reduced-motion makes the quarter dial transition effectively instant', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('/demo');
  const duration = await page.locator('.dial-hand').evaluate(element => getComputedStyle(element).transitionDuration);
  expect(Number.parseFloat(duration)).toBeLessThanOrEqual(0.00001);
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

test('submission review dialog is keyboard-operable and has no serious Axe findings when an integration is available', async ({ page, request }) => {
  const health = await (await request.get('/health')).json() as { hmrc_integration_configured?: boolean; hmrc_integration_mode?: string };
  await page.goto('/records');
  if (!health.hmrc_integration_configured) {
    await expect(page.getByRole('button', { name: /submit to HMRC|HMRC sandbox/i })).toHaveCount(0);
    await expect(page.getByText('No approved direct-submission integration is configured.')).toBeVisible();
    return;
  }
  const { reviewAction, reviewConfirmation, confirmAction } = submissionLabels(health.hmrc_integration_mode || 'approved_provider');
  await page.evaluate(() => localStorage.setItem('quarterly-ready:document', JSON.stringify({
    schemaVersion: 1, businessName: 'Maya Patel Tutoring', quarterLabel: '6 April to 5 July 2026',
    quarterStart: '2026-04-06', quarterEnd: '2026-07-05', figuresReviewed: true, markedReady: true,
    packDownloaded: true, updatedAt: new Date().toISOString(),
    transactions: [{ id: 'income-1', date: '2026-04-09', description: 'Lesson', amountPence: 4500, kind: 'income', category: 'Sales' }]
  })));
  await page.reload();
  await page.getByRole('button', { name: reviewAction }).click();
  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();
  await expect(page.getByLabel(reviewConfirmation)).toBeFocused();
  await page.keyboard.press('Space');
  await expect(page.getByRole('button', { name: confirmAction })).toBeEnabled();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter(item => ['serious', 'critical'].includes(item.impact || ''))).toEqual([]);
  await page.keyboard.press('Escape');
  await expect(dialog).toBeHidden();
});

test('@claim:conditional-submission @regression:hmrc-capability shows direct submission only when the server confirms an approved integration', async ({ page, browser, request }) => {
  const health = await (await request.get('/health')).json() as { hmrc_integration_configured?: boolean; hmrc_integration_mode?: string };
  const { reviewAction } = submissionLabels(health.hmrc_integration_mode || 'approved_provider');
  await page.goto('/records');
  if (health.hmrc_integration_configured) await expect(page.getByRole('button', { name: reviewAction })).toBeVisible();
  else await expect(page.getByRole('button', { name: /submit to HMRC|HMRC sandbox/i })).toHaveCount(0);
  const unavailableContext = await browser.newContext();
  const unavailable = await unavailableContext.newPage();
  await unavailable.route('**/health', route => route.fulfill({
    contentType: 'application/json',
    body: JSON.stringify({ status: 'ok', build_sha: 'test', safe_qa_fixtures: true, hmrc_integration_configured: false }),
  }));
  await unavailable.goto('/records');
  await expect(unavailable.getByRole('button', { name: reviewAction })).toHaveCount(0);
  await expect(unavailable.getByText('No approved direct-submission integration is configured.')).toBeVisible();
  await unavailableContext.close();
});

test('@regression:hmrc-capability hides direct submission when no approved integration is configured', async ({ page }) => {
  await page.route('**/health', route => route.fulfill({
    contentType: 'application/json',
    body: JSON.stringify({ status: 'ok', build_sha: 'test', safe_qa_fixtures: true, hmrc_integration_configured: false }),
  }));
  await page.goto('/records');
  await expect(page.getByRole('button', { name: /submit to HMRC|HMRC sandbox/i })).toHaveCount(0);
  await expect(page.getByText('No approved direct-submission integration is configured.')).toBeVisible();
});

test('@regression:hmrc-sandbox-copy makes the non-filing boundary explicit before confirmation', async ({ page }) => {
  await page.route('**/health', route => route.fulfill({
    contentType: 'application/json',
    body: JSON.stringify({
      status: 'ok', build_sha: 'test', safe_qa_fixtures: true,
      hmrc_integration_configured: true, hmrc_integration_mode: 'hmrc_sandbox_no_filing',
    }),
  }));
  await page.goto('/records');
  await expect(page.getByText('HMRC non-filing sandbox')).toBeVisible();
  await expect(page.getByText('It files no return and sends HMRC no records.')).toBeVisible();
  const button = page.getByRole('button', { name: 'Review in HMRC sandbox' });
  await expect(button).toBeVisible();
  await expect(button).toBeDisabled();
});

test('@regression:hmrc-copy does not claim a deployed sandbox when direct submission is unavailable', async ({ page }) => {
  await page.route('**/health', route => route.fulfill({
    contentType: 'application/json',
    body: JSON.stringify({ status: 'ok', build_sha: 'test', safe_qa_fixtures: true, hmrc_integration_configured: false }),
  }));
  await page.goto('/privacy');
  await expect(page.getByText('Quarterly Ready can send a reviewed update only when an approved HMRC integration is configured.')).toBeVisible();
  await expect(page.getByText(/The deployed integration is a non-filing HMRC sandbox/i)).toHaveCount(0);
});
