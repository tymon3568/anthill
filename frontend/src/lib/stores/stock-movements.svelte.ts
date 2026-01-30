// =============================================================================
// Stock Movements Store - Svelte 5 Runes-based State Management
// Handles Transfers and Stock Takes
// =============================================================================

import type {
	TransferStatus,
	TransferPriority,
	TransferType,
	CreateTransferRequest,
	PaginationInfo
} from '$lib/types/inventory';
import type {
	StockTake,
	StockTakeLine,
	StockTakeStatus,
	CreateStockTakeRequest,
	CountStockTakeRequest,
	StockTakeAdjustment
} from '$lib/types/stock-take';
import { transferApi } from '$lib/api/inventory/transfers';
import { stockTakeApi } from '$lib/api/inventory/stock-take';

// =============================================================================
// Transfer Types (for Store)
// =============================================================================

export interface TransferResponse {
	transferId: string;
	transferNumber: string;
	tenantId: string;
	sourceWarehouseId: string;
	sourceWarehouseName?: string;
	destinationWarehouseId: string;
	destinationWarehouseName?: string;
	status: TransferStatus;
	transferType: TransferType;
	priority: TransferPriority;
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

export interface TransferListParams {
	sourceWarehouseId?: string;
	destinationWarehouseId?: string;
	status?: TransferStatus;
	search?: string;
	createdAfter?: string;
	createdBefore?: string;
	page?: number;
	pageSize?: number;
}

// =============================================================================
// Transfer State & Store
// =============================================================================

interface TransferState {
	items: TransferResponse[];
	selected: TransferResponse | null;
	pagination: PaginationInfo | null;
	isLoading: boolean;
	error: string | null;
}

export const transferState = $state<TransferState>({
	items: [],
	selected: null,
	pagination: null,
	isLoading: false,
	error: null
});

export const transferStore = {
	/**
	 * Load transfers with optional filtering
	 */
	async load(params: TransferListParams = {}): Promise<void> {
		transferState.isLoading = true;
		transferState.error = null;

		const response = await transferApi.list(params);

		if (response.success && response.data) {
			// Backend returns 'items', map to TransferResponse[]
			const data = response.data as unknown as {
				items: TransferResponse[];
				total: number;
				page: number;
				pageSize: number;
				totalPages: number;
			};
			transferState.items = data.items.map((item: Record<string, unknown>) => ({
				...item,
				transferNumber: item.transferNumber ?? item.transfer_number,
				transferId: item.transferId ?? item.transfer_id,
				tenantId: item.tenantId ?? item.tenant_id,
				sourceWarehouseId: item.sourceWarehouseId ?? item.source_warehouse_id,
				destinationWarehouseId: item.destinationWarehouseId ?? item.destination_warehouse_id,
				transferType: item.transferType ?? item.transfer_type,
				referenceNumber: item.referenceNumber ?? item.reference_number,
				expectedShipDate: item.expectedShipDate ?? item.expected_ship_date,
				expectedReceiveDate: item.expectedReceiveDate ?? item.expected_receive_date,
				actualShipDate: item.actualShipDate ?? item.actual_ship_date,
				actualReceiveDate: item.actualReceiveDate ?? item.actual_receive_date,
				shippingMethod: item.shippingMethod ?? item.shipping_method,
				createdBy: item.createdBy ?? item.created_by,
				confirmedBy: item.confirmedBy ?? item.confirmed_by,
				shippedBy: item.shippedBy ?? item.shipped_by,
				receivedBy: item.receivedBy ?? item.received_by,
				createdAt: item.createdAt ?? item.created_at,
				updatedAt: item.updatedAt ?? item.updated_at
			})) as TransferResponse[];
			transferState.pagination = {
				page: data.page,
				pageSize: data.pageSize,
				totalItems: data.total,
				totalPages: data.totalPages,
				hasNext: data.page < data.totalPages,
				hasPrev: data.page > 1
			};
		} else {
			transferState.error = response.error || 'Failed to load transfers';
		}

		transferState.isLoading = false;
	},

	/**
	 * Get a single transfer by ID
	 */
	async get(transferId: string): Promise<TransferResponse | null> {
		transferState.isLoading = true;
		transferState.error = null;

		const response = await transferApi.get(transferId);

		if (response.success && response.data) {
			transferState.selected = response.data as unknown as TransferResponse;
			transferState.isLoading = false;
			return transferState.selected;
		} else {
			transferState.error = response.error || 'Failed to load transfer';
			transferState.isLoading = false;
			return null;
		}
	},

	/**
	 * Create a new transfer
	 */
	async create(
		data: CreateTransferRequest
	): Promise<{ transferId: string; transferNumber: string } | null> {
		transferState.isLoading = true;
		transferState.error = null;

		const response = await transferApi.create(data);

		if (response.success && response.data) {
			transferState.isLoading = false;
			return {
				transferId: response.data.transferId,
				transferNumber: response.data.transferNumber
			};
		} else {
			transferState.error = response.error || 'Failed to create transfer';
			transferState.isLoading = false;
			return null;
		}
	},

	/**
	 * Confirm a draft transfer
	 */
	async confirm(transferId: string, notes?: string): Promise<boolean> {
		transferState.isLoading = true;
		transferState.error = null;

		const response = await transferApi.confirm(transferId, { notes });

		if (response.success) {
			// Update local state
			transferState.items = transferState.items.map((t) =>
				t.transferId === transferId ? { ...t, status: 'confirmed' as TransferStatus } : t
			);
			if (transferState.selected?.transferId === transferId) {
				transferState.selected = {
					...transferState.selected,
					status: 'confirmed' as TransferStatus
				};
			}
			transferState.isLoading = false;
			return true;
		} else {
			transferState.error = response.error || 'Failed to confirm transfer';
			transferState.isLoading = false;
			return false;
		}
	},

	/**
	 * Ship a confirmed transfer
	 */
	async ship(transferId: string, notes?: string): Promise<boolean> {
		transferState.isLoading = true;
		transferState.error = null;

		const response = await transferApi.ship(transferId, notes);

		if (response.success) {
			// Update local state
			transferState.items = transferState.items.map((t) =>
				t.transferId === transferId ? { ...t, status: 'shipped' as TransferStatus } : t
			);
			if (transferState.selected?.transferId === transferId) {
				transferState.selected = { ...transferState.selected, status: 'shipped' as TransferStatus };
			}
			transferState.isLoading = false;
			return true;
		} else {
			transferState.error = response.error || 'Failed to ship transfer';
			transferState.isLoading = false;
			return false;
		}
	},

	/**
	 * Receive a shipped transfer
	 */
	async receive(transferId: string, notes?: string): Promise<boolean> {
		transferState.isLoading = true;
		transferState.error = null;

		const response = await transferApi.receive(transferId, { notes });

		if (response.success) {
			// Update local state
			transferState.items = transferState.items.map((t) =>
				t.transferId === transferId ? { ...t, status: 'received' as TransferStatus } : t
			);
			if (transferState.selected?.transferId === transferId) {
				transferState.selected = {
					...transferState.selected,
					status: 'received' as TransferStatus
				};
			}
			transferState.isLoading = false;
			return true;
		} else {
			transferState.error = response.error || 'Failed to receive transfer';
			transferState.isLoading = false;
			return false;
		}
	},

	/**
	 * Cancel a transfer
	 */
	async cancel(transferId: string, reason?: string): Promise<boolean> {
		transferState.isLoading = true;
		transferState.error = null;

		const response = await transferApi.cancel(transferId, reason);

		if (response.success) {
			// Update local state
			transferState.items = transferState.items.map((t) =>
				t.transferId === transferId ? { ...t, status: 'cancelled' as TransferStatus } : t
			);
			if (transferState.selected?.transferId === transferId) {
				transferState.selected = {
					...transferState.selected,
					status: 'cancelled' as TransferStatus
				};
			}
			transferState.isLoading = false;
			return true;
		} else {
			transferState.error = response.error || 'Failed to cancel transfer';
			transferState.isLoading = false;
			return false;
		}
	},

	/**
	 * Select a transfer
	 */
	select(transfer: TransferResponse | null): void {
		transferState.selected = transfer;
	},

	/**
	 * Clear transfer state
	 */
	clear(): void {
		transferState.items = [];
		transferState.selected = null;
		transferState.pagination = null;
		transferState.isLoading = false;
		transferState.error = null;
	}
};

// =============================================================================
// Stock Take State & Store
// =============================================================================

interface StockTakeState {
	items: StockTake[];
	selected: StockTake | null;
	lines: StockTakeLine[];
	adjustments: StockTakeAdjustment[];
	pagination: { page: number; limit: number; total: number; totalPages: number } | null;
	isLoading: boolean;
	error: string | null;
}

export const stockTakeState = $state<StockTakeState>({
	items: [],
	selected: null,
	lines: [],
	adjustments: [],
	pagination: null,
	isLoading: false,
	error: null
});

export const stockTakeStore = {
	/**
	 * Load stock takes with optional filtering
	 */
	async load(
		params: { warehouseId?: string; status?: StockTakeStatus; page?: number; limit?: number } = {}
	): Promise<void> {
		stockTakeState.isLoading = true;
		stockTakeState.error = null;

		const response = await stockTakeApi.list(params);

		if (response.success && response.data) {
			stockTakeState.items = response.data.stockTakes;
			stockTakeState.pagination = response.data.pagination;
		} else {
			stockTakeState.error = response.error || 'Failed to load stock takes';
		}

		stockTakeState.isLoading = false;
	},

	/**
	 * Get a single stock take by ID with lines
	 */
	async get(stockTakeId: string): Promise<StockTake | null> {
		stockTakeState.isLoading = true;
		stockTakeState.error = null;

		const response = await stockTakeApi.get(stockTakeId);

		if (response.success && response.data) {
			stockTakeState.selected = response.data.stockTake;
			stockTakeState.lines = response.data.lines;
			stockTakeState.isLoading = false;
			return response.data.stockTake;
		} else {
			stockTakeState.error = response.error || 'Failed to load stock take';
			stockTakeState.isLoading = false;
			return null;
		}
	},

	/**
	 * Create a new stock take session
	 */
	async create(data: CreateStockTakeRequest): Promise<StockTake | null> {
		stockTakeState.isLoading = true;
		stockTakeState.error = null;

		const response = await stockTakeApi.create(data);

		if (response.success && response.data) {
			stockTakeState.items = [response.data.stockTake, ...stockTakeState.items];
			stockTakeState.isLoading = false;
			return response.data.stockTake;
		} else {
			stockTakeState.error = response.error || 'Failed to create stock take';
			stockTakeState.isLoading = false;
			return null;
		}
	},

	/**
	 * Submit counted quantities
	 */
	async count(stockTakeId: string, data: CountStockTakeRequest): Promise<boolean> {
		stockTakeState.isLoading = true;
		stockTakeState.error = null;

		const response = await stockTakeApi.count(stockTakeId, data);

		if (response.success && response.data) {
			// Update lines in local state
			stockTakeState.lines = stockTakeState.lines.map((line) => {
				const updated = response.data!.lines.find((l) => l.lineId === line.lineId);
				return updated || line;
			});
			// Update status to in_progress if it was draft
			if (stockTakeState.selected?.status === 'draft') {
				stockTakeState.selected = { ...stockTakeState.selected, status: 'in_progress' };
				stockTakeState.items = stockTakeState.items.map((st) =>
					st.stockTakeId === stockTakeId ? { ...st, status: 'in_progress' as StockTakeStatus } : st
				);
			}
			stockTakeState.isLoading = false;
			return true;
		} else {
			stockTakeState.error = response.error || 'Failed to submit counts';
			stockTakeState.isLoading = false;
			return false;
		}
	},

	/**
	 * Finalize stock take and create adjustments
	 */
	async finalize(stockTakeId: string): Promise<boolean> {
		stockTakeState.isLoading = true;
		stockTakeState.error = null;

		const response = await stockTakeApi.finalize(stockTakeId);

		if (response.success && response.data) {
			stockTakeState.selected = response.data.stockTake;
			stockTakeState.adjustments = response.data.adjustments;
			// Update in list
			stockTakeState.items = stockTakeState.items.map((st) =>
				st.stockTakeId === stockTakeId ? response.data!.stockTake : st
			);
			stockTakeState.isLoading = false;
			return true;
		} else {
			stockTakeState.error = response.error || 'Failed to finalize stock take';
			stockTakeState.isLoading = false;
			return false;
		}
	},

	/**
	 * Select a stock take
	 */
	select(stockTake: StockTake | null): void {
		stockTakeState.selected = stockTake;
	},

	/**
	 * Clear stock take state
	 */
	clear(): void {
		stockTakeState.items = [];
		stockTakeState.selected = null;
		stockTakeState.lines = [];
		stockTakeState.adjustments = [];
		stockTakeState.pagination = null;
		stockTakeState.isLoading = false;
		stockTakeState.error = null;
	}
};
