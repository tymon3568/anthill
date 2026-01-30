// =============================================================================
// Product Images API Client
// Handles all product image-related API operations for inventory service
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	ProductImage,
	ProductImagesListResponse,
	UploadImageResponse,
	DeleteImageResponse,
	UpdateProductImageRequest,
	ReorderImagesRequest
} from '$lib/types/product-image';

/**
 * Get the base path for product images API
 */
const getBasePath = (productId: string) => `/inventory/products/${productId}/images`;

/**
 * Product Images API client for inventory service
 */
export const productImageApi = {
	/**
	 * List all images for a product
	 */
	async list(productId: string): Promise<ApiResponse<ProductImagesListResponse>> {
		return apiClient.get<ProductImagesListResponse>(getBasePath(productId));
	},

	/**
	 * Get a single image by ID
	 */
	async get(productId: string, imageId: string): Promise<ApiResponse<ProductImage>> {
		return apiClient.get<ProductImage>(`${getBasePath(productId)}/${imageId}`);
	},

	/**
	 * Upload a new product image
	 */
	async upload(productId: string, file: File): Promise<ApiResponse<UploadImageResponse>> {
		const formData = new FormData();
		formData.append('file', file);

		return apiClient.upload<UploadImageResponse>(getBasePath(productId), formData);
	},

	/**
	 * Update image metadata (alt text)
	 */
	async update(
		productId: string,
		imageId: string,
		data: UpdateProductImageRequest
	): Promise<ApiResponse<ProductImage>> {
		return apiClient.put<ProductImage>(
			`${getBasePath(productId)}/${imageId}`,
			data as Record<string, unknown>
		);
	},

	/**
	 * Delete a product image
	 */
	async delete(productId: string, imageId: string): Promise<ApiResponse<DeleteImageResponse>> {
		return apiClient.delete<DeleteImageResponse>(`${getBasePath(productId)}/${imageId}`);
	},

	/**
	 * Reorder images for a product
	 */
	async reorder(productId: string, imageIds: string[]): Promise<ApiResponse<void>> {
		const request: ReorderImagesRequest = { imageIds };
		return apiClient.post<void>(
			`${getBasePath(productId)}/reorder`,
			request as unknown as Record<string, unknown>
		);
	},

	/**
	 * Set an image as the primary image
	 */
	async setPrimary(productId: string, imageId: string): Promise<ApiResponse<void>> {
		return apiClient.post<void>(`${getBasePath(productId)}/${imageId}/set-primary`, {});
	}
};
