// =============================================================================
// Stock Take Types
// TypeScript types for Stock Take module
// =============================================================================

import type { PaginationInfo } from './inventory';

// =============================================================================
// Enums
// =============================================================================

export type StockTakeStatus = 'draft' | 'scheduled' | 'in_progress' | 'completed' | 'cancelled';

// =============================================================================
// Domain Types
// =============================================================================

/**
 * Stock Take entity - represents a physical inventory count session
 */
export interface StockTake {
	stockTakeId: string;
	tenantId: string;
	stockTakeNumber: string;
	warehouseId: string;
	status: StockTakeStatus;
	startedAt: string | null;
	completedAt: string | null;
	createdBy: string;
	updatedBy: string | null;
	notes: string | null;
	createdAt: string;
	updatedAt: string;
	// Joined/computed fields
	warehouse?: {
		warehouseId: string;
		warehouseCode: string;
		warehouseName: string;
	};
	createdByUser?: {
		userId: string;
		fullName: string;
	};
	lineCount?: number;
	countedLineCount?: number;
	totalVariance?: number;
}

/**
 * Stock Take Line - represents a single product line in a stock take
 */
export interface StockTakeLine {
	lineId: string;
	tenantId: string;
	stockTakeId: string;
	productId: string;
	expectedQuantity: number;
	actualQuantity: number | null;
	differenceQuantity: number | null;
	countedBy: string | null;
	countedAt: string | null;
	notes: string | null;
	createdAt: string;
	updatedAt: string;
	// Joined fields
	product?: {
		productId: string;
		sku: string;
		name: string;
	};
	countedByUser?: {
		userId: string;
		fullName: string;
	};
}

// =============================================================================
// Request Types
// =============================================================================

/**
 * Request to create a new stock take
 */
export interface CreateStockTakeRequest {
	warehouseId: string;
	notes?: string | null;
}

/**
 * Individual count item for submitting counts
 */
export interface CountItem {
	lineId: string;
	actualQuantity: number;
	notes?: string | null;
}

/**
 * Request to submit counted quantities
 */
export interface CountStockTakeRequest {
	items: CountItem[];
}

/**
 * Query parameters for listing stock takes
 */
export interface StockTakeListParams {
	warehouseId?: string | null;
	status?: StockTakeStatus | null;
	page?: number;
	limit?: number;
	search?: string;
}

// =============================================================================
// Response Types
// =============================================================================

/**
 * Response for stock take creation
 */
export interface CreateStockTakeResponse {
	stockTake: StockTake;
}

/**
 * Response for count submission
 */
export interface CountStockTakeResponse {
	lines: StockTakeLine[];
}

/**
 * Stock adjustment generated from stock take discrepancies
 */
export interface StockTakeAdjustment {
	adjustmentId: string;
	productId: string;
	warehouseId: string;
	quantity: number;
	reason: string;
	adjustedAt: string;
}

/**
 * Response for stock take finalization
 */
export interface FinalizeStockTakeResponse {
	stockTake: StockTake;
	adjustments: StockTakeAdjustment[];
}

/**
 * Response for stock take list
 */
export interface StockTakeListResponse {
	stockTakes: StockTake[];
	pagination: PaginationInfo;
}

/**
 * Response for getting a single stock take with lines
 */
export interface StockTakeDetailResponse {
	stockTake: StockTake;
	lines: StockTakeLine[];
}

// =============================================================================
// UI Helper Types
// =============================================================================

/**
 * Stock take with computed UI fields
 */
export interface StockTakeWithProgress extends StockTake {
	progress: number; // 0-100 percentage
	countedCount: number;
	totalCount: number;
	hasVariance: boolean;
}

/**
 * Stock take line with computed UI fields
 */
export interface StockTakeLineWithVariance extends StockTakeLine {
	variancePercentage: number | null;
	isPositiveVariance: boolean;
	isNegativeVariance: boolean;
	isCounted: boolean;
}

/**
 * Summary statistics for stock takes
 */
export interface StockTakeSummary {
	total: number;
	draft: number;
	scheduled: number;
	inProgress: number;
	completed: number;
	cancelled: number;
}

/**
 * Filter state for stock take list
 */
export interface StockTakeFilters {
	status: StockTakeStatus | 'all';
	warehouseId: string | 'all';
	search: string;
}
