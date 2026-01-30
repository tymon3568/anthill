// =============================================================================
// Replenishment API Client
// Handles reorder rules and replenishment checking
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	ReorderRule,
	CreateReorderRule,
	UpdateReorderRule,
	ReplenishmentCheckResult,
	PaginationParams
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/replenishment';

interface ReorderRuleListParams extends PaginationParams {
	productId?: string;
	warehouseId?: string;
}

interface ReorderRuleListResponse {
	rules: ReorderRule[];
	pagination: {
		page: number;
		pageSize: number;
		totalItems: number;
		totalPages: number;
		hasNext: boolean;
		hasPrev: boolean;
	};
}

interface ReplenishmentCheckParams {
	productId?: string;
	warehouseId?: string;
	autoTrigger?: boolean;
}

/**
 * Replenishment API client for inventory service
 */
export const replenishmentApi = {
	// =========================================================================
	// Reorder Rules
	// =========================================================================

	/**
	 * List reorder rules
	 */
	async listRules(
		params: ReorderRuleListParams = {}
	): Promise<ApiResponse<ReorderRuleListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<ReorderRuleListResponse>(`${BASE_PATH}/rules${query}`);
	},

	/**
	 * Get a reorder rule by ID
	 */
	async getRule(ruleId: string): Promise<ApiResponse<ReorderRule>> {
		return apiClient.get<ReorderRule>(`${BASE_PATH}/rules/${ruleId}`);
	},

	/**
	 * Create a new reorder rule
	 */
	async createRule(data: CreateReorderRule): Promise<ApiResponse<ReorderRule>> {
		return apiClient.post<ReorderRule>(`${BASE_PATH}/rules`, toRecord(data));
	},

	/**
	 * Update an existing reorder rule
	 */
	async updateRule(ruleId: string, data: UpdateReorderRule): Promise<ApiResponse<ReorderRule>> {
		return apiClient.patch<ReorderRule>(`${BASE_PATH}/rules/${ruleId}`, toRecord(data));
	},

	/**
	 * Delete a reorder rule
	 */
	async deleteRule(ruleId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/rules/${ruleId}`);
	},

	// =========================================================================
	// Replenishment Checks
	// =========================================================================

	/**
	 * Check replenishment needs for products
	 */
	async check(
		params: ReplenishmentCheckParams = {}
	): Promise<ApiResponse<ReplenishmentCheckResult[]>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<ReplenishmentCheckResult[]>(`${BASE_PATH}/check${query}`);
	},

	/**
	 * Trigger replenishment for a specific product
	 */
	async trigger(
		productId: string,
		warehouseId?: string
	): Promise<ApiResponse<ReplenishmentCheckResult>> {
		return apiClient.post<ReplenishmentCheckResult>(`${BASE_PATH}/trigger`, {
			productId,
			warehouseId
		});
	}
};
