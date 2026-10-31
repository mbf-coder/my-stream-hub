import { defineConfig } from '@playwright/test';

// Phase-1 has no e2e flows yet (diagnostics UI only). This config reserves the
// runner for Phase-2 studio tests: `npm run test:e2e` currently passes vacuously.
export default defineConfig({
  testDir: './tests',
  timeout: 30_000,
  reporter: 'list',
});
