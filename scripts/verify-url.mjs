import process from 'node:process';
import { chromium } from '@playwright/test';

const target = process.argv[2];

if (!target) {
  throw new Error('Usage: scripts/verify-url.sh <http(s) URL>');
}

const url = new URL(target);
if (!['http:', 'https:'].includes(url.protocol)) {
  throw new Error('verify-url accepts an http(s) URL.');
}

const browser = await chromium.launch({ channel: 'chromium', args: ['--disable-gpu'] });
const page = await browser.newPage();
const errors = [];
page.on('console', message => {
  if (message.type() === 'error') errors.push(`console: ${message.text()}`);
});
page.on('pageerror', error => errors.push(`page: ${error.message}`));

try {
  const response = await page.goto(url.href, { waitUntil: 'networkidle' });
  if (!response?.ok()) throw new Error(`${url.href} returned ${response?.status() ?? 'no response'}`);

  const result = await page.evaluate(() => ({
    title: document.title.trim(),
    language: document.documentElement.lang.trim(),
    mainCount: document.querySelectorAll('main').length,
    headingCount: document.querySelectorAll('h1').length,
    missingAlt: [...document.images]
      .filter(image => !image.hasAttribute('alt'))
      .map(image => image.currentSrc || image.src),
  }));

  if (!result.title) throw new Error('document title is missing');
  if (!result.language) throw new Error('html lang is missing');
  if (result.mainCount !== 1) throw new Error(`expected one main landmark, found ${result.mainCount}`);
  if (result.headingCount !== 1) throw new Error(`expected one h1, found ${result.headingCount}`);
  if (result.missingAlt.length) throw new Error(`images without alt text: ${result.missingAlt.join(', ')}`);
  if (errors.length) throw new Error(errors.join('\n'));

  console.log(JSON.stringify({ url: page.url(), ...result, console_errors: 0, status: 'ok' }));
} finally {
  await browser.close();
}
