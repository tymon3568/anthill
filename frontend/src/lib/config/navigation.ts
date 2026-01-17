import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
import PackageIcon from '@lucide/svelte/icons/package';
import ShoppingCartIcon from '@lucide/svelte/icons/shopping-cart';
import PlugIcon from '@lucide/svelte/icons/plug';
import SettingsIcon from '@lucide/svelte/icons/settings';
import ShieldIcon from '@lucide/svelte/icons/shield';
import type { Component } from 'svelte';

export interface NavItem {
	title: string;
	url: string;
	icon?: Component;
	isActive?: boolean;
	badge?: string | number;
	items?: NavSubItem[];
	/** If true, only show this item for admin/owner roles */
	adminOnly?: boolean;
}

export interface NavSubItem {
	title: string;
	url: string;
	badge?: string | number;
}

export interface NavGroup {
	label: string;
	items: NavItem[];
}

// Main navigation items for the sidebar
export const mainNavigation: NavItem[] = [
	{
		title: 'Dashboard',
		url: '/dashboard',
		icon: LayoutDashboardIcon
	},
	{
		title: 'Inventory',
		url: '/inventory',
		icon: PackageIcon,
		items: [
			{ title: 'Products', url: '/inventory/products' },
			{ title: 'Categories', url: '/inventory/categories' },
			{ title: 'Stock Levels', url: '/inventory/stock' },
			{ title: 'Adjustments', url: '/inventory/adjustments' },
			{ title: 'Warehouses', url: '/inventory/warehouses' }
		]
	},
	{
		title: 'Orders',
		url: '/orders',
		icon: ShoppingCartIcon,
		items: [
			{ title: 'Sales Orders', url: '/orders/sales' },
			{ title: 'Purchase Orders', url: '/orders/purchase' },
			{ title: 'Returns', url: '/orders/returns' }
		]
	},
	{
		title: 'Integrations',
		url: '/integrations',
		icon: PlugIcon,
		items: [
			{ title: 'Marketplaces', url: '/integrations/marketplaces' },
			{ title: 'Sync Status', url: '/integrations/sync' },
			{ title: 'Webhooks', url: '/integrations/webhooks' }
		]
	}
];

// Settings navigation - shown at bottom of sidebar
export const settingsNavigation: NavItem[] = [
	{
		title: 'Admin',
		url: '/admin',
		icon: ShieldIcon,
		adminOnly: true,
		items: [
			{ title: 'Users', url: '/admin/users' },
			{ title: 'Roles', url: '/admin/roles' },
			{ title: 'Invitations', url: '/admin/invitations' }
		]
	},
	{
		title: 'Settings',
		url: '/settings',
		icon: SettingsIcon,
		items: [
			{ title: 'Profile', url: '/settings' },
			{ title: 'Organization', url: '/settings/tenant' },
			{ title: 'Payments', url: '/settings/payment' }
		]
	}
];

// Grouped navigation for command palette search
export const navigationGroups: NavGroup[] = [
	{
		label: 'Main',
		items: mainNavigation
	},
	{
		label: 'Settings',
		items: settingsNavigation
	}
];

// Flatten navigation for search
export function getAllNavigationItems(): (NavItem | NavSubItem)[] {
	const items: (NavItem | NavSubItem)[] = [];

	const processItems = (navItems: NavItem[]) => {
		for (const item of navItems) {
			items.push(item);
			if (item.items) {
				items.push(...item.items);
			}
		}
	};

	processItems(mainNavigation);
	processItems(settingsNavigation);

	return items;
}

// Check if a path is active
export function isPathActive(currentPath: string, itemPath: string): boolean {
	if (itemPath === '/dashboard') {
		return currentPath === '/dashboard';
	}
	// Check for exact match or path prefix followed by / or end of string
	if (currentPath === itemPath) {
		return true;
	}
	// Ensure we match path boundaries (e.g., /inventory matches /inventory/products but not /inventory-new)
	return currentPath.startsWith(itemPath + '/');
}

// Check if a parent item has an active child
export function hasActiveChild(currentPath: string, items?: NavSubItem[]): boolean {
	if (!items) return false;
	return items.some((item) => isPathActive(currentPath, item.url));
}
