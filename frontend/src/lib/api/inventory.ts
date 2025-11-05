import { apiClient, createPaginationParams } from './client';
import type {
	ApiResponse,
	PaginatedResponse,
	Product,
	Category,
	ProductForm,
	InventoryTransaction,
	Order
} from '$lib/types';

export const inventoryApi = {
	// Products
	async getProducts(
		page: number = 1,
		limit: number = 10,
		search?: string
	): Promise<ApiResponse<PaginatedResponse<Product>>> {
		const params = createPaginationParams(page, limit);
		if (search) params.append('search', search);

		return apiClient.get<PaginatedResponse<Product>>(`/products?${params}`);
	},

	async getProduct(id: string): Promise<ApiResponse<Product>> {
		return apiClient.get<Product>(`/products/${id}`);
	},

	async createProduct(product: ProductForm): Promise<ApiResponse<Product>> {
		return apiClient.post<Product>('/products', product as unknown as Record<string, unknown>);
	},

	async updateProduct(id: string, product: Partial<ProductForm>): Promise<ApiResponse<Product>> {
		return apiClient.put<Product>(`/products/${id}`, product as unknown as Record<string, unknown>);
	},

	async deleteProduct(id: string): Promise<ApiResponse<void>> {
		return apiClient.delete(`/products/${id}`);
	},

	// Categories
	async getCategories(): Promise<ApiResponse<Category[]>> {
		return apiClient.get<Category[]>('/categories');
	},

	async createCategory(name: string, description?: string): Promise<ApiResponse<Category>> {
		return apiClient.post<Category>('/categories', { name, description });
	},

	async updateCategory(
		id: string,
		name: string,
		description?: string
	): Promise<ApiResponse<Category>> {
		return apiClient.put<Category>(`/categories/${id}`, { name, description });
	},

	async deleteCategory(id: string): Promise<ApiResponse<void>> {
		return apiClient.delete(`/categories/${id}`);
	},

	// Inventory Transactions
	async getTransactions(
		productId?: string,
		page: number = 1,
		limit: number = 10
	): Promise<ApiResponse<PaginatedResponse<InventoryTransaction>>> {
		const params = createPaginationParams(page, limit);
		if (productId) params.append('product_id', productId);

		return apiClient.get<PaginatedResponse<InventoryTransaction>>(
			`/inventory/transactions?${params}`
		);
	},

	async createTransaction(
		transaction: Omit<InventoryTransaction, 'id' | 'createdAt' | 'tenantId'>
	): Promise<ApiResponse<InventoryTransaction>> {
		return apiClient.post<InventoryTransaction>('/inventory/transactions', transaction);
	},

	// Orders
	async getOrders(
		page: number = 1,
		limit: number = 10,
		status?: string
	): Promise<ApiResponse<PaginatedResponse<Order>>> {
		const params = createPaginationParams(page, limit);
		if (status) params.append('status', status);

		return apiClient.get<PaginatedResponse<Order>>(`/orders?${params}`);
	},

	async getOrder(id: string): Promise<ApiResponse<Order>> {
		return apiClient.get<Order>(`/orders/${id}`);
	},

	async createOrder(
		order: Omit<Order, 'id' | 'orderNumber' | 'createdAt' | 'updatedAt' | 'tenantId'>
	): Promise<ApiResponse<Order>> {
		return apiClient.post<Order>('/orders', order);
	},

	async updateOrderStatus(id: string, status: Order['status']): Promise<ApiResponse<Order>> {
		return apiClient.patch<Order>(`/orders/${id}/status`, { status });
	}
};
