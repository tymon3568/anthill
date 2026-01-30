// =============================================================================
// Price Lists API Client
// Handles all price list related API operations
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	PriceList,
	PriceListItem,
	CustomerPriceList,
	CreatePriceListInput,
	UpdatePriceListInput,
	CreatePriceListItemInput,
	UpdatePriceListItemInput,
	AssignCustomerInput,
	PriceListFilters
} from '$lib/types/pricing';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/pricing/price-lists';

// =============================================================================
// Response Types
// =============================================================================

export interface PriceListListResponse {
	priceLists: PriceList[];
	pagination: {
		page: number;
		pageSize: number;
		totalItems: number;
		totalPages: number;
	};
}

export interface PriceListItemListResponse {
	items: PriceListItem[];
	pagination: {
		page: number;
		pageSize: number;
		totalItems: number;
		totalPages: number;
	};
}

export interface ListParams extends PriceListFilters {
	page?: number;
	pageSize?: number;
	sortBy?: string;
	sortOrder?: 'asc' | 'desc';
}

export interface ItemListParams {
	page?: number;
	pageSize?: number;
	applyTo?: string;
	search?: string;
}

// =============================================================================
// Price Lists API
// =============================================================================

export const priceListApi = {
	/**
	 * List price lists with optional filtering and pagination
	 */
	async list(params: ListParams = {}): Promise<ApiResponse<PriceListListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<PriceListListResponse>(`${BASE_PATH}${query}`);
	},

	/**
	 * Get a single price list by ID
	 */
	async get(priceListId: string): Promise<ApiResponse<PriceList>> {
		return apiClient.get<PriceList>(`${BASE_PATH}/${priceListId}`);
	},

	/**
	 * Create a new price list
	 */
	async create(data: CreatePriceListInput): Promise<ApiResponse<PriceList>> {
		return apiClient.post<PriceList>(BASE_PATH, toRecord(data));
	},

	/**
	 * Update an existing price list
	 */
	async update(priceListId: string, data: UpdatePriceListInput): Promise<ApiResponse<PriceList>> {
		return apiClient.patch<PriceList>(`${BASE_PATH}/${priceListId}`, toRecord(data));
	},

	/**
	 * Delete a price list
	 */
	async delete(priceListId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/${priceListId}`);
	},

	/**
	 * Set a price list as default
	 */
	async setDefault(priceListId: string): Promise<ApiResponse<PriceList>> {
		return apiClient.post<PriceList>(`${BASE_PATH}/${priceListId}/set-default`, {});
	},

	/**
	 * Bulk activate price lists
	 */
	async bulkActivate(ids: string[]): Promise<ApiResponse<{ updated: number }>> {
		return apiClient.post<{ updated: number }>(`${BASE_PATH}/bulk/activate`, { ids });
	},

	/**
	 * Bulk deactivate price lists
	 */
	async bulkDeactivate(ids: string[]): Promise<ApiResponse<{ updated: number }>> {
		return apiClient.post<{ updated: number }>(`${BASE_PATH}/bulk/deactivate`, { ids });
	},

	/**
	 * Bulk delete price lists
	 */
	async bulkDelete(ids: string[]): Promise<ApiResponse<{ deleted: number }>> {
		return apiClient.post<{ deleted: number }>(`${BASE_PATH}/bulk/delete`, { ids });
	}
};

// =============================================================================
// Price List Items API
// =============================================================================

export const priceListItemApi = {
	/**
	 * List items for a price list
	 */
	async list(
		priceListId: string,
		params: ItemListParams = {}
	): Promise<ApiResponse<PriceListItemListResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<PriceListItemListResponse>(`${BASE_PATH}/${priceListId}/items${query}`);
	},

	/**
	 * Get a single item
	 */
	async get(priceListId: string, itemId: string): Promise<ApiResponse<PriceListItem>> {
		return apiClient.get<PriceListItem>(`${BASE_PATH}/${priceListId}/items/${itemId}`);
	},

	/**
	 * Create a new item
	 */
	async create(
		priceListId: string,
		data: CreatePriceListItemInput
	): Promise<ApiResponse<PriceListItem>> {
		return apiClient.post<PriceListItem>(`${BASE_PATH}/${priceListId}/items`, toRecord(data));
	},

	/**
	 * Update an existing item
	 */
	async update(
		priceListId: string,
		itemId: string,
		data: UpdatePriceListItemInput
	): Promise<ApiResponse<PriceListItem>> {
		return apiClient.patch<PriceListItem>(
			`${BASE_PATH}/${priceListId}/items/${itemId}`,
			toRecord(data)
		);
	},

	/**
	 * Delete an item
	 */
	async delete(priceListId: string, itemId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/${priceListId}/items/${itemId}`);
	},

	/**
	 * Bulk create items
	 */
	async bulkCreate(
		priceListId: string,
		items: CreatePriceListItemInput[]
	): Promise<ApiResponse<{ created: number; items: PriceListItem[] }>> {
		return apiClient.post<{ created: number; items: PriceListItem[] }>(
			`${BASE_PATH}/${priceListId}/items/bulk`,
			{ items }
		);
	},

	/**
	 * Bulk delete items
	 */
	async bulkDelete(
		priceListId: string,
		itemIds: string[]
	): Promise<ApiResponse<{ deleted: number }>> {
		return apiClient.post<{ deleted: number }>(`${BASE_PATH}/${priceListId}/items/bulk/delete`, {
			itemIds
		});
	}
};

// =============================================================================
// Customer Price Lists API
// =============================================================================

export const customerPriceListApi = {
	/**
	 * List customers assigned to a price list
	 */
	async list(priceListId: string): Promise<ApiResponse<CustomerPriceList[]>> {
		return apiClient.get<CustomerPriceList[]>(`${BASE_PATH}/${priceListId}/customers`);
	},

	/**
	 * Assign a customer to a price list
	 */
	async assign(
		priceListId: string,
		customerId: string,
		data: AssignCustomerInput = {}
	): Promise<ApiResponse<CustomerPriceList>> {
		return apiClient.post<CustomerPriceList>(`${BASE_PATH}/${priceListId}/customers`, {
			customerId,
			...toRecord(data)
		});
	},

	/**
	 * Update customer assignment
	 */
	async update(
		priceListId: string,
		customerId: string,
		data: AssignCustomerInput
	): Promise<ApiResponse<CustomerPriceList>> {
		return apiClient.patch<CustomerPriceList>(
			`${BASE_PATH}/${priceListId}/customers/${customerId}`,
			toRecord(data)
		);
	},

	/**
	 * Unassign a customer from a price list
	 */
	async unassign(priceListId: string, customerId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/${priceListId}/customers/${customerId}`);
	},

	/**
	 * Get price lists for a customer
	 */
	async getForCustomer(customerId: string): Promise<ApiResponse<CustomerPriceList[]>> {
		return apiClient.get<CustomerPriceList[]>(`/pricing/customers/${customerId}/price-lists`);
	}
};
