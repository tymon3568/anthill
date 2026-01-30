// =============================================================================
// Stock Adjustments API Client
// Handles stock adjustment operations
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	StockAdjustment,
	CreateAdjustmentRequest,
	CreateAdjustmentResponse,
	AdjustmentListParams,
	AdjustmentListResponse,
	AdjustmentSummary
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/adjustments';

/**
 * Transform backend adjustment to frontend StockAdjustment format
 * Note: apiClient already transforms snake_case to camelCase
 * Backend returns adjustment documents without line details (product, quantity, reason)
 */
function transformAdjustmentFromBackend(adj: Record<string, unknown>): StockAdjustment {
	return {
		adjustmentId: (adj.adjustmentId as string) || '',
		tenantId: (adj.tenantId as string) || '',
		moveId: (adj.moveId as string) || '',
		productId: (adj.productId as string) || '',
		warehouseId: (adj.warehouseId as string) || '',
		reasonCode: (adj.reasonCode as string) || '',
		quantity: (adj.quantity as number) || 0,
		notes: (adj.notes as string) || null,
		approvedBy: (adj.approvedBy as string) || (adj.createdBy as string) || '',
		createdAt: (adj.createdAt as string) || '',
		updatedAt: (adj.updatedAt as string) || ''
	};
}

/**
 * Transform backend list response to frontend format
 * Note: apiClient already transforms snake_case to camelCase
 */
function transformListResponse(response: Record<string, unknown>): AdjustmentListResponse {
	const adjustments = (response.adjustments as Record<string, unknown>[]) || [];
	return {
		adjustments: adjustments.map(transformAdjustmentFromBackend),
		pagination: {
			page: (response.page as number) || 1,
			pageSize: (response.pageSize as number) || 50,
			totalCount: (response.totalCount as number) || 0,
			totalPages: Math.ceil(
				((response.totalCount as number) || 0) / ((response.pageSize as number) || 50)
			)
		}
	};
}

/**
 * Map frontend reason code to backend reason code format
 * Frontend uses SCREAMING_SNAKE_CASE, backend uses snake_case
 */
function mapReasonCodeToBackend(code: string): string {
	const mapping: Record<string, string> = {
		COUNT_ERROR: 'count_correction',
		STOCK_TAKE_VARIANCE: 'count_correction',
		FOUND: 'found',
		DAMAGE: 'damaged',
		EXPIRED: 'expired',
		SCRAP: 'other',
		THEFT: 'theft',
		LOST: 'lost',
		WRITE_OFF: 'other',
		SAMPLE: 'promotion',
		PROMOTION: 'promotion',
		INTERNAL_USE: 'other',
		RETURN_TO_STOCK: 'return_to_stock',
		CORRECTION: 'system_correction'
	};
	return mapping[code] || 'other';
}

/**
 * Transform frontend CreateAdjustmentRequest to backend format
 * Backend expects snake_case field names and different structure
 */
function transformCreateRequest(data: CreateAdjustmentRequest): Record<string, unknown> {
	return {
		warehouse_id: data.warehouseId,
		notes: data.notes || null,
		lines: data.items.map((item) => ({
			product_id: item.productId,
			variant_id: null,
			adjustment_type: item.quantity < 0 ? 'decrease' : 'increase',
			qty: Math.abs(item.quantity),
			reason_code: mapReasonCodeToBackend(item.reasonCode),
			reason_notes: item.notes || null,
			location_id: item.locationId || null,
			lot_id: item.lotSerialId || null,
			serial_id: null
		}))
	};
}

/**
 * Stock Adjustments API client for inventory service
 */
export const adjustmentApi = {
	/**
	 * List adjustments with optional filtering and pagination
	 */
	async list(params: AdjustmentListParams = {}): Promise<ApiResponse<AdjustmentListResponse>> {
		const query = buildQueryString(toRecord(params));
		const response = await apiClient.get<Record<string, unknown>>(`${BASE_PATH}${query}`);

		if (response.success && response.data) {
			return {
				...response,
				data: transformListResponse(response.data)
			};
		}

		return response as ApiResponse<AdjustmentListResponse>;
	},

	/**
	 * Get a single adjustment by ID
	 */
	async get(adjustmentId: string): Promise<ApiResponse<StockAdjustment>> {
		return apiClient.get<StockAdjustment>(`${BASE_PATH}/${adjustmentId}`);
	},

	/**
	 * Create new adjustment(s)
	 * Supports multiple items in a single request
	 */
	async create(data: CreateAdjustmentRequest): Promise<ApiResponse<CreateAdjustmentResponse>> {
		const transformedData = transformCreateRequest(data);
		return apiClient.post<CreateAdjustmentResponse>(BASE_PATH, transformedData);
	},

	/**
	 * Delete an adjustment (soft delete)
	 * Only allowed for adjustments that haven't been posted
	 */
	async delete(adjustmentId: string): Promise<ApiResponse<{ success: boolean }>> {
		return apiClient.delete<{ success: boolean }>(`${BASE_PATH}/${adjustmentId}`);
	},

	/**
	 * Get adjustment summary/analytics
	 */
	async getSummary(params?: {
		warehouseId?: string;
		dateFrom?: string;
		dateTo?: string;
	}): Promise<ApiResponse<AdjustmentSummary>> {
		const query = buildQueryString(params ?? {});
		return apiClient.get<AdjustmentSummary>(`${BASE_PATH}/summary${query}`);
	},

	/**
	 * Get adjustments for a specific product
	 */
	async getByProduct(
		productId: string,
		params?: { warehouseId?: string; limit?: number }
	): Promise<ApiResponse<AdjustmentListResponse>> {
		const query = buildQueryString({
			productId,
			...params
		});
		return apiClient.get<AdjustmentListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get adjustments for a specific warehouse
	 */
	async getByWarehouse(
		warehouseId: string,
		params?: AdjustmentListParams
	): Promise<ApiResponse<AdjustmentListResponse>> {
		const query = buildQueryString({
			warehouseId,
			...params
		});
		return apiClient.get<AdjustmentListResponse>(`${BASE_PATH}${query}`);
	}
};
