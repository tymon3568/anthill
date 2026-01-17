import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';

// Order types
export interface Order {
	id: string;
	orderNumber: string;
	customerId: string;
	customerName: string;
	customerEmail: string;
	status: OrderStatus;
	items: OrderItem[];
	subtotal: number;
	tax: number;
	shipping: number;
	total: number;
	shippingAddress: Address;
	billingAddress: Address;
	paymentMethod: string;
	paymentStatus: 'pending' | 'paid' | 'failed' | 'refunded';
	notes?: string;
	createdAt: string;
	updatedAt: string;
}

export type OrderStatus =
	| 'pending'
	| 'confirmed'
	| 'processing'
	| 'shipped'
	| 'delivered'
	| 'cancelled'
	| 'refunded';

export interface OrderItem {
	id: string;
	productId: string;
	productName: string;
	sku: string;
	quantity: number;
	unitPrice: number;
	total: number;
}

export interface Address {
	name: string;
	line1: string;
	line2?: string;
	city: string;
	state: string;
	postalCode: string;
	country: string;
	phone?: string;
}

export interface OrderListParams {
	page?: number;
	perPage?: number;
	status?: OrderStatus;
	search?: string;
	dateFrom?: string;
	dateTo?: string;
	sortBy?: string;
	sortOrder?: 'asc' | 'desc';
}

export interface OrderListResponse {
	items: Order[];
	total: number;
	page: number;
	perPage: number;
	totalPages: number;
}

// Orders API client
export const ordersApi = {
	async list(params: OrderListParams = {}): Promise<ApiResponse<OrderListResponse>> {
		const queryParams = new URLSearchParams();
		Object.entries(params).forEach(([key, value]) => {
			if (value !== undefined) queryParams.set(key, String(value));
		});
		const query = queryParams.toString();
		return apiClient.get<OrderListResponse>(`/orders${query ? `?${query}` : ''}`);
	},

	async get(id: string): Promise<ApiResponse<Order>> {
		return apiClient.get<Order>(`/orders/${id}`);
	},

	async updateStatus(id: string, status: OrderStatus): Promise<ApiResponse<Order>> {
		return apiClient.put<Order>(`/orders/${id}/status`, { status });
	},

	async cancel(id: string, reason?: string): Promise<ApiResponse<Order>> {
		return apiClient.post<Order>(`/orders/${id}/cancel`, { reason });
	}
};

// Mock data
export const mockOrders: Order[] = [
	{
		id: '1',
		orderNumber: 'ORD-2026-001',
		customerId: 'cust-1',
		customerName: 'John Doe',
		customerEmail: 'john@example.com',
		status: 'processing',
		items: [
			{
				id: '1',
				productId: 'p1',
				productName: 'Wireless Headphones',
				sku: 'PROD-001',
				quantity: 1,
				unitPrice: 149.99,
				total: 149.99
			}
		],
		subtotal: 149.99,
		tax: 15.0,
		shipping: 10.0,
		total: 174.99,
		shippingAddress: {
			name: 'John Doe',
			line1: '123 Main St',
			city: 'New York',
			state: 'NY',
			postalCode: '10001',
			country: 'US'
		},
		billingAddress: {
			name: 'John Doe',
			line1: '123 Main St',
			city: 'New York',
			state: 'NY',
			postalCode: '10001',
			country: 'US'
		},
		paymentMethod: 'Credit Card',
		paymentStatus: 'paid',
		createdAt: '2026-01-17T10:30:00Z',
		updatedAt: '2026-01-17T10:30:00Z'
	},
	{
		id: '2',
		orderNumber: 'ORD-2026-002',
		customerId: 'cust-2',
		customerName: 'Jane Smith',
		customerEmail: 'jane@example.com',
		status: 'shipped',
		items: [
			{
				id: '2',
				productId: 'p2',
				productName: 'USB-C Cable',
				sku: 'PROD-002',
				quantity: 2,
				unitPrice: 19.99,
				total: 39.98
			}
		],
		subtotal: 39.98,
		tax: 4.0,
		shipping: 5.0,
		total: 48.98,
		shippingAddress: {
			name: 'Jane Smith',
			line1: '456 Oak Ave',
			city: 'Los Angeles',
			state: 'CA',
			postalCode: '90001',
			country: 'US'
		},
		billingAddress: {
			name: 'Jane Smith',
			line1: '456 Oak Ave',
			city: 'Los Angeles',
			state: 'CA',
			postalCode: '90001',
			country: 'US'
		},
		paymentMethod: 'PayPal',
		paymentStatus: 'paid',
		createdAt: '2026-01-16T14:20:00Z',
		updatedAt: '2026-01-17T08:15:00Z'
	}
];
