// =============================================================================
// Stock Take API Client
// Handles physical inventory counting operations
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/stock-takes';

// =============================================================================
// Types
// =============================================================================

export type StockTakeStatus = 'draft' | 'in_progress' | 'completed' | 'cancelled';

export interface StockTakeResponse {
	stockTakeId: string;
	stockTakeNumber: string;
	tenantId: string;
	warehouseId: string;
	warehouseName?: string;
	status: StockTakeStatus;
	startedAt?: string | null;
	completedAt?: string | null;
	createdBy: string;
	notes?: string | null;
	createdAt: string;
	updatedAt: string;
}

export interface StockTakeLineResponse {
	lineId: string;
	stockTakeId: string;
	productId: string;
	productSku?: string;
	productName?: string;
	expectedQuantity: number;
	actualQuantity?: number | null;
	differenceQuantity?: number | null;
	countedBy?: string | null;
	countedAt?: string | null;
	notes?: string | null;
	createdAt: string;
	updatedAt: string;
}

export interface StockTakeListResponse {
	stockTakes: StockTakeResponse[];
	pagination: {
		page: number;
		limit: number;
		total: number;
		totalPages: number;
	};
}

export interface StockTakeDetailResponse {
	stockTake: StockTakeResponse;
	lines: StockTakeLineResponse[];
}

export interface CreateStockTakeRequest {
	warehouseId: string;
	notes?: string | null;
}

export interface CreateStockTakeResponse {
	stockTake: StockTakeResponse;
}

export interface CountItemRequest {
	productId: string;
	actualQuantity: number;
	notes?: string | null;
}

export interface CountStockTakeRequest {
	items: CountItemRequest[];
}

export interface CountStockTakeResponse {
	lines: StockTakeLineResponse[];
}

export interface StockAdjustmentResponse {
	adjustmentId: string;
	productId: string;
	warehouseId: string;
	quantity: number;
	reason: string;
	adjustedAt: string;
}

export interface FinalizeStockTakeResponse {
	stockTake: StockTakeResponse;
	adjustments: StockAdjustmentResponse[];
}

export interface StockTakeListParams {
	warehouseId?: string;
	status?: StockTakeStatus;
	page?: number;
	limit?: number;
}

/**
 * Stock Take API client for inventory service
 */
export const stockTakeApi = {
	/**
	 * List stock takes with optional filtering
	 */
	async list(params: StockTakeListParams = {}): Promise<ApiResponse<StockTakeListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<StockTakeListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get a single stock take by ID with its lines
	 */
	async get(stockTakeId: string): Promise<ApiResponse<StockTakeDetailResponse>> {
		return apiClient.get<StockTakeDetailResponse>(`${BASE_PATH}/${stockTakeId}`);
	},

	/**
	 * Create a new stock take session
	 */
	async create(data: CreateStockTakeRequest): Promise<ApiResponse<CreateStockTakeResponse>> {
		return apiClient.post<CreateStockTakeResponse>(BASE_PATH, toRecord(data));
	},

	/**
	 * Submit counted quantities for stock take lines
	 */
	async count(
		stockTakeId: string,
		data: CountStockTakeRequest
	): Promise<ApiResponse<CountStockTakeResponse>> {
		return apiClient.post<CountStockTakeResponse>(
			`${BASE_PATH}/${stockTakeId}/count`,
			toRecord(data)
		);
	},

	/**
	 * Finalize stock take and create adjustments
	 */
	async finalize(stockTakeId: string): Promise<ApiResponse<FinalizeStockTakeResponse>> {
		return apiClient.post<FinalizeStockTakeResponse>(`${BASE_PATH}/${stockTakeId}/finalize`);
	}
};
