// =============================================================================
// Inventory API Module Index
// Re-exports all inventory API clients for convenient imports
// =============================================================================

export { categoryApi } from './categories';
export { productApi } from './products';
export { productImageApi } from './product-images';
export { productImportApi } from './product-import';
export { warehouseApi } from './warehouses';
export { receiptApi } from './receipts';
export { lotSerialApi } from './lot-serials';
export { transferApi } from './transfers';
export { reconciliationApi } from './reconciliation';
export { rmaApi } from './rma';
export { qualityApi } from './quality';
export { pickingApi } from './picking';
export { putawayApi } from './putaway';
export { replenishmentApi } from './replenishment';
export { valuationApi } from './valuation';
export { reportsApi } from './reports';
export { stockLevelApi } from './stock-levels';
export { adjustmentApi } from './adjustments';
export { stockTakeApi } from './stock-take';

// Re-export types for convenience
export type * from '$lib/types/inventory';
