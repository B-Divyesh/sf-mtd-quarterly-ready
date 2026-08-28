import { defineConfig } from 'vitest/config';

export default defineConfig({
  root: 'frontend',
  test: { environment: 'node', include: ['src/**/*.test.ts'] },
});
