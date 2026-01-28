// =============================================================================
// Receipt (GRN) API Client
// Handles goods receipt note operations for inventory service
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	ReceiptResponse,
	ReceiptCreateRequest,
	ReceiptListResponse,
	ReceiptListParams
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/receipts';

/**
 * Receipt (GRN) API client for inventory service
 */
export const receiptApi = {
	/**
	 * List receipts with optional filtering and pagination
	 */
	async list(params: ReceiptListParams = {}): Promise<ApiResponse<ReceiptListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<ReceiptListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get a single receipt by ID
	 */
	async get(receiptId: string): Promise<ApiResponse<ReceiptResponse>> {
		return apiClient.get<ReceiptResponse>(`${BASE_PATH}/${receiptId}`);
	},

	/**
	 * Create a new receipt (Goods Receipt Note)
	 */
	async create(data: ReceiptCreateRequest): Promise<ApiResponse<ReceiptResponse>> {
		return apiClient.post<ReceiptResponse>(BASE_PATH, toRecord(data));
	},

	/**
	 * Confirm a draft receipt
	 */
	async confirm(receiptId: string): Promise<ApiResponse<ReceiptResponse>> {
		return apiClient.post<ReceiptResponse>(`${BASE_PATH}/${receiptId}/confirm`);
	},

	/**
	 * Cancel a receipt
	 */
	async cancel(receiptId: string): Promise<ApiResponse<ReceiptResponse>> {
		return apiClient.post<ReceiptResponse>(`${BASE_PATH}/${receiptId}/cancel`);
	},

	/**
	 * Complete a receipt (finalize receiving)
	 */
	async complete(receiptId: string): Promise<ApiResponse<ReceiptResponse>> {
		return apiClient.post<ReceiptResponse>(`${BASE_PATH}/${receiptId}/complete`);
	}
};
