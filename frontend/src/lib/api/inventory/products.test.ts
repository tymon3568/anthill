// =============================================================================
// Inventory Product API Tests
// Tests for inventory/products.ts API client
// =============================================================================

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { productApi } from './products';
import { apiClient } from '$lib/api/client';
import type {
	ProductResponse,
	ProductCreateRequest,
	ProductUpdateRequest,
	ProductListResponse,
	ProductListParams,
	MoveToCategoryRequest,
	BulkOperationResponse,
	PaginationInfo
} from '$lib/types/inventory';
import type { ApiResponse } from '$lib/types';

// Mock the apiClient
vi.mock('$lib/api/client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		put: vi.fn(),
		delete: vi.fn()
	}
}));

// =============================================================================
// Mock Data
// =============================================================================

const mockPagination: PaginationInfo = {
	page: 1,
	pageSize: 20,
	totalItems: 100,
	totalPages: 5,
	hasNext: true,
	hasPrev: false
};

const mockProduct: ProductResponse = {
	productId: 'prod-001',
	tenantId: 'tenant-001',
	sku: 'SKU-001',
	name: 'Test Product',
	description: 'A test product description',
	productType: 'physical',
	trackInventory: true,
	trackingMethod: 'lot',
	salePrice: 99.99,
	costPrice: 50.0,
	currencyCode: 'VND',
	weightGrams: 500,
	dimensions: { length: 10, width: 5, height: 3 },
	attributes: { color: 'red', size: 'M' },
	defaultUomId: 'uom-001',
	itemGroupId: 'group-001',
	isActive: true,
	isSellable: true,
	isPurchaseable: true,
	createdAt: '2026-01-01T00:00:00Z',
	updatedAt: '2026-01-15T00:00:00Z'
};

const mockProduct2: ProductResponse = {
	productId: 'prod-002',
	tenantId: 'tenant-001',
	sku: 'SKU-002',
	name: 'Another Product',
	description: 'Another test product',
	productType: 'digital',
	trackInventory: false,
	trackingMethod: 'none',
	salePrice: 29.99,
	costPrice: 10.0,
	currencyCode: 'VND',
	weightGrams: undefined,
	dimensions: undefined,
	attributes: undefined,
	defaultUomId: undefined,
	itemGroupId: undefined,
	isActive: true,
	isSellable: true,
	isPurchaseable: false,
	createdAt: '2026-01-05T00:00:00Z',
	updatedAt: '2026-01-20T00:00:00Z'
};

const mockProductListResponse: ProductListResponse = {
	products: [mockProduct, mockProduct2],
	pagination: mockPagination
};

const mockBulkOperationResponse: BulkOperationResponse = {
	success: true,
	affectedCount: 3,
	message: 'Operation completed successfully'
};

// =============================================================================
// Tests
// =============================================================================

describe('Inventory productApi', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	// =========================================================================
	// list() tests
	// =========================================================================
	describe('list', () => {
		it('should list products without params', async () => {
			const mockResponse: ApiResponse<ProductListResponse> = {
				success: true,
				data: mockProductListResponse
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productApi.list();

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/products');
			expect(result).toEqual(mockResponse);
		});

		it('should list products with pagination params', async () => {
			const mockResponse: ApiResponse<ProductListResponse> = {
				success: true,
				data: mockProductListResponse
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const params: ProductListParams = {
				page: 2,
				pageSize: 10,
				sortBy: 'name',
				sortDir: 'asc'
			};

			const result = await productApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith(
				'/inventory/products?page=2&pageSize=10&sortBy=name&sortDir=asc'
			);
			expect(result).toEqual(mockResponse);
		});

		it('should list products with filter params', async () => {
			const mockResponse: ApiResponse<ProductListResponse> = {
				success: true,
				data: mockProductListResponse
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const params: ProductListParams = {
				productType: 'physical',
				isActive: true,
				isSellable: true,
				isPurchaseable: false,
				search: 'test'
			};

			const result = await productApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith(
				'/inventory/products?productType=physical&isActive=true&isSellable=true&isPurchaseable=false&search=test'
			);
			expect(result).toEqual(mockResponse);
		});

		it('should filter out undefined and null params', async () => {
			const mockResponse: ApiResponse<ProductListResponse> = {
				success: true,
				data: mockProductListResponse
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const params: ProductListParams = {
				page: 1,
				productType: undefined,
				isActive: null,
				search: 'query'
			};

			const result = await productApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/products?page=1&search=query');
			expect(result).toEqual(mockResponse);
		});

		it('should handle API error', async () => {
			const mockError: ApiResponse<ProductListResponse> = {
				success: false,
				error: 'Failed to fetch products'
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockError);

			const result = await productApi.list();

			expect(result).toEqual(mockError);
		});
	});

	// =========================================================================
	// get() tests
	// =========================================================================
	describe('get', () => {
		it('should get a product by ID', async () => {
			const mockResponse: ApiResponse<ProductResponse> = {
				success: true,
				data: mockProduct
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productApi.get('prod-001');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/products/prod-001');
			expect(result).toEqual(mockResponse);
		});

		it('should handle product not found', async () => {
			const mockError: ApiResponse<ProductResponse> = {
				success: false,
				error: 'Product not found'
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockError);

			const result = await productApi.get('non-existent');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/products/non-existent');
			expect(result).toEqual(mockError);
		});
	});

	// =========================================================================
	// getBySku() tests
	// =========================================================================
	describe('getBySku', () => {
		it('should get a product by SKU', async () => {
			const mockResponse: ApiResponse<ProductResponse> = {
				success: true,
				data: mockProduct
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productApi.getBySku('SKU-001');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/products/sku/SKU-001');
			expect(result).toEqual(mockResponse);
		});

		it('should encode special characters in SKU', async () => {
			const mockResponse: ApiResponse<ProductResponse> = {
				success: true,
				data: mockProduct
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productApi.getBySku('SKU/001&test');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/products/sku/SKU%2F001%26test');
			expect(result).toEqual(mockResponse);
		});

		it('should handle SKU not found', async () => {
			const mockError: ApiResponse<ProductResponse> = {
				success: false,
				error: 'Product with SKU not found'
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockError);

			const result = await productApi.getBySku('INVALID-SKU');

			expect(result).toEqual(mockError);
		});
	});

	// =========================================================================
	// create() tests
	// =========================================================================
	describe('create', () => {
		it('should create a new product with all fields', async () => {
			const mockResponse: ApiResponse<ProductResponse> = {
				success: true,
				data: mockProduct
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createData: ProductCreateRequest = {
				sku: 'SKU-001',
				name: 'Test Product',
				description: 'A test product description',
				productType: 'physical',
				trackInventory: true,
				trackingMethod: 'lot',
				salePrice: 99.99,
				costPrice: 50.0,
				currencyCode: 'VND',
				weightGrams: 500,
				dimensions: { length: 10, width: 5, height: 3 },
				attributes: { color: 'red', size: 'M' },
				defaultUomId: 'uom-001',
				itemGroupId: 'group-001',
				isActive: true,
				isSellable: true,
				isPurchaseable: true
			};

			const result = await productApi.create(createData);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/products', createData);
			expect(result).toEqual(mockResponse);
		});

		it('should create a product with minimal fields', async () => {
			const mockResponse: ApiResponse<ProductResponse> = {
				success: true,
				data: mockProduct2
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createData: ProductCreateRequest = {
				sku: 'SKU-002',
				name: 'Simple Product',
				productType: 'digital',
				currencyCode: 'VND'
			};

			const result = await productApi.create(createData);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/products', createData);
			expect(result).toEqual(mockResponse);
		});

		it('should handle validation error on create', async () => {
			const mockError: ApiResponse<ProductResponse> = {
				success: false,
				error: 'SKU already exists'
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockError);

			const createData: ProductCreateRequest = {
				sku: 'SKU-001',
				name: 'Duplicate Product',
				productType: 'physical',
				currencyCode: 'VND'
			};

			const result = await productApi.create(createData);

			expect(result).toEqual(mockError);
		});
	});

	// =========================================================================
	// update() tests
	// =========================================================================
	describe('update', () => {
		it('should update a product with all fields', async () => {
			const updatedProduct = { ...mockProduct, name: 'Updated Product' };
			const mockResponse: ApiResponse<ProductResponse> = {
				success: true,
				data: updatedProduct
			};
			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const updateData: ProductUpdateRequest = {
				name: 'Updated Product',
				description: 'Updated description',
				salePrice: 149.99,
				isActive: true
			};

			const result = await productApi.update('prod-001', updateData);

			expect(apiClient.put).toHaveBeenCalledWith('/inventory/products/prod-001', updateData);
			expect(result).toEqual(mockResponse);
		});

		it('should update only specific fields', async () => {
			const updatedProduct = { ...mockProduct, isActive: false };
			const mockResponse: ApiResponse<ProductResponse> = {
				success: true,
				data: updatedProduct
			};
			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const updateData: ProductUpdateRequest = {
				isActive: false
			};

			const result = await productApi.update('prod-001', updateData);

			expect(apiClient.put).toHaveBeenCalledWith('/inventory/products/prod-001', updateData);
			expect(result).toEqual(mockResponse);
		});

		it('should handle update product not found', async () => {
			const mockError: ApiResponse<ProductResponse> = {
				success: false,
				error: 'Product not found'
			};
			vi.mocked(apiClient.put).mockResolvedValue(mockError);

			const updateData: ProductUpdateRequest = {
				name: 'New Name'
			};

			const result = await productApi.update('non-existent', updateData);

			expect(result).toEqual(mockError);
		});
	});

	// =========================================================================
	// delete() tests
	// =========================================================================
	describe('delete', () => {
		it('should delete a product', async () => {
			const mockResponse: ApiResponse<void> = {
				success: true
			};
			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await productApi.delete('prod-001');

			expect(apiClient.delete).toHaveBeenCalledWith('/inventory/products/prod-001');
			expect(result).toEqual(mockResponse);
		});

		it('should handle delete product not found', async () => {
			const mockError: ApiResponse<void> = {
				success: false,
				error: 'Product not found'
			};
			vi.mocked(apiClient.delete).mockResolvedValue(mockError);

			const result = await productApi.delete('non-existent');

			expect(result).toEqual(mockError);
		});

		it('should handle delete constraint error', async () => {
			const mockError: ApiResponse<void> = {
				success: false,
				error: 'Cannot delete product with existing inventory'
			};
			vi.mocked(apiClient.delete).mockResolvedValue(mockError);

			const result = await productApi.delete('prod-001');

			expect(result).toEqual(mockError);
		});
	});

	// =========================================================================
	// moveToCategory() tests
	// =========================================================================
	describe('moveToCategory', () => {
		it('should move products to a category', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: mockBulkOperationResponse
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const moveData: MoveToCategoryRequest = {
				productIds: ['prod-001', 'prod-002', 'prod-003'],
				categoryId: 'cat-001'
			};

			const result = await productApi.moveToCategory(moveData);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/products/move-to-category', moveData);
			expect(result).toEqual(mockResponse);
		});

		it('should handle move to non-existent category', async () => {
			const mockError: ApiResponse<BulkOperationResponse> = {
				success: false,
				error: 'Category not found'
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockError);

			const moveData: MoveToCategoryRequest = {
				productIds: ['prod-001'],
				categoryId: 'non-existent'
			};

			const result = await productApi.moveToCategory(moveData);

			expect(result).toEqual(mockError);
		});

		it('should handle partial success in move operation', async () => {
			const partialSuccessResponse: BulkOperationResponse = {
				success: true,
				affectedCount: 2,
				message: '2 of 3 products moved successfully'
			};
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: partialSuccessResponse
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const moveData: MoveToCategoryRequest = {
				productIds: ['prod-001', 'prod-002', 'prod-003'],
				categoryId: 'cat-001'
			};

			const result = await productApi.moveToCategory(moveData);

			expect(result.data?.affectedCount).toBe(2);
		});
	});

	// =========================================================================
	// bulkActivate() tests
	// =========================================================================
	describe('bulkActivate', () => {
		it('should bulk activate products', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: mockBulkOperationResponse
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const productIds = ['prod-001', 'prod-002', 'prod-003'];

			const result = await productApi.bulkActivate(productIds);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/products/bulk/activate', {
				productIds
			});
			expect(result).toEqual(mockResponse);
		});

		it('should handle empty product IDs array', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: { success: true, affectedCount: 0, message: 'No products to activate' }
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await productApi.bulkActivate([]);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/products/bulk/activate', {
				productIds: []
			});
			expect(result.data?.affectedCount).toBe(0);
		});

		it('should handle bulk activate error', async () => {
			const mockError: ApiResponse<BulkOperationResponse> = {
				success: false,
				error: 'Failed to activate products'
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockError);

			const result = await productApi.bulkActivate(['prod-001']);

			expect(result).toEqual(mockError);
		});
	});

	// =========================================================================
	// bulkDeactivate() tests
	// =========================================================================
	describe('bulkDeactivate', () => {
		it('should bulk deactivate products', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: mockBulkOperationResponse
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const productIds = ['prod-001', 'prod-002', 'prod-003'];

			const result = await productApi.bulkDeactivate(productIds);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/products/bulk/deactivate', {
				productIds
			});
			expect(result).toEqual(mockResponse);
		});

		it('should handle bulk deactivate with single product', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: { success: true, affectedCount: 1, message: '1 product deactivated' }
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await productApi.bulkDeactivate(['prod-001']);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/products/bulk/deactivate', {
				productIds: ['prod-001']
			});
			expect(result.data?.affectedCount).toBe(1);
		});
	});

	// =========================================================================
	// bulkDelete() tests
	// =========================================================================
	describe('bulkDelete', () => {
		it('should bulk delete products', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: mockBulkOperationResponse
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const productIds = ['prod-001', 'prod-002', 'prod-003'];

			const result = await productApi.bulkDelete(productIds);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/products/bulk/delete', {
				productIds
			});
			expect(result).toEqual(mockResponse);
		});

		it('should handle bulk delete constraint error', async () => {
			const mockError: ApiResponse<BulkOperationResponse> = {
				success: false,
				error: 'Some products have existing inventory and cannot be deleted'
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockError);

			const result = await productApi.bulkDelete(['prod-001', 'prod-002']);

			expect(result).toEqual(mockError);
		});

		it('should handle bulk delete partial success', async () => {
			const partialSuccessResponse: BulkOperationResponse = {
				success: false,
				affectedCount: 1,
				message: 'Only 1 of 3 products could be deleted'
			};
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: partialSuccessResponse
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await productApi.bulkDelete(['prod-001', 'prod-002', 'prod-003']);

			expect(result.data?.success).toBe(false);
			expect(result.data?.affectedCount).toBe(1);
		});
	});
});
