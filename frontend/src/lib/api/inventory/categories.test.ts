// =============================================================================
// Inventory Category API Tests
// Tests for inventory/categories.ts API client
// =============================================================================

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { categoryApi } from './categories';
import { apiClient } from '$lib/api/client';
import type {
	CategoryResponse,
	CategoryCreateRequest,
	CategoryUpdateRequest,
	CategoryListResponse,
	CategoryListParams,
	CategoryStatsResponse,
	BulkCategoryIds,
	BulkOperationResponse,
	PaginationInfo
} from '$lib/types/inventory';
import type { ApiResponse } from '$lib/types';

// Mock the apiClient
vi.mock('$lib/api/client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		patch: vi.fn(),
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

const mockCategory: CategoryResponse = {
	categoryId: 'cat-001',
	tenantId: 'tenant-001',
	name: 'Electronics',
	slug: 'electronics',
	code: 'ELEC',
	description: 'Electronic products and devices',
	parentCategoryId: null,
	path: '/electronics',
	level: 0,
	displayOrder: 1,
	isActive: true,
	isVisible: true,
	productCount: 45,
	totalProductCount: 125,
	icon: '🔌',
	color: '#3B82F6',
	imageUrl: 'https://example.com/electronics.jpg',
	metaTitle: 'Electronics - Best Deals',
	metaDescription: 'Shop the best electronics',
	metaKeywords: 'electronics, gadgets, devices',
	breadcrumbs: [{ categoryId: 'cat-001', name: 'Electronics', slug: 'electronics', level: 0 }],
	createdAt: '2026-01-01T00:00:00Z',
	updatedAt: '2026-01-15T00:00:00Z'
};

const mockChildCategory: CategoryResponse = {
	categoryId: 'cat-002',
	tenantId: 'tenant-001',
	name: 'Computers',
	slug: 'computers',
	code: 'COMP',
	description: 'Desktop and laptop computers',
	parentCategoryId: 'cat-001',
	path: '/electronics/computers',
	level: 1,
	displayOrder: 1,
	isActive: true,
	isVisible: true,
	productCount: 30,
	totalProductCount: 45,
	icon: '💻',
	color: '#10B981',
	imageUrl: null,
	metaTitle: null,
	metaDescription: null,
	metaKeywords: null,
	breadcrumbs: [
		{ categoryId: 'cat-001', name: 'Electronics', slug: 'electronics', level: 0 },
		{ categoryId: 'cat-002', name: 'Computers', slug: 'computers', level: 1 }
	],
	createdAt: '2026-01-02T00:00:00Z',
	updatedAt: '2026-01-16T00:00:00Z'
};

const mockInactiveCategory: CategoryResponse = {
	categoryId: 'cat-003',
	tenantId: 'tenant-001',
	name: 'Discontinued',
	slug: 'discontinued',
	code: 'DISC',
	description: 'Discontinued products',
	parentCategoryId: null,
	path: '/discontinued',
	level: 0,
	displayOrder: 99,
	isActive: false,
	isVisible: false,
	productCount: 0,
	totalProductCount: 0,
	icon: null,
	color: null,
	imageUrl: null,
	metaTitle: null,
	metaDescription: null,
	metaKeywords: null,
	breadcrumbs: [],
	createdAt: '2026-01-03T00:00:00Z',
	updatedAt: '2026-01-17T00:00:00Z'
};

const mockCategoryListResponse: CategoryListResponse = {
	categories: [mockCategory, mockChildCategory, mockInactiveCategory],
	pagination: mockPagination
};

const mockCategoryStats: CategoryStatsResponse = {
	categoryId: 'cat-001',
	name: 'Electronics',
	level: 0,
	productCount: 45,
	totalProductCount: 125,
	subcategoryCount: 5,
	activeProductCount: 120,
	inactiveProductCount: 5
};

const mockBulkOperationResponse: BulkOperationResponse = {
	success: true,
	affectedCount: 3,
	message: 'Operation completed successfully'
};

// =============================================================================
// Test Suites
// =============================================================================

describe('categoryApi', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	// ===========================================================================
	// List Categories
	// ===========================================================================

	describe('list', () => {
		it('should list categories without params', async () => {
			const mockResponse: ApiResponse<CategoryListResponse> = {
				success: true,
				data: mockCategoryListResponse
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.list();

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/categories');
			expect(result).toEqual(mockResponse);
			expect(result.data?.categories).toHaveLength(3);
		});

		it('should list categories with pagination params', async () => {
			const params: CategoryListParams = { page: 2, pageSize: 10 };
			const mockResponse: ApiResponse<CategoryListResponse> = {
				success: true,
				data: mockCategoryListResponse
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/categories?page=2&pageSize=10');
			expect(result).toEqual(mockResponse);
		});

		it('should list categories with filter params', async () => {
			const params: CategoryListParams = {
				isActive: true,
				isVisible: true,
				level: 0
			};
			const mockResponse: ApiResponse<CategoryListResponse> = {
				success: true,
				data: { categories: [mockCategory], pagination: mockPagination }
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith(
				'/inventory/categories?isActive=true&isVisible=true&level=0'
			);
			expect(result.data?.categories).toHaveLength(1);
		});

		it('should list categories with search param', async () => {
			const params: CategoryListParams = { search: 'electronics' };
			const mockResponse: ApiResponse<CategoryListResponse> = {
				success: true,
				data: { categories: [mockCategory], pagination: mockPagination }
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/categories?search=electronics');
			expect(result).toEqual(mockResponse);
		});

		it('should list categories with parent filter', async () => {
			const params: CategoryListParams = { parentId: 'cat-001' };
			const mockResponse: ApiResponse<CategoryListResponse> = {
				success: true,
				data: { categories: [mockChildCategory], pagination: mockPagination }
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/categories?parentId=cat-001');
			expect(result.data?.categories[0].parentCategoryId).toBe('cat-001');
		});

		it('should list categories with sorting', async () => {
			const params: CategoryListParams = { sortBy: 'name', sortDir: 'asc' };
			const mockResponse: ApiResponse<CategoryListResponse> = {
				success: true,
				data: mockCategoryListResponse
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/categories?sortBy=name&sortDir=asc');
			expect(result).toEqual(mockResponse);
		});

		it('should handle list error', async () => {
			const mockResponse: ApiResponse<CategoryListResponse> = {
				success: false,
				error: 'Failed to fetch categories'
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.list();

			expect(result.success).toBe(false);
			expect(result.error).toBe('Failed to fetch categories');
		});
	});

	// ===========================================================================
	// Get Category Tree
	// ===========================================================================

	describe('getTree', () => {
		it('should get full category tree without root id', async () => {
			const mockResponse: ApiResponse<CategoryResponse[]> = {
				success: true,
				data: [mockCategory, mockChildCategory]
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.getTree();

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/categories/tree');
			expect(result.data).toHaveLength(2);
		});

		it('should get subtree with root id', async () => {
			const mockResponse: ApiResponse<CategoryResponse[]> = {
				success: true,
				data: [mockChildCategory]
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.getTree('cat-001');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/categories/tree?root_id=cat-001');
			expect(result.data).toHaveLength(1);
		});
	});

	// ===========================================================================
	// Get Single Category
	// ===========================================================================

	describe('get', () => {
		it('should get category by id', async () => {
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: true,
				data: mockCategory
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.get('cat-001');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/categories/cat-001');
			expect(result.data?.categoryId).toBe('cat-001');
			expect(result.data?.name).toBe('Electronics');
		});

		it('should handle not found error', async () => {
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: false,
				error: 'Category not found'
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.get('non-existent');

			expect(result.success).toBe(false);
			expect(result.error).toBe('Category not found');
		});
	});

	// ===========================================================================
	// Create Category
	// ===========================================================================

	describe('create', () => {
		it('should create a root category', async () => {
			const createData: CategoryCreateRequest = {
				name: 'New Category',
				slug: 'new-category',
				code: 'NEW',
				description: 'A new category',
				isActive: true,
				isVisible: true,
				displayOrder: 5
			};
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: true,
				data: { ...mockCategory, ...createData, categoryId: 'cat-new' }
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await categoryApi.create(createData);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/categories', createData);
			expect(result.success).toBe(true);
			expect(result.data?.name).toBe('New Category');
		});

		it('should create a child category', async () => {
			const createData: CategoryCreateRequest = {
				name: 'Laptops',
				slug: 'laptops',
				parentCategoryId: 'cat-002',
				isActive: true
			};
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: true,
				data: { ...mockChildCategory, ...createData, categoryId: 'cat-new', level: 2 }
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await categoryApi.create(createData);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/categories', createData);
			expect(result.data?.parentCategoryId).toBe('cat-002');
		});

		it('should create category with SEO fields', async () => {
			const createData: CategoryCreateRequest = {
				name: 'SEO Category',
				metaTitle: 'SEO Title',
				metaDescription: 'SEO Description',
				metaKeywords: 'seo, keywords'
			};
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: true,
				data: { ...mockCategory, ...createData }
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await categoryApi.create(createData);

			expect(result.data?.metaTitle).toBe('SEO Title');
			expect(result.data?.metaDescription).toBe('SEO Description');
		});

		it('should handle validation error on create', async () => {
			const createData: CategoryCreateRequest = { name: '' };
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: false,
				error: 'Name is required'
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await categoryApi.create(createData);

			expect(result.success).toBe(false);
			expect(result.error).toBe('Name is required');
		});

		it('should handle duplicate code error', async () => {
			const createData: CategoryCreateRequest = { name: 'Test', code: 'ELEC' };
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: false,
				error: 'Category code already exists'
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await categoryApi.create(createData);

			expect(result.success).toBe(false);
			expect(result.error).toBe('Category code already exists');
		});
	});

	// ===========================================================================
	// Update Category
	// ===========================================================================

	describe('update', () => {
		it('should update category name', async () => {
			const updateData: CategoryUpdateRequest = { name: 'Updated Electronics' };
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: true,
				data: { ...mockCategory, name: 'Updated Electronics' }
			};
			vi.mocked(apiClient.patch).mockResolvedValue(mockResponse);

			const result = await categoryApi.update('cat-001', updateData);

			expect(apiClient.patch).toHaveBeenCalledWith('/inventory/categories/cat-001', updateData);
			expect(result.data?.name).toBe('Updated Electronics');
		});

		it('should update category active status', async () => {
			const updateData: CategoryUpdateRequest = { isActive: false };
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: true,
				data: { ...mockCategory, isActive: false }
			};
			vi.mocked(apiClient.patch).mockResolvedValue(mockResponse);

			const result = await categoryApi.update('cat-001', updateData);

			expect(result.data?.isActive).toBe(false);
		});

		it('should update category parent (move)', async () => {
			const updateData: CategoryUpdateRequest = { parentCategoryId: 'cat-003' };
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: true,
				data: { ...mockChildCategory, parentCategoryId: 'cat-003', level: 1 }
			};
			vi.mocked(apiClient.patch).mockResolvedValue(mockResponse);

			const result = await categoryApi.update('cat-002', updateData);

			expect(result.data?.parentCategoryId).toBe('cat-003');
		});

		it('should update display order', async () => {
			const updateData: CategoryUpdateRequest = { displayOrder: 10 };
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: true,
				data: { ...mockCategory, displayOrder: 10 }
			};
			vi.mocked(apiClient.patch).mockResolvedValue(mockResponse);

			const result = await categoryApi.update('cat-001', updateData);

			expect(result.data?.displayOrder).toBe(10);
		});

		it('should handle not found on update', async () => {
			const updateData: CategoryUpdateRequest = { name: 'Test' };
			const mockResponse: ApiResponse<CategoryResponse> = {
				success: false,
				error: 'Category not found'
			};
			vi.mocked(apiClient.patch).mockResolvedValue(mockResponse);

			const result = await categoryApi.update('non-existent', updateData);

			expect(result.success).toBe(false);
			expect(result.error).toBe('Category not found');
		});
	});

	// ===========================================================================
	// Delete Category
	// ===========================================================================

	describe('delete', () => {
		it('should delete category', async () => {
			const mockResponse: ApiResponse<void> = { success: true };
			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await categoryApi.delete('cat-003');

			expect(apiClient.delete).toHaveBeenCalledWith('/inventory/categories/cat-003');
			expect(result.success).toBe(true);
		});

		it('should handle not found on delete', async () => {
			const mockResponse: ApiResponse<void> = {
				success: false,
				error: 'Category not found'
			};
			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await categoryApi.delete('non-existent');

			expect(result.success).toBe(false);
		});

		it('should handle delete with products error', async () => {
			const mockResponse: ApiResponse<void> = {
				success: false,
				error: 'Cannot delete category with products. Reassign products first.'
			};
			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await categoryApi.delete('cat-001');

			expect(result.success).toBe(false);
			expect(result.error).toBe('Cannot delete category with products. Reassign products first.');
		});
	});

	// ===========================================================================
	// Get Category Stats
	// ===========================================================================

	describe('getStats', () => {
		it('should get category statistics', async () => {
			const mockResponse: ApiResponse<CategoryStatsResponse> = {
				success: true,
				data: mockCategoryStats
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await categoryApi.getStats('cat-001');

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/categories/cat-001/stats');
			expect(result.data?.productCount).toBe(45);
			expect(result.data?.subcategoryCount).toBe(5);
			expect(result.data?.activeProductCount).toBe(120);
		});
	});

	// ===========================================================================
	// Bulk Operations
	// ===========================================================================

	describe('bulkActivate', () => {
		it('should bulk activate categories', async () => {
			const bulkData: BulkCategoryIds = { categoryIds: ['cat-001', 'cat-002', 'cat-003'] };
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: mockBulkOperationResponse
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await categoryApi.bulkActivate(bulkData);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/categories/bulk/activate', bulkData);
			expect(result.data?.affectedCount).toBe(3);
		});

		it('should handle partial activation', async () => {
			const bulkData: BulkCategoryIds = { categoryIds: ['cat-001', 'non-existent'] };
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: { success: true, affectedCount: 1, message: '1 of 2 categories activated' }
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await categoryApi.bulkActivate(bulkData);

			expect(result.data?.affectedCount).toBe(1);
		});
	});

	describe('bulkDeactivate', () => {
		it('should bulk deactivate categories', async () => {
			const bulkData: BulkCategoryIds = { categoryIds: ['cat-001', 'cat-002'] };
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: { success: true, affectedCount: 2, message: '2 categories deactivated' }
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await categoryApi.bulkDeactivate(bulkData);

			expect(apiClient.post).toHaveBeenCalledWith(
				'/inventory/categories/bulk/deactivate',
				bulkData
			);
			expect(result.data?.affectedCount).toBe(2);
		});
	});

	describe('bulkDelete', () => {
		it('should bulk delete categories', async () => {
			const bulkData: BulkCategoryIds = { categoryIds: ['cat-003'] };
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: true,
				data: { success: true, affectedCount: 1, message: '1 category deleted' }
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await categoryApi.bulkDelete(bulkData);

			expect(apiClient.post).toHaveBeenCalledWith('/inventory/categories/bulk/delete', bulkData);
			expect(result.data?.affectedCount).toBe(1);
		});

		it('should handle bulk delete with dependencies', async () => {
			const bulkData: BulkCategoryIds = { categoryIds: ['cat-001', 'cat-002'] };
			const mockResponse: ApiResponse<BulkOperationResponse> = {
				success: false,
				error: 'Some categories have products and cannot be deleted'
			};
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await categoryApi.bulkDelete(bulkData);

			expect(result.success).toBe(false);
			expect(result.error).toBe('Some categories have products and cannot be deleted');
		});
	});
});
