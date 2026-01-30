// =============================================================================
// Stock Take API Client
// Handles stock take (physical inventory count) operations
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	StockTake,
	StockTakeLine,
	CreateStockTakeRequest,
	CreateStockTakeResponse,
	CountStockTakeRequest,
	CountStockTakeResponse,
	FinalizeStockTakeResponse,
	StockTakeListParams,
	StockTakeListResponse,
	StockTakeDetailResponse
} from '$lib/types/stock-take';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/stock-takes';

/**
 * Transform backend stock take to frontend format
 * Note: apiClient already transforms snake_case to camelCase
 */
function transformStockTakeFromBackend(st: Record<string, unknown>): StockTake {
	return {
		stockTakeId: (st.stockTakeId as string) || '',
		tenantId: (st.tenantId as string) || '',
		stockTakeNumber: (st.stockTakeNumber as string) || '',
		warehouseId: (st.warehouseId as string) || '',
		status: (st.status as StockTake['status']) || 'draft',
		startedAt: (st.startedAt as string) || null,
		completedAt: (st.completedAt as string) || null,
		createdBy: (st.createdBy as string) || '',
		updatedBy: (st.updatedBy as string) || null,
		notes: (st.notes as string) || null,
		createdAt: (st.createdAt as string) || '',
		updatedAt: (st.updatedAt as string) || '',
		warehouse: st.warehouse as StockTake['warehouse'],
		createdByUser: st.createdByUser as StockTake['createdByUser'],
		lineCount: st.lineCount as number | undefined,
		countedLineCount: st.countedLineCount as number | undefined,
		totalVariance: st.totalVariance as number | undefined
	};
}

/**
 * Transform backend stock take line to frontend format
 */
function transformLineFromBackend(line: Record<string, unknown>): StockTakeLine {
	return {
		lineId: (line.lineId as string) || '',
		tenantId: (line.tenantId as string) || '',
		stockTakeId: (line.stockTakeId as string) || '',
		productId: (line.productId as string) || '',
		expectedQuantity: (line.expectedQuantity as number) || 0,
		actualQuantity: line.actualQuantity as number | null,
		differenceQuantity: line.differenceQuantity as number | null,
		countedBy: (line.countedBy as string) || null,
		countedAt: (line.countedAt as string) || null,
		notes: (line.notes as string) || null,
		createdAt: (line.createdAt as string) || '',
		updatedAt: (line.updatedAt as string) || '',
		product: line.product as StockTakeLine['product'],
		countedByUser: line.countedByUser as StockTakeLine['countedByUser']
	};
}

/**
 * Transform backend list response to frontend format
 */
function transformListResponse(response: Record<string, unknown>): StockTakeListResponse {
	const stockTakes = (response.stockTakes as Record<string, unknown>[]) || [];
	const pagination = response.pagination as Record<string, unknown> | undefined;

	return {
		stockTakes: stockTakes.map(transformStockTakeFromBackend),
		pagination: {
			page: (pagination?.page as number) || 1,
			pageSize: (pagination?.pageSize as number) || (pagination?.limit as number) || 50,
			totalItems: (pagination?.totalItems as number) || (pagination?.total as number) || 0,
			totalPages:
				(pagination?.totalPages as number) ||
				Math.ceil(
					((pagination?.total as number) || 0) /
						((pagination?.pageSize as number) || (pagination?.limit as number) || 50)
				),
			hasNext: (pagination?.hasNext as boolean) || false,
			hasPrev: (pagination?.hasPrev as boolean) || false
		}
	};
}

/**
 * Transform create request to backend format (snake_case)
 */
function transformCreateRequest(data: CreateStockTakeRequest): Record<string, unknown> {
	return {
		warehouse_id: data.warehouseId,
		notes: data.notes || null
	};
}

/**
 * Transform count request to backend format (snake_case)
 */
function transformCountRequest(data: CountStockTakeRequest): Record<string, unknown> {
	return {
		items: data.items.map((item) => ({
			line_id: item.lineId,
			actual_quantity: item.actualQuantity,
			notes: item.notes || null
		}))
	};
}

/**
 * Stock Take API client for inventory service
 */
export const stockTakeApi = {
	/**
	 * List stock takes with optional filtering and pagination
	 */
	async list(params: StockTakeListParams = {}): Promise<ApiResponse<StockTakeListResponse>> {
		const query = buildQueryString(toRecord(params));
		const response = await apiClient.get<Record<string, unknown>>(`${BASE_PATH}${query}`);

		if (response.success && response.data) {
			return {
				...response,
				data: transformListResponse(response.data)
			};
		}

		return response as unknown as ApiResponse<StockTakeListResponse>;
	},

	/**
	 * Get a single stock take with all its lines
	 */
	async get(stockTakeId: string): Promise<ApiResponse<StockTakeDetailResponse>> {
		const response = await apiClient.get<Record<string, unknown>>(`${BASE_PATH}/${stockTakeId}`);

		if (response.success && response.data) {
			const data = response.data;
			return {
				...response,
				data: {
					stockTake: transformStockTakeFromBackend(
						(data.stockTake as Record<string, unknown>) || data
					),
					lines: ((data.lines as Record<string, unknown>[]) || []).map(transformLineFromBackend)
				}
			};
		}

		return response as unknown as ApiResponse<StockTakeDetailResponse>;
	},

	/**
	 * Create a new stock take
	 */
	async create(data: CreateStockTakeRequest): Promise<ApiResponse<CreateStockTakeResponse>> {
		const transformedData = transformCreateRequest(data);
		const response = await apiClient.post<Record<string, unknown>>(BASE_PATH, transformedData);

		if (response.success && response.data) {
			return {
				...response,
				data: {
					stockTake: transformStockTakeFromBackend(
						(response.data.stockTake as Record<string, unknown>) || response.data
					)
				}
			};
		}

		return response as unknown as ApiResponse<CreateStockTakeResponse>;
	},

	/**
	 * Submit counted quantities for stock take lines
	 * This also updates status to 'in_progress' if it was 'draft'
	 */
	async submitCounts(
		stockTakeId: string,
		data: CountStockTakeRequest
	): Promise<ApiResponse<CountStockTakeResponse>> {
		const transformedData = transformCountRequest(data);
		const response = await apiClient.post<Record<string, unknown>>(
			`${BASE_PATH}/${stockTakeId}/count`,
			transformedData
		);

		if (response.success && response.data) {
			const lines = (response.data.lines as Record<string, unknown>[]) || [];
			return {
				...response,
				data: {
					lines: lines.map(transformLineFromBackend)
				}
			};
		}

		return response as unknown as ApiResponse<CountStockTakeResponse>;
	},

	/**
	 * Finalize a stock take
	 * This completes the stock take and creates adjustments for variances
	 */
	async finalize(stockTakeId: string): Promise<ApiResponse<FinalizeStockTakeResponse>> {
		const response = await apiClient.post<Record<string, unknown>>(
			`${BASE_PATH}/${stockTakeId}/finalize`,
			{}
		);

		if (response.success && response.data) {
			const data = response.data;
			return {
				...response,
				data: {
					stockTake: transformStockTakeFromBackend(
						(data.stockTake as Record<string, unknown>) || {}
					),
					adjustments: ((data.adjustments as Record<string, unknown>[]) || []).map((adj) => ({
						adjustmentId: (adj.adjustmentId as string) || '',
						productId: (adj.productId as string) || '',
						warehouseId: (adj.warehouseId as string) || '',
						quantity: (adj.quantity as number) || 0,
						reason: (adj.reason as string) || '',
						adjustedAt: (adj.adjustedAt as string) || ''
					}))
				}
			};
		}

		return response as unknown as ApiResponse<FinalizeStockTakeResponse>;
	},

	// NOTE: Delete endpoint is not implemented in backend yet
	// Uncomment when backend adds DELETE /stock-takes/{id} endpoint
	// async delete(stockTakeId: string): Promise<ApiResponse<{ success: boolean }>> {
	// 	return apiClient.delete<{ success: boolean }>(`${BASE_PATH}/${stockTakeId}`);
	// },

	/**
	 * Get stock takes for a specific warehouse
	 */
	async getByWarehouse(
		warehouseId: string,
		params?: Omit<StockTakeListParams, 'warehouseId'>
	): Promise<ApiResponse<StockTakeListResponse>> {
		return this.list({ ...params, warehouseId });
	},

	/**
	 * Get in-progress stock takes
	 */
	async getInProgress(
		params?: Omit<StockTakeListParams, 'status'>
	): Promise<ApiResponse<StockTakeListResponse>> {
		return this.list({ ...params, status: 'in_progress' });
	}
};
