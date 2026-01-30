// =============================================================================
// Reconciliation API Client
// Handles stock reconciliation and cycle counting operations
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	StockReconciliation,
	CreateReconciliationRequest,
	CreateReconciliationResponse,
	CountReconciliationRequest,
	CountReconciliationResponse,
	ApproveReconciliationRequest,
	ApproveReconciliationResponse,
	FinalizeReconciliationResponse,
	ReconciliationListResponse,
	ReconciliationDetailResponse,
	ReconciliationAnalyticsResponse,
	VarianceAnalysisResponse,
	ScanBarcodeRequest,
	ScanBarcodeResponse,
	ReconciliationStatus,
	CycleType,
	PaginationParams
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/reconciliation';

interface ReconciliationListParams extends PaginationParams {
	warehouseId?: string;
	status?: ReconciliationStatus;
	cycleType?: CycleType;
	search?: string;
}

/**
 * Reconciliation API client for inventory service
 */
export const reconciliationApi = {
	/**
	 * List reconciliations with optional filtering
	 */
	async list(
		params: ReconciliationListParams = {}
	): Promise<ApiResponse<ReconciliationListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<ReconciliationListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get a single reconciliation by ID
	 */
	async get(reconciliationId: string): Promise<ApiResponse<ReconciliationDetailResponse>> {
		return apiClient.get<ReconciliationDetailResponse>(`${BASE_PATH}/${reconciliationId}`);
	},

	/**
	 * Create a new reconciliation/cycle count
	 */
	async create(
		data: CreateReconciliationRequest
	): Promise<ApiResponse<CreateReconciliationResponse>> {
		return apiClient.post<CreateReconciliationResponse>(BASE_PATH, toRecord(data));
	},

	/**
	 * Start a reconciliation (change status to in_progress)
	 */
	async start(reconciliationId: string): Promise<ApiResponse<StockReconciliation>> {
		return apiClient.post<StockReconciliation>(`${BASE_PATH}/${reconciliationId}/start`);
	},

	/**
	 * Submit count data for reconciliation items
	 */
	async count(
		reconciliationId: string,
		data: CountReconciliationRequest
	): Promise<ApiResponse<CountReconciliationResponse>> {
		return apiClient.post<CountReconciliationResponse>(
			`${BASE_PATH}/${reconciliationId}/count`,
			toRecord(data)
		);
	},

	/**
	 * Scan barcode for quick counting
	 */
	async scanBarcode(
		reconciliationId: string,
		data: ScanBarcodeRequest
	): Promise<ApiResponse<ScanBarcodeResponse>> {
		return apiClient.post<ScanBarcodeResponse>(
			`${BASE_PATH}/${reconciliationId}/scan`,
			toRecord(data)
		);
	},

	/**
	 * Approve a completed reconciliation
	 */
	async approve(
		reconciliationId: string,
		data: ApproveReconciliationRequest = {}
	): Promise<ApiResponse<ApproveReconciliationResponse>> {
		return apiClient.post<ApproveReconciliationResponse>(
			`${BASE_PATH}/${reconciliationId}/approve`,
			toRecord(data)
		);
	},

	/**
	 * Finalize reconciliation and create stock adjustments
	 */
	async finalize(reconciliationId: string): Promise<ApiResponse<FinalizeReconciliationResponse>> {
		return apiClient.post<FinalizeReconciliationResponse>(
			`${BASE_PATH}/${reconciliationId}/finalize`
		);
	},

	/**
	 * Cancel a reconciliation
	 */
	async cancel(reconciliationId: string): Promise<ApiResponse<StockReconciliation>> {
		return apiClient.post<StockReconciliation>(`${BASE_PATH}/${reconciliationId}/cancel`);
	},

	/**
	 * Get reconciliation analytics
	 */
	async getAnalytics(warehouseId?: string): Promise<ApiResponse<ReconciliationAnalyticsResponse>> {
		const query = warehouseId ? `?warehouse_id=${warehouseId}` : '';
		return apiClient.get<ReconciliationAnalyticsResponse>(`${BASE_PATH}/analytics${query}`);
	},

	/**
	 * Get variance analysis for a reconciliation
	 */
	async getVarianceAnalysis(
		reconciliationId: string
	): Promise<ApiResponse<VarianceAnalysisResponse>> {
		return apiClient.get<VarianceAnalysisResponse>(
			`${BASE_PATH}/${reconciliationId}/variance-analysis`
		);
	}
};
