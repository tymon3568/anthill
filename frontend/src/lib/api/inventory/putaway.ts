// =============================================================================
// Putaway API Client
// Handles putaway suggestions and confirmation operations
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	PutawayRequest,
	PutawayResponse,
	ConfirmPutawayRequest,
	ConfirmPutawayResponse
} from '$lib/types/inventory';
import { toRecord } from './utils';

const BASE_PATH = '/inventory/putaway';

/**
 * Putaway API client for inventory service
 */
export const putawayApi = {
	/**
	 * Get putaway suggestions for a product
	 */
	async getSuggestions(data: PutawayRequest): Promise<ApiResponse<PutawayResponse>> {
		return apiClient.post<PutawayResponse>(`${BASE_PATH}/suggest`, toRecord(data));
	},

	/**
	 * Confirm putaway and create stock movements
	 */
	async confirm(data: ConfirmPutawayRequest): Promise<ApiResponse<ConfirmPutawayResponse>> {
		return apiClient.post<ConfirmPutawayResponse>(`${BASE_PATH}/confirm`, toRecord(data));
	}
};
