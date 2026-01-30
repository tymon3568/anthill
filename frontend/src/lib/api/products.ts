/**
 * Product API Client
 *
 * Handles all product-related API calls including:
 * - Product CRUD operations
 * - Product variants
 * - Unit of Measures
 */

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	Product,
	ProductVariant,
	UnitOfMeasure,
	ProductListParams,
	ProductListResponse,
	CreateProductRequest,
	UpdateProductRequest,
	CreateVariantRequest,
	UpdateVariantRequest
} from '$lib/types/products';

// ============================================================
// PRODUCT API
// ============================================================

export const productsApi = {
	/**
	 * List products with filtering and pagination
	 */
	async list(params: ProductListParams = {}): Promise<ApiResponse<ProductListResponse>> {
		const queryParams = new URLSearchParams();

		if (params.page) queryParams.set('page', params.page.toString());
		if (params.limit) queryParams.set('limit', params.limit.toString());
		if (params.search) queryParams.set('search', params.search);
		if (params.productType) queryParams.set('product_type', params.productType);
		if (params.categoryId) queryParams.set('category_id', params.categoryId);
		if (params.isActive !== undefined) queryParams.set('is_active', params.isActive.toString());
		if (params.trackingMethod) queryParams.set('tracking_method', params.trackingMethod);
		if (params.sortBy) queryParams.set('sort_by', params.sortBy);
		if (params.sortOrder) queryParams.set('sort_order', params.sortOrder);

		const query = queryParams.toString();
		return apiClient.get<ProductListResponse>(`/products${query ? `?${query}` : ''}`);
	},

	/**
	 * Get single product by ID
	 */
	async get(id: string): Promise<ApiResponse<Product>> {
		return apiClient.get<Product>(`/products/${id}`);
	},

	/**
	 * Create new product
	 */
	async create(data: CreateProductRequest): Promise<ApiResponse<Product>> {
		return apiClient.post<Product>('/products', data as unknown as Record<string, unknown>);
	},

	/**
	 * Update existing product
	 */
	async update(id: string, data: UpdateProductRequest): Promise<ApiResponse<Product>> {
		return apiClient.put<Product>(`/products/${id}`, data as unknown as Record<string, unknown>);
	},

	/**
	 * Delete product (soft delete)
	 */
	async delete(id: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/products/${id}`);
	},

	/**
	 * Bulk delete products
	 */
	async bulkDelete(ids: string[]): Promise<ApiResponse<{ deleted: number }>> {
		return apiClient.post<{ deleted: number }>('/products/bulk-delete', { ids });
	},

	/**
	 * Export products to CSV
	 */
	async export(params: ProductListParams = {}): Promise<ApiResponse<Blob>> {
		const queryParams = new URLSearchParams();
		if (params.search) queryParams.set('search', params.search);
		if (params.productType) queryParams.set('product_type', params.productType);
		if (params.isActive !== undefined) queryParams.set('is_active', params.isActive.toString());

		const query = queryParams.toString();
		return apiClient.get<Blob>(`/products/export${query ? `?${query}` : ''}`);
	}
};

// ============================================================
// PRODUCT VARIANTS API
// ============================================================

export const variantsApi = {
	/**
	 * List variants for a product
	 */
	async list(productId: string): Promise<ApiResponse<ProductVariant[]>> {
		return apiClient.get<ProductVariant[]>(`/inventory/variants?parentProductId=${productId}`);
	},

	/**
	 * Get single variant
	 */
	async get(_productId: string, variantId: string): Promise<ApiResponse<ProductVariant>> {
		return apiClient.get<ProductVariant>(`/inventory/variants/${variantId}`);
	},

	/**
	 * Create variant for a product
	 */
	async create(
		productId: string,
		data: CreateVariantRequest
	): Promise<ApiResponse<ProductVariant>> {
		return apiClient.post<ProductVariant>('/inventory/variants', {
			...data,
			parentProductId: productId
		} as unknown as Record<string, unknown>);
	},

	/**
	 * Update variant
	 */
	async update(
		_productId: string,
		variantId: string,
		data: UpdateVariantRequest
	): Promise<ApiResponse<ProductVariant>> {
		return apiClient.put<ProductVariant>(
			`/inventory/variants/${variantId}`,
			data as unknown as Record<string, unknown>
		);
	},

	/**
	 * Delete variant
	 */
	async delete(_productId: string, variantId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/inventory/variants/${variantId}`);
	}
};

// ============================================================
// UNIT OF MEASURES API
// ============================================================

export const uomApi = {
	/**
	 * List all UOMs
	 */
	async list(): Promise<ApiResponse<UnitOfMeasure[]>> {
		return apiClient.get<UnitOfMeasure[]>('/inventory/uom');
	},

	/**
	 * Get single UOM
	 */
	async get(id: string): Promise<ApiResponse<UnitOfMeasure>> {
		return apiClient.get<UnitOfMeasure>(`/inventory/uom/${id}`);
	},

	/**
	 * Create UOM
	 */
	async create(
		data: Omit<UnitOfMeasure, 'uomId' | 'tenantId' | 'createdAt' | 'updatedAt'>
	): Promise<ApiResponse<UnitOfMeasure>> {
		return apiClient.post<UnitOfMeasure>(
			'/inventory/uom',
			data as unknown as Record<string, unknown>
		);
	},

	/**
	 * Update UOM
	 */
	async update(id: string, data: Partial<UnitOfMeasure>): Promise<ApiResponse<UnitOfMeasure>> {
		return apiClient.put<UnitOfMeasure>(
			`/inventory/uom/${id}`,
			data as unknown as Record<string, unknown>
		);
	},

	/**
	 * Delete UOM
	 */
	async delete(id: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`/inventory/uom/${id}`);
	}
};

// ============================================================
// RE-EXPORT TYPES AND MOCK DATA
// ============================================================

export type {
	Product,
	ProductVariant,
	UnitOfMeasure,
	ProductListParams,
	ProductListResponse,
	CreateProductRequest,
	UpdateProductRequest,
	CreateVariantRequest,
	UpdateVariantRequest
} from '$lib/types/products';

export { mockProducts, mockVariants, mockUoms } from '$lib/types/products';
