// =============================================================================
// Product API Client
// Handles all product-related API operations for inventory service
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	ProductResponse,
	ProductCreateRequest,
	ProductUpdateRequest,
	ProductListResponse,
	ProductListParams,
	MoveToCategoryRequest,
	BulkOperationResponse
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/products';

/**
 * Product API client for inventory service
 */
export const productApi = {
	/**
	 * List products with optional filtering and pagination
	 */
	async list(params: ProductListParams = {}): Promise<ApiResponse<ProductListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<ProductListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get a single product by ID
	 */
	async get(productId: string): Promise<ApiResponse<ProductResponse>> {
		return apiClient.get<ProductResponse>(`${BASE_PATH}/${productId}`);
	},

	/**
	 * Get a product by SKU
	 */
	async getBySku(sku: string): Promise<ApiResponse<ProductResponse>> {
		return apiClient.get<ProductResponse>(`${BASE_PATH}/sku/${encodeURIComponent(sku)}`);
	},

	/**
	 * Create a new product
	 */
	async create(data: ProductCreateRequest): Promise<ApiResponse<ProductResponse>> {
		return apiClient.post<ProductResponse>(BASE_PATH, toRecord(data));
	},

	/**
	 * Update an existing product
	 */
	async update(
		productId: string,
		data: ProductUpdateRequest
	): Promise<ApiResponse<ProductResponse>> {
		// Use PUT to match backend handler and Casbin policies (PUT, not PATCH)
		return apiClient.put<ProductResponse>(`${BASE_PATH}/${productId}`, toRecord(data));
	},

	/**
	 * Delete a product
	 */
	async delete(productId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/${productId}`);
	},

	/**
	 * Move products to a category
	 */
	async moveToCategory(data: MoveToCategoryRequest): Promise<ApiResponse<BulkOperationResponse>> {
		return apiClient.post<BulkOperationResponse>(`${BASE_PATH}/move-to-category`, toRecord(data));
	},

	/**
	 * Bulk activate products
	 */
	async bulkActivate(productIds: string[]): Promise<ApiResponse<BulkOperationResponse>> {
		return apiClient.post<BulkOperationResponse>(`${BASE_PATH}/bulk/activate`, { productIds });
	},

	/**
	 * Bulk deactivate products
	 */
	async bulkDeactivate(productIds: string[]): Promise<ApiResponse<BulkOperationResponse>> {
		return apiClient.post<BulkOperationResponse>(`${BASE_PATH}/bulk/deactivate`, { productIds });
	},

	/**
	 * Bulk delete products
	 */
	async bulkDelete(productIds: string[]): Promise<ApiResponse<BulkOperationResponse>> {
		return apiClient.post<BulkOperationResponse>(`${BASE_PATH}/bulk/delete`, { productIds });
	}
};
