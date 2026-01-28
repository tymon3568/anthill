import { describe, it, expect } from 'vitest';
import {
	mainNavigation,
	settingsNavigation,
	navigationGroups,
	getAllNavigationItems,
	isPathActive,
	hasActiveChild,
	type NavItem,
	type NavSubItem
} from './navigation';

describe('Navigation Configuration', () => {
	describe('mainNavigation', () => {
		it('should have Dashboard as first item', () => {
			expect(mainNavigation[0].title).toBe('Dashboard');
			expect(mainNavigation[0].url).toBe('/dashboard');
		});

		it('should include all main navigation sections', () => {
			const titles = mainNavigation.map((item) => item.title);
			expect(titles).toContain('Dashboard');
			expect(titles).toContain('Inventory');
			expect(titles).toContain('Orders');
			expect(titles).toContain('Integrations');
		});

		it('should have icons for all main items', () => {
			mainNavigation.forEach((item) => {
				expect(item.icon).toBeDefined();
			});
		});

		it('should have sub-items for Inventory section', () => {
			const inventory = mainNavigation.find((item) => item.title === 'Inventory');
			expect(inventory?.items).toBeDefined();
			expect(inventory?.items?.length).toBeGreaterThan(0);

			const subTitles = inventory?.items?.map((sub) => sub.title);
			expect(subTitles).toContain('Products');
			expect(subTitles).toContain('Categories');
			expect(subTitles).toContain('Stock Levels');
		});

		it('should have sub-items for Orders section', () => {
			const orders = mainNavigation.find((item) => item.title === 'Orders');
			expect(orders?.items).toBeDefined();
			expect(orders?.items?.length).toBeGreaterThan(0);

			const subTitles = orders?.items?.map((sub) => sub.title);
			expect(subTitles).toContain('Sales Orders');
			expect(subTitles).toContain('Purchase Orders');
		});
	});

	describe('settingsNavigation', () => {
		it('should have Admin and Settings items', () => {
			expect(settingsNavigation.length).toBe(2);
			expect(settingsNavigation[0].title).toBe('Admin');
			expect(settingsNavigation[0].url).toBe('/admin');
			expect(settingsNavigation[1].title).toBe('Settings');
			expect(settingsNavigation[1].url).toBe('/settings');
		});

		it('should have icon for Settings', () => {
			expect(settingsNavigation[0].icon).toBeDefined();
			expect(settingsNavigation[1].icon).toBeDefined();
		});

		it('should have admin sub-items', () => {
			const admin = settingsNavigation[0];
			expect(admin.items).toBeDefined();

			const subTitles = admin.items?.map((sub) => sub.title);
			expect(subTitles).toContain('Users');
			expect(subTitles).toContain('Roles');
			expect(subTitles).toContain('Invitations');
		});

		it('should have settings sub-items', () => {
			const settings = settingsNavigation[1];
			expect(settings.items).toBeDefined();

			const subTitles = settings.items?.map((sub) => sub.title);
			expect(subTitles).toContain('Profile');
			expect(subTitles).toContain('Organization');
		});
	});

	describe('navigationGroups', () => {
		it('should have Main and Settings groups', () => {
			const labels = navigationGroups.map((group) => group.label);
			expect(labels).toContain('Main');
			expect(labels).toContain('Settings');
		});

		it('should contain mainNavigation in Main group', () => {
			const mainGroup = navigationGroups.find((g) => g.label === 'Main');
			expect(mainGroup?.items).toBe(mainNavigation);
		});

		it('should contain settingsNavigation in Settings group', () => {
			const settingsGroup = navigationGroups.find((g) => g.label === 'Settings');
			expect(settingsGroup?.items).toBe(settingsNavigation);
		});
	});

	describe('getAllNavigationItems', () => {
		it('should return a flat array of all navigation items', () => {
			const items = getAllNavigationItems();
			expect(Array.isArray(items)).toBe(true);
			expect(items.length).toBeGreaterThan(0);
		});

		it('should include main nav items', () => {
			const items = getAllNavigationItems();
			const titles = items.map((item) => item.title);
			expect(titles).toContain('Dashboard');
			expect(titles).toContain('Inventory');
		});

		it('should include sub-items', () => {
			const items = getAllNavigationItems();
			const titles = items.map((item) => item.title);
			expect(titles).toContain('Products');
			expect(titles).toContain('Sales Orders');
		});

		it('should include settings items', () => {
			const items = getAllNavigationItems();
			const titles = items.map((item) => item.title);
			expect(titles).toContain('Settings');
			expect(titles).toContain('Profile');
		});

		it('should not have unexpected duplicates', () => {
			const items = getAllNavigationItems();
			const urls = items.map((item) => item.url);
			const uniqueUrls = [...new Set(urls)];
			// Note: /settings appears twice (Settings parent and Profile sub-item)
			// This is intentional as Profile points to the settings root
			const expectedDuplicates = 1; // /settings
			expect(urls.length).toBe(uniqueUrls.length + expectedDuplicates);
		});
	});

	describe('isPathActive', () => {
		it('should return true for exact dashboard match', () => {
			expect(isPathActive('/dashboard', '/dashboard')).toBe(true);
		});

		it('should return false for dashboard when on sub-path', () => {
			// Dashboard is special - exact match only
			expect(isPathActive('/dashboard/overview', '/dashboard')).toBe(false);
		});

		it('should return true for path prefix match (non-dashboard)', () => {
			expect(isPathActive('/inventory/products', '/inventory')).toBe(true);
			expect(isPathActive('/inventory/categories', '/inventory')).toBe(true);
			expect(isPathActive('/orders/sales', '/orders')).toBe(true);
		});

		it('should return true for exact match on sub-paths', () => {
			expect(isPathActive('/inventory/products', '/inventory/products')).toBe(true);
			expect(isPathActive('/settings/profile', '/settings/profile')).toBe(true);
		});

		it('should return false for non-matching paths', () => {
			expect(isPathActive('/inventory', '/orders')).toBe(false);
			expect(isPathActive('/dashboard', '/settings')).toBe(false);
		});

		it('should return false for partial non-prefix matches', () => {
			expect(isPathActive('/inventory-new', '/inventory')).toBe(false);
		});

		it('should handle root path', () => {
			expect(isPathActive('/', '/')).toBe(true);
			// Root path should only match exactly, not all paths
			expect(isPathActive('/dashboard', '/')).toBe(false);
		});
	});

	describe('hasActiveChild', () => {
		const testSubItems: NavSubItem[] = [
			{ title: 'Products', url: '/inventory/products' },
			{ title: 'Categories', url: '/inventory/categories' },
			{ title: 'Stock', url: '/inventory/stock-levels' }
		];

		it('should return true when current path matches a sub-item', () => {
			expect(hasActiveChild('/inventory/products', testSubItems)).toBe(true);
			expect(hasActiveChild('/inventory/categories', testSubItems)).toBe(true);
		});

		it('should return true when current path is a sub-path of a sub-item', () => {
			expect(hasActiveChild('/inventory/products/123', testSubItems)).toBe(true);
			expect(hasActiveChild('/inventory/categories/edit', testSubItems)).toBe(true);
		});

		it('should return false when no sub-item matches', () => {
			expect(hasActiveChild('/orders/sales', testSubItems)).toBe(false);
			expect(hasActiveChild('/dashboard', testSubItems)).toBe(false);
		});

		it('should return false for undefined items', () => {
			expect(hasActiveChild('/inventory/products', undefined)).toBe(false);
		});

		it('should return false for empty items array', () => {
			expect(hasActiveChild('/inventory/products', [])).toBe(false);
		});

		it('should handle the parent path without matching', () => {
			// Parent path /inventory should not trigger hasActiveChild
			// because none of the sub-items are /inventory exactly
			expect(hasActiveChild('/inventory', testSubItems)).toBe(false);
		});
	});

	describe('NavItem type validation', () => {
		it('should have required properties on NavItem', () => {
			const item: NavItem = mainNavigation[0];
			expect(item.title).toBeDefined();
			expect(typeof item.title).toBe('string');
			expect(item.url).toBeDefined();
			expect(typeof item.url).toBe('string');
		});

		it('should have optional icon property', () => {
			const itemWithIcon = mainNavigation.find((item) => item.icon);
			expect(itemWithIcon?.icon).toBeDefined();
		});

		it('should have optional items property for sub-navigation', () => {
			const itemWithSubs = mainNavigation.find((item) => item.items && item.items.length > 0);
			expect(itemWithSubs?.items).toBeDefined();
			expect(Array.isArray(itemWithSubs?.items)).toBe(true);
		});
	});

	describe('URL structure validation', () => {
		it('should have all URLs starting with /', () => {
			const items = getAllNavigationItems();
			items.forEach((item) => {
				expect(item.url.startsWith('/')).toBe(true);
			});
		});

		it('should not have trailing slashes in URLs', () => {
			const items = getAllNavigationItems();
			items.forEach((item) => {
				if (item.url !== '/') {
					expect(item.url.endsWith('/')).toBe(false);
				}
			});
		});

		it('should have valid URL paths (no spaces or special chars)', () => {
			const items = getAllNavigationItems();
			const validPathRegex = /^\/[a-z0-9-/]*$/;
			items.forEach((item) => {
				expect(validPathRegex.test(item.url)).toBe(true);
			});
		});
	});
});
