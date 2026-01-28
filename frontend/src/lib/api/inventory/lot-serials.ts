// =============================================================================
// Lot/Serial API Client
// Handles lot and serial number tracking operations for inventory service
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	LotSerial,
	CreateLotSerialRequest,
	LotSerialLifecycle,
	QuarantineResponse,
	PaginationParams
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/lot-serials';

interface LotSerialListParams extends PaginationParams {
	productId?: string;
	status?: string;
	trackingType?: 'lot' | 'serial';
	warehouseId?: string;
	search?: string;
}

interface LotSerialListResponse {
	lotSerials: LotSerial[];
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
 * Lot/Serial API client for inventory service
 */
export const lotSerialApi = {
	/**
	 * List lot/serial numbers with optional filtering
	 */
	async list(params: LotSerialListParams = {}): Promise<ApiResponse<LotSerialListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<LotSerialListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get a single lot/serial by ID
	 */
	async get(lotSerialId: string): Promise<ApiResponse<LotSerial>> {
		return apiClient.get<LotSerial>(`${BASE_PATH}/${lotSerialId}`);
	},

	/**
	 * Create a new lot/serial record
	 */
	async create(data: CreateLotSerialRequest): Promise<ApiResponse<LotSerial>> {
		return apiClient.post<LotSerial>(BASE_PATH, toRecord(data));
	},

	/**
	 * Get lot/serial lifecycle (full history including stock moves and quality checks)
	 */
	async getLifecycle(lotSerialId: string): Promise<ApiResponse<LotSerialLifecycle>> {
		return apiClient.get<LotSerialLifecycle>(`${BASE_PATH}/${lotSerialId}/lifecycle`);
	},

	/**
	 * Quarantine a batch of lot/serial numbers
	 */
	async quarantine(
		lotSerialIds: string[],
		reason?: string
	): Promise<ApiResponse<QuarantineResponse>> {
		return apiClient.post<QuarantineResponse>(`${BASE_PATH}/quarantine`, {
			lotSerialIds,
			reason
		});
	},

	/**
	 * Release lot/serial numbers from quarantine
	 */
	async release(lotSerialIds: string[], reason?: string): Promise<ApiResponse<QuarantineResponse>> {
		return apiClient.post<QuarantineResponse>(`${BASE_PATH}/release`, {
			lotSerialIds,
			reason
		});
	},

	/**
	 * Get expiring lot/serial numbers
	 */
	async getExpiring(
		daysAhead: number = 30,
		warehouseId?: string
	): Promise<ApiResponse<LotSerialListResponse>> {
		const params: Record<string, unknown> = { daysAhead };
		if (warehouseId) params.warehouseId = warehouseId;
		const query = buildQueryString(params);
		return apiClient.get<LotSerialListResponse>(`${BASE_PATH}/expiring${query}`);
	}
};
