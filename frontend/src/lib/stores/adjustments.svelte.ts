// =============================================================================
// Stock Adjustments Store - Svelte 5 Runes-based State Management
// Handles stock adjustment CRUD operations
// =============================================================================

import type {
	StockAdjustment,
	CreateAdjustmentRequest,
	AdjustmentListParams,
	AdjustmentSummary,
	PaginationInfo
} from '$lib/types/inventory';
import { adjustmentApi } from '$lib/api/inventory/adjustments';

// =============================================================================
// Adjustment State
// =============================================================================

interface AdjustmentState {
	items: StockAdjustment[];
	selected: StockAdjustment | null;
	pagination: PaginationInfo | null;
	summary: AdjustmentSummary | null;
	isLoading: boolean;
	error: string | null;
}

export const adjustmentState = $state<AdjustmentState>({
	items: [],
	selected: null,
	pagination: null,
	summary: null,
	isLoading: false,
	error: null
});

// =============================================================================
// Adjustment Store Actions
// =============================================================================

export const adjustmentStore = {
	/**
	 * Load adjustments with optional filtering and pagination
	 */
	async load(params: AdjustmentListParams = {}): Promise<void> {
		adjustmentState.isLoading = true;
		adjustmentState.error = null;

		const response = await adjustmentApi.list(params);

		if (response.success && response.data) {
			adjustmentState.items = response.data.adjustments;
			adjustmentState.pagination = response.data.pagination ?? null;
		} else {
			adjustmentState.error = response.error || 'Failed to load adjustments';
		}

		adjustmentState.isLoading = false;
	},

	/**
	 * Get a single adjustment by ID
	 */
	async get(adjustmentId: string): Promise<StockAdjustment | null> {
		adjustmentState.isLoading = true;
		adjustmentState.error = null;

		const response = await adjustmentApi.get(adjustmentId);

		if (response.success && response.data) {
			adjustmentState.selected = response.data;
			adjustmentState.isLoading = false;
			return response.data;
		} else {
			adjustmentState.error = response.error || 'Failed to load adjustment';
			adjustmentState.isLoading = false;
			return null;
		}
	},

	/**
	 * Create new adjustment(s)
	 * Note: Does not add to local state since the list page will reload fresh data
	 */
	async create(data: CreateAdjustmentRequest): Promise<boolean> {
		adjustmentState.isLoading = true;
		adjustmentState.error = null;

		const response = await adjustmentApi.create(data);

		if (response.success && response.data) {
			// Success - don't add to local state, let the list page reload
			adjustmentState.isLoading = false;
			return true;
		} else {
			adjustmentState.error = response.error || 'Failed to create adjustment';
			adjustmentState.isLoading = false;
			return false;
		}
	},

	/**
	 * Delete an adjustment (soft delete)
	 */
	async delete(adjustmentId: string): Promise<boolean> {
		adjustmentState.isLoading = true;
		adjustmentState.error = null;

		const response = await adjustmentApi.delete(adjustmentId);

		if (response.success) {
			// Remove from local state
			adjustmentState.items = adjustmentState.items.filter((a) => a.adjustmentId !== adjustmentId);
			if (adjustmentState.selected?.adjustmentId === adjustmentId) {
				adjustmentState.selected = null;
			}
			adjustmentState.isLoading = false;
			return true;
		} else {
			adjustmentState.error = response.error || 'Failed to delete adjustment';
			adjustmentState.isLoading = false;
			return false;
		}
	},

	/**
	 * Load adjustment summary/analytics
	 */
	async loadSummary(params?: {
		warehouseId?: string;
		dateFrom?: string;
		dateTo?: string;
	}): Promise<void> {
		adjustmentState.isLoading = true;
		adjustmentState.error = null;

		const response = await adjustmentApi.getSummary(params);

		if (response.success && response.data) {
			adjustmentState.summary = response.data;
		} else {
			adjustmentState.error = response.error || 'Failed to load summary';
		}

		adjustmentState.isLoading = false;
	},

	/**
	 * Load adjustments for a specific product
	 */
	async loadByProduct(
		productId: string,
		params?: { warehouseId?: string; limit?: number }
	): Promise<void> {
		adjustmentState.isLoading = true;
		adjustmentState.error = null;

		const response = await adjustmentApi.getByProduct(productId, params);

		if (response.success && response.data) {
			adjustmentState.items = response.data.adjustments;
			adjustmentState.pagination = response.data.pagination ?? null;
		} else {
			adjustmentState.error = response.error || 'Failed to load adjustments';
		}

		adjustmentState.isLoading = false;
	},

	/**
	 * Load adjustments for a specific warehouse
	 */
	async loadByWarehouse(warehouseId: string, params?: AdjustmentListParams): Promise<void> {
		adjustmentState.isLoading = true;
		adjustmentState.error = null;

		const response = await adjustmentApi.getByWarehouse(warehouseId, params);

		if (response.success && response.data) {
			adjustmentState.items = response.data.adjustments;
			adjustmentState.pagination = response.data.pagination ?? null;
		} else {
			adjustmentState.error = response.error || 'Failed to load adjustments';
		}

		adjustmentState.isLoading = false;
	},

	/**
	 * Select an adjustment
	 */
	select(adjustment: StockAdjustment | null): void {
		adjustmentState.selected = adjustment;
	},

	/**
	 * Clear all adjustment state
	 */
	clear(): void {
		adjustmentState.items = [];
		adjustmentState.selected = null;
		adjustmentState.pagination = null;
		adjustmentState.summary = null;
		adjustmentState.isLoading = false;
		adjustmentState.error = null;
	},

	/**
	 * Clear error state
	 */
	clearError(): void {
		adjustmentState.error = null;
	}
};
