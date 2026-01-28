/**
 * Product Types - Based on docs/database-erd.dbml
 *
 * Tables: products, product_variants, unit_of_measures, uom_conversions
 */

// ============================================================
// PRODUCT TYPES
// ============================================================

export type ProductType = 'goods' | 'service' | 'consumable';
export type TrackingMethod = 'none' | 'lot' | 'serial';
export type ProductStatus = 'active' | 'inactive' | 'archived';

/**
 * Product entity matching database schema
 */
export interface Product {
	productId: string;
	tenantId: string;
	sku: string;
	name: string;
	description?: string;
	productType: ProductType;
	categoryId?: string;
	itemGroupId?: string;
	trackInventory: boolean;
	trackingMethod: TrackingMethod;
	defaultUomId?: string;
	defaultUom?: UnitOfMeasure;
	salePrice: number; // cents/xu
	costPrice?: number; // cents/xu
	currencyCode: string;
	weightGrams?: number;
	dimensions?: ProductDimensions;
	attributes?: Record<string, unknown>;
	isActive: boolean;
	isSellable: boolean;
	isPurchaseable: boolean;
	createdAt: string;
	updatedAt: string;
	deletedAt?: string;
	// Computed/joined fields
	variantCount?: number;
	categoryName?: string;
}

export interface ProductDimensions {
	lengthMm?: number;
	widthMm?: number;
	heightMm?: number;
}

/**
 * Product variant matching database schema
 */
export interface ProductVariant {
	variantId: string;
	tenantId: string;
	parentProductId: string;
	variantAttributes: Record<string, string>; // e.g., {color: "red", size: "L"}
	sku: string;
	barcode?: string;
	priceDifference: number; // cents - delta from parent product
	isActive: boolean;
	createdAt: string;
	updatedAt: string;
	deletedAt?: string;
}

// ============================================================
// UNIT OF MEASURE TYPES
// ============================================================

export type UomType = 'unit' | 'weight' | 'length' | 'volume' | 'time';

/**
 * Unit of Measure matching database schema
 */
export interface UnitOfMeasure {
	uomId: string;
	tenantId: string;
	uomCode: string; // PC, KG, M, etc.
	uomName: string;
	uomType: UomType;
	isBaseUnit: boolean;
	baseUomId?: string;
	conversionFactor?: number;
	isActive: boolean;
	createdAt: string;
	updatedAt: string;
	deletedAt?: string;
}

/**
 * UOM Conversion matching database schema
 */
export interface UomConversion {
	conversionId: string;
	tenantId: string;
	fromUomId: string;
	toUomId: string;
	conversionFactor: number;
	isActive: boolean;
	createdAt: string;
	updatedAt: string;
}

// ============================================================
// API REQUEST/RESPONSE TYPES
// ============================================================

export interface ProductListParams {
	page?: number;
	limit?: number;
	search?: string;
	productType?: ProductType;
	categoryId?: string;
	isActive?: boolean;
	trackingMethod?: TrackingMethod;
	sortBy?: 'name' | 'sku' | 'salePrice' | 'createdAt' | 'updatedAt';
	sortOrder?: 'asc' | 'desc';
}

export interface ProductListResponse {
	data: Product[];
	meta: {
		page: number;
		limit: number;
		total: number;
		totalPages: number;
	};
}

export interface CreateProductRequest {
	sku: string;
	name: string;
	description?: string;
	productType: ProductType;
	categoryId?: string;
	trackInventory?: boolean;
	trackingMethod?: TrackingMethod;
	defaultUomId?: string;
	salePrice: number;
	costPrice?: number;
	currencyCode?: string;
	weightGrams?: number;
	dimensions?: ProductDimensions;
	attributes?: Record<string, unknown>;
	isActive?: boolean;
	isSellable?: boolean;
	isPurchaseable?: boolean;
}

export interface UpdateProductRequest extends Partial<CreateProductRequest> {
	// All fields optional for updates
}

export interface CreateVariantRequest {
	sku: string;
	barcode?: string;
	variantAttributes: Record<string, string>;
	priceDifference?: number;
	isActive?: boolean;
}

export interface UpdateVariantRequest extends Partial<CreateVariantRequest> {
	// All fields optional for updates
}

// ============================================================
// FORM TYPES
// ============================================================

export interface ProductFormData {
	sku: string;
	name: string;
	description: string;
	productType: ProductType;
	categoryId: string;
	trackInventory: boolean;
	trackingMethod: TrackingMethod;
	defaultUomId: string;
	salePrice: number;
	costPrice: number;
	currencyCode: string;
	weightGrams: number;
	lengthMm: number;
	widthMm: number;
	heightMm: number;
	isActive: boolean;
	isSellable: boolean;
	isPurchaseable: boolean;
}

export interface VariantFormData {
	sku: string;
	barcode: string;
	priceDifference: number;
	isActive: boolean;
	attributes: { key: string; value: string }[];
}

// ============================================================
// MOCK DATA
// ============================================================

export const mockUoms: UnitOfMeasure[] = [
	{
		uomId: 'uom-1',
		tenantId: 'tenant-1',
		uomCode: 'PC',
		uomName: 'Piece',
		uomType: 'unit',
		isBaseUnit: true,
		isActive: true,
		createdAt: '2026-01-01T00:00:00Z',
		updatedAt: '2026-01-01T00:00:00Z'
	},
	{
		uomId: 'uom-2',
		tenantId: 'tenant-1',
		uomCode: 'BOX',
		uomName: 'Box',
		uomType: 'unit',
		isBaseUnit: false,
		baseUomId: 'uom-1',
		conversionFactor: 12,
		isActive: true,
		createdAt: '2026-01-01T00:00:00Z',
		updatedAt: '2026-01-01T00:00:00Z'
	},
	{
		uomId: 'uom-3',
		tenantId: 'tenant-1',
		uomCode: 'KG',
		uomName: 'Kilogram',
		uomType: 'weight',
		isBaseUnit: true,
		isActive: true,
		createdAt: '2026-01-01T00:00:00Z',
		updatedAt: '2026-01-01T00:00:00Z'
	},
	{
		uomId: 'uom-4',
		tenantId: 'tenant-1',
		uomCode: 'M',
		uomName: 'Meter',
		uomType: 'length',
		isBaseUnit: true,
		isActive: true,
		createdAt: '2026-01-01T00:00:00Z',
		updatedAt: '2026-01-01T00:00:00Z'
	}
];

export const mockProducts: Product[] = [
	{
		productId: 'prod-1',
		tenantId: 'tenant-1',
		sku: 'LAPTOP-001',
		name: 'Laptop Pro 15"',
		description: 'High-performance laptop with 16GB RAM',
		productType: 'goods',
		trackInventory: true,
		trackingMethod: 'serial',
		defaultUomId: 'uom-1',
		salePrice: 2500000000, // 25,000,000 VND
		costPrice: 2000000000,
		currencyCode: 'VND',
		weightGrams: 1800,
		dimensions: { lengthMm: 350, widthMm: 240, heightMm: 18 },
		isActive: true,
		isSellable: true,
		isPurchaseable: true,
		createdAt: '2026-01-15T10:00:00Z',
		updatedAt: '2026-01-20T14:30:00Z',
		variantCount: 3
	},
	{
		productId: 'prod-2',
		tenantId: 'tenant-1',
		sku: 'MOUSE-001',
		name: 'Wireless Mouse',
		description: 'Ergonomic wireless mouse with Bluetooth',
		productType: 'goods',
		trackInventory: true,
		trackingMethod: 'none',
		defaultUomId: 'uom-1',
		salePrice: 50000000, // 500,000 VND
		costPrice: 30000000,
		currencyCode: 'VND',
		weightGrams: 85,
		isActive: true,
		isSellable: true,
		isPurchaseable: true,
		createdAt: '2026-01-10T09:00:00Z',
		updatedAt: '2026-01-18T11:00:00Z',
		variantCount: 0
	},
	{
		productId: 'prod-3',
		tenantId: 'tenant-1',
		sku: 'CHAIR-001',
		name: 'Office Chair Premium',
		description: 'Ergonomic office chair with lumbar support',
		productType: 'goods',
		trackInventory: true,
		trackingMethod: 'lot',
		defaultUomId: 'uom-1',
		salePrice: 350000000, // 3,500,000 VND
		costPrice: 200000000,
		currencyCode: 'VND',
		weightGrams: 15000,
		dimensions: { lengthMm: 650, widthMm: 650, heightMm: 1200 },
		isActive: true,
		isSellable: true,
		isPurchaseable: true,
		createdAt: '2026-01-05T08:00:00Z',
		updatedAt: '2026-01-22T09:00:00Z',
		variantCount: 2
	},
	{
		productId: 'prod-4',
		tenantId: 'tenant-1',
		sku: 'WARRANTY-1Y',
		name: 'Extended Warranty - 1 Year',
		description: 'One year extended warranty service',
		productType: 'service',
		trackInventory: false,
		trackingMethod: 'none',
		defaultUomId: 'uom-1',
		salePrice: 100000000, // 1,000,000 VND
		currencyCode: 'VND',
		isActive: true,
		isSellable: true,
		isPurchaseable: false,
		createdAt: '2026-01-01T00:00:00Z',
		updatedAt: '2026-01-01T00:00:00Z',
		variantCount: 0
	},
	{
		productId: 'prod-5',
		tenantId: 'tenant-1',
		sku: 'PAPER-A4',
		name: 'A4 Paper (Ream)',
		description: 'Standard A4 paper, 500 sheets per ream',
		productType: 'consumable',
		trackInventory: true,
		trackingMethod: 'none',
		defaultUomId: 'uom-1',
		salePrice: 8000000, // 80,000 VND
		costPrice: 5000000,
		currencyCode: 'VND',
		weightGrams: 2500,
		isActive: false,
		isSellable: true,
		isPurchaseable: true,
		createdAt: '2026-01-02T00:00:00Z',
		updatedAt: '2026-01-15T00:00:00Z',
		variantCount: 0
	}
];

export const mockVariants: ProductVariant[] = [
	{
		variantId: 'var-1',
		tenantId: 'tenant-1',
		parentProductId: 'prod-1',
		sku: 'LAPTOP-001-BLK-256',
		variantAttributes: { color: 'Black', storage: '256GB' },
		priceDifference: 0,
		isActive: true,
		createdAt: '2026-01-15T10:00:00Z',
		updatedAt: '2026-01-15T10:00:00Z'
	},
	{
		variantId: 'var-2',
		tenantId: 'tenant-1',
		parentProductId: 'prod-1',
		sku: 'LAPTOP-001-BLK-512',
		variantAttributes: { color: 'Black', storage: '512GB' },
		priceDifference: 300000000, // +3,000,000 VND
		isActive: true,
		createdAt: '2026-01-15T10:00:00Z',
		updatedAt: '2026-01-15T10:00:00Z'
	},
	{
		variantId: 'var-3',
		tenantId: 'tenant-1',
		parentProductId: 'prod-1',
		sku: 'LAPTOP-001-SLV-256',
		variantAttributes: { color: 'Silver', storage: '256GB' },
		priceDifference: 50000000, // +500,000 VND
		isActive: true,
		createdAt: '2026-01-15T10:00:00Z',
		updatedAt: '2026-01-15T10:00:00Z'
	}
];
