import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';

// Dashboard metrics types
export interface DashboardMetrics {
	totalProducts: number;
	lowStockAlerts: number;
	categories: number;
	totalOrders: number;
	inventoryValue: number;
	pendingShipments: number;
}

export interface SalesTrendItem {
	label: string;
	value: number;
	date: string;
}

export interface InventoryCategoryItem {
	category: string;
	count: number;
	value: number;
}

export interface RecentOrder {
	id: string;
	orderNumber: string;
	customerName: string;
	total: number;
	status: 'pending' | 'processing' | 'shipped' | 'delivered' | 'cancelled';
	createdAt: string;
}

export interface DashboardData {
	metrics: DashboardMetrics;
	salesTrend: SalesTrendItem[];
	inventoryByCategory: InventoryCategoryItem[];
	recentOrders: RecentOrder[];
}

// Dashboard API client
export const dashboardApi = {
	// Get dashboard metrics
	async getMetrics(): Promise<ApiResponse<DashboardMetrics>> {
		return apiClient.get<DashboardMetrics>('/dashboard/metrics');
	},

	// Get sales trend data
	async getSalesTrend(period: 'week' | 'month' | 'year' = 'month'): Promise<ApiResponse<SalesTrendItem[]>> {
		return apiClient.get<SalesTrendItem[]>(`/dashboard/sales-trend?period=${period}`);
	},

	// Get inventory by category
	async getInventoryByCategory(): Promise<ApiResponse<InventoryCategoryItem[]>> {
		return apiClient.get<InventoryCategoryItem[]>('/dashboard/inventory-categories');
	},

	// Get recent orders
	async getRecentOrders(limit: number = 10): Promise<ApiResponse<RecentOrder[]>> {
		return apiClient.get<RecentOrder[]>(`/dashboard/recent-orders?limit=${limit}`);
	},

	// Get all dashboard data in one call
	async getDashboardData(): Promise<ApiResponse<DashboardData>> {
		return apiClient.get<DashboardData>('/dashboard');
	}
};

// Mock data for development/testing
export const mockDashboardData: DashboardData = {
	metrics: {
		totalProducts: 1248,
		lowStockAlerts: 23,
		categories: 45,
		totalOrders: 892,
		inventoryValue: 124500,
		pendingShipments: 67
	},
	salesTrend: [
		{ label: 'Jan', value: 4200, date: '2026-01-01' },
		{ label: 'Feb', value: 3800, date: '2026-02-01' },
		{ label: 'Mar', value: 5100, date: '2026-03-01' },
		{ label: 'Apr', value: 4600, date: '2026-04-01' },
		{ label: 'May', value: 5800, date: '2026-05-01' },
		{ label: 'Jun', value: 6200, date: '2026-06-01' }
	],
	inventoryByCategory: [
		{ category: 'Electronics', count: 320, value: 45000 },
		{ category: 'Clothing', count: 450, value: 32000 },
		{ category: 'Home', count: 280, value: 28000 },
		{ category: 'Sports', count: 180, value: 15000 },
		{ category: 'Books', count: 120, value: 4500 }
	],
	recentOrders: [
		{
			id: '1',
			orderNumber: 'ORD-12345',
			customerName: 'John Doe',
			total: 299.99,
			status: 'processing',
			createdAt: '2026-01-17T10:30:00Z'
		},
		{
			id: '2',
			orderNumber: 'ORD-12344',
			customerName: 'Jane Smith',
			total: 149.50,
			status: 'shipped',
			createdAt: '2026-01-17T09:15:00Z'
		},
		{
			id: '3',
			orderNumber: 'ORD-12343',
			customerName: 'Bob Wilson',
			total: 89.00,
			status: 'delivered',
			createdAt: '2026-01-16T16:45:00Z'
		}
	]
};
