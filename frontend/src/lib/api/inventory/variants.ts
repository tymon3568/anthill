// =============================================================================
// Variant API Client
// Handles cross-product variant operations for inventory service
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	VariantResponse,
	VariantCreateRequest,
	VariantUpdateRequest,
	VariantListResponse,
	VariantListParams,
	BulkOperationResponse
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/variants';

/**
 * Variant API client for inventory service
 * Provides cross-product variant management capabilities
 */
export const variantApi = {
	/**
	 * List variants with optional filtering and pagination
	 * Supports search across SKU, barcode, and parent product name
	 */
	async list(params: VariantListParams = {}): Promise<ApiResponse<VariantListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<VariantListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get a single variant by ID
	 */
	async get(variantId: string): Promise<ApiResponse<VariantResponse>> {
		return apiClient.get<VariantResponse>(`${BASE_PATH}/${variantId}`);
	},

	/**
	 * Get a variant by SKU
	 */
	async getBySku(sku: string): Promise<ApiResponse<VariantResponse>> {
		return apiClient.get<VariantResponse>(`${BASE_PATH}/by-sku/${encodeURIComponent(sku)}`);
	},

	/**
	 * Get a variant by barcode
	 */
	async getByBarcode(barcode: string): Promise<ApiResponse<VariantResponse>> {
		return apiClient.get<VariantResponse>(`${BASE_PATH}/by-barcode/${encodeURIComponent(barcode)}`);
	},

	/**
	 * List variants for a specific product
	 */
	async listByProduct(
		parentProductId: string,
		params: Omit<VariantListParams, 'parentProductId'> = {}
	): Promise<ApiResponse<VariantListResponse>> {
		const fullParams: VariantListParams = { ...params, parentProductId };
		const query = buildQueryString(toRecord(fullParams));
		return apiClient.get<VariantListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Create a new variant
	 */
	async create(data: VariantCreateRequest): Promise<ApiResponse<VariantResponse>> {
		return apiClient.post<VariantResponse>(BASE_PATH, toRecord(data));
	},

	/**
	 * Update an existing variant
	 */
	async update(
		variantId: string,
		data: VariantUpdateRequest
	): Promise<ApiResponse<VariantResponse>> {
		return apiClient.put<VariantResponse>(`${BASE_PATH}/${variantId}`, toRecord(data));
	},

	/**
	 * Delete a variant (soft delete)
	 */
	async delete(variantId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/${variantId}`);
	},

	/**
	 * Bulk activate variants
	 */
	async bulkActivate(variantIds: string[]): Promise<ApiResponse<BulkOperationResponse>> {
		return apiClient.post<BulkOperationResponse>(`${BASE_PATH}/bulk/activate`, { variantIds });
	},

	/**
	 * Bulk deactivate variants
	 */
	async bulkDeactivate(variantIds: string[]): Promise<ApiResponse<BulkOperationResponse>> {
		return apiClient.post<BulkOperationResponse>(`${BASE_PATH}/bulk/deactivate`, { variantIds });
	},

	/**
	 * Bulk delete variants
	 */
	async bulkDelete(variantIds: string[]): Promise<ApiResponse<BulkOperationResponse>> {
		return apiClient.post<BulkOperationResponse>(`${BASE_PATH}/bulk/delete`, { variantIds });
	}
};
