import { defineConfig, devices } from '@playwright/test';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import path from 'node:path';

const repositoryRoot = fileURLToPath(new URL('.', import.meta.url));

export default defineConfig({
  testDir: './tests',
  timeout: 30_000,
  expect: { timeout: 7_000 },
  fullyParallel: false,
  workers: 1,
  reporter: [['list'], ['html', { open: 'never' }]],
  use: {
    baseURL: 'http://127.0.0.1:4173',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
  ],
  webServer: {
    command: 'npm run build && cargo run',
    cwd: repositoryRoot,
    env: {
      ...process.env,
      PORT: '4173',
      DATA_DIR: path.join(tmpdir(), 'quarterly-ready-test'),
      FRONTEND_DIR: path.join(repositoryRoot, 'dist'),
      SAFE_QA_FIXTURES: '1',
      HMRC_INTEGRATION_URL: 'https://approved-integration.test/mtd/periodic-update',
      HMRC_INTEGRATION_TOKEN: 'playwright-test-token',
    },
    url: 'http://127.0.0.1:4173/health',
    reuseExistingServer: true,
    timeout: 600_000,
  },
});
