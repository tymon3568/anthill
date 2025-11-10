import { expect, test } from '@playwright/test';

test('home page loads without errors', async ({ page }) => {
	await page.goto('/');
	// Just check that the page loads and has some basic content
	await expect(page.locator('body')).toBeVisible();
	// Check that we don't have any console errors
	const errors = [];
	page.on('console', msg => {
		if (msg.type() === 'error') {
			errors.push(msg.text());
		}
	});
	await page.waitForTimeout(1000);
	expect(errors.length).toBe(0);
});

test('home page shows loading state initially', async ({ page }) => {
	await page.goto('/');
	// Check that loading text appears
	await expect(page.locator('text=/Loading/i')).toBeVisible();
});
