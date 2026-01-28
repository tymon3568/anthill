// =============================================================================
// Category API Client
// Handles all category-related API operations for inventory service
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	CategoryResponse,
	CategoryCreateRequest,
	CategoryUpdateRequest,
	CategoryListResponse,
	CategoryListParams,
	CategoryStatsResponse,
	BulkCategoryIds,
	BulkOperationResponse
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/categories';

/**
 * Category API client for inventory service
 */
export const categoryApi = {
	/**
	 * List categories with optional filtering and pagination
	 */
	async list(params: CategoryListParams = {}): Promise<ApiResponse<CategoryListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<CategoryListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get category tree structure
	 */
	async getTree(rootId?: string): Promise<ApiResponse<CategoryResponse[]>> {
		const query = rootId ? `?root_id=${rootId}` : '';
		return apiClient.get<CategoryResponse[]>(`${BASE_PATH}/tree${query}`);
	},

	/**
	 * Get a single category by ID
	 */
	async get(categoryId: string): Promise<ApiResponse<CategoryResponse>> {
		return apiClient.get<CategoryResponse>(`${BASE_PATH}/${categoryId}`);
	},

	/**
	 * Create a new category
	 */
	async create(data: CategoryCreateRequest): Promise<ApiResponse<CategoryResponse>> {
		return apiClient.post<CategoryResponse>(BASE_PATH, toRecord(data));
	},

	/**
	 * Update an existing category
	 */
	async update(
		categoryId: string,
		data: CategoryUpdateRequest
	): Promise<ApiResponse<CategoryResponse>> {
		return apiClient.put<CategoryResponse>(`${BASE_PATH}/${categoryId}`, toRecord(data));
	},

	/**
	 * Delete a category
	 */
	async delete(categoryId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/${categoryId}`);
	},

	/**
	 * Get category statistics
	 */
	async getStats(categoryId: string): Promise<ApiResponse<CategoryStatsResponse>> {
		return apiClient.get<CategoryStatsResponse>(`${BASE_PATH}/${categoryId}/stats`);
	},

	/**
	 * Bulk activate categories
	 */
	async bulkActivate(data: BulkCategoryIds): Promise<ApiResponse<BulkOperationResponse>> {
		return apiClient.post<BulkOperationResponse>(`${BASE_PATH}/bulk/activate`, toRecord(data));
	},

	/**
	 * Bulk deactivate categories
	 */
	async bulkDeactivate(data: BulkCategoryIds): Promise<ApiResponse<BulkOperationResponse>> {
		return apiClient.post<BulkOperationResponse>(`${BASE_PATH}/bulk/deactivate`, toRecord(data));
	},

	/**
	 * Bulk delete categories
	 */
	async bulkDelete(data: BulkCategoryIds): Promise<ApiResponse<BulkOperationResponse>> {
		return apiClient.post<BulkOperationResponse>(`${BASE_PATH}/bulk/delete`, toRecord(data));
	}
};
