// =============================================================================
// Quality Control API Client
// Handles quality control points and inspections
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	QualityControlPoint,
	CreateQualityControlPoint,
	UpdateQualityControlPoint,
	QcPointType,
	PaginationParams
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/quality';

interface QcPointListParams extends PaginationParams {
	qcType?: QcPointType;
	productId?: string;
	warehouseId?: string;
	active?: boolean;
}

interface QcPointListResponse {
	qcPoints: QualityControlPoint[];
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
 * Quality Control API client for inventory service
 */
export const qualityApi = {
	/**
	 * List quality control points with optional filtering
	 */
	async list(params: QcPointListParams = {}): Promise<ApiResponse<QcPointListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<QcPointListResponse>(`${BASE_PATH}/points${query}`);
	},

	/**
	 * Get a single QC point by ID
	 */
	async get(qcPointId: string): Promise<ApiResponse<QualityControlPoint>> {
		return apiClient.get<QualityControlPoint>(`${BASE_PATH}/points/${qcPointId}`);
	},

	/**
	 * Create a new quality control point
	 */
	async create(data: CreateQualityControlPoint): Promise<ApiResponse<QualityControlPoint>> {
		return apiClient.post<QualityControlPoint>(`${BASE_PATH}/points`, toRecord(data));
	},

	/**
	 * Update an existing QC point
	 */
	async update(
		qcPointId: string,
		data: UpdateQualityControlPoint
	): Promise<ApiResponse<QualityControlPoint>> {
		return apiClient.patch<QualityControlPoint>(`${BASE_PATH}/points/${qcPointId}`, toRecord(data));
	},

	/**
	 * Delete a QC point
	 */
	async delete(qcPointId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/points/${qcPointId}`);
	},

	/**
	 * Activate a QC point
	 */
	async activate(qcPointId: string): Promise<ApiResponse<QualityControlPoint>> {
		return apiClient.post<QualityControlPoint>(`${BASE_PATH}/points/${qcPointId}/activate`);
	},

	/**
	 * Deactivate a QC point
	 */
	async deactivate(qcPointId: string): Promise<ApiResponse<QualityControlPoint>> {
		return apiClient.post<QualityControlPoint>(`${BASE_PATH}/points/${qcPointId}/deactivate`);
	}
};
