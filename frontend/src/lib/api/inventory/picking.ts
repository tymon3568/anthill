// =============================================================================
// Picking API Client
// Handles picking optimization and task management
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	PickingOptimizationRequest,
	PickingPlanResponse,
	ConfirmPickingPlanRequest,
	PickingMethodResponse,
	CreatePickingMethodRequest,
	UpdatePickingMethodRequest,
	PaginationParams
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/picking';

interface PickingMethodListParams extends PaginationParams {
	warehouseId?: string;
	isActive?: boolean;
}

interface PickingMethodListResponse {
	methods: PickingMethodResponse[];
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
 * Picking API client for inventory service
 */
export const pickingApi = {
	// =========================================================================
	// Picking Optimization
	// =========================================================================

	/**
	 * Generate an optimized picking plan for orders
	 */
	async optimize(data: PickingOptimizationRequest): Promise<ApiResponse<PickingPlanResponse>> {
		return apiClient.post<PickingPlanResponse>(`${BASE_PATH}/optimize`, toRecord(data));
	},

	/**
	 * Get a picking plan by ID
	 */
	async getPlan(planId: string): Promise<ApiResponse<PickingPlanResponse>> {
		return apiClient.get<PickingPlanResponse>(`${BASE_PATH}/plans/${planId}`);
	},

	/**
	 * Confirm a picking plan and start execution
	 */
	async confirmPlan(
		data: ConfirmPickingPlanRequest
	): Promise<ApiResponse<{ confirmed: boolean; planId: string }>> {
		return apiClient.post<{ confirmed: boolean; planId: string }>(
			`${BASE_PATH}/confirm`,
			toRecord(data)
		);
	},

	// =========================================================================
	// Picking Methods
	// =========================================================================

	/**
	 * List picking methods
	 */
	async listMethods(
		params: PickingMethodListParams = {}
	): Promise<ApiResponse<PickingMethodListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<PickingMethodListResponse>(`${BASE_PATH}/methods${query}`);
	},

	/**
	 * Get a picking method by ID
	 */
	async getMethod(methodId: string): Promise<ApiResponse<PickingMethodResponse>> {
		return apiClient.get<PickingMethodResponse>(`${BASE_PATH}/methods/${methodId}`);
	},

	/**
	 * Create a new picking method
	 */
	async createMethod(
		data: CreatePickingMethodRequest
	): Promise<ApiResponse<PickingMethodResponse>> {
		return apiClient.post<PickingMethodResponse>(`${BASE_PATH}/methods`, toRecord(data));
	},

	/**
	 * Update a picking method
	 */
	async updateMethod(
		methodId: string,
		data: UpdatePickingMethodRequest
	): Promise<ApiResponse<PickingMethodResponse>> {
		return apiClient.patch<PickingMethodResponse>(
			`${BASE_PATH}/methods/${methodId}`,
			toRecord(data)
		);
	},

	/**
	 * Delete a picking method
	 */
	async deleteMethod(methodId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/methods/${methodId}`);
	}
};
