// Pricing Types
// Based on docs/database-erd.dbml and docs/pricing-strategy.md

export type PriceListType = 'sale' | 'purchase';
export type BasedOn = 'fixed' | 'base_price' | 'other_pricelist';
export type ComputeMethod = 'fixed' | 'percentage' | 'formula' | 'margin';
export type ApplyTo = 'product' | 'variant' | 'category' | 'all';
export type RoundingMethod = 'none' | 'round_up' | 'round_down' | 'round_nearest' | 'round_to_99';
export type RuleType =
	| 'discount_percentage'
	| 'discount_amount'
	| 'fixed_price'
	| 'free_item'
	| 'buy_x_get_y'
	| 'bundle_price';
export type ApplyOn = 'line' | 'order';

// =====================================
// Price List Types
// =====================================

export interface PriceList {
	priceListId: string;
	tenantId: string;
	name: string;
	code: string;
	description?: string;
	currencyCode: string;
	priceListType: PriceListType;
	basedOn: BasedOn;
	parentPriceListId?: string;
	defaultPercentage: number;
	validFrom?: Date;
	validTo?: Date;
	priority: number;
	isDefault: boolean;
	isActive: boolean;
	createdAt: Date;
	updatedAt: Date;

	// Computed/Joined
	itemCount?: number;
	customerCount?: number;
	parentPriceList?: { name: string; code: string };
}

export interface PriceListItem {
	itemId: string;
	tenantId: string;
	priceListId: string;
	applyTo: ApplyTo;
	productId?: string;
	variantId?: string;
	categoryId?: string;
	minQuantity: number;
	maxQuantity?: number;
	computeMethod: ComputeMethod;
	fixedPrice?: number;
	percentage?: number;
	marginPercentage?: number;
	formula?: string;
	roundingMethod: RoundingMethod;
	roundingPrecision: number;
	validFrom?: Date;
	validTo?: Date;
	createdAt: Date;
	updatedAt: Date;

	// Joined data for display
	product?: { name: string; sku: string; salePrice: number };
	variant?: { sku: string; attributes: Record<string, string>; priceDifference: number };
	category?: { name: string; code?: string };
}

export interface CustomerPriceList {
	id: string;
	tenantId: string;
	customerId: string;
	priceListId: string;
	priority: number;
	validFrom?: Date;
	validTo?: Date;
	createdAt: Date;

	// Joined
	customer?: { name: string; email: string };
	priceList?: { name: string; code: string };
}

// =====================================
// Pricing Rules Types
// =====================================

export interface RuleConditions {
	// Product targeting
	products?: string[];
	variants?: string[];
	categories?: string[];
	categoryIds?: string[]; // Alias for categories for backward compatibility
	excludeProducts?: string[];

	// Quantity conditions
	minQuantity?: number;
	maxQuantity?: number;

	// Amount conditions
	minOrderAmount?: number;
	maxOrderAmount?: number;

	// Customer targeting
	customerIds?: string[];
	customerGroups?: string[];

	// Time conditions
	weekdays?: number[]; // 0=Sunday, 1=Monday, etc.
	validDays?: (string | number)[]; // Days of week (0-6)
	validHoursStart?: string | number; // Start hour (0-23) or "HH:MM"
	validHoursEnd?: string | number; // End hour (0-23) or "HH:MM"
	timeRange?: {
		start: string; // "09:00"
		end: string; // "17:00"
	};

	// Special conditions
	firstOrderOnly?: boolean;
	newCustomerDays?: number;
}

export interface PricingRule {
	ruleId: string;
	tenantId: string;
	name: string;
	code?: string;
	description?: string;
	ruleType: RuleType;
	conditions: RuleConditions;

	// Discount values
	discountPercentage?: number;
	discountAmount?: number;
	fixedPrice?: number;

	// Free item / Buy X Get Y
	freeProductId?: string;
	freeVariantId?: string;
	freeQuantity?: number;
	buyQuantity?: number;
	getQuantity?: number;

	// Limits
	maxDiscountAmount?: number;
	usageLimit?: number;
	usageCount?: number;
	perCustomerLimit?: number;

	// Validity
	validFrom?: Date;
	validTo?: Date;

	// Priority & combination
	priority: number;
	isCombinable: boolean;
	exclusiveGroup?: string;
	applyOn: ApplyOn;

	isActive: boolean;
	createdAt: Date;
	updatedAt: Date;

	// Joined data
	freeProduct?: { name: string; sku: string };
	freeVariant?: { sku: string; attributes: Record<string, string> };
}

export interface PricingRuleUsage {
	usageId: string;
	tenantId: string;
	ruleId: string;
	customerId?: string;
	orderId?: string;
	discountAmount: number;
	usedAt: Date;
}

// =====================================
// Price Calculation Types
// =====================================

export interface PriceRequest {
	productId: string;
	variantId?: string;
	customerId?: string;
	quantity: number;
	date?: Date;
	currencyCode?: string;
}

export interface PriceDiscount {
	type: 'pricelist' | 'rule';
	id: string;
	name: string;
	description?: string;
	percentage?: number;
	amount: number;
}

export interface PriceResult {
	// Prices
	basePrice: number;
	listPrice: number;
	finalPrice: number;
	unitPrice: number;
	lineTotal: number;

	// Breakdown
	priceListUsed?: {
		id: string;
		name: string;
		code: string;
	};

	discounts: PriceDiscount[];

	// Totals
	totalDiscount: number;
	totalSavings: number;
	savingsPercentage: number;

	// Margin (if cost price available)
	costPrice?: number;
	marginAmount?: number;
	marginPercentage?: number;

	// Meta
	currency: string;
	calculatedAt: Date;
}

export interface BulkPriceRequest {
	items: Array<{
		productId: string;
		variantId?: string;
		quantity: number;
	}>;
	customerId?: string;
	date?: Date;
}

export interface BulkPriceResult {
	items: PriceResult[];
	subtotal: number;
	totalDiscount: number;
	grandTotal: number;
	currency: string;
}

export interface QuantityBreak {
	minQty: number;
	maxQty?: number;
	unitPrice: number;
	discount?: number;
	discountPercentage?: number;
}

export interface ActivePromotion {
	id: string;
	name: string;
	description: string;
	ruleType: RuleType;
	discountPercentage?: number;
	discountAmount?: number;
	validFrom?: Date;
	validTo?: Date;
}

// =====================================
// Price History Types
// =====================================

export interface PriceHistory {
	historyId: string;
	tenantId: string;
	entityType: 'product' | 'variant' | 'price_list_item';
	entityId: string;
	fieldName: string;
	oldValue?: string;
	newValue?: string;
	changeReason?: string;
	changedAt: Date;
	changedBy?: string;
	metadata?: Record<string, unknown>;

	// Joined
	user?: { fullName: string; email: string };
}

// =====================================
// Input Types for API
// =====================================

export interface CreatePriceListInput {
	name: string;
	code: string;
	description?: string;
	currencyCode?: string;
	priceListType?: PriceListType;
	basedOn?: BasedOn;
	parentPriceListId?: string;
	defaultPercentage?: number;
	validFrom?: Date;
	validTo?: Date;
	priority?: number;
	isDefault?: boolean;
	isActive?: boolean;
}

export interface UpdatePriceListInput extends Partial<CreatePriceListInput> {}

export interface CreatePriceListItemInput {
	applyTo: ApplyTo;
	productId?: string;
	variantId?: string;
	categoryId?: string;
	minQuantity?: number;
	maxQuantity?: number;
	computeMethod: ComputeMethod;
	fixedPrice?: number;
	percentage?: number;
	marginPercentage?: number;
	formula?: string;
	roundingMethod?: RoundingMethod;
	roundingPrecision?: number;
	validFrom?: Date;
	validTo?: Date;
}

export interface UpdatePriceListItemInput extends Partial<CreatePriceListItemInput> {}

export interface CreatePricingRuleInput {
	name: string;
	code?: string;
	description?: string;
	ruleType: RuleType;
	conditions?: RuleConditions;
	discountPercentage?: number;
	discountAmount?: number;
	fixedPrice?: number;
	freeProductId?: string;
	freeVariantId?: string;
	freeQuantity?: number;
	buyQuantity?: number;
	getQuantity?: number;
	maxDiscountAmount?: number;
	usageLimit?: number;
	perCustomerLimit?: number;
	validFrom?: Date;
	validTo?: Date;
	priority?: number;
	isCombinable?: boolean;
	exclusiveGroup?: string;
	applyOn?: ApplyOn;
	isActive?: boolean;
}

export interface UpdatePricingRuleInput extends Partial<CreatePricingRuleInput> {}

export interface AssignCustomerInput {
	priority?: number;
	validFrom?: Date;
	validTo?: Date;
}

// =====================================
// List/Filter Types
// =====================================

export interface PriceListFilters {
	search?: string;
	priceListType?: PriceListType;
	isActive?: boolean;
	currencyCode?: string;
}

export interface PricingRuleFilters {
	search?: string;
	ruleType?: RuleType;
	isActive?: boolean;
	status?: 'active' | 'scheduled' | 'expired' | 'inactive';
}

// =====================================
// Mock Data for Development
// =====================================

export const mockPriceLists: PriceList[] = [
	{
		priceListId: 'pl-001',
		tenantId: 'tenant-001',
		name: 'Retail Price',
		code: 'RETAIL',
		description: 'Standard retail pricing for all customers',
		currencyCode: 'VND',
		priceListType: 'sale',
		basedOn: 'fixed',
		defaultPercentage: 0,
		priority: 100,
		isDefault: true,
		isActive: true,
		createdAt: new Date('2026-01-01'),
		updatedAt: new Date('2026-01-20'),
		itemCount: 150,
		customerCount: 0
	},
	{
		priceListId: 'pl-002',
		tenantId: 'tenant-001',
		name: 'Wholesale',
		code: 'WHOLESALE',
		description: 'Special pricing for wholesale customers',
		currencyCode: 'VND',
		priceListType: 'sale',
		basedOn: 'base_price',
		defaultPercentage: -15,
		priority: 50,
		isDefault: false,
		isActive: true,
		createdAt: new Date('2026-01-05'),
		updatedAt: new Date('2026-01-18'),
		itemCount: 150,
		customerCount: 12
	},
	{
		priceListId: 'pl-003',
		tenantId: 'tenant-001',
		name: 'VIP Members',
		code: 'VIP',
		description: 'Exclusive pricing for VIP members',
		currencyCode: 'VND',
		priceListType: 'sale',
		basedOn: 'base_price',
		defaultPercentage: -20,
		priority: 30,
		isDefault: false,
		isActive: true,
		createdAt: new Date('2026-01-10'),
		updatedAt: new Date('2026-01-22'),
		itemCount: 45,
		customerCount: 25
	},
	{
		priceListId: 'pl-004',
		tenantId: 'tenant-001',
		name: 'Tet 2026',
		code: 'TET2026',
		description: 'Seasonal promotion for Tet holiday',
		currencyCode: 'VND',
		priceListType: 'sale',
		basedOn: 'base_price',
		defaultPercentage: -10,
		validFrom: new Date('2026-01-25'),
		validTo: new Date('2026-02-10'),
		priority: 20,
		isDefault: false,
		isActive: true,
		createdAt: new Date('2026-01-15'),
		updatedAt: new Date('2026-01-23'),
		itemCount: 30,
		customerCount: 0
	},
	{
		priceListId: 'pl-005',
		tenantId: 'tenant-001',
		name: 'Supplier A',
		code: 'SUPP-A',
		description: 'Purchase prices from Supplier A',
		currencyCode: 'VND',
		priceListType: 'purchase',
		basedOn: 'fixed',
		defaultPercentage: 0,
		priority: 100,
		isDefault: false,
		isActive: true,
		createdAt: new Date('2026-01-02'),
		updatedAt: new Date('2026-01-19'),
		itemCount: 80,
		customerCount: 0
	}
];

export const mockPricingRules: PricingRule[] = [
	{
		ruleId: 'rule-001',
		tenantId: 'tenant-001',
		name: 'Buy 3 Get 10% Off',
		code: 'BUY3-10OFF',
		description: 'Buy 3 or more items and get 10% discount',
		ruleType: 'discount_percentage',
		conditions: {
			minQuantity: 3
		},
		discountPercentage: 10,
		freeQuantity: 0,
		usageCount: 156,
		priority: 100,
		isCombinable: true,
		applyOn: 'line',
		isActive: true,
		createdAt: new Date('2026-01-01'),
		updatedAt: new Date('2026-01-20')
	},
	{
		ruleId: 'rule-002',
		tenantId: 'tenant-001',
		name: 'Electronics 15% Off',
		code: 'ELEC15',
		description: '15% discount on all electronics',
		ruleType: 'discount_percentage',
		conditions: {
			categories: ['cat-electronics']
		},
		discountPercentage: 15,
		freeQuantity: 0,
		usageCount: 89,
		validFrom: new Date('2026-01-15'),
		validTo: new Date('2026-02-15'),
		priority: 50,
		isCombinable: false,
		applyOn: 'line',
		isActive: true,
		createdAt: new Date('2026-01-10'),
		updatedAt: new Date('2026-01-18')
	},
	{
		ruleId: 'rule-003',
		tenantId: 'tenant-001',
		name: 'Free Shipping Over 500K',
		code: 'FREESHIP500',
		description: 'Free shipping for orders over 500,000 VND',
		ruleType: 'discount_amount',
		conditions: {
			minOrderAmount: 50000000 // 500,000 VND in cents
		},
		discountAmount: 3000000, // 30,000 VND shipping fee
		freeQuantity: 0,
		usageCount: 234,
		priority: 200,
		isCombinable: true,
		applyOn: 'order',
		isActive: true,
		createdAt: new Date('2026-01-01'),
		updatedAt: new Date('2026-01-22')
	},
	{
		ruleId: 'rule-004',
		tenantId: 'tenant-001',
		name: 'First Order 20% Off',
		code: 'FIRST20',
		description: '20% discount for first-time customers',
		ruleType: 'discount_percentage',
		conditions: {
			firstOrderOnly: true
		},
		discountPercentage: 20,
		maxDiscountAmount: 20000000, // Max 200,000 VND
		freeQuantity: 0,
		usageCount: 45,
		priority: 10,
		isCombinable: false,
		applyOn: 'order',
		isActive: true,
		createdAt: new Date('2026-01-05'),
		updatedAt: new Date('2026-01-21')
	},
	{
		ruleId: 'rule-005',
		tenantId: 'tenant-001',
		name: 'Buy 2 Get 1 Free',
		code: 'B2G1',
		description: 'Buy 2 items, get 1 free (accessories only)',
		ruleType: 'buy_x_get_y',
		conditions: {
			categories: ['cat-accessories']
		},
		buyQuantity: 2,
		getQuantity: 1,
		freeQuantity: 1,
		usageCount: 67,
		priority: 40,
		isCombinable: false,
		applyOn: 'line',
		isActive: false,
		createdAt: new Date('2025-12-01'),
		updatedAt: new Date('2026-01-10')
	}
];
