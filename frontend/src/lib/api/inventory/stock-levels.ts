// =============================================================================
// Stock Levels API Client
// Handles stock level queries for inventory service
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type { StockLevelListResponse, StockLevelListParams } from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/stock-levels';

/**
 * Stock Levels API client for inventory service
 */
export const stockLevelApi = {
	/**
	 * List stock levels with optional filtering and pagination
	 */
	async list(params: StockLevelListParams = {}): Promise<ApiResponse<StockLevelListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<StockLevelListResponse>(`${BASE_PATH}${query}`);
	}
};
