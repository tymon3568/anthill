// =============================================================================
// Inventory Variant API Tests
// Tests for inventory/variants.ts API client
// =============================================================================

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { variantApi } from './variants';
import { apiClient } from '$lib/api/client';
import type {
	VariantResponse,
	VariantCreateRequest,
	VariantUpdateRequest,
	VariantListResponse,
	VariantListParams,
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
	totalItems: 50,
	totalPages: 3,
	hasNext: true,
	hasPrev: false
};

const mockVariant: VariantResponse = {
	variantId: 'var-001',
	tenantId: 'tenant-001',
	parentProductId: 'prod-001',
	variantAttributes: { color: 'Black', size: 'L' },
	sku: 'LAPTOP-001-BLK-L',
	barcode: '1234567890123',
	priceDifference: 500000,
	isActive: true,
	createdAt: '2026-01-01T00:00:00Z',
	updatedAt: '2026-01-15T00:00:00Z',
	parentProductName: 'Laptop Pro 15"',
	parentProductSku: 'LAPTOP-001'
};

const mockVariant2: VariantResponse = {
	variantId: 'var-002',
	tenantId: 'tenant-001',
	parentProductId: 'prod-001',
	variantAttributes: { color: 'Silver', size: 'M' },
	sku: 'LAPTOP-001-SLV-M',
	barcode: '1234567890124',
	priceDifference: 0,
	isActive: true,
	createdAt: '2026-01-02T00:00:00Z',
	updatedAt: '2026-01-16T00:00:00Z',
	parentProductName: 'Laptop Pro 15"',
	parentProductSku: 'LAPTOP-001'
};

const mockInactiveVariant: VariantResponse = {
	variantId: 'var-003',
	tenantId: 'tenant-001',
	parentProductId: 'prod-002',
	variantAttributes: { color: 'Red' },
	sku: 'PHONE-001-RED',
	barcode: null,
	priceDifference: -100000,
	isActive: false,
	createdAt: '2026-01-03T00:00:00Z',
	updatedAt: '2026-01-17T00:00:00Z',
	parentProductName: 'Smartphone X',
	parentProductSku: 'PHONE-001'
};

const mockVariantListResponse: VariantListResponse = {
	variants: [mockVariant, mockVariant2, mockInactiveVariant],
	pagination: mockPagination
};

const mockBulkOperationResponse: BulkOperationResponse = {
	success: true,
	affectedCount: 3,
	message: 'Operation completed successfully'
};

// =============================================================================
// Test Suites
// =============================================================================

describe('variantApi', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	// =========================================================================
	// list() tests
	// =========================================================================

	describe('list', () => {
		it('should list variants without parameters', async () => {
			const mockResponse: ApiResponse<VariantListResponse> = {
				success: true,
				data: mockVariantListResponse
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.list();

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/variants');
			expect(result.success).toBe(true);
			expect(result.data?.variants).toHaveLength(3);
		});

		it('should list variants with pagination params', async () => {
			const mockResponse: ApiResponse<VariantListResponse> = {
				success: true,
				data: mockVariantListResponse
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const params: VariantListParams = { page: 2, pageSize: 10 };
			await variantApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/variants?page=2&pageSize=10');
		});

		it('should list variants with search param', async () => {
			const mockResponse: ApiResponse<VariantListResponse> = {
				success: true,
				data: { variants: [mockVariant], pagination: { ...mockPagination, totalItems: 1 } }
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const params: VariantListParams = { search: 'LAPTOP' };
			const result = await variantApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/variants?search=LAPTOP');
			expect(result.data?.variants).toHaveLength(1);
		});

		it('should list variants with isActive filter', async () => {
			const mockResponse: ApiResponse<VariantListResponse> = {
				success: true,
				data: { variants: [mockVariant, mockVariant2], pagination: mockPagination }
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const params: VariantListParams = { isActive: true };
			await variantApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/variants?isActive=true');
		});

		it('should list variants with parentProductId filter', async () => {
			const mockResponse: ApiResponse<VariantListResponse> = {
				success: true,
				data: { variants: [mockVariant, mockVariant2], pagination: mockPagination }
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const params: VariantListParams = { parentProductId: 'prod-001' };
			await variantApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/variants?parentProductId=prod-001');
		});

		it('should list variants with multiple params', async () => {
			const mockResponse: ApiResponse<VariantListResponse> = {
				success: true,
				data: mockVariantListResponse
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const params: VariantListParams = {
				page: 1,
				pageSize: 20,
				search: 'BLK',
				isActive: true,
				sortBy: 'sku',
				sortDir: 'asc'
			};
			await variantApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith(
				'/inventory/variants?page=1&pageSize=20&search=BLK&isActive=true&sortBy=sku&sortDir=asc'
			);
		});

		it('should handle error response', async () => {
			const mockResponse: ApiResponse<VariantListResponse> = {
				success: false,
				error: 'Failed to fetch variants'
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.list();

			expect(result.success).toBe(false);
			expect(result.error).toBe('Failed to fetch variants');
		});
	});

	// =========================================================================
	// get() tests
	// =========================================================================

	describe('get', () => {
		it('should get variant by ID', async () => {
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: mockVariant
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.get('var-001');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/variants/var-001');
			expect(result.success).toBe(true);
			expect(result.data?.variantId).toBe('var-001');
		});

		it('should handle not found error', async () => {
			const mockResponse: ApiResponse<VariantResponse> = {
				success: false,
				error: 'Variant not found'
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.get('non-existent');

			expect(result.success).toBe(false);
			expect(result.error).toBe('Variant not found');
		});
	});

	// =========================================================================
	// getBySku() tests
	// =========================================================================

	describe('getBySku', () => {
		it('should get variant by SKU', async () => {
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: mockVariant
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.getBySku('LAPTOP-001-BLK-L');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/variants/by-sku/LAPTOP-001-BLK-L');
			expect(result.success).toBe(true);
			expect(result.data?.sku).toBe('LAPTOP-001-BLK-L');
		});

		it('should encode special characters in SKU', async () => {
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: mockVariant
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			await variantApi.getBySku('SKU/WITH/SLASHES');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/variants/by-sku/SKU%2FWITH%2FSLASHES');
		});
	});

	// =========================================================================
	// getByBarcode() tests
	// =========================================================================

	describe('getByBarcode', () => {
		it('should get variant by barcode', async () => {
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: mockVariant
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.getByBarcode('1234567890123');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/variants/by-barcode/1234567890123');
			expect(result.success).toBe(true);
			expect(result.data?.barcode).toBe('1234567890123');
		});

		it('should handle barcode not found', async () => {
			const mockResponse: ApiResponse<VariantResponse> = {
				success: false,
				error: 'Variant with barcode not found'
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.getByBarcode('0000000000000');

			expect(result.success).toBe(false);
			expect(result.error).toBe('Variant with barcode not found');
		});
	});

	// =========================================================================
	// listByProduct() tests
	// =========================================================================

	describe('listByProduct', () => {
		it('should list variants for a specific product', async () => {
			const mockResponse: ApiResponse<VariantListResponse> = {
				success: true,
				data: { variants: [mockVariant, mockVariant2], pagination: mockPagination }
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.listByProduct('prod-001');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/variants?parentProductId=prod-001');
			expect(result.success).toBe(true);
			expect(result.data?.variants).toHaveLength(2);
		});

		it('should list variants for a product with additional params', async () => {
			const mockResponse: ApiResponse<VariantListResponse> = {
				success: true,
				data: { variants: [mockVariant], pagination: mockPagination }
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			await variantApi.listByProduct('prod-001', { isActive: true, page: 1 });

			expect(apiClient.get).toHaveBeenCalledWith(
				'/inventory/variants?isActive=true&page=1&parentProductId=prod-001'
			);
		});
	});

	// =========================================================================
	// create() tests
	// =========================================================================

	describe('create', () => {
		it('should create a new variant', async () => {
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: mockVariant
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createRequest: VariantCreateRequest = {
				parentProductId: 'prod-001',
				sku: 'LAPTOP-001-BLK-L',
				barcode: '1234567890123',
				variantAttributes: { color: 'Black', size: 'L' },
				priceDifference: 500000,
				isActive: true
			};

			const result = await variantApi.create(createRequest);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/variants', createRequest);
			expect(result.success).toBe(true);
			expect(result.data?.sku).toBe('LAPTOP-001-BLK-L');
		});

		it('should create variant with minimal data', async () => {
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: { ...mockVariant, barcode: null, priceDifference: 0 }
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createRequest: VariantCreateRequest = {
				parentProductId: 'prod-001',
				sku: 'LAPTOP-001-NEW',
				variantAttributes: { color: 'Blue' }
			};

			const result = await variantApi.create(createRequest);

			expect(result.success).toBe(true);
		});

		it('should handle duplicate SKU error', async () => {
			const mockResponse: ApiResponse<VariantResponse> = {
				success: false,
				error: 'Variant with SKU already exists'
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createRequest: VariantCreateRequest = {
				parentProductId: 'prod-001',
				sku: 'EXISTING-SKU',
				variantAttributes: { color: 'Red' }
			};

			const result = await variantApi.create(createRequest);

			expect(result.success).toBe(false);
			expect(result.error).toBe('Variant with SKU already exists');
		});
	});

	// =========================================================================
	// update() tests
	// =========================================================================

	describe('update', () => {
		it('should update a variant', async () => {
			const updatedVariant = { ...mockVariant, priceDifference: 750000 };
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: updatedVariant
			};
			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const updateRequest: VariantUpdateRequest = {
				priceDifference: 750000
			};

			const result = await variantApi.update('var-001', updateRequest);

			expect(apiClient.put).toHaveBeenCalledWith('/inventory/variants/var-001', updateRequest);
			expect(result.success).toBe(true);
			expect(result.data?.priceDifference).toBe(750000);
		});

		it('should update variant attributes', async () => {
			const updatedVariant = {
				...mockVariant,
				variantAttributes: { color: 'Navy', size: 'XL' }
			};
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: updatedVariant
			};
			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const updateRequest: VariantUpdateRequest = {
				variantAttributes: { color: 'Navy', size: 'XL' }
			};

			const result = await variantApi.update('var-001', updateRequest);

			expect(result.success).toBe(true);
			expect(result.data?.variantAttributes.color).toBe('Navy');
		});

		it('should deactivate a variant', async () => {
			const deactivatedVariant = { ...mockVariant, isActive: false };
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: deactivatedVariant
			};
			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const result = await variantApi.update('var-001', { isActive: false });

			expect(result.success).toBe(true);
			expect(result.data?.isActive).toBe(false);
		});
	});

	// =========================================================================
	// delete() tests
	// =========================================================================

	describe('delete', () => {
		it('should delete a variant', async () => {
			const mockResponse: ApiResponse<void> = {
				success: true
			};
			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await variantApi.delete('var-001');

			expect(apiClient.delete).toHaveBeenCalledWith('/inventory/variants/var-001');
			expect(result.success).toBe(true);
		});

		it('should handle delete error', async () => {
			const mockResponse: ApiResponse<void> = {
				success: false,
				error: 'Cannot delete variant with inventory'
			};
			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await variantApi.delete('var-001');

			expect(result.success).toBe(false);
			expect(result.error).toBe('Cannot delete variant with inventory');
		});
	});

	// =========================================================================
	// bulkActivate() tests
	// =========================================================================

	describe('bulkActivate', () => {
		it('should activate multiple variants', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: mockBulkOperationResponse
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const variantIds = ['var-001', 'var-002', 'var-003'];
			const result = await variantApi.bulkActivate(variantIds);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/variants/bulk/activate', {
				variantIds
			});
			expect(result.success).toBe(true);
			expect(result.data?.affectedCount).toBe(3);
		});

		it('should handle partial activation', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: { success: true, affectedCount: 2, message: '2 of 3 variants activated' }
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await variantApi.bulkActivate(['var-001', 'var-002', 'non-existent']);

			expect(result.data?.affectedCount).toBe(2);
		});
	});

	// =========================================================================
	// bulkDeactivate() tests
	// =========================================================================

	describe('bulkDeactivate', () => {
		it('should deactivate multiple variants', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: mockBulkOperationResponse
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const variantIds = ['var-001', 'var-002'];
			const result = await variantApi.bulkDeactivate(variantIds);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/variants/bulk/deactivate', {
				variantIds
			});
			expect(result.success).toBe(true);
		});
	});

	// =========================================================================
	// bulkDelete() tests
	// =========================================================================

	describe('bulkDelete', () => {
		it('should delete multiple variants', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: mockBulkOperationResponse
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const variantIds = ['var-001', 'var-002', 'var-003'];
			const result = await variantApi.bulkDelete(variantIds);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/variants/bulk/delete', {
				variantIds
			});
			expect(result.success).toBe(true);
			expect(result.data?.affectedCount).toBe(3);
		});

		it('should handle deletion with constraints', async () => {
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: false,
				error: 'Some variants have inventory and cannot be deleted'
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await variantApi.bulkDelete(['var-with-inventory']);

			expect(result.success).toBe(false);
			expect(result.error).toContain('cannot be deleted');
		});
	});

	// =========================================================================
	// Edge cases
	// =========================================================================

	describe('edge cases', () => {
		it('should handle empty variant list response', async () => {
			const mockResponse: ApiResponse<VariantListResponse> = {
				success: true,
				data: {
					variants: [],
					pagination: { ...mockPagination, totalItems: 0, totalPages: 0, hasNext: false }
				}
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.list({ search: 'nonexistent' });

			expect(result.success).toBe(true);
			expect(result.data?.variants).toHaveLength(0);
			expect(result.data?.pagination.totalItems).toBe(0);
		});

		it('should handle variant with negative price difference', async () => {
			const discountVariant: VariantResponse = {
				...mockVariant,
				priceDifference: -200000
			};
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: discountVariant
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.get('var-discount');

			expect(result.success).toBe(true);
			expect(result.data?.priceDifference).toBeLessThan(0);
		});

		it('should handle variant with empty attributes', async () => {
			const mockResponse: ApiResponse<VariantResponse> = {
				success: true,
				data: { ...mockVariant, variantAttributes: {} }
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantApi.get('var-empty-attrs');

			expect(result.success).toBe(true);
			expect(Object.keys(result.data?.variantAttributes || {})).toHaveLength(0);
		});

		it('should handle network error', async () => {
			vi.mocked(apiClient.get).mockRejectedValue(new Error('Network error'));

			await expect(variantApi.list()).rejects.toThrow('Network error');
		});
	});
});
