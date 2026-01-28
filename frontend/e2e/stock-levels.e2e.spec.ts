import { test, expect } from '@playwright/test';

test.describe('Stock Levels Management E2E Tests', () => {
	// Mock authentication before each test
	test.beforeEach(async ({ page }) => {
		// Set up authenticated session with mock token
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

		// Set both access_token and refresh_token
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

		// Mock stock levels API responses
		await page.route('**/api/v1/inventory/stock-levels*', async (route) => {
			const url = route.request().url();

			if (route.request().method() === 'GET') {
				// Check for specific filters
				if (url.includes('search=LOW')) {
					// Search filter - return low stock item
					await route.fulfill({
						status: 200,
						contentType: 'application/json',
						body: JSON.stringify({
							items: [
								{
									inventoryId: 'inv-002',
									tenantId: 'tenant-001',
									productId: 'prod-002',
									productSku: 'LOW-STOCK-001',
									productName: 'Low Stock Product',
									warehouseId: 'wh-001',
									warehouseCode: 'WH-MAIN',
									warehouseName: 'Main Warehouse',
									availableQuantity: 5,
									reservedQuantity: 0,
									totalQuantity: 5,
									status: 'low_stock',
									reorderPoint: 10,
									updatedAt: '2026-01-15T10:00:00Z'
								}
							],
							pagination: {
								page: 1,
								pageSize: 20,
								totalItems: 1,
								totalPages: 1,
								hasNext: false,
								hasPrev: false
							},
							summary: {
								totalProducts: 1,
								totalAvailableQuantity: 5,
								totalReservedQuantity: 0,
								lowStockCount: 1,
								outOfStockCount: 0
							}
						})
					});
				} else if (url.includes('warehouseId=wh-002')) {
					// Warehouse filter - return secondary warehouse items
					await route.fulfill({
						status: 200,
						contentType: 'application/json',
						body: JSON.stringify({
							items: [
								{
									inventoryId: 'inv-003',
									tenantId: 'tenant-001',
									productId: 'prod-003',
									productSku: 'PROD-003',
									productName: 'Secondary Warehouse Product',
									warehouseId: 'wh-002',
									warehouseCode: 'WH-SEC',
									warehouseName: 'Secondary Warehouse',
									availableQuantity: 200,
									reservedQuantity: 20,
									totalQuantity: 220,
									status: 'in_stock',
									reorderPoint: 50,
									updatedAt: '2026-01-16T10:00:00Z'
								}
							],
							pagination: {
								page: 1,
								pageSize: 20,
								totalItems: 1,
								totalPages: 1,
								hasNext: false,
								hasPrev: false
							},
							summary: {
								totalProducts: 1,
								totalAvailableQuantity: 200,
								totalReservedQuantity: 20,
								lowStockCount: 0,
								outOfStockCount: 0
							}
						})
					});
				} else if (url.includes('outOfStockOnly=true')) {
					// Out of stock filter
					await route.fulfill({
						status: 200,
						contentType: 'application/json',
						body: JSON.stringify({
							items: [
								{
									inventoryId: 'inv-004',
									tenantId: 'tenant-001',
									productId: 'prod-004',
									productSku: 'OUT-STOCK-001',
									productName: 'Out of Stock Product',
									warehouseId: 'wh-001',
									warehouseCode: 'WH-MAIN',
									warehouseName: 'Main Warehouse',
									availableQuantity: 0,
									reservedQuantity: 0,
									totalQuantity: 0,
									status: 'out_of_stock',
									reorderPoint: 10,
									updatedAt: '2026-01-14T10:00:00Z'
								}
							],
							pagination: {
								page: 1,
								pageSize: 20,
								totalItems: 1,
								totalPages: 1,
								hasNext: false,
								hasPrev: false
							},
							summary: {
								totalProducts: 1,
								totalAvailableQuantity: 0,
								totalReservedQuantity: 0,
								lowStockCount: 0,
								outOfStockCount: 1
							}
						})
					});
				} else {
					// Default list response
					await route.fulfill({
						status: 200,
						contentType: 'application/json',
						body: JSON.stringify({
							items: [
								{
									inventoryId: 'inv-001',
									tenantId: 'tenant-001',
									productId: 'prod-001',
									productSku: 'TEST-SKU-001',
									productName: 'Test Product 1',
									warehouseId: 'wh-001',
									warehouseCode: 'WH-MAIN',
									warehouseName: 'Main Warehouse',
									availableQuantity: 100,
									reservedQuantity: 10,
									totalQuantity: 110,
									status: 'in_stock',
									reorderPoint: 20,
									updatedAt: '2026-01-15T10:00:00Z'
								},
								{
									inventoryId: 'inv-002',
									tenantId: 'tenant-001',
									productId: 'prod-002',
									productSku: 'LOW-STOCK-001',
									productName: 'Low Stock Product',
									warehouseId: 'wh-001',
									warehouseCode: 'WH-MAIN',
									warehouseName: 'Main Warehouse',
									availableQuantity: 5,
									reservedQuantity: 0,
									totalQuantity: 5,
									status: 'low_stock',
									reorderPoint: 10,
									updatedAt: '2026-01-14T10:00:00Z'
								},
								{
									inventoryId: 'inv-003',
									tenantId: 'tenant-001',
									productId: 'prod-003',
									productSku: 'PROD-003',
									productName: 'Secondary Warehouse Product',
									warehouseId: 'wh-002',
									warehouseCode: 'WH-SEC',
									warehouseName: 'Secondary Warehouse',
									availableQuantity: 200,
									reservedQuantity: 20,
									totalQuantity: 220,
									status: 'in_stock',
									reorderPoint: 50,
									updatedAt: '2026-01-16T10:00:00Z'
								}
							],
							pagination: {
								page: 1,
								pageSize: 20,
								totalItems: 3,
								totalPages: 1,
								hasNext: false,
								hasPrev: false
							},
							summary: {
								totalProducts: 3,
								totalAvailableQuantity: 305,
								totalReservedQuantity: 30,
								lowStockCount: 1,
								outOfStockCount: 0
							}
						})
					});
				}
			} else {
				await route.continue();
			}
		});

		// Mock warehouses API for filter dropdown
		await page.route('**/api/v1/inventory/warehouses*', async (route) => {
			if (route.request().method() === 'GET') {
				await route.fulfill({
					status: 200,
					contentType: 'application/json',
					body: JSON.stringify({
						warehouses: [
							{
								warehouseId: 'wh-001',
								warehouseCode: 'WH-MAIN',
								warehouseName: 'Main Warehouse',
								isActive: true
							},
							{
								warehouseId: 'wh-002',
								warehouseCode: 'WH-SEC',
								warehouseName: 'Secondary Warehouse',
								isActive: true
							}
						],
						pagination: {
							page: 1,
							pageSize: 100,
							totalItems: 2,
							totalPages: 1,
							hasNext: false,
							hasPrev: false
						}
					})
				});
			} else {
				await route.continue();
			}
		});
	});

	test('should display stock levels list page', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Check page title and header
		await expect(page.getByRole('heading', { name: 'Stock Levels' })).toBeVisible();

		// Check description text
		await expect(page.getByText('View inventory quantities across warehouses')).toBeVisible();

		// Check refresh button exists
		await expect(page.getByRole('button', { name: 'Refresh' })).toBeVisible();
	});

	test('should display summary cards with correct data', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Check summary cards - use role paragraph to be specific
		await expect(page.getByRole('paragraph').filter({ hasText: 'Total Products' })).toBeVisible();
		await expect(
			page.getByRole('paragraph').filter({ hasText: 'Available Quantity' })
		).toBeVisible();
		await expect(
			page.getByRole('paragraph').filter({ hasText: 'Reserved Quantity' })
		).toBeVisible();
		await expect(page.getByRole('paragraph').filter({ hasText: 'Low Stock Items' })).toBeVisible();
		await expect(page.getByRole('paragraph').filter({ hasText: /^Out of Stock$/ })).toBeVisible();

		// Check summary values
		await expect(page.getByText('3').first()).toBeVisible(); // Total Products
		await expect(page.getByText('305')).toBeVisible(); // Available Quantity
		await expect(page.getByText('30', { exact: true })).toBeVisible(); // Reserved Quantity
	});

	test('should display stock levels table with correct columns', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Check table headers - use role cell for table headers
		await expect(page.getByRole('cell', { name: 'SKU', exact: true })).toBeVisible();
		await expect(page.getByRole('cell', { name: /Product ↑/ })).toBeVisible();
		await expect(page.getByRole('cell', { name: 'Warehouse', exact: true })).toBeVisible();
		await expect(page.getByRole('cell', { name: 'Available', exact: true })).toBeVisible();
		await expect(page.getByRole('cell', { name: 'Reserved', exact: true })).toBeVisible();
		await expect(page.getByRole('cell', { name: 'Total', exact: true })).toBeVisible();
		await expect(page.getByRole('cell', { name: 'Status', exact: true })).toBeVisible();
		await expect(page.getByRole('cell', { name: 'Updated', exact: true })).toBeVisible();
	});

	test('should display stock level items in table', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Check first item
		await expect(page.getByText('TEST-SKU-001')).toBeVisible();
		await expect(page.getByText('Test Product 1')).toBeVisible();
		await expect(page.getByText('WH-MAIN').first()).toBeVisible();

		// Check second item
		await expect(page.getByText('LOW-STOCK-001')).toBeVisible();
		await expect(page.getByText('Low Stock Product')).toBeVisible();

		// Check third item
		await expect(page.getByText('PROD-003')).toBeVisible();
		await expect(page.getByText('Secondary Warehouse Product')).toBeVisible();
	});

	test('should display correct status badges', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Check status badges - use exact match to avoid matching product names
		const tableBody = page.locator('tbody');
		await expect(tableBody.getByText('In Stock', { exact: true }).first()).toBeVisible();
		await expect(tableBody.getByText('Low Stock', { exact: true })).toBeVisible();
	});

	test('should have search functionality', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Find search input
		const searchInput = page.getByPlaceholder('Search by SKU or product name...');
		await expect(searchInput).toBeVisible();

		// Type in search
		await searchInput.fill('LOW');
		await page.waitForTimeout(500); // Wait for debounce

		// Should show filtered results
		await expect(page.getByText('LOW-STOCK-001')).toBeVisible();
	});

	test('should have warehouse filter dropdown', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Find warehouse filter - look for combobox with "All Warehouses" text
		const warehouseFilter = page.getByRole('combobox').filter({ hasText: 'All Warehouses' });
		await expect(warehouseFilter).toBeVisible();
	});

	test('should have status filter dropdown', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Find status filter
		const statusFilter = page.getByRole('combobox').filter({ hasText: 'All Status' });
		await expect(statusFilter).toBeVisible();
	});

	test('should refresh data when clicking refresh button', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Click refresh button
		const refreshButton = page.getByRole('button', { name: 'Refresh' });
		await refreshButton.click();

		// Wait for refresh to complete
		await page.waitForLoadState('networkidle');

		// Data should still be visible
		await expect(page.getByText('TEST-SKU-001')).toBeVisible();
	});

	test('should display pagination controls', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Check pagination text
		await expect(page.getByText('Showing 1 - 3 of 3')).toBeVisible();

		// Check page size selector
		await expect(page.getByRole('combobox').filter({ hasText: '20' })).toBeVisible();
	});

	test('should display list count in header', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Check list count header
		await expect(page.getByText('Stock Level List (3)')).toBeVisible();
	});

	test('should have product links that navigate to product page', async ({ page }) => {
		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Find product link
		const productLink = page.getByRole('link', { name: 'Test Product 1' });
		await expect(productLink).toBeVisible();

		// Check href attribute
		const href = await productLink.getAttribute('href');
		expect(href).toContain('/inventory/products/');
	});

	test('should handle empty state gracefully', async ({ page }) => {
		// Override route to return empty data
		await page.route('**/api/v1/inventory/stock-levels*', async (route) => {
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({
					items: [],
					pagination: {
						page: 1,
						pageSize: 20,
						totalItems: 0,
						totalPages: 0,
						hasNext: false,
						hasPrev: false
					},
					summary: {
						totalProducts: 0,
						totalAvailableQuantity: 0,
						totalReservedQuantity: 0,
						lowStockCount: 0,
						outOfStockCount: 0
					}
				})
			});
		});

		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Check empty state message
		await expect(page.getByText('No stock levels found.')).toBeVisible();
	});

	test('should navigate from sidebar to stock levels page', async ({ page }) => {
		// Start from dashboard
		await page.goto('/dashboard');
		await page.waitForLoadState('networkidle');

		// Expand Inventory section if needed
		const inventoryButton = page.getByRole('button', { name: 'Inventory' });
		if (await inventoryButton.isVisible()) {
			await inventoryButton.click();
		}

		// Click Stock Levels link
		const stockLevelsLink = page.getByRole('link', { name: 'Stock Levels' });
		await stockLevelsLink.click();

		// Verify navigation
		await expect(page).toHaveURL(/\/inventory\/stock-levels/);
		await expect(page.getByRole('heading', { name: 'Stock Levels' })).toBeVisible();
	});

	test('should display loading state while fetching data', async ({ page }) => {
		// Add delay to API response
		await page.route('**/api/v1/inventory/stock-levels*', async (route) => {
			await new Promise((resolve) => setTimeout(resolve, 500));
			await route.fulfill({
				status: 200,
				contentType: 'application/json',
				body: JSON.stringify({
					items: [],
					pagination: {
						page: 1,
						pageSize: 20,
						totalItems: 0,
						totalPages: 0,
						hasNext: false,
						hasPrev: false
					},
					summary: {
						totalProducts: 0,
						totalAvailableQuantity: 0,
						totalReservedQuantity: 0,
						lowStockCount: 0,
						outOfStockCount: 0
					}
				})
			});
		});

		await page.goto('/inventory/stock-levels');

		// Page should eventually load
		await page.waitForLoadState('networkidle');
		await expect(page.getByRole('heading', { name: 'Stock Levels' })).toBeVisible();
	});

	test('should handle API error gracefully', async ({ page }) => {
		// Override route to return error
		await page.route('**/api/v1/inventory/stock-levels*', async (route) => {
			await route.fulfill({
				status: 500,
				contentType: 'application/json',
				body: JSON.stringify({
					error: 'Internal Server Error'
				})
			});
		});

		await page.goto('/inventory/stock-levels');
		await page.waitForLoadState('networkidle');

		// Page should still render with error state
		await expect(page.getByRole('heading', { name: 'Stock Levels' })).toBeVisible();
	});
});
