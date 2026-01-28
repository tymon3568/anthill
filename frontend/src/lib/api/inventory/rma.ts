// =============================================================================
// RMA API Client
// Handles Return Merchandise Authorization operations
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	CreateRmaRequest,
	CreateRmaResponse,
	ApproveRmaRequest,
	ApproveRmaResponse,
	ReceiveRmaRequest,
	ReceiveRmaResponse,
	RmaStatus,
	PaginationParams
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/rma';

interface RmaListParams extends PaginationParams {
	customerId?: string;
	status?: RmaStatus;
	search?: string;
	createdAfter?: string;
	createdBefore?: string;
}

interface RmaResponse {
	rmaId: string;
	rmaNumber: string;
	tenantId: string;
	customerId: string;
	originalDeliveryId: string;
	status: RmaStatus;
	returnReason?: string | null;
	notes?: string | null;
	createdBy: string;
	approvedBy?: string | null;
	receivedBy?: string | null;
	processedBy?: string | null;
	approvedAt?: string | null;
	receivedAt?: string | null;
	processedAt?: string | null;
	createdAt: string;
	updatedAt: string;
}

interface RmaListResponse {
	rmas: RmaResponse[];
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
 * RMA API client for inventory service
 */
export const rmaApi = {
	/**
	 * List RMAs with optional filtering
	 */
	async list(params: RmaListParams = {}): Promise<ApiResponse<RmaListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<RmaListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get a single RMA by ID
	 */
	async get(rmaId: string): Promise<ApiResponse<RmaResponse>> {
		return apiClient.get<RmaResponse>(`${BASE_PATH}/${rmaId}`);
	},

	/**
	 * Create a new RMA
	 */
	async create(data: CreateRmaRequest): Promise<ApiResponse<CreateRmaResponse>> {
		return apiClient.post<CreateRmaResponse>(BASE_PATH, toRecord(data));
	},

	/**
	 * Approve or reject an RMA
	 */
	async approve(rmaId: string, data: ApproveRmaRequest): Promise<ApiResponse<ApproveRmaResponse>> {
		return apiClient.post<ApproveRmaResponse>(`${BASE_PATH}/${rmaId}/approve`, toRecord(data));
	},

	/**
	 * Receive returned items for an approved RMA
	 */
	async receive(rmaId: string, data: ReceiveRmaRequest): Promise<ApiResponse<ReceiveRmaResponse>> {
		return apiClient.post<ReceiveRmaResponse>(`${BASE_PATH}/${rmaId}/receive`, toRecord(data));
	},

	/**
	 * Process an RMA (restock, scrap, refund, or exchange)
	 */
	async process(
		rmaId: string,
		notes?: string
	): Promise<ApiResponse<{ rmaId: string; status: RmaStatus; processedAt: string }>> {
		return apiClient.post<{ rmaId: string; status: RmaStatus; processedAt: string }>(
			`${BASE_PATH}/${rmaId}/process`,
			notes ? { notes } : undefined
		);
	}
};
