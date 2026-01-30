// =============================================================================
// Pricing Rules API Client
// Handles all pricing rule related API operations
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	PricingRule,
	PricingRuleUsage,
	CreatePricingRuleInput,
	UpdatePricingRuleInput,
	PricingRuleFilters
} from '$lib/types/pricing';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/pricing/rules';

// =============================================================================
// Response Types
// =============================================================================

export interface PricingRuleListResponse {
	rules: PricingRule[];
	pagination: {
		page: number;
		pageSize: number;
		totalItems: number;
		totalPages: number;
	};
}

export interface RuleListParams extends PricingRuleFilters {
	page?: number;
	pageSize?: number;
	sortBy?: string;
	sortOrder?: 'asc' | 'desc';
}

export interface RuleUsageParams {
	page?: number;
	pageSize?: number;
	startDate?: string;
	endDate?: string;
}

// =============================================================================
// Pricing Rules API
// =============================================================================

export const pricingRuleApi = {
	/**
	 * List pricing rules with optional filtering and pagination
	 */
	async list(params: RuleListParams = {}): Promise<ApiResponse<PricingRuleListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<PricingRuleListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get a single pricing rule by ID
	 */
	async get(ruleId: string): Promise<ApiResponse<PricingRule>> {
		return apiClient.get<PricingRule>(`${BASE_PATH}/${ruleId}`);
	},

	/**
	 * Create a new pricing rule
	 */
	async create(data: CreatePricingRuleInput): Promise<ApiResponse<PricingRule>> {
		return apiClient.post<PricingRule>(BASE_PATH, toRecord(data));
	},

	/**
	 * Update an existing pricing rule
	 */
	async update(ruleId: string, data: UpdatePricingRuleInput): Promise<ApiResponse<PricingRule>> {
		return apiClient.patch<PricingRule>(`${BASE_PATH}/${ruleId}`, toRecord(data));
	},

	/**
	 * Delete a pricing rule
	 */
	async delete(ruleId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/${ruleId}`);
	},

	/**
	 * Duplicate a pricing rule
	 */
	async duplicate(ruleId: string): Promise<ApiResponse<PricingRule>> {
		return apiClient.post<PricingRule>(`${BASE_PATH}/${ruleId}/duplicate`, {});
	},

	/**
	 * Activate a pricing rule
	 */
	async activate(ruleId: string): Promise<ApiResponse<PricingRule>> {
		return apiClient.post<PricingRule>(`${BASE_PATH}/${ruleId}/activate`, {});
	},

	/**
	 * Deactivate a pricing rule
	 */
	async deactivate(ruleId: string): Promise<ApiResponse<PricingRule>> {
		return apiClient.post<PricingRule>(`${BASE_PATH}/${ruleId}/deactivate`, {});
	},

	/**
	 * Get usage statistics for a pricing rule
	 */
	async getUsage(
		ruleId: string,
		params: RuleUsageParams = {}
	): Promise<
		ApiResponse<{
			usage: PricingRuleUsage[];
			summary: {
				totalUsageCount: number;
				totalDiscountAmount: number;
				uniqueCustomers: number;
				uniqueOrders: number;
			};
		}>
	> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get(`${BASE_PATH}/${ruleId}/usage${query}`);
	},

	/**
	 * Bulk activate pricing rules
	 */
	async bulkActivate(ids: string[]): Promise<ApiResponse<{ updated: number }>> {
		return apiClient.post<{ updated: number }>(`${BASE_PATH}/bulk/activate`, { ids });
	},

	/**
	 * Bulk deactivate pricing rules
	 */
	async bulkDeactivate(ids: string[]): Promise<ApiResponse<{ updated: number }>> {
		return apiClient.post<{ updated: number }>(`${BASE_PATH}/bulk/deactivate`, { ids });
	},

	/**
	 * Bulk delete pricing rules
	 */
	async bulkDelete(ids: string[]): Promise<ApiResponse<{ deleted: number }>> {
		return apiClient.post<{ deleted: number }>(`${BASE_PATH}/bulk/delete`, { ids });
	}
};
