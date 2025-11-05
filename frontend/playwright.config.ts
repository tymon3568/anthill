import { defineConfig } from '@playwright/test';

export default defineConfig({
	webServer: {
		command: 'npx vite dev --port 5173',
		port: 5173,
		reuseExistingServer: !process.env.CI
	},
	testDir: 'e2e',
	use: {
		actionTimeout: 10000,
		navigationTimeout: 30000,
	},
	expect: {
		timeout: 10000,
	},
});
