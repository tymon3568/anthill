// =============================================================================
// Reports API Client
// Handles inventory reporting operations
// =============================================================================

import { apiClient } from '$lib/api/client';
import type { ApiResponse } from '$lib/types';
import type {
	LowStockQuery,
	LowStockEntry,
	DeadStockQuery,
	DeadStockEntry,
	StockAgingQuery,
	StockAgingEntry,
	InventoryTurnoverQuery,
	InventoryTurnoverEntry,
	StockLedgerQuery,
	StockLedgerEntry
} from '$lib/types/inventory';
import { buildQueryString, toRecord } from './utils';

const BASE_PATH = '/inventory/reports';

interface LowStockResponse {
	items: LowStockEntry[];
	totalCount: number;
}

interface DeadStockResponse {
	items: DeadStockEntry[];
	totalCount: number;
}

interface StockAgingResponse {
	items: StockAgingEntry[];
	totalCount: number;
}

interface InventoryTurnoverResponse {
	items: InventoryTurnoverEntry[];
	totalCount: number;
}

interface StockLedgerResponse {
	entries: StockLedgerEntry[];
	totalCount: number;
	openingBalance: number;
	closingBalance: number;
}

/**
 * Reports API client for inventory service
 */
export const reportsApi = {
	/**
	 * Get low stock report
	 */
	async getLowStock(params: LowStockQuery = {}): Promise<ApiResponse<LowStockResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<LowStockResponse>(`${BASE_PATH}/low-stock${query}`);
	},

	/**
	 * Get dead stock report (items with no movement for X days)
	 */
	async getDeadStock(params: DeadStockQuery = {}): Promise<ApiResponse<DeadStockResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<DeadStockResponse>(`${BASE_PATH}/dead-stock${query}`);
	},

	/**
	 * Get stock aging report
	 */
	async getStockAging(params: StockAgingQuery = {}): Promise<ApiResponse<StockAgingResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<StockAgingResponse>(`${BASE_PATH}/aging${query}`);
	},

	/**
	 * Get inventory turnover report
	 */
	async getInventoryTurnover(
		params: InventoryTurnoverQuery = {}
	): Promise<ApiResponse<InventoryTurnoverResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<InventoryTurnoverResponse>(`${BASE_PATH}/turnover${query}`);
	},

	/**
	 * Get stock ledger for a product
	 */
	async getStockLedger(params: StockLedgerQuery): Promise<ApiResponse<StockLedgerResponse>> {
		const query = buildQueryString(toRecord(params));
		return apiClient.get<StockLedgerResponse>(`${BASE_PATH}/ledger${query}`);
	}
};
