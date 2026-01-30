// =============================================================================
// Stock Levels API Tests
// Tests for inventory/stock-levels.ts API client
// =============================================================================

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { stockLevelApi } from './stock-levels';
import { apiClient } from '$lib/api/client';
import type {
	StockLevelListResponse,
	StockLevelListParams,
	StockLevelResponse,
	StockLevelSummary,
	PaginationInfo
} from '$lib/types/inventory';
import type { ApiResponse } from '$lib/types';

// Mock the apiClient
vi.mock('$lib/api/client', () => ({
	apiClient: {
		get: vi.fn()
	}
}));

// =============================================================================
// Mock Data
// =============================================================================

const mockPagination: PaginationInfo = {
	page: 1,
	pageSize: 20,
	totalItems: 3,
	totalPages: 1,
	hasNext: false,
	hasPrev: false
};

const mockSummary: StockLevelSummary = {
	totalProducts: 3,
	totalAvailableQuantity: 500,
	totalReservedQuantity: 50,
	lowStockCount: 1,
	outOfStockCount: 0
};

const mockStockLevel1: StockLevelResponse = {
	inventoryId: 'inv-001',
	tenantId: 'tenant-001',
	productId: 'prod-001',
	productSku: 'SKU-001',
	productName: 'Test Product 1',
	warehouseId: 'wh-001',
	warehouseCode: 'WH-MAIN',
	warehouseName: 'Main Warehouse',
	availableQuantity: 100,
	reservedQuantity: 10,
	totalQuantity: 110,
	status: 'in_stock',
	reorderPoint: 20,
	updatedAt: '2026-01-15T10:00:00Z'
};

const mockStockLevel2: StockLevelResponse = {
	inventoryId: 'inv-002',
	tenantId: 'tenant-001',
	productId: 'prod-002',
	productSku: 'SKU-002',
	productName: 'Test Product 2',
	warehouseId: 'wh-001',
	warehouseCode: 'WH-MAIN',
	warehouseName: 'Main Warehouse',
	availableQuantity: 5,
	reservedQuantity: 0,
	totalQuantity: 5,
	status: 'low_stock',
	reorderPoint: 10,
	updatedAt: '2026-01-14T10:00:00Z'
};

const mockStockLevel3: StockLevelResponse = {
	inventoryId: 'inv-003',
	tenantId: 'tenant-001',
	productId: 'prod-003',
	productSku: 'SKU-003',
	productName: 'Test Product 3',
	warehouseId: 'wh-002',
	warehouseCode: 'WH-SEC',
	warehouseName: 'Secondary Warehouse',
	availableQuantity: 395,
	reservedQuantity: 40,
	totalQuantity: 435,
	status: 'in_stock',
	reorderPoint: null,
	updatedAt: '2026-01-16T10:00:00Z'
};

const mockListResponse: StockLevelListResponse = {
	items: [mockStockLevel1, mockStockLevel2, mockStockLevel3],
	pagination: mockPagination,
	summary: mockSummary
};

// =============================================================================
// Tests
// =============================================================================

describe('Stock Levels API Client', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('stockLevelApi.list', () => {
		it('should list stock levels without params', async () => {
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: mockListResponse,
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/stock-levels');
			expect(result.data).toEqual(mockListResponse);
			expect(result.data?.items).toHaveLength(3);
			expect(result.data?.summary.totalProducts).toBe(3);
		});

		it('should list stock levels with pagination params', async () => {
			const params: StockLevelListParams = {
				page: 2,
				pageSize: 10
			};
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: { ...mockListResponse, pagination: { ...mockPagination, page: 2, pageSize: 10 } },
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/stock-levels?page=2&pageSize=10');
			expect(result.data?.pagination.page).toBe(2);
			expect(result.data?.pagination.pageSize).toBe(10);
		});

		it('should list stock levels with warehouse filter', async () => {
			const params: StockLevelListParams = {
				warehouseId: 'wh-001'
			};
			const filteredItems = [mockStockLevel1, mockStockLevel2];
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: {
					items: filteredItems,
					pagination: { ...mockPagination, totalItems: 2 },
					summary: { ...mockSummary, totalProducts: 2 }
				},
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/stock-levels?warehouseId=wh-001');
			expect(result.data?.items).toHaveLength(2);
			expect(result.data?.items.every((i) => i.warehouseId === 'wh-001')).toBe(true);
		});

		it('should list stock levels with product filter', async () => {
			const params: StockLevelListParams = {
				productId: 'prod-001'
			};
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: {
					items: [mockStockLevel1],
					pagination: { ...mockPagination, totalItems: 1 },
					summary: { ...mockSummary, totalProducts: 1 }
				},
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/stock-levels?productId=prod-001');
			expect(result.data?.items).toHaveLength(1);
			expect(result.data?.items[0].productId).toBe('prod-001');
		});

		it('should list stock levels with search param', async () => {
			const params: StockLevelListParams = {
				search: 'SKU-001'
			};
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: {
					items: [mockStockLevel1],
					pagination: { ...mockPagination, totalItems: 1 },
					summary: mockSummary
				},
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/stock-levels?search=SKU-001');
			expect(result.data?.items).toHaveLength(1);
		});

		it('should list stock levels with lowStockOnly filter', async () => {
			const params: StockLevelListParams = {
				lowStockOnly: true
			};
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: {
					items: [mockStockLevel2],
					pagination: { ...mockPagination, totalItems: 1 },
					summary: mockSummary
				},
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/stock-levels?lowStockOnly=true');
			expect(result.data?.items[0].status).toBe('low_stock');
		});

		it('should list stock levels with outOfStockOnly filter', async () => {
			const params: StockLevelListParams = {
				outOfStockOnly: true
			};
			const outOfStockItem: StockLevelResponse = {
				...mockStockLevel1,
				availableQuantity: 0,
				totalQuantity: 0,
				status: 'out_of_stock'
			};
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: {
					items: [outOfStockItem],
					pagination: { ...mockPagination, totalItems: 1 },
					summary: { ...mockSummary, outOfStockCount: 1 }
				},
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith('/inventory/stock-levels?outOfStockOnly=true');
			expect(result.data?.items[0].status).toBe('out_of_stock');
		});

		it('should list stock levels with sorting params', async () => {
			const params: StockLevelListParams = {
				sortBy: 'available_quantity',
				sortDir: 'desc'
			};
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: mockListResponse,
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith(
				'/inventory/stock-levels?sortBy=available_quantity&sortDir=desc'
			);
			expect(result.status).toBe(200);
		});

		it('should list stock levels with all params combined', async () => {
			const params: StockLevelListParams = {
				page: 1,
				pageSize: 50,
				warehouseId: 'wh-001',
				search: 'test',
				sortBy: 'productName',
				sortDir: 'asc'
			};
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: mockListResponse,
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list(params);

			expect(apiClient.get).toHaveBeenCalledWith(
				'/inventory/stock-levels?page=1&pageSize=50&warehouseId=wh-001&search=test&sortBy=productName&sortDir=asc'
			);
			expect(result.status).toBe(200);
		});

		it('should handle list error', async () => {
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				error: 'Unauthorized',
				status: 401
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();

			expect(result.status).toBe(401);
			expect(result.error).toBe('Unauthorized');
			expect(result.data).toBeUndefined();
		});

		it('should handle server error', async () => {
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				error: 'Internal Server Error',
				status: 500
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();

			expect(result.status).toBe(500);
			expect(result.error).toBe('Internal Server Error');
		});

		it('should handle network error', async () => {
			vi.mocked(apiClient.get).mockRejectedValue(new Error('Network error'));

			await expect(stockLevelApi.list()).rejects.toThrow('Network error');
		});

		it('should return empty items when no stock levels exist', async () => {
			const emptyResponse: StockLevelListResponse = {
				items: [],
				pagination: {
					page: 1,
					pageSize: 20,
					totalItems: 0,
					totalPages: 0,
					hasNext: false,
					hasPrev: false
				},
				summary: {
					totalProducts: 0,
					totalAvailableQuantity: 0,
					totalReservedQuantity: 0,
					lowStockCount: 0,
					outOfStockCount: 0
				}
			};
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: emptyResponse,
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();

			expect(result.data?.items).toHaveLength(0);
			expect(result.data?.summary.totalProducts).toBe(0);
			expect(result.data?.pagination.totalItems).toBe(0);
		});
	});

	describe('response data structure', () => {
		it('should have correct stock level item structure', async () => {
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: mockListResponse,
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();
			const item = result.data?.items[0];

			expect(item).toHaveProperty('inventoryId');
			expect(item).toHaveProperty('tenantId');
			expect(item).toHaveProperty('productId');
			expect(item).toHaveProperty('productSku');
			expect(item).toHaveProperty('productName');
			expect(item).toHaveProperty('warehouseId');
			expect(item).toHaveProperty('warehouseCode');
			expect(item).toHaveProperty('warehouseName');
			expect(item).toHaveProperty('availableQuantity');
			expect(item).toHaveProperty('reservedQuantity');
			expect(item).toHaveProperty('totalQuantity');
			expect(item).toHaveProperty('status');
			expect(item).toHaveProperty('updatedAt');
		});

		it('should have correct summary structure', async () => {
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: mockListResponse,
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();
			const summary = result.data?.summary;

			expect(summary).toHaveProperty('totalProducts');
			expect(summary).toHaveProperty('totalAvailableQuantity');
			expect(summary).toHaveProperty('totalReservedQuantity');
			expect(summary).toHaveProperty('lowStockCount');
			expect(summary).toHaveProperty('outOfStockCount');
		});

		it('should calculate totalQuantity correctly', async () => {
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: mockListResponse,
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();
			const item = result.data?.items[0];

			expect(item?.totalQuantity).toBe(item!.availableQuantity + item!.reservedQuantity);
		});

		it('should handle nullable reorderPoint', async () => {
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: mockListResponse,
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();

			// First item has reorderPoint
			expect(result.data?.items[0].reorderPoint).toBe(20);
			// Third item has null reorderPoint
			expect(result.data?.items[2].reorderPoint).toBeNull();
		});
	});

	describe('stock status values', () => {
		it('should handle in_stock status', async () => {
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: {
					items: [mockStockLevel1],
					pagination: mockPagination,
					summary: mockSummary
				},
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();

			expect(result.data?.items[0].status).toBe('in_stock');
		});

		it('should handle low_stock status', async () => {
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: {
					items: [mockStockLevel2],
					pagination: mockPagination,
					summary: mockSummary
				},
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();

			expect(result.data?.items[0].status).toBe('low_stock');
		});

		it('should handle out_of_stock status', async () => {
			const outOfStockItem: StockLevelResponse = {
				...mockStockLevel1,
				availableQuantity: 0,
				reservedQuantity: 0,
				totalQuantity: 0,
				status: 'out_of_stock'
			};
			const mockResponse: ApiResponse<StockLevelListResponse> = {
				data: {
					items: [outOfStockItem],
					pagination: mockPagination,
					summary: { ...mockSummary, outOfStockCount: 1 }
				},
				status: 200
			};
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await stockLevelApi.list();

			expect(result.data?.items[0].status).toBe('out_of_stock');
			expect(result.data?.items[0].availableQuantity).toBe(0);
		});
	});
});
