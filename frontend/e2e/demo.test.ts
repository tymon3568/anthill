import { expect, test } from '@playwright/test';

test('home page redirects appropriately', async ({ page }) => {
	await page.goto('/');

	// Home page should redirect to login (since no auth in E2E)
	await page.waitForURL('/login');
	expect(page.url()).toContain('/login');
});
