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

test('all internal links return successful pages', async ({ page, request }) => {
  await page.goto('/');
  const links = await page.locator('a[href]').evaluateAll(items => [...new Set(items.map(item => (item as HTMLAnchorElement).href).filter(href => new URL(href).origin === location.origin))]);
  for (const link of links) expect((await request.get(link)).status(), link).toBeLessThan(400);
});
