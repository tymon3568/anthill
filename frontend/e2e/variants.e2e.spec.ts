import { test, expect } from '@playwright/test';

test.describe('Variants Management E2E Tests', () => {
	// Mock authentication before each test
	test.beforeEach(async ({ page }) => {
		// Set up authenticated session with mock token
		// Token payload must match what validateAndParseToken expects
		const tokenPayload = {
			sub: 'user-123',
			email: 'user@example.com',
			tenant_id: 'tenant-001',
			name: 'Test User',
			role: 'admin',
			exp: Math.floor(Date.now() / 1000) + 3600,
			iat: Math.floor(Date.now() / 1000) - 60
		};
		const token =
			'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.' +
			Buffer.from(JSON.stringify(tokenPayload)).toString('base64url') +
			'.test-signature';

		// Set both access_token and refresh_token (hooks.server.ts checks these)
		await page.context().addCookies([
			{
				name: 'access_token',
				value: token,
				domain: 'localhost',
				path: '/',
				httpOnly: true,
				secure: false
			},
			{
				name: 'refresh_token',
				value: 'mock-refresh-token',
				domain: 'localhost',
				path: '/',
				httpOnly: true,
				secure: false
			}
		]);

		// Mock variants API responses
		await page.route('**/api/v1/inventory/variants*', async (route) => {
			const url = route.request().url();

			if (route.request().method() === 'GET' && !url.includes('/by-')) {
				// List variants
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify({
						variants: [
							{
								variantId: 'var-001',
								tenantId: 'tenant-001',
								parentProductId: 'prod-001',
								variantAttributes: { color: 'Black', size: 'L' },
								sku: 'LAPTOP-001-BLK-L',
								barcode: '1234567890123',
								priceDifference: 500000,
								isActive: true,
								createdAt: '2026-01-01T00:00:00Z',
								updatedAt: '2026-01-15T00:00:00Z',
								parentProductName: 'Laptop Pro 15"',
								parentProductSku: 'LAPTOP-001'
							},
							{
								variantId: 'var-002',
								tenantId: 'tenant-001',
								parentProductId: 'prod-001',
								variantAttributes: { color: 'Silver', size: 'M' },
								sku: 'LAPTOP-001-SLV-M',
								barcode: '1234567890124',
								priceDifference: 0,
								isActive: true,
								createdAt: '2026-01-02T00:00:00Z',
								updatedAt: '2026-01-16T00:00:00Z',
								parentProductName: 'Laptop Pro 15"',
								parentProductSku: 'LAPTOP-001'
							},
							{
								variantId: 'var-003',
								tenantId: 'tenant-001',
								parentProductId: 'prod-002',
								variantAttributes: { color: 'Red' },
								sku: 'PHONE-001-RED',
								barcode: null,
								priceDifference: -100000,
								isActive: false,
								createdAt: '2026-01-03T00:00:00Z',
								updatedAt: '2026-01-17T00:00:00Z',
								parentProductName: 'Smartphone X',
								parentProductSku: 'PHONE-001'
							}
						],
						pagination: {
							page: 1,
							pageSize: 10,
							totalItems: 3,
							totalPages: 1,
							hasNext: false,
							hasPrev: false
						}
					})
				});
			} else if (url.includes('/bulk/activate')) {
				// Bulk activate
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify({
						success: true,
						affectedCount: 2,
						message: 'Variants activated successfully'
					})
				});
			} else if (url.includes('/bulk/deactivate')) {
				// Bulk deactivate
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify({
						success: true,
						affectedCount: 2,
						message: 'Variants deactivated successfully'
					})
				});
			} else if (url.includes('/bulk/delete')) {
				// Bulk delete
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify({
						success: true,
						affectedCount: 2,
						message: 'Variants deleted successfully'
					})
				});
			} else {
				await route.continue();
			}
		});
	});

	test('should display variants list page', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Check page title and header - use specific selector to avoid multiple h1 matches
		await expect(page.getByRole('heading', { name: 'Product Variants' })).toBeVisible();
		await expect(page.locator('text=/View and search variants/i')).toBeVisible();
	});

	test('should display variants table with data', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Wait for table to load
		await page.waitForSelector('table');

		// Check table headers
		await expect(page.locator('th:has-text("SKU")')).toBeVisible();
		await expect(page.locator('th:has-text("Parent Product")')).toBeVisible();
		await expect(page.locator('th:has-text("Attributes")')).toBeVisible();
		await expect(page.locator('th:has-text("Barcode")')).toBeVisible();
		await expect(page.locator('th:has-text("Price Diff")')).toBeVisible();
		await expect(page.locator('th:has-text("Status")')).toBeVisible();

		// Check variant data is displayed
		await expect(page.locator('text=LAPTOP-001-BLK-L')).toBeVisible();
		await expect(page.locator('text=LAPTOP-001-SLV-M')).toBeVisible();
		await expect(page.locator('text=PHONE-001-RED')).toBeVisible();
	});

	test('should display variant attributes as badges', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Check attribute badges are displayed
		await expect(page.locator('text=/color: Black/i')).toBeVisible();
		await expect(page.locator('text=/size: L/i')).toBeVisible();
		await expect(page.locator('text=/color: Silver/i')).toBeVisible();
	});

	test('should display status badges correctly', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Check active and inactive badges (inside Badge components in table)
		// Use more specific selectors to find badges in the table, not the select options
		const tableBody = page.locator('tbody');
		await expect(tableBody.locator('text=Active').first()).toBeVisible();
		await expect(tableBody.locator('text=Inactive').first()).toBeVisible();
	});

	test('should have search functionality', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Check search input exists
		const searchInput = page.locator('input[placeholder*="Search"]');
		await expect(searchInput).toBeVisible();

		// Type in search
		await searchInput.fill('LAPTOP');

		// Wait for debounce
		await page.waitForTimeout(500);

		// Search should trigger (mock API will return same results)
		await expect(page.locator('text=LAPTOP-001-BLK-L')).toBeVisible();
	});

	test('should have status filter', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Check filter select exists
		const statusFilter = page.locator('select').first();
		await expect(statusFilter).toBeVisible();

		// Select active filter
		await statusFilter.selectOption('active');

		// Wait for filter to apply
		await page.waitForTimeout(300);
	});

	test('should clear filters when clear button is clicked', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Add a filter
		const searchInput = page.locator('input[placeholder*="Search"]');
		await searchInput.fill('test');

		// Wait for debounce
		await page.waitForTimeout(500);

		// Clear button should appear
		const clearButton = page.locator('button:has-text("Clear")');
		await expect(clearButton).toBeVisible();

		// Click clear
		await clearButton.click();

		// Search should be cleared
		await expect(searchInput).toHaveValue('');
	});

	test('should select individual variants with checkbox', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Wait for table
		await page.waitForSelector('table');

		// Click first row checkbox
		const firstCheckbox = page.locator('tbody tr:first-child input[type="checkbox"]');
		await firstCheckbox.click();

		// Bulk actions should appear
		await expect(page.locator('text=/1 selected/i')).toBeVisible();
	});

	test('should select all variants with header checkbox', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Wait for table
		await page.waitForSelector('table');

		// Click header checkbox
		const headerCheckbox = page.locator('thead input[type="checkbox"]');
		await headerCheckbox.click();

		// All should be selected
		await expect(page.locator('text=/3 selected/i')).toBeVisible();
	});

	test('should show bulk action buttons when variants are selected', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Select a variant
		const firstCheckbox = page.locator('tbody tr:first-child input[type="checkbox"]');
		await firstCheckbox.click();

		// Wait for bulk actions bar to appear
		await page.waitForSelector('text=/1 selected/i');

		// Bulk action buttons should be visible - use exact match to avoid Activate matching Deactivate
		await expect(page.getByRole('button', { name: 'Activate', exact: true })).toBeVisible();
		await expect(page.getByRole('button', { name: 'Deactivate', exact: true })).toBeVisible();
		await expect(page.getByRole('button', { name: 'Delete', exact: true })).toBeVisible();
	});

	test('should link to parent product', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Find link to parent product
		const productLink = page.locator('a[href*="/inventory/products/prod-001"]').first();
		await expect(productLink).toBeVisible();

		// Check link text
		await expect(productLink).toContainText(/Laptop Pro/i);
	});

	test('should display price difference with formatting', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Positive price difference should show + sign
		await expect(page.locator('text=/\\+.*500/i')).toBeVisible();

		// Negative price difference should show - sign
		await expect(page.locator('text=/-.*100/i')).toBeVisible();
	});

	test('should have pagination controls', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Check pagination info
		await expect(page.locator('text=/Showing 1 - 3 of 3/i')).toBeVisible();

		// Check page size selector
		await expect(page.locator('select:has-text("10")')).toBeVisible();
	});

	test('should handle empty state', async ({ page }) => {
		// Override mock to return empty
		await page.route('**/api/v1/inventory/variants*', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({
					variants: [],
					pagination: {
						page: 1,
						pageSize: 10,
						totalItems: 0,
						totalPages: 0,
						hasNext: false,
						hasPrev: false
					}
				})
			});
		});

		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Should show empty state message
		await expect(page.locator('text=/No variants found/i')).toBeVisible();
	});

	test('should handle API error gracefully', async ({ page }) => {
		// Override mock to return error
		await page.route('**/api/v1/inventory/variants*', async (route) => {
			await route.fulfill({
				status: 500,
				contentType: 'application/json',
				body: JSON.stringify({
					error: 'Internal server error'
				})
			});
		});

		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Should show error message (the actual error text from API or fallback)
		await expect(
			page.locator('text=/Failed to load variants|Internal server error/i')
		).toBeVisible();

		// Should show retry button
		await expect(page.locator('button:has-text("Try Again")')).toBeVisible();
	});

	test('should show loading state', async ({ page }) => {
		// Add delay to API response
		await page.route('**/api/v1/inventory/variants*', async (route) => {
			await new Promise((resolve) => setTimeout(resolve, 1000));
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({
					variants: [],
					pagination: {
						page: 1,
						pageSize: 10,
						totalItems: 0,
						totalPages: 0,
						hasNext: false,
						hasPrev: false
					}
				})
			});
		});

		await page.goto('/inventory/variants');

		// Should show loading indicator
		await expect(page.locator('text=/Loading variants/i')).toBeVisible();
	});

	test('should navigate to product detail when clicking View', async ({ page }) => {
		await page.goto('/inventory/variants');
		await page.waitForLoadState('networkidle');

		// Wait for table to load
		await page.waitForSelector('tbody tr');

		// Click view button (it's an anchor styled as button)
		const viewButton = page.locator('tbody tr:first-child a:has-text("View")');
		await viewButton.click();

		// Should navigate to product detail - wait for URL change
		await page.waitForURL(/\/inventory\/products\/prod-001/);
		await expect(page).toHaveURL(/\/inventory\/products\/prod-001/);
	});
});
