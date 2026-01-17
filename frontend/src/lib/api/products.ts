import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';

// Product types
export interface Product {
	id: string;
	sku: string;
	name: string;
	description?: string;
	categoryId: string;
	categoryName?: string;
	price: number;
	costPrice?: number;
	quantity: number;
	minQuantity: number;
	maxQuantity?: number;
	unit: string;
	status: 'active' | 'inactive' | 'discontinued';
	warehouseId?: string;
	warehouseName?: string;
	imageUrl?: string;
	tags?: string[];
	createdAt: string;
	updatedAt: string;
}

export interface ProductListParams {
	page?: number;
	perPage?: number;
	search?: string;
	categoryId?: string;
	status?: string;
	warehouseId?: string;
	sortBy?: string;
	sortOrder?: 'asc' | 'desc';
	lowStock?: boolean;
}

export interface ProductListResponse {
	items: Product[];
	total: number;
	page: number;
	perPage: number;
	totalPages: number;
}

export interface CreateProductRequest {
	sku: string;
	name: string;
	description?: string;
	categoryId: string;
	price: number;
	costPrice?: number;
	quantity: number;
	minQuantity: number;
	maxQuantity?: number;
	unit: string;
	warehouseId?: string;
	imageUrl?: string;
	tags?: string[];
}

export interface UpdateProductRequest extends Partial<CreateProductRequest> {
	status?: 'active' | 'inactive' | 'discontinued';
}

export interface Category {
	id: string;
	name: string;
	parentId?: string;
	productCount?: number;
}

// Products API client
export const productsApi = {
	// List products with filtering and pagination
	async list(params: ProductListParams = {}): Promise<ApiResponse<ProductListResponse>> {
		const queryParams = new URLSearchParams();

		if (params.page) queryParams.set('page', params.page.toString());
		if (params.perPage) queryParams.set('per_page', params.perPage.toString());
		if (params.search) queryParams.set('search', params.search);
		if (params.categoryId) queryParams.set('category_id', params.categoryId);
		if (params.status) queryParams.set('status', params.status);
		if (params.warehouseId) queryParams.set('warehouse_id', params.warehouseId);
		if (params.sortBy) queryParams.set('sort_by', params.sortBy);
		if (params.sortOrder) queryParams.set('sort_order', params.sortOrder);
		if (params.lowStock) queryParams.set('low_stock', 'true');

		const query = queryParams.toString();
		return apiClient.get<ProductListResponse>(`/products${query ? `?${query}` : ''}`);
	},

	// Get single product
	async get(id: string): Promise<ApiResponse<Product>> {
		return apiClient.get<Product>(`/products/${id}`);
	},

	// Create product
	async create(data: CreateProductRequest): Promise<ApiResponse<Product>> {
		return apiClient.post<Product>('/products', data as unknown as Record<string, unknown>);
	},

	// Update product
	async update(id: string, data: UpdateProductRequest): Promise<ApiResponse<Product>> {
		return apiClient.put<Product>(`/products/${id}`, data as unknown as Record<string, unknown>);
	},

	// Delete product
	async delete(id: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/products/${id}`);
	},

	// Bulk delete products
	async bulkDelete(ids: string[]): Promise<ApiResponse<{ deleted: number }>> {
		return apiClient.post<{ deleted: number }>('/products/bulk-delete', { ids });
	},

	// Get categories
	async getCategories(): Promise<ApiResponse<Category[]>> {
		return apiClient.get<Category[]>('/categories');
	},

	// Update stock quantity
	async updateStock(
		id: string,
		quantity: number,
		reason?: string
	): Promise<ApiResponse<Product>> {
		return apiClient.post<Product>(`/products/${id}/stock`, { quantity, reason });
	}
};

// Mock data for development
export const mockProducts: Product[] = [
	{
		id: '1',
		sku: 'PROD-001',
		name: 'Wireless Bluetooth Headphones',
		description: 'Premium noise-cancelling headphones',
		categoryId: 'cat-1',
		categoryName: 'Electronics',
		price: 149.99,
		costPrice: 75.0,
		quantity: 45,
		minQuantity: 10,
		unit: 'pcs',
		status: 'active',
		warehouseId: 'wh-1',
		warehouseName: 'Main Warehouse',
		tags: ['audio', 'wireless'],
		createdAt: '2026-01-10T10:00:00Z',
		updatedAt: '2026-01-15T14:30:00Z'
	},
	{
		id: '2',
		sku: 'PROD-002',
		name: 'USB-C Charging Cable 2m',
		description: 'Fast charging cable',
		categoryId: 'cat-1',
		categoryName: 'Electronics',
		price: 19.99,
		costPrice: 5.0,
		quantity: 8,
		minQuantity: 20,
		unit: 'pcs',
		status: 'active',
		warehouseId: 'wh-1',
		warehouseName: 'Main Warehouse',
		tags: ['cable', 'usb-c'],
		createdAt: '2026-01-08T09:00:00Z',
		updatedAt: '2026-01-16T11:00:00Z'
	},
	{
		id: '3',
		sku: 'PROD-003',
		name: 'Ergonomic Office Chair',
		description: 'Adjustable lumbar support',
		categoryId: 'cat-2',
		categoryName: 'Furniture',
		price: 299.99,
		costPrice: 150.0,
		quantity: 12,
		minQuantity: 5,
		unit: 'pcs',
		status: 'active',
		warehouseId: 'wh-2',
		warehouseName: 'Secondary Warehouse',
		tags: ['office', 'chair'],
		createdAt: '2026-01-05T08:00:00Z',
		updatedAt: '2026-01-17T09:00:00Z'
	}
];

export const mockCategories: Category[] = [
	{ id: 'cat-1', name: 'Electronics', productCount: 156 },
	{ id: 'cat-2', name: 'Furniture', productCount: 45 },
	{ id: 'cat-3', name: 'Clothing', productCount: 230 },
	{ id: 'cat-4', name: 'Sports', productCount: 89 }
];
