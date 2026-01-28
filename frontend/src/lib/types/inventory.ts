// =============================================================================
// Inventory Service Types
// Generated from OpenAPI spec: shared/openapi/inventory.yaml
// =============================================================================

// =============================================================================
// Common Types & Enums
// =============================================================================

export type SortDirection = 'asc' | 'desc';

export type CategorySortField = 'name' | 'display_order' | 'created_at' | 'updated_at';

export type ProductTrackingMethod = 'none' | 'lot' | 'serial';

export type LotSerialTrackingType = 'lot' | 'serial';

export type LotSerialStatus = 'active' | 'expired' | 'quarantined' | 'disposed' | 'reserved';

export type QcPointType = 'Incoming' | 'Outgoing' | 'Internal';

export type ReconciliationStatus = 'draft' | 'in_progress' | 'completed' | 'cancelled';

export type CycleType = 'full' | 'abc_a' | 'abc_b' | 'abc_c' | 'location' | 'random';

export type TransferStatus =
	| 'draft'
	| 'confirmed'
	| 'partially_picked'
	| 'picked'
	| 'partially_shipped'
	| 'shipped'
	| 'received'
	| 'cancelled';

export type TransferPriority = 'low' | 'normal' | 'high' | 'urgent';

export type TransferType = 'manual' | 'auto_replenishment' | 'emergency' | 'consolidation';

export type RmaStatus = 'draft' | 'approved' | 'received' | 'processed' | 'rejected';

export type RmaCondition = 'new' | 'used' | 'damaged' | 'defective';

export type RmaAction = 'restock' | 'scrap' | 'refund' | 'exchange';

export type ValuationMethod = 'fifo' | 'avco' | 'standard';

// =============================================================================
// Pagination Types
// =============================================================================

export interface PaginationInfo {
	page: number;
	pageSize: number;
	totalItems: number;
	totalPages: number;
	hasNext: boolean;
	hasPrev: boolean;
}

export interface PaginationParams {
	page?: number;
	pageSize?: number;
	sortBy?: string;
	sortDir?: SortDirection;
}

// =============================================================================
// Category Types
// =============================================================================

export interface CategoryBreadcrumb {
	categoryId: string;
	name: string;
	slug?: string | null;
	level: number;
}

export interface CategoryResponse {
	categoryId: string;
	tenantId: string;
	name: string;
	slug?: string | null;
	code?: string | null;
	description?: string | null;
	parentCategoryId?: string | null;
	path: string;
	level: number;
	displayOrder: number;
	isActive: boolean;
	isVisible: boolean;
	productCount: number;
	totalProductCount: number;
	icon?: string | null;
	color?: string | null;
	imageUrl?: string | null;
	metaTitle?: string | null;
	metaDescription?: string | null;
	metaKeywords?: string | null;
	breadcrumbs?: CategoryBreadcrumb[] | null;
	createdAt: string;
	updatedAt: string;
}

export interface CategoryCreateRequest {
	name: string;
	slug?: string | null;
	code?: string | null;
	description?: string | null;
	parentCategoryId?: string | null;
	displayOrder?: number;
	isActive?: boolean;
	isVisible?: boolean;
	icon?: string | null;
	color?: string | null;
	imageUrl?: string | null;
	metaTitle?: string | null;
	metaDescription?: string | null;
	metaKeywords?: string | null;
}

export interface CategoryUpdateRequest {
	name?: string | null;
	slug?: string | null;
	code?: string | null;
	description?: string | null;
	parentCategoryId?: string | null;
	displayOrder?: number | null;
	isActive?: boolean | null;
	isVisible?: boolean | null;
	icon?: string | null;
	color?: string | null;
	imageUrl?: string | null;
	metaTitle?: string | null;
	metaDescription?: string | null;
	metaKeywords?: string | null;
}

export interface CategoryListResponse {
	categories: CategoryResponse[];
	pagination: PaginationInfo;
}

export interface CategoryListParams extends PaginationParams {
	parentId?: string | null;
	level?: number | null;
	isActive?: boolean | null;
	isVisible?: boolean | null;
	search?: string | null;
	sortBy?: CategorySortField;
}

export interface CategoryStatsResponse {
	categoryId: string;
	name: string;
	level: number;
	productCount: number;
	totalProductCount: number;
	subcategoryCount: number;
	activeProductCount: number;
	inactiveProductCount: number;
}

export interface BulkCategoryIds {
	categoryIds: string[];
}

export interface BulkOperationResponse {
	success: boolean;
	affectedCount: number;
	message: string;
}

// =============================================================================
// Product Types
// =============================================================================

export interface ProductResponse {
	productId: string;
	tenantId: string;
	sku: string;
	name: string;
	description?: string | null;
	productType: string;
	categoryId?: string | null;
	trackInventory: boolean;
	trackingMethod: ProductTrackingMethod;
	salePrice?: number | null;
	costPrice?: number | null;
	currencyCode: string;
	weightGrams?: number | null;
	dimensions?: Record<string, unknown>;
	attributes?: Record<string, unknown>;
	defaultUomId?: string | null;
	itemGroupId?: string | null;
	isActive: boolean;
	isSellable: boolean;
	isPurchaseable: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface ProductCreateRequest {
	sku: string;
	name: string;
	description?: string | null;
	productType: string;
	categoryId?: string | null;
	trackInventory?: boolean | null;
	trackingMethod?: ProductTrackingMethod | null;
	salePrice?: number | null;
	costPrice?: number | null;
	currencyCode: string;
	weightGrams?: number | null;
	dimensions?: Record<string, unknown>;
	attributes?: Record<string, unknown>;
	defaultUomId?: string | null;
	itemGroupId?: string | null;
	isActive?: boolean | null;
	isSellable?: boolean | null;
	isPurchaseable?: boolean | null;
}

export interface ProductUpdateRequest {
	name?: string | null;
	description?: string | null;
	productType?: string | null;
	categoryId?: string | null;
	trackInventory?: boolean | null;
	trackingMethod?: ProductTrackingMethod | null;
	salePrice?: number | null;
	costPrice?: number | null;
	currencyCode?: string | null;
	weightGrams?: number | null;
	dimensions?: Record<string, unknown>;
	attributes?: Record<string, unknown>;
	defaultUomId?: string | null;
	itemGroupId?: string | null;
	isActive?: boolean | null;
	isSellable?: boolean | null;
	isPurchaseable?: boolean | null;
}

export interface ProductListResponse {
	products: ProductResponse[];
	pagination: PaginationInfo;
}

export interface ProductListParams extends PaginationParams {
	productType?: string | null;
	categoryId?: string | null;
	isActive?: boolean | null;
	isSellable?: boolean | null;
	isPurchaseable?: boolean | null;
	search?: string | null;
}

export interface MoveToCategoryRequest {
	productIds: string[];
	categoryId: string;
}

// =============================================================================
// Variant Types
// =============================================================================

export interface VariantResponse {
	variantId: string;
	tenantId: string;
	parentProductId: string;
	variantAttributes: Record<string, string>;
	sku: string;
	barcode?: string | null;
	priceDifference: number; // BIGINT as cents - delta from parent product
	isActive: boolean;
	createdAt: string;
	updatedAt: string;
	// Joined fields from API
	parentProductName?: string | null;
	parentProductSku?: string | null;
}

export interface VariantCreateRequest {
	parentProductId: string;
	sku: string;
	barcode?: string | null;
	variantAttributes: Record<string, string>;
	priceDifference?: number;
	isActive?: boolean;
}

export interface VariantUpdateRequest {
	sku?: string | null;
	barcode?: string | null;
	variantAttributes?: Record<string, string> | null;
	priceDifference?: number | null;
	isActive?: boolean | null;
}

export interface VariantListResponse {
	variants: VariantResponse[];
	pagination: PaginationInfo;
}

export interface VariantListParams extends PaginationParams {
	parentProductId?: string | null;
	isActive?: boolean | null;
	search?: string | null;
}

export interface BulkVariantIds {
	variantIds: string[];
}

// =============================================================================
// Warehouse Types
// =============================================================================

export interface WarehouseResponse {
	warehouseId: string;
	tenantId: string;
	warehouseCode: string;
	warehouseName: string;
	warehouseType: string;
	description?: string | null;
	parentWarehouseId?: string | null;
	address?: Record<string, unknown>;
	contactInfo?: Record<string, unknown>;
	capacityInfo?: Record<string, unknown>;
	isActive: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreateWarehouseRequest {
	warehouseCode: string;
	warehouseName: string;
	warehouseType: string;
	description?: string | null;
	parentWarehouseId?: string | null;
	address?: Record<string, unknown>;
	contactInfo?: Record<string, unknown>;
	capacityInfo?: Record<string, unknown>;
}

export interface UpdateWarehouseRequest {
	warehouseCode?: string;
	warehouseName?: string;
	warehouseType?: string;
	description?: string | null;
	parentWarehouseId?: string | null;
	address?: Record<string, unknown>;
	contactInfo?: Record<string, unknown>;
	capacityInfo?: Record<string, unknown>;
	isActive?: boolean;
}

export interface WarehouseZoneResponse {
	zoneId: string;
	tenantId: string;
	warehouseId: string;
	zoneCode: string;
	zoneName: string;
	zoneType: string;
	description?: string | null;
	zoneAttributes?: Record<string, unknown>;
	capacityInfo?: Record<string, unknown>;
	isActive: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreateWarehouseZoneRequest {
	zoneCode: string;
	zoneName: string;
	zoneType: string;
	description?: string | null;
	zoneAttributes?: Record<string, unknown>;
	capacityInfo?: Record<string, unknown>;
}

export interface WarehouseLocationResponse {
	locationId: string;
	tenantId: string;
	warehouseId: string;
	zoneId?: string | null;
	locationCode: string;
	locationName?: string | null;
	locationType: string;
	description?: string | null;
	coordinates?: Record<string, unknown>;
	dimensions?: Record<string, unknown>;
	capacityInfo?: Record<string, unknown>;
	locationAttributes?: Record<string, unknown>;
	isActive: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreateWarehouseLocationRequest {
	locationCode: string;
	locationType: string;
	locationName?: string | null;
	description?: string | null;
	zoneId?: string | null;
	coordinates?: Record<string, unknown>;
	dimensions?: Record<string, unknown>;
	capacityInfo?: Record<string, unknown>;
	locationAttributes?: Record<string, unknown>;
}

// =============================================================================
// Receipt (GRN) Types
// =============================================================================

export interface ReceiptItemCreateRequest {
	productId: string;
	expectedQuantity: number;
	receivedQuantity: number;
	unitCost?: number | null;
	uomId?: string | null;
	lotNumber?: string | null;
	serialNumbers?: string[];
	expiryDate?: string | null;
	notes?: string | null;
}

export interface ReceiptCreateRequest {
	warehouseId: string;
	supplierId?: string | null;
	referenceNumber?: string | null;
	expectedDeliveryDate?: string | null;
	currencyCode: string;
	notes?: string | null;
	items: ReceiptItemCreateRequest[];
}

export interface ReceiptItemResponse {
	receiptItemId: string;
	receiptId: string;
	tenantId: string;
	productId: string;
	expectedQuantity: number;
	receivedQuantity: number;
	unitCost?: number | null;
	lineTotal?: number | null;
	uomId?: string | null;
	lotNumber?: string | null;
	serialNumbers?: string[];
	expiryDate?: string | null;
	notes?: string | null;
	createdAt: string;
	updatedAt: string;
}

export interface ReceiptResponse {
	receiptId: string;
	receiptNumber: string;
	tenantId: string;
	warehouseId: string;
	supplierId?: string | null;
	referenceNumber?: string | null;
	status: string;
	receiptDate: string;
	expectedDeliveryDate?: string | null;
	actualDeliveryDate?: string | null;
	createdBy: string;
	currencyCode: string;
	totalQuantity?: number | null;
	totalValue?: number | null;
	notes?: string | null;
	items: ReceiptItemResponse[];
	createdAt: string;
	updatedAt: string;
}

export interface ReceiptSummaryResponse {
	receiptId: string;
	receiptNumber: string;
	warehouseId: string;
	supplierId?: string | null;
	referenceNumber?: string | null;
	status: string;
	receiptDate: string;
	currencyCode: string;
	totalQuantity?: number | null;
	totalValue?: number | null;
	itemCount: number;
	createdAt: string;
}

export interface ReceiptListResponse {
	receipts: ReceiptSummaryResponse[];
	pagination: PaginationInfo;
}

export interface ReceiptListParams extends PaginationParams {
	warehouseId?: string | null;
	supplierId?: string | null;
	status?: string | null;
	search?: string | null;
	createdAfter?: string | null;
	createdBefore?: string | null;
}

// =============================================================================
// Lot/Serial Types
// =============================================================================

export interface LotSerial {
	lotSerialId: string;
	tenantId: string;
	productId: string;
	trackingType: LotSerialTrackingType;
	lotNumber?: string | null;
	serialNumber?: string | null;
	status: LotSerialStatus;
	initialQuantity?: number | null;
	remainingQuantity?: number | null;
	expiryDate?: string | null;
	warehouseId?: string | null;
	locationId?: string | null;
	createdBy: string;
	updatedBy?: string | null;
	createdAt: string;
	updatedAt: string;
	deletedAt?: string | null;
}

export interface CreateLotSerialRequest {
	productId: string;
	trackingType: LotSerialTrackingType;
	lotNumber?: string | null;
	serialNumber?: string | null;
	status: LotSerialStatus;
	initialQuantity?: number | null;
	remainingQuantity?: number | null;
	expiryDate?: string | null;
}

export interface LotSerialLifecycle {
	lotSerial: LotSerial;
	stockMoves: StockMove[];
	qualityChecks: unknown[];
	supplierName?: string | null;
	purchaseOrderNumber?: string | null;
	currentWarehouseName?: string | null;
	currentLocationCode?: string | null;
	coaLink?: string | null;
}

export interface QuarantineResponse {
	quarantinedCount: number;
}

// =============================================================================
// Stock Movement Types
// =============================================================================

export interface StockMove {
	moveId: string;
	tenantId: string;
	productId: string;
	moveType: string;
	quantity: number;
	unitCost?: number | null;
	totalCost?: number | null;
	sourceLocationId?: string | null;
	destinationLocationId?: string | null;
	lotSerialId?: string | null;
	referenceType: string;
	referenceId: string;
	moveReason?: string | null;
	idempotencyKey: string;
	batchInfo?: Record<string, unknown>;
	metadata?: Record<string, unknown>;
	moveDate: string;
	createdAt: string;
}

// =============================================================================
// Transfer Types
// =============================================================================

export interface CreateTransferItemRequest {
	productId: string;
	quantity: number;
	uomId?: string | null;
	lineNumber: number;
	unitCost?: number | null;
	notes?: string | null;
	/** Source zone within source warehouse (optional, for precise tracking) */
	sourceZoneId?: string | null;
	/** Source location/bin within source warehouse (optional) */
	sourceLocationId?: string | null;
	/** Destination zone within destination warehouse (optional) */
	destinationZoneId?: string | null;
	/** Destination location/bin within destination warehouse (optional) */
	destinationLocationId?: string | null;
}

export interface CreateTransferRequest {
	sourceWarehouseId: string;
	destinationWarehouseId: string;
	transferType?: TransferType;
	priority?: TransferPriority;
	referenceNumber?: string | null;
	reason?: string | null;
	expectedShipDate?: string | null;
	expectedReceiveDate?: string | null;
	shippingMethod?: string | null;
	notes?: string | null;
	items: CreateTransferItemRequest[];
}

export interface CreateTransferResponse {
	transferId: string;
	transferNumber: string;
	status: TransferStatus;
	itemsCount: number;
}

export interface ConfirmTransferRequest {
	notes?: string | null;
}

export interface ConfirmTransferResponse {
	transferId: string;
	status: TransferStatus;
	confirmedAt: string;
}

export interface ReceiveTransferRequest {
	notes?: string | null;
}

export interface ReceiveTransferResponse {
	transferId: string;
	status: TransferStatus;
	receivedAt: string;
	stockMovesCreated: number;
}

// =============================================================================
// Reconciliation Types
// =============================================================================

export interface ReconciliationCountItem {
	productId: string;
	warehouseId: string;
	locationId?: string | null;
	countedQuantity: number;
	unitCost?: number | null;
	notes?: string | null;
}

export interface CreateReconciliationRequest {
	name: string;
	cycleType: CycleType;
	warehouseId?: string | null;
	description?: string | null;
	notes?: string | null;
	productFilter?: Record<string, unknown>;
	locationFilter?: Record<string, unknown>;
}

export interface CreateReconciliationResponse {
	reconciliation: StockReconciliation;
}

export interface StockReconciliation {
	reconciliationId: string;
	tenantId: string;
	reconciliationNumber: string;
	name: string;
	status: ReconciliationStatus;
	cycleType: CycleType;
	warehouseId?: string | null;
	description?: string | null;
	notes?: string | null;
	productFilter?: Record<string, unknown>;
	locationFilter?: Record<string, unknown>;
	totalItems: number;
	countedItems: number;
	totalVariance: number;
	createdBy: string;
	approvedBy?: string | null;
	startedAt?: string | null;
	completedAt?: string | null;
	approvedAt?: string | null;
	createdAt: string;
	updatedAt: string;
}

export interface StockReconciliationItem {
	tenantId: string;
	reconciliationId: string;
	productId: string;
	warehouseId: string;
	locationId?: string | null;
	expectedQuantity: number;
	countedQuantity?: number | null;
	variance?: number | null;
	variancePercentage?: number | null;
	varianceValue?: number | null;
	unitCost?: number | null;
	countedBy?: string | null;
	countedAt?: string | null;
	notes?: string | null;
	createdAt: string;
	updatedAt: string;
}

export interface CountReconciliationRequest {
	items: ReconciliationCountItem[];
}

export interface CountReconciliationResponse {
	items: StockReconciliationItem[];
}

export interface ApproveReconciliationRequest {
	notes?: string | null;
}

export interface ApproveReconciliationResponse {
	reconciliation: StockReconciliation;
}

export interface FinalizeReconciliationResponse {
	reconciliation: StockReconciliation;
	adjustments: StockAdjustment[];
}

export interface ReconciliationListResponse {
	reconciliations: StockReconciliation[];
	pagination: PaginationInfo;
}

export interface ReconciliationDetailResponse {
	reconciliation: StockReconciliation;
	items: StockReconciliationItem[];
}

export interface ReconciliationAnalyticsResponse {
	totalReconciliations: number;
	completedReconciliations: number;
	highVarianceItems: number;
	averageVariancePercentage?: number | null;
	totalVarianceValue?: number | null;
	accuracyRate?: number | null;
}

export interface VarianceRange {
	range: string;
	count: number;
	totalVarianceValue?: number | null;
}

export interface VarianceAnalysisResponse {
	reconciliation: StockReconciliation;
	varianceRanges: VarianceRange[];
	topVarianceItems: StockReconciliationItem[];
}

export interface ScanBarcodeRequest {
	barcode: string;
	quantity: number;
	locationId?: string | null;
	notes?: string | null;
}

export interface ScanBarcodeResponse {
	item: StockReconciliationItem;
	isNewCount: boolean;
}

export interface StockAdjustment {
	adjustmentId: string;
	productId: string;
	warehouseId: string;
	quantity: number;
	reason: string;
	adjustedAt: string;
}

// =============================================================================
// RMA Types
// =============================================================================

export interface CreateRmaItemRequest {
	productId: string;
	variantId?: string | null;
	quantityReturned: number;
	condition: RmaCondition;
	action: RmaAction;
	unitCost?: number | null;
	notes?: string | null;
}

export interface CreateRmaRequest {
	customerId: string;
	originalDeliveryId: string;
	returnReason?: string | null;
	notes?: string | null;
	items: CreateRmaItemRequest[];
}

export interface CreateRmaResponse {
	rmaId: string;
	rmaNumber: string;
	status: RmaStatus;
	createdAt: string;
}

export interface ApproveRmaRequest {
	approved: boolean;
	notes?: string | null;
}

export interface ApproveRmaResponse {
	rmaId: string;
	status: RmaStatus;
	approvedAt: string;
}

export interface ReceiveRmaItemRequest {
	rmaItemId: string;
	receivedQuantity: number;
	condition: RmaCondition;
	notes?: string | null;
}

export interface ReceiveRmaRequest {
	receivedItems: ReceiveRmaItemRequest[];
	notes?: string | null;
}

export interface ReceiveRmaResponse {
	rmaId: string;
	status: RmaStatus;
	receivedAt: string;
	stockMovesCreated: number;
}

// =============================================================================
// Quality Control Types
// =============================================================================

export interface QualityControlPoint {
	qcPointId: string;
	tenantId: string;
	name: string;
	qcType: QcPointType;
	productId?: string | null;
	warehouseId?: string | null;
	active: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreateQualityControlPoint {
	name: string;
	type: QcPointType;
	productId?: string | null;
	warehouseId?: string | null;
}

export interface UpdateQualityControlPoint {
	name?: string | null;
	type?: QcPointType | null;
	productId?: string | null;
	warehouseId?: string | null;
	active?: boolean | null;
}

// =============================================================================
// Picking Types
// =============================================================================

export interface PickingTask {
	taskId: string;
	orderId: string;
	productId: string;
	productCode: string;
	productName: string;
	quantity: number;
	locationId: string;
	locationCode: string;
	sequence: number;
	estimatedTimeSeconds?: number | null;
}

export interface PickingMetrics {
	taskCount: number;
	totalDistanceMeters?: number | null;
	totalEstimatedTimeSeconds?: number | null;
	efficiencyScore?: number | null;
	travelTimeReductionPercent?: number | null;
}

export interface PickingOptimizationRequest {
	warehouseId: string;
	orderIds: string[];
	criteria: string[];
	methodId?: string | null;
	constraints?: Record<string, unknown>;
}

export interface PickingPlanResponse {
	planId: string;
	methodId: string;
	methodName: string;
	methodType: string;
	warehouseId: string;
	orderIds: string[];
	tasks: PickingTask[];
	metrics: PickingMetrics;
	generatedAt: string;
}

export interface ConfirmPickingPlanRequest {
	planId: string;
	notes?: string | null;
}

export interface PickingMethodResponse {
	methodId: string;
	tenantId: string;
	name: string;
	methodType: string;
	warehouseId: string;
	description?: string | null;
	config: Record<string, unknown>;
	isActive: boolean;
	isDefault: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreatePickingMethodRequest {
	name: string;
	methodType: string;
	warehouseId: string;
	description?: string | null;
	config: Record<string, unknown>;
	isDefault?: boolean | null;
}

export interface UpdatePickingMethodRequest {
	name?: string | null;
	description?: string | null;
	config?: Record<string, unknown>;
	isActive?: boolean | null;
	isDefault?: boolean | null;
}

// =============================================================================
// Putaway Types
// =============================================================================

export interface PutawaySuggestion {
	locationId: string;
	locationCode: string;
	warehouseId: string;
	zone?: string | null;
	aisle?: string | null;
	rack?: string | null;
	level?: number | null;
	position?: number | null;
	currentStock: number;
	availableCapacity?: number | null;
	score: number;
	ruleApplied?: string | null;
}

export interface PutawayRequest {
	productId: string;
	quantity: number;
	warehouseId?: string | null;
	productCategoryId?: string | null;
	preferredLocationType?: string | null;
	attributes?: Record<string, unknown>;
}

export interface PutawayResponse {
	suggestions: PutawaySuggestion[];
	totalQuantity: number;
	allocatedQuantity: number;
}

export interface PutawayAllocation {
	locationId: string;
	quantity: number;
	unitCost?: number | null;
}

export interface ConfirmPutawayRequest {
	productId: string;
	allocations: PutawayAllocation[];
	referenceType: string;
	referenceId: string;
}

export interface ConfirmPutawayResponse {
	stockMovesCreated: string[];
	totalQuantityPutaway: number;
}

// =============================================================================
// Replenishment Types
// =============================================================================

export interface ReorderRule {
	ruleId: string;
	tenantId: string;
	productId: string;
	warehouseId?: string | null;
	reorderPoint: number;
	minQuantity: number;
	maxQuantity: number;
	leadTimeDays: number;
	safetyStock: number;
	createdAt: string;
	updatedAt: string;
	deletedAt?: string | null;
}

export interface CreateReorderRule {
	productId: string;
	warehouseId?: string | null;
	reorderPoint: number;
	minQuantity: number;
	maxQuantity: number;
	leadTimeDays: number;
	safetyStock: number;
}

export interface UpdateReorderRule {
	reorderPoint?: number | null;
	minQuantity?: number | null;
	maxQuantity?: number | null;
	leadTimeDays?: number | null;
	safetyStock?: number | null;
}

export interface ReplenishmentCheckResult {
	productId: string;
	warehouseId?: string | null;
	currentQuantity: number;
	projectedQuantity: number;
	reorderPoint: number;
	suggestedOrderQuantity: number;
	needsReplenishment: boolean;
	actionTaken?: string | null;
}

// =============================================================================
// Valuation Types
// =============================================================================

export interface ValuationDto {
	valuationId: string;
	tenantId: string;
	productId: string;
	valuationMethod: ValuationMethod;
	totalQuantity: number;
	totalValue: number;
	currentUnitCost?: number | null;
	standardCost?: number | null;
	lastUpdated: string;
}

export interface ValuationLayerDto {
	layerId: string;
	tenantId: string;
	productId: string;
	quantity: number;
	unitCost: number;
	totalValue: number;
	createdAt: string;
}

export interface ValuationLayersResponse {
	layers: ValuationLayerDto[];
}

export interface ValuationHistoryDto {
	historyId: string;
	valuationId: string;
	tenantId: string;
	productId: string;
	valuationMethod: ValuationMethod;
	totalQuantity: number;
	totalValue: number;
	unitCost?: number | null;
	standardCost?: number | null;
	changeReason?: string | null;
	changedAt: string;
}

export interface ValuationHistoryResponse {
	history: ValuationHistoryDto[];
	totalCount: number;
}

export interface SetValuationMethodPayload {
	valuationMethod: ValuationMethod;
}

export interface SetStandardCostPayload {
	standardCost: number;
}

export interface RevaluationPayload {
	newUnitCost: number;
	reason: string;
}

export interface CostAdjustmentPayload {
	adjustmentAmount: number;
	reason: string;
}

// =============================================================================
// Stock Level Types
// =============================================================================

export type StockStatus = 'in_stock' | 'low_stock' | 'out_of_stock';

export interface StockLevelResponse {
	inventoryId: string;
	tenantId: string;
	productId: string;
	productSku: string;
	productName: string;
	warehouseId: string;
	warehouseCode: string;
	warehouseName: string;
	availableQuantity: number;
	reservedQuantity: number;
	totalQuantity: number;
	status: StockStatus;
	reorderPoint?: number | null;
	updatedAt: string;
}

export interface StockLevelSummary {
	totalProducts: number;
	totalAvailableQuantity: number;
	totalReservedQuantity: number;
	lowStockCount: number;
	outOfStockCount: number;
}

export interface StockLevelListResponse {
	items: StockLevelResponse[];
	pagination: PaginationInfo;
	summary: StockLevelSummary;
}

export interface StockLevelListParams extends PaginationParams {
	warehouseId?: string | null;
	productId?: string | null;
	search?: string | null;
	lowStockOnly?: boolean | null;
	outOfStockOnly?: boolean | null;
}

// =============================================================================
// Report Types
// =============================================================================

export interface LowStockQuery {
	warehouseId?: string | null;
}

export interface LowStockEntry {
	productId: string;
	productName: string;
	warehouseId?: string | null;
	warehouseName?: string | null;
	currentStock: number;
	reorderPoint: number;
}

export interface DeadStockQuery {
	warehouseId?: string | null;
	daysThreshold?: number | null;
}

export interface DeadStockEntry {
	productId: string;
	productName: string;
	warehouseId: string;
	warehouseName: string;
	currentStock: number;
	lastOutboundDate?: string | null;
	daysSinceLastOutbound?: number | null;
}

export interface StockAgingQuery {
	warehouseId?: string | null;
}

export interface StockAgingEntry {
	productId: string;
	productName: string;
	warehouseId?: string | null;
	warehouseName?: string | null;
	currentStock: number;
	agingBucket: string;
	daysSinceLastInbound?: number | null;
}

export interface InventoryTurnoverQuery {
	period?: string;
}

export interface InventoryTurnoverEntry {
	productId: string;
	productName: string;
	turnoverRatio: number;
	cogs: number;
	avgInventoryValue: number;
	period: string;
}

export interface StockLedgerQuery {
	productId: string;
	warehouseId?: string | null;
	dateFrom?: string | null;
	dateTo?: string | null;
}

export interface StockLedgerEntry {
	moveId: string;
	moveDate: string;
	referenceType: string;
	referenceId: string;
	description?: string | null;
	quantityIn?: number | null;
	quantityOut?: number | null;
	balance: number;
	unitCost?: number | null;
	totalCost?: number | null;
}

// =============================================================================
// Error Types
// =============================================================================

export interface ErrorResponse {
	error: string;
	code: string;
}

// =============================================================================
// Health Check Types
// =============================================================================

export interface HealthResponse {
	status: string;
	version: string;
	timestamp: string;
	database: string;
	nats: string;
}
