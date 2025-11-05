import { test, expect } from '@playwright/test';

test.describe('Authentication E2E Tests', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/login');
		await page.waitForLoadState('networkidle');
		await page.waitForSelector('h1:has-text("Welcome back")', { state: 'visible' });
	});

	test('should display login page by default', async ({ page }) => {
		// Check if we're on login page
		await expect(page.locator('h1')).toContainText(/Welcome back/i);
		await expect(page.locator('text=/Sign in to your account/i')).toBeVisible();
			await expect(page.locator('input[type="email"]')).toBeVisible();
			await expect(page.locator('input[type="password"]')).toBeVisible();
			await expect(page.getByRole('button', { name: 'Sign In' })).toBeVisible();
	});

	test('should show validation errors for empty login form', async ({ page }) => {
		// Submit form to trigger validation
		await page.getByRole('button', { name: 'Sign In' }).click();

		// Wait for validation errors to appear
		await page.waitForTimeout(1000);

		// Check for validation errors
		await expect(page.locator('text=/Email is required/i')).toBeVisible();
		await expect(page.locator('text=/Password is required/i')).toBeVisible();
	});

	test('should show validation error for invalid email', async ({ page }) => {
		// Fill invalid email
		await page.locator('input[type="email"]').fill('invalid-email');
		await page.locator('input[type="password"]').fill('password123');

		// Focus and blur email field to trigger validation
		await page.locator('input[type="email"]').focus();
		await page.locator('input[type="email"]').blur();

		// Check for email validation error
		await expect(page.locator('text=/valid email/i')).toBeVisible();
	});

	test('should show validation error for short password', async ({ page }) => {
		// Fill valid email but short password
		await page.locator('input[type="email"]').fill('user@example.com');
		await page.locator('input[type="password"]').fill('123');

		// Focus and blur password field to trigger validation
		await page.locator('input[type="password"]').focus();
		await page.locator('input[type="password"]').blur();

		// Check for password validation error
		await expect(page.locator('text=/at least.*characters/i')).toBeVisible();
	});

	test('should navigate to register page', async ({ page }) => {
		// Click register link
		await page.locator('a[href*="register"]').click();

		// Check if we're on register page
		await expect(page.locator('h1')).toContainText(/Create your account/i);
		await expect(page.locator('text=/Join Anthill to manage your inventory/i')).toBeVisible();
		await expect(page.locator('input[type="email"]')).toBeVisible();
		// Use specific ID for password field to avoid ambiguity
		await expect(page.locator('#password')).toBeVisible();
		await expect(page.locator('#confirmPassword')).toBeVisible();
	});

	test('should show validation errors for empty register form', async ({ page }) => {
		// Navigate to register
		await page.locator('a[href*="register"]').click();

		// Wait for register page to load
		await page.waitForURL('**/register');

		// Submit form to trigger validation
		await page.getByRole('button', { name: 'Create Account' }).click();

		// Wait for validation errors to appear
		await page.waitForTimeout(1000);

		// Check for validation errors
		await expect(page.locator('text=/Name is required/i')).toBeVisible();
		await expect(page.locator('text=/Email is required/i')).toBeVisible();
		await expect(page.locator('text=/Password.*required/i')).toBeVisible();
	});

	test('should show password strength indicator', async ({ page }) => {
		// Navigate to register
		await page.locator('a[href*="register"]').click();

		// Wait for register page to load
		await page.waitForURL('**/register');

		// Type weak password
		await page.locator('#password').fill('weak');

		// Check for weak password indicator
		await expect(page.locator('text=/weak/i')).toBeVisible();

		// Type strong password
		await page.locator('#password').fill('StrongPass123!');

		// Check for strong password indicator
		await expect(page.locator('text=/strong/i')).toBeVisible();
	});

	test('should show password confirmation mismatch error', async ({ page }) => {
		// Navigate to register
		await page.locator('a[href*="register"]').click();

		// Wait for register page to load
		await page.waitForURL('**/register');

		// Fill form with mismatched passwords
		await page.getByLabel('Full Name').fill('John Doe');
		await page.getByLabel('Email').fill('user@example.com');
		await page.locator('#password').fill('StrongPass123!');
		await page.locator('#confirmPassword').fill('DifferentPass123!');

		// Submit form to trigger validation
		await page.getByRole('button', { name: 'Create Account' }).click();

		// Wait for password mismatch error to appear
		await expect(page.locator('text=/Passwords do not match/i')).toBeVisible({
			timeout: 5000
		});
	});

	test('should validate name field', async ({ page }) => {
		// Navigate to register
		await page.goto('/login');
		await page.locator('a[href*="register"]').click();

		// Wait for register page to load
		await page.waitForURL('**/register');

		// Fill invalid name (too short)
		await page.locator('input[placeholder*="name" i]').fill('J');

		// Focus and blur name field to trigger validation
		await page.locator('input[placeholder*="name" i]').focus();
		await page.locator('input[placeholder*="name" i]').blur();

		// Check for name validation error
		await expect(page.locator('text=/at least.*characters/i')).toBeVisible();
	});

	test('should validate name with invalid characters', async ({ page }) => {
		// Navigate to register
		await page.locator('a[href*="register"]').click();

		// Wait for register page to load
		await page.waitForURL('**/register');

		// Fill name with numbers
		await page.locator('input[placeholder*="name" i]').fill('John123');

		// Focus and blur name field to trigger validation
		await page.locator('input[placeholder*="name" i]').focus();
		await page.locator('input[placeholder*="name" i]').blur();

		// Check for name validation error
		await expect(page.locator('text=/letters and spaces/i')).toBeVisible();
	});

	test('should have accessible form elements', async ({ page }) => {
		// Navigate to login page first
		await page.goto('/login');

		// Check login form accessibility - fields should have proper attributes
		await expect(page.getByLabel('Email')).toBeVisible();
		await expect(page.getByLabel('Password')).toBeVisible();
		await expect(page.getByRole('button', { name: 'Sign In' })).toBeVisible();

		// Navigate to register
		await page.locator('a[href*="register"]').click();
		await page.waitForURL('**/register');

		// Check register form accessibility - all fields should be visible
		await expect(page.getByLabel('Full Name')).toBeVisible();
		await expect(page.getByLabel('Email')).toBeVisible();
		await expect(page.locator('#password')).toBeVisible();
		await expect(page.locator('#confirmPassword')).toBeVisible();
	});

	test('should have proper form labels', async ({ page }) => {
		// Navigate to login page first
		await page.goto('/login');

		// Check login form labels
		await expect(page.getByLabel('Email')).toBeVisible();
		await expect(page.getByLabel('Password')).toBeVisible();

		// Navigate to register
		await page.locator('a[href*="register"]').click();
		await page.waitForURL('**/register');

		// Check register form labels
		await expect(page.getByLabel('Full Name')).toBeVisible();
		await expect(page.getByLabel('Email')).toBeVisible();
		// Use ID selector to avoid strict mode violation with multiple password fields
		await expect(page.locator('#password')).toBeVisible();
		await expect(page.locator('#confirmPassword')).toBeVisible();
	});
});
