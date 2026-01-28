// =============================================================================
// Price Calculation API Client
// Handles price calculation and preview operations
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	PriceRequest,
	PriceResult,
	BulkPriceRequest,
	BulkPriceResult,
	QuantityBreak,
	ActivePromotion
} from '$lib/types/pricing';
import { toRecord } from './utils';

const BASE_PATH = '/pricing/calculate';

// =============================================================================
// Price Calculation API
// =============================================================================

export const priceCalculationApi = {
	/**
	 * Calculate price for a single product
	 */
	async calculate(request: PriceRequest): Promise<ApiResponse<PriceResult>> {
		return apiClient.post<PriceResult>(BASE_PATH, toRecord(request));
	},

	/**
	 * Calculate prices for multiple products (bulk)
	 */
	async calculateBulk(request: BulkPriceRequest): Promise<ApiResponse<BulkPriceResult>> {
		return apiClient.post<BulkPriceResult>(`${BASE_PATH}/bulk`, toRecord(request));
	},

	/**
	 * Get quantity breaks for a product
	 */
	async getQuantityBreaks(
		productId: string,
		options: {
			variantId?: string;
			customerId?: string;
			priceListId?: string;
		} = {}
	): Promise<ApiResponse<QuantityBreak[]>> {
		const params = new URLSearchParams();
		if (options.variantId) params.append('variantId', options.variantId);
		if (options.customerId) params.append('customerId', options.customerId);
		if (options.priceListId) params.append('priceListId', options.priceListId);
		const query = params.toString();
		return apiClient.get<QuantityBreak[]>(
			`${BASE_PATH}/quantity-breaks/${productId}${query ? `?${query}` : ''}`
		);
	},

	/**
	 * Get active promotions for a product or order
	 */
	async getActivePromotions(options: {
		productId?: string;
		variantId?: string;
		customerId?: string;
		categoryId?: string;
	}): Promise<ApiResponse<ActivePromotion[]>> {
		const params = new URLSearchParams();
		if (options.productId) params.append('productId', options.productId);
		if (options.variantId) params.append('variantId', options.variantId);
		if (options.customerId) params.append('customerId', options.customerId);
		if (options.categoryId) params.append('categoryId', options.categoryId);
		const query = params.toString();
		return apiClient.get<ActivePromotion[]>(
			`/pricing/promotions/active${query ? `?${query}` : ''}`
		);
	},

	/**
	 * Preview price with a specific price list (for testing)
	 */
	async previewWithPriceList(
		priceListId: string,
		request: PriceRequest
	): Promise<ApiResponse<PriceResult>> {
		return apiClient.post<PriceResult>(`${BASE_PATH}/preview/${priceListId}`, toRecord(request));
	},

	/**
	 * Simulate pricing rule application
	 */
	async simulateRule(
		ruleId: string,
		request: PriceRequest
	): Promise<
		ApiResponse<{
			wouldApply: boolean;
			reason?: string;
			priceWithRule: PriceResult;
			priceWithoutRule: PriceResult;
			savings: number;
		}>
	> {
		return apiClient.post(`/pricing/rules/${ruleId}/simulate`, toRecord(request));
	}
};
