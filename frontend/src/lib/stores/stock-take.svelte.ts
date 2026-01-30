// =============================================================================
// Stock Take Store - Svelte 5 Runes-based State Management
// Handles stock take (physical inventory count) operations
// =============================================================================

import type { PaginationInfo } from '$lib/types/inventory';
import type {
	StockTake,
	StockTakeLine,
	CreateStockTakeRequest,
	CountStockTakeRequest,
	StockTakeListParams,
	StockTakeDetailResponse,
	StockTakeSummary,
	StockTakeFilters,
	StockTakeWithProgress,
	StockTakeLineWithVariance
} from '$lib/types/stock-take';
import { stockTakeApi } from '$lib/api/inventory/stock-take';

// =============================================================================
// Stock Take State
// =============================================================================

interface StockTakeState {
	items: StockTake[];
	selected: StockTake | null;
	selectedLines: StockTakeLine[];
	pagination: PaginationInfo | null;
	summary: StockTakeSummary | null;
	filters: StockTakeFilters;
	isLoading: boolean;
	isSubmitting: boolean;
	error: string | null;
}

const defaultFilters: StockTakeFilters = {
	status: 'all',
	warehouseId: 'all',
	search: ''
};

export const stockTakeState = $state<StockTakeState>({
	items: [],
	selected: null,
	selectedLines: [],
	pagination: null,
	summary: null,
	filters: { ...defaultFilters },
	isLoading: false,
	isSubmitting: false,
	error: null
});

// =============================================================================
// Computed / Derived Values
// =============================================================================

/**
 * Get items with progress computed
 */
export function getItemsWithProgress(): StockTakeWithProgress[] {
	return stockTakeState.items.map((item) => {
		const totalCount = item.lineCount ?? 0;
		const countedCount = item.countedLineCount ?? 0;
		const progress = totalCount > 0 ? Math.round((countedCount / totalCount) * 100) : 0;

		return {
			...item,
			progress,
			countedCount,
			totalCount,
			hasVariance: (item.totalVariance ?? 0) !== 0
		};
	});
}

/**
 * Get lines with variance computed
 */
export function getLinesWithVariance(): StockTakeLineWithVariance[] {
	return stockTakeState.selectedLines.map((line) => {
		const variancePercentage =
			line.differenceQuantity !== null && line.expectedQuantity > 0
				? Math.round((line.differenceQuantity / line.expectedQuantity) * 100)
				: null;

		return {
			...line,
			variancePercentage,
			isPositiveVariance: (line.differenceQuantity ?? 0) > 0,
			isNegativeVariance: (line.differenceQuantity ?? 0) < 0,
			isCounted: line.actualQuantity !== null
		};
	});
}

/**
 * Calculate summary from current items
 */
export function calculateSummary(): StockTakeSummary {
	const items = stockTakeState.items;
	return {
		total: items.length,
		draft: items.filter((i) => i.status === 'draft').length,
		scheduled: items.filter((i) => i.status === 'scheduled').length,
		inProgress: items.filter((i) => i.status === 'in_progress').length,
		completed: items.filter((i) => i.status === 'completed').length,
		cancelled: items.filter((i) => i.status === 'cancelled').length
	};
}

/**
 * Get count progress for selected stock take
 */
export function getCountProgress(): { counted: number; total: number; percentage: number } {
	const lines = stockTakeState.selectedLines;
	const total = lines.length;
	const counted = lines.filter((l) => l.actualQuantity !== null).length;
	const percentage = total > 0 ? Math.round((counted / total) * 100) : 0;

	return { counted, total, percentage };
}

// =============================================================================
// Stock Take Store Actions
// =============================================================================

export const stockTakeStore = {
	/**
	 * Load stock takes with optional filtering and pagination
	 */
	async load(params: StockTakeListParams = {}): Promise<void> {
		stockTakeState.isLoading = true;
		stockTakeState.error = null;

		// Apply filters from state if not provided
		const filters = stockTakeState.filters;
		const finalParams: StockTakeListParams = {
			...params,
			status: params.status ?? (filters.status !== 'all' ? filters.status : undefined),
			warehouseId:
				params.warehouseId ?? (filters.warehouseId !== 'all' ? filters.warehouseId : undefined),
			search: params.search ?? (filters.search || undefined)
		};

		const response = await stockTakeApi.list(finalParams);

		if (response.success && response.data) {
			stockTakeState.items = response.data.stockTakes;
			stockTakeState.pagination = response.data.pagination ?? null;
			stockTakeState.summary = calculateSummary();
		} else {
			stockTakeState.error = response.error || 'Failed to load stock takes';
		}

		stockTakeState.isLoading = false;
	},

	/**
	 * Get a single stock take with its lines
	 */
	async get(stockTakeId: string): Promise<StockTakeDetailResponse | null> {
		stockTakeState.isLoading = true;
		stockTakeState.error = null;

		const response = await stockTakeApi.get(stockTakeId);

		if (response.success && response.data) {
			stockTakeState.selected = response.data.stockTake;
			stockTakeState.selectedLines = response.data.lines;
			stockTakeState.isLoading = false;
			return response.data;
		} else {
			stockTakeState.error = response.error || 'Failed to load stock take';
			stockTakeState.isLoading = false;
			return null;
		}
	},

	/**
	 * Create a new stock take
	 */
	async create(data: CreateStockTakeRequest): Promise<StockTake | null> {
		stockTakeState.isSubmitting = true;
		stockTakeState.error = null;

		const response = await stockTakeApi.create(data);

		if (response.success && response.data) {
			stockTakeState.isSubmitting = false;
			return response.data.stockTake;
		} else {
			stockTakeState.error = response.error || 'Failed to create stock take';
			stockTakeState.isSubmitting = false;
			return null;
		}
	},

	/**
	 * Submit counted quantities for stock take lines
	 */
	async submitCounts(stockTakeId: string, data: CountStockTakeRequest): Promise<boolean> {
		stockTakeState.isSubmitting = true;
		stockTakeState.error = null;

		const response = await stockTakeApi.submitCounts(stockTakeId, data);

		if (response.success && response.data) {
			// Update local lines with the returned updated lines
			const updatedLines = response.data.lines;
			stockTakeState.selectedLines = stockTakeState.selectedLines.map((line) => {
				const updated = updatedLines.find((u) => u.lineId === line.lineId);
				return updated ?? line;
			});

			// Also update the selected stock take status if it changed
			if (stockTakeState.selected && stockTakeState.selected.status === 'draft') {
				stockTakeState.selected = {
					...stockTakeState.selected,
					status: 'in_progress'
				};
			}

			stockTakeState.isSubmitting = false;
			return true;
		} else {
			stockTakeState.error = response.error || 'Failed to submit counts';
			stockTakeState.isSubmitting = false;
			return false;
		}
	},

	/**
	 * Finalize a stock take (complete and create adjustments)
	 */
	async finalize(stockTakeId: string): Promise<boolean> {
		stockTakeState.isSubmitting = true;
		stockTakeState.error = null;

		const response = await stockTakeApi.finalize(stockTakeId);

		if (response.success && response.data) {
			// Update the selected stock take
			stockTakeState.selected = response.data.stockTake;

			// Update in list if present
			stockTakeState.items = stockTakeState.items.map((item) =>
				item.stockTakeId === stockTakeId ? response.data!.stockTake : item
			);

			stockTakeState.isSubmitting = false;
			return true;
		} else {
			stockTakeState.error = response.error || 'Failed to finalize stock take';
			stockTakeState.isSubmitting = false;
			return false;
		}
	},

	// NOTE: Delete is not implemented in backend yet
	// This method will fail until backend adds DELETE endpoint
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	async delete(_stockTakeId: string): Promise<boolean> {
		stockTakeState.error = 'Delete functionality is not available yet';
		return false;
	},

	/**
	 * Update a single line's count locally (optimistic update)
	 */
	updateLineCount(lineId: string, actualQuantity: number, notes?: string): void {
		stockTakeState.selectedLines = stockTakeState.selectedLines.map((line) => {
			if (line.lineId === lineId) {
				const differenceQuantity = actualQuantity - line.expectedQuantity;
				return {
					...line,
					actualQuantity,
					differenceQuantity,
					notes: notes ?? line.notes
				};
			}
			return line;
		});
	},

	/**
	 * Set filters and reload
	 */
	async setFilters(filters: Partial<StockTakeFilters>): Promise<void> {
		stockTakeState.filters = { ...stockTakeState.filters, ...filters };
		await this.load();
	},

	/**
	 * Reset filters to default
	 */
	async resetFilters(): Promise<void> {
		stockTakeState.filters = { ...defaultFilters };
		await this.load();
	},

	/**
	 * Select a stock take
	 */
	select(stockTake: StockTake | null): void {
		stockTakeState.selected = stockTake;
		if (!stockTake) {
			stockTakeState.selectedLines = [];
		}
	},

	/**
	 * Clear all stock take state
	 */
	clear(): void {
		stockTakeState.items = [];
		stockTakeState.selected = null;
		stockTakeState.selectedLines = [];
		stockTakeState.pagination = null;
		stockTakeState.summary = null;
		stockTakeState.filters = { ...defaultFilters };
		stockTakeState.isLoading = false;
		stockTakeState.isSubmitting = false;
		stockTakeState.error = null;
	},

	/**
	 * Clear error state
	 */
	clearError(): void {
		stockTakeState.error = null;
	},

	/**
	 * Check if a stock take can be edited
	 */
	canEdit(stockTake: StockTake): boolean {
		return stockTake.status === 'draft';
	},

	/**
	 * Check if a stock take can be started
	 */
	canStart(stockTake: StockTake): boolean {
		return stockTake.status === 'draft' || stockTake.status === 'scheduled';
	},

	/**
	 * Check if a stock take can be finalized
	 */
	canFinalize(stockTake: StockTake, lines: StockTakeLine[]): boolean {
		if (stockTake.status !== 'in_progress') return false;
		// All lines must be counted
		return lines.every((line) => line.actualQuantity !== null);
	},

	/**
	 * Check if a stock take can be deleted
	 */
	canDelete(stockTake: StockTake): boolean {
		return stockTake.status === 'draft' || stockTake.status === 'cancelled';
	},

	/**
	 * Check if a stock take can be cancelled
	 */
	canCancel(stockTake: StockTake): boolean {
		return stockTake.status !== 'completed' && stockTake.status !== 'cancelled';
	}
};
