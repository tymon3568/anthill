// =============================================================================
// Valuation API Client
// Handles inventory valuation operations (FIFO, AVCO, Standard Cost)
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	ValuationDto,
	ValuationLayersResponse,
	ValuationHistoryResponse,
	SetValuationMethodPayload,
	SetStandardCostPayload,
	RevaluationPayload,
	CostAdjustmentPayload,
	PaginationParams
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/valuation';

interface ValuationListParams extends PaginationParams {
	warehouseId?: string;
}

interface ValuationListResponse {
	valuations: ValuationDto[];
	pagination: {
		page: number;
		pageSize: number;
		totalItems: number;
		totalPages: number;
		hasNext: boolean;
		hasPrev: boolean;
	};
}

/**
 * Valuation API client for inventory service
 */
export const valuationApi = {
	/**
	 * List all product valuations
	 */
	async list(params: ValuationListParams = {}): Promise<ApiResponse<ValuationListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<ValuationListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get valuation for a specific product
	 */
	async get(productId: string): Promise<ApiResponse<ValuationDto>> {
		return apiClient.get<ValuationDto>(`${BASE_PATH}/${productId}`);
	},

	/**
	 * Get FIFO layers for a product
	 */
	async getLayers(productId: string): Promise<ApiResponse<ValuationLayersResponse>> {
		return apiClient.get<ValuationLayersResponse>(`${BASE_PATH}/${productId}/layers`);
	},

	/**
	 * Get valuation history for a product
	 */
	async getHistory(
		productId: string,
		params: PaginationParams = {}
	): Promise<ApiResponse<ValuationHistoryResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<ValuationHistoryResponse>(`${BASE_PATH}/${productId}/history${query}`);
	},

	/**
	 * Set valuation method for a product
	 */
	async setMethod(
		productId: string,
		data: SetValuationMethodPayload
	): Promise<ApiResponse<ValuationDto>> {
		return apiClient.post<ValuationDto>(`${BASE_PATH}/${productId}/method`, toRecord(data));
	},

	/**
	 * Set standard cost for a product
	 */
	async setStandardCost(
		productId: string,
		data: SetStandardCostPayload
	): Promise<ApiResponse<ValuationDto>> {
		return apiClient.post<ValuationDto>(`${BASE_PATH}/${productId}/standard-cost`, toRecord(data));
	},

	/**
	 * Perform revaluation for a product
	 */
	async revalue(productId: string, data: RevaluationPayload): Promise<ApiResponse<ValuationDto>> {
		return apiClient.post<ValuationDto>(`${BASE_PATH}/${productId}/revalue`, toRecord(data));
	},

	/**
	 * Apply cost adjustment for a product
	 */
	async adjustCost(
		productId: string,
		data: CostAdjustmentPayload
	): Promise<ApiResponse<ValuationDto>> {
		return apiClient.post<ValuationDto>(`${BASE_PATH}/${productId}/adjust`, toRecord(data));
	}
};
