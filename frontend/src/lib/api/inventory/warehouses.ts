// =============================================================================
// Warehouse API Client
// Handles warehouse, zone, and location operations for inventory service
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	WarehouseResponse,
	CreateWarehouseRequest,
	WarehouseZoneResponse,
	CreateWarehouseZoneRequest,
	WarehouseLocationResponse,
	CreateWarehouseLocationRequest,
	PaginationParams,
	LotSerial
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/warehouses';

interface WarehouseListResponse {
	warehouses: WarehouseResponse[];
	pagination: {
		page: number;
		pageSize: number;
		totalItems: number;
		totalPages: number;
		hasNext: boolean;
		hasPrev: boolean;
	};
}

interface ZoneListResponse {
	zones: WarehouseZoneResponse[];
	pagination: {
		page: number;
		pageSize: number;
		totalItems: number;
		totalPages: number;
		hasNext: boolean;
		hasPrev: boolean;
	};
}

interface LocationListResponse {
	locations: WarehouseLocationResponse[];
	pagination: {
		page: number;
		pageSize: number;
		totalItems: number;
		totalPages: number;
		hasNext: boolean;
		hasPrev: boolean;
	};
}

interface LocationStockResponse {
	lotSerials: LotSerial[];
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
 * Warehouse API client for inventory service
 */
export const warehouseApi = {
	// =========================================================================
	// Warehouse Operations
	// =========================================================================

	/**
	 * List all warehouses
	 * Note: Backend returns raw array, we wrap it in expected format
	 */
	async list(params: PaginationParams = {}): Promise<ApiResponse<WarehouseListResponse>> {
		const query = buildQueryString(toRecord(params));
		const response = await apiClient.get<WarehouseResponse[]>(`${BASE_PATH}${query}`);

		// Backend returns raw array, wrap in expected format
		if (response.success && response.data) {
			const warehouses = response.data;
			return {
				success: true,
				data: {
					warehouses,
					pagination: {
						page: 1,
						pageSize: warehouses.length,
						totalItems: warehouses.length,
						totalPages: 1,
						hasNext: false,
						hasPrev: false
					}
				}
			};
		}

		return {
			success: false,
			error: response.error || 'Failed to load warehouses'
		};
	},

	/**
	 * Get a single warehouse by ID
	 */
	async get(warehouseId: string): Promise<ApiResponse<WarehouseResponse>> {
		return apiClient.get<WarehouseResponse>(`${BASE_PATH}/${warehouseId}`);
	},

	/**
	 * Create a new warehouse
	 */
	async create(data: CreateWarehouseRequest): Promise<ApiResponse<WarehouseResponse>> {
		return apiClient.post<WarehouseResponse>(BASE_PATH, toRecord(data));
	},

	/**
	 * Update an existing warehouse
	 */
	async update(
		warehouseId: string,
		data: Partial<CreateWarehouseRequest>
	): Promise<ApiResponse<WarehouseResponse>> {
		return apiClient.patch<WarehouseResponse>(`${BASE_PATH}/${warehouseId}`, toRecord(data));
	},

	/**
	 * Delete a warehouse
	 */
	async delete(warehouseId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/${warehouseId}`);
	},

	// =========================================================================
	// Zone Operations
	// =========================================================================

	/**
	 * List zones for a warehouse
	 * Note: Backend returns raw array, we wrap it in expected format
	 */
	async listZones(
		warehouseId: string,
		params: PaginationParams = {}
	): Promise<ApiResponse<ZoneListResponse>> {
		const query = buildQueryString(toRecord(params));
		const response = await apiClient.get<WarehouseZoneResponse[]>(
			`${BASE_PATH}/${warehouseId}/zones${query}`
		);

		// Backend returns raw array, wrap in expected format
		if (response.success && response.data) {
			const zones = response.data;
			return {
				success: true,
				data: {
					zones,
					pagination: {
						page: 1,
						pageSize: zones.length,
						totalItems: zones.length,
						totalPages: 1,
						hasNext: false,
						hasPrev: false
					}
				}
			};
		}

		return {
			success: false,
			error: response.error || 'Failed to load zones'
		};
	},

	/**
	 * Get a single zone
	 */
	async getZone(warehouseId: string, zoneId: string): Promise<ApiResponse<WarehouseZoneResponse>> {
		return apiClient.get<WarehouseZoneResponse>(`${BASE_PATH}/${warehouseId}/zones/${zoneId}`);
	},

	/**
	 * Create a new zone in a warehouse
	 */
	async createZone(
		warehouseId: string,
		data: CreateWarehouseZoneRequest
	): Promise<ApiResponse<WarehouseZoneResponse>> {
		return apiClient.post<WarehouseZoneResponse>(
			`${BASE_PATH}/${warehouseId}/zones`,
			toRecord(data)
		);
	},

	/**
	 * Update a zone
	 */
	async updateZone(
		warehouseId: string,
		zoneId: string,
		data: Partial<CreateWarehouseZoneRequest>
	): Promise<ApiResponse<WarehouseZoneResponse>> {
		return apiClient.patch<WarehouseZoneResponse>(
			`${BASE_PATH}/${warehouseId}/zones/${zoneId}`,
			toRecord(data)
		);
	},

	/**
	 * Delete a zone
	 */
	async deleteZone(warehouseId: string, zoneId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/${warehouseId}/zones/${zoneId}`);
	},

	// =========================================================================
	// Location Operations
	// =========================================================================

	/**
	 * List locations for a warehouse
	 * Note: Backend returns raw array, we wrap it in expected format
	 */
	async listLocations(
		warehouseId: string,
		params: PaginationParams & { zoneId?: string } = {}
	): Promise<ApiResponse<LocationListResponse>> {
		const query = buildQueryString(toRecord(params));
		const response = await apiClient.get<WarehouseLocationResponse[]>(
			`${BASE_PATH}/${warehouseId}/locations${query}`
		);

		// Backend returns raw array, wrap in expected format
		if (response.success && response.data) {
			const locations = response.data;
			return {
				success: true,
				data: {
					locations,
					pagination: {
						page: 1,
						pageSize: locations.length,
						totalItems: locations.length,
						totalPages: 1,
						hasNext: false,
						hasPrev: false
					}
				}
			};
		}

		return {
			success: false,
			error: response.error || 'Failed to load locations'
		};
	},

	/**
	 * Get a single location
	 */
	async getLocation(
		warehouseId: string,
		locationId: string
	): Promise<ApiResponse<WarehouseLocationResponse>> {
		return apiClient.get<WarehouseLocationResponse>(
			`${BASE_PATH}/${warehouseId}/locations/${locationId}`
		);
	},

	/**
	 * Create a new location in a warehouse
	 */
	async createLocation(
		warehouseId: string,
		data: CreateWarehouseLocationRequest
	): Promise<ApiResponse<WarehouseLocationResponse>> {
		return apiClient.post<WarehouseLocationResponse>(
			`${BASE_PATH}/${warehouseId}/locations`,
			toRecord(data)
		);
	},

	/**
	 * Update a location
	 */
	async updateLocation(
		warehouseId: string,
		locationId: string,
		data: Partial<CreateWarehouseLocationRequest>
	): Promise<ApiResponse<WarehouseLocationResponse>> {
		return apiClient.patch<WarehouseLocationResponse>(
			`${BASE_PATH}/${warehouseId}/locations/${locationId}`,
			toRecord(data)
		);
	},

	/**
	 * Delete a location
	 */
	async deleteLocation(warehouseId: string, locationId: string): Promise<ApiResponse<void>> {
		return apiClient.delete<void>(`${BASE_PATH}/${warehouseId}/locations/${locationId}`);
	},

	/**
	 * Get stock (lot/serials) at a specific location
	 */
	async getLocationStock(
		warehouseId: string,
		locationId: string,
		params: PaginationParams = {}
	): Promise<ApiResponse<LocationStockResponse>> {
		const query = buildQueryString(toRecord({ ...params, locationId }));
		return apiClient.get<LocationStockResponse>(`/inventory/lot-serials${query}`);
	}
};
