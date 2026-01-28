// =============================================================================
// Transfer API Client
// Handles stock transfer operations between warehouses
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	CreateTransferRequest,
	CreateTransferResponse,
	ConfirmTransferRequest,
	ConfirmTransferResponse,
	ReceiveTransferRequest,
	ReceiveTransferResponse,
	TransferStatus,
	PaginationParams
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/transfers';

interface TransferListParams extends PaginationParams {
	sourceWarehouseId?: string;
	destinationWarehouseId?: string;
	status?: TransferStatus;
	search?: string;
	createdAfter?: string;
	createdBefore?: string;
}

interface TransferResponse {
	transferId: string;
	transferNumber: string;
	tenantId: string;
	sourceWarehouseId: string;
	destinationWarehouseId: string;
	status: TransferStatus;
	transferType: string;
	priority: string;
	referenceNumber?: string | null;
	reason?: string | null;
	expectedShipDate?: string | null;
	expectedReceiveDate?: string | null;
	actualShipDate?: string | null;
	actualReceiveDate?: string | null;
	shippingMethod?: string | null;
	notes?: string | null;
	createdBy: string;
	confirmedBy?: string | null;
	shippedBy?: string | null;
	receivedBy?: string | null;
	createdAt: string;
	updatedAt: string;
}

interface TransferListResponse {
	transfers: TransferResponse[];
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
 * Transfer API client for inventory service
 */
export const transferApi = {
	/**
	 * List transfers with optional filtering
	 */
	async list(params: TransferListParams = {}): Promise<ApiResponse<TransferListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<TransferListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get a single transfer by ID
	 */
	async get(transferId: string): Promise<ApiResponse<TransferResponse>> {
		return apiClient.get<TransferResponse>(`${BASE_PATH}/${transferId}`);
	},

	/**
	 * Create a new transfer
	 */
	async create(data: CreateTransferRequest): Promise<ApiResponse<CreateTransferResponse>> {
		return apiClient.post<CreateTransferResponse>(BASE_PATH, toRecord(data));
	},

	/**
	 * Confirm a draft transfer
	 */
	async confirm(
		transferId: string,
		data: ConfirmTransferRequest = {}
	): Promise<ApiResponse<ConfirmTransferResponse>> {
		return apiClient.post<ConfirmTransferResponse>(
			`${BASE_PATH}/${transferId}/confirm`,
			toRecord(data)
		);
	},

	/**
	 * Ship a confirmed transfer
	 */
	async ship(
		transferId: string,
		notes?: string
	): Promise<ApiResponse<{ transferId: string; status: TransferStatus; shippedAt: string }>> {
		return apiClient.post<{ transferId: string; status: TransferStatus; shippedAt: string }>(
			`${BASE_PATH}/${transferId}/ship`,
			notes ? { notes } : undefined
		);
	},

	/**
	 * Receive a shipped transfer
	 */
	async receive(
		transferId: string,
		data: ReceiveTransferRequest = {}
	): Promise<ApiResponse<ReceiveTransferResponse>> {
		return apiClient.post<ReceiveTransferResponse>(
			`${BASE_PATH}/${transferId}/receive`,
			toRecord(data)
		);
	},

	/**
	 * Cancel a transfer
	 */
	async cancel(
		transferId: string,
		reason?: string
	): Promise<ApiResponse<{ transferId: string; status: TransferStatus }>> {
		return apiClient.post<{ transferId: string; status: TransferStatus }>(
			`${BASE_PATH}/${transferId}/cancel`,
			reason ? { reason } : undefined
		);
	}
};
