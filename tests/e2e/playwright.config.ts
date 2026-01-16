import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { open: 'never' }],
    ['json', { outputFile: 'results/test-results.json' }],
    ['list'],
  ],
  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:3000',
    trace: 'on-first-retry',
    extraHTTPHeaders: {
      'Content-Type': 'application/json',
    },
  },
  projects: [
    {
      name: 'api',
      testMatch: /.*\.api\.spec\.ts/,
      use: {},
    },
    {
      name: 'flows',
      testMatch: /.*\.flow\.spec\.ts/,
      use: {},
    },
    {
      name: 'security',
      testMatch: /.*\.security\.spec\.ts/,
      use: {},
    },
  ],
  globalSetup: './global-setup.ts',
  globalTeardown: './global-teardown.ts',
  timeout: 30000,
  expect: {
    timeout: 5000,
  },
});
