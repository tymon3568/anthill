// =============================================================================
// Product Types Tests
// Tests for types/products.ts - type validation and mock data integrity
// =============================================================================

import { describe, it, expect } from 'vitest';
import {
	mockProducts,
	mockVariants,
	mockUoms,
	type Product,
	type ProductVariant,
	type UnitOfMeasure,
	type ProductType,
	type TrackingMethod,
	type ProductStatus,
	type UomType,
	type ProductDimensions,
	type ProductListParams,
	type ProductListResponse,
	type CreateProductRequest,
	type UpdateProductRequest,
	type CreateVariantRequest,
	type UpdateVariantRequest,
	type ProductFormData,
	type VariantFormData,
	type UomConversion
} from './products';

// =============================================================================
// Mock Data Validation Tests
// =============================================================================

describe('mockProducts', () => {
	it('should have valid products array', () => {
		expect(mockProducts).toBeDefined();
		expect(Array.isArray(mockProducts)).toBe(true);
		expect(mockProducts.length).toBeGreaterThan(0);
	});

	it('should have all required fields for each product', () => {
		for (const product of mockProducts) {
			expect(product.productId).toBeDefined();
			expect(typeof product.productId).toBe('string');
			expect(product.tenantId).toBeDefined();
			expect(typeof product.tenantId).toBe('string');
			expect(product.sku).toBeDefined();
			expect(typeof product.sku).toBe('string');
			expect(product.name).toBeDefined();
			expect(typeof product.name).toBe('string');
			expect(product.productType).toBeDefined();
			expect(['goods', 'service', 'consumable']).toContain(product.productType);
			expect(typeof product.trackInventory).toBe('boolean');
			expect(['none', 'lot', 'serial']).toContain(product.trackingMethod);
			expect(typeof product.salePrice).toBe('number');
			expect(product.currencyCode).toBeDefined();
			expect(typeof product.isActive).toBe('boolean');
			expect(typeof product.isSellable).toBe('boolean');
			expect(typeof product.isPurchaseable).toBe('boolean');
			expect(product.createdAt).toBeDefined();
			expect(product.updatedAt).toBeDefined();
		}
	});

	it('should have unique product IDs', () => {
		const productIds = mockProducts.map((p) => p.productId);
		const uniqueIds = new Set(productIds);
		expect(uniqueIds.size).toBe(productIds.length);
	});

	it('should have unique SKUs', () => {
		const skus = mockProducts.map((p) => p.sku);
		const uniqueSkus = new Set(skus);
		expect(uniqueSkus.size).toBe(skus.length);
	});

	it('should have valid product types', () => {
		const validTypes: ProductType[] = ['goods', 'service', 'consumable'];
		for (const product of mockProducts) {
			expect(validTypes).toContain(product.productType);
		}
	});

	it('should have valid tracking methods', () => {
		const validMethods: TrackingMethod[] = ['none', 'lot', 'serial'];
		for (const product of mockProducts) {
			expect(validMethods).toContain(product.trackingMethod);
		}
	});

	it('should have non-negative prices', () => {
		for (const product of mockProducts) {
			expect(product.salePrice).toBeGreaterThanOrEqual(0);
			if (product.costPrice !== undefined) {
				expect(product.costPrice).toBeGreaterThanOrEqual(0);
			}
		}
	});

	it('should have valid ISO date strings', () => {
		const isoDateRegex = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/;
		for (const product of mockProducts) {
			expect(product.createdAt).toMatch(isoDateRegex);
			expect(product.updatedAt).toMatch(isoDateRegex);
		}
	});

	it('should have at least one product of each type', () => {
		const types = mockProducts.map((p) => p.productType);
		expect(types).toContain('goods');
		expect(types).toContain('service');
		expect(types).toContain('consumable');
	});

	it('should have products with valid dimensions when present', () => {
		const productsWithDimensions = mockProducts.filter((p) => p.dimensions !== undefined);
		expect(productsWithDimensions.length).toBeGreaterThan(0);

		for (const product of productsWithDimensions) {
			const dims = product.dimensions!;
			if (dims.lengthMm !== undefined) {
				expect(dims.lengthMm).toBeGreaterThan(0);
			}
			if (dims.widthMm !== undefined) {
				expect(dims.widthMm).toBeGreaterThan(0);
			}
			if (dims.heightMm !== undefined) {
				expect(dims.heightMm).toBeGreaterThan(0);
			}
		}
	});
});

describe('mockVariants', () => {
	it('should have valid variants array', () => {
		expect(mockVariants).toBeDefined();
		expect(Array.isArray(mockVariants)).toBe(true);
		expect(mockVariants.length).toBeGreaterThan(0);
	});

	it('should have all required fields for each variant', () => {
		for (const variant of mockVariants) {
			expect(variant.variantId).toBeDefined();
			expect(typeof variant.variantId).toBe('string');
			expect(variant.tenantId).toBeDefined();
			expect(typeof variant.tenantId).toBe('string');
			expect(variant.parentProductId).toBeDefined();
			expect(typeof variant.parentProductId).toBe('string');
			expect(variant.sku).toBeDefined();
			expect(typeof variant.sku).toBe('string');
			expect(variant.variantAttributes).toBeDefined();
			expect(typeof variant.variantAttributes).toBe('object');
			expect(typeof variant.priceDifference).toBe('number');
			expect(typeof variant.isActive).toBe('boolean');
			expect(variant.createdAt).toBeDefined();
			expect(variant.updatedAt).toBeDefined();
		}
	});

	it('should have unique variant IDs', () => {
		const variantIds = mockVariants.map((v) => v.variantId);
		const uniqueIds = new Set(variantIds);
		expect(uniqueIds.size).toBe(variantIds.length);
	});

	it('should have unique SKUs', () => {
		const skus = mockVariants.map((v) => v.sku);
		const uniqueSkus = new Set(skus);
		expect(uniqueSkus.size).toBe(skus.length);
	});

	it('should reference existing parent products', () => {
		const productIds = mockProducts.map((p) => p.productId);
		for (const variant of mockVariants) {
			expect(productIds).toContain(variant.parentProductId);
		}
	});

	it('should have non-empty variant attributes', () => {
		for (const variant of mockVariants) {
			const attrKeys = Object.keys(variant.variantAttributes);
			expect(attrKeys.length).toBeGreaterThan(0);
		}
	});

	it('should have valid ISO date strings', () => {
		const isoDateRegex = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/;
		for (const variant of mockVariants) {
			expect(variant.createdAt).toMatch(isoDateRegex);
			expect(variant.updatedAt).toMatch(isoDateRegex);
		}
	});
});

describe('mockUoms', () => {
	it('should have valid UOMs array', () => {
		expect(mockUoms).toBeDefined();
		expect(Array.isArray(mockUoms)).toBe(true);
		expect(mockUoms.length).toBeGreaterThan(0);
	});

	it('should have all required fields for each UOM', () => {
		for (const uom of mockUoms) {
			expect(uom.uomId).toBeDefined();
			expect(typeof uom.uomId).toBe('string');
			expect(uom.tenantId).toBeDefined();
			expect(typeof uom.tenantId).toBe('string');
			expect(uom.uomCode).toBeDefined();
			expect(typeof uom.uomCode).toBe('string');
			expect(uom.uomName).toBeDefined();
			expect(typeof uom.uomName).toBe('string');
			expect(uom.uomType).toBeDefined();
			expect(['unit', 'weight', 'length', 'volume', 'time']).toContain(uom.uomType);
			expect(typeof uom.isBaseUnit).toBe('boolean');
			expect(typeof uom.isActive).toBe('boolean');
			expect(uom.createdAt).toBeDefined();
			expect(uom.updatedAt).toBeDefined();
		}
	});

	it('should have unique UOM IDs', () => {
		const uomIds = mockUoms.map((u) => u.uomId);
		const uniqueIds = new Set(uomIds);
		expect(uniqueIds.size).toBe(uomIds.length);
	});

	it('should have unique UOM codes', () => {
		const codes = mockUoms.map((u) => u.uomCode);
		const uniqueCodes = new Set(codes);
		expect(uniqueCodes.size).toBe(codes.length);
	});

	it('should have valid UOM types', () => {
		const validTypes: UomType[] = ['unit', 'weight', 'length', 'volume', 'time'];
		for (const uom of mockUoms) {
			expect(validTypes).toContain(uom.uomType);
		}
	});

	it('should have at least one base unit', () => {
		const baseUnits = mockUoms.filter((u) => u.isBaseUnit);
		expect(baseUnits.length).toBeGreaterThan(0);
	});

	it('should have non-base units with conversion factors', () => {
		const nonBaseUnits = mockUoms.filter((u) => !u.isBaseUnit);
		for (const uom of nonBaseUnits) {
			expect(uom.baseUomId).toBeDefined();
			expect(uom.conversionFactor).toBeDefined();
			expect(uom.conversionFactor).toBeGreaterThan(0);
		}
	});

	it('should reference existing base UOMs', () => {
		const uomIds = mockUoms.map((u) => u.uomId);
		const nonBaseUnits = mockUoms.filter((u) => !u.isBaseUnit);
		for (const uom of nonBaseUnits) {
			expect(uomIds).toContain(uom.baseUomId);
		}
	});
});

// =============================================================================
// Type Structure Tests
// =============================================================================

describe('Type structures', () => {
	describe('Product interface', () => {
		it('should accept valid product object', () => {
			const product: Product = {
				productId: 'test-1',
				tenantId: 'tenant-1',
				sku: 'TEST-SKU',
				name: 'Test Product',
				productType: 'goods',
				trackInventory: true,
				trackingMethod: 'lot',
				salePrice: 100000,
				currencyCode: 'VND',
				isActive: true,
				isSellable: true,
				isPurchaseable: true,
				createdAt: '2026-01-01T00:00:00Z',
				updatedAt: '2026-01-01T00:00:00Z'
			};
			expect(product).toBeDefined();
			expect(product.productId).toBe('test-1');
		});

		it('should accept product with all optional fields', () => {
			const product: Product = {
				productId: 'test-2',
				tenantId: 'tenant-1',
				sku: 'TEST-SKU-2',
				name: 'Full Product',
				description: 'A full description',
				productType: 'goods',
				itemGroupId: 'group-1',
				trackInventory: true,
				trackingMethod: 'serial',
				defaultUomId: 'uom-1',
				defaultUom: mockUoms[0],
				salePrice: 200000,
				costPrice: 100000,
				currencyCode: 'VND',
				weightGrams: 500,
				dimensions: { lengthMm: 100, widthMm: 50, heightMm: 25 },
				attributes: { color: 'red' },
				isActive: true,
				isSellable: true,
				isPurchaseable: true,
				createdAt: '2026-01-01T00:00:00Z',
				updatedAt: '2026-01-01T00:00:00Z',
				deletedAt: undefined,
				variantCount: 5,
				categoryName: 'Electronics'
			};
			expect(product).toBeDefined();
			expect(product.description).toBe('A full description');
			expect(product.dimensions?.lengthMm).toBe(100);
		});
	});

	describe('ProductVariant interface', () => {
		it('should accept valid variant object', () => {
			const variant: ProductVariant = {
				variantId: 'var-1',
				tenantId: 'tenant-1',
				parentProductId: 'prod-1',
				variantAttributes: { color: 'red', size: 'M' },
				sku: 'VAR-SKU-1',
				priceDifference: 50000,
				isActive: true,
				createdAt: '2026-01-01T00:00:00Z',
				updatedAt: '2026-01-01T00:00:00Z'
			};
			expect(variant).toBeDefined();
			expect(variant.variantAttributes.color).toBe('red');
		});

		it('should accept variant with optional barcode', () => {
			const variant: ProductVariant = {
				variantId: 'var-2',
				tenantId: 'tenant-1',
				parentProductId: 'prod-1',
				variantAttributes: { size: 'L' },
				sku: 'VAR-SKU-2',
				barcode: '1234567890123',
				priceDifference: 0,
				isActive: true,
				createdAt: '2026-01-01T00:00:00Z',
				updatedAt: '2026-01-01T00:00:00Z'
			};
			expect(variant.barcode).toBe('1234567890123');
		});
	});

	describe('UnitOfMeasure interface', () => {
		it('should accept valid base unit', () => {
			const uom: UnitOfMeasure = {
				uomId: 'uom-test-1',
				tenantId: 'tenant-1',
				uomCode: 'EA',
				uomName: 'Each',
				uomType: 'unit',
				isBaseUnit: true,
				isActive: true,
				createdAt: '2026-01-01T00:00:00Z',
				updatedAt: '2026-01-01T00:00:00Z'
			};
			expect(uom).toBeDefined();
			expect(uom.isBaseUnit).toBe(true);
		});

		it('should accept valid derived unit with conversion', () => {
			const uom: UnitOfMeasure = {
				uomId: 'uom-test-2',
				tenantId: 'tenant-1',
				uomCode: 'DZ',
				uomName: 'Dozen',
				uomType: 'unit',
				isBaseUnit: false,
				baseUomId: 'uom-test-1',
				conversionFactor: 12,
				isActive: true,
				createdAt: '2026-01-01T00:00:00Z',
				updatedAt: '2026-01-01T00:00:00Z'
			};
			expect(uom.isBaseUnit).toBe(false);
			expect(uom.conversionFactor).toBe(12);
		});
	});

	describe('UomConversion interface', () => {
		it('should accept valid conversion object', () => {
			const conversion: UomConversion = {
				conversionId: 'conv-1',
				tenantId: 'tenant-1',
				fromUomId: 'uom-1',
				toUomId: 'uom-2',
				conversionFactor: 12,
				isActive: true,
				createdAt: '2026-01-01T00:00:00Z',
				updatedAt: '2026-01-01T00:00:00Z'
			};
			expect(conversion).toBeDefined();
			expect(conversion.conversionFactor).toBe(12);
		});
	});

	describe('ProductDimensions interface', () => {
		it('should accept full dimensions', () => {
			const dims: ProductDimensions = {
				lengthMm: 100,
				widthMm: 50,
				heightMm: 25
			};
			expect(dims.lengthMm).toBe(100);
		});

		it('should accept partial dimensions', () => {
			const dims: ProductDimensions = {
				lengthMm: 100
			};
			expect(dims.lengthMm).toBe(100);
			expect(dims.widthMm).toBeUndefined();
		});

		it('should accept empty dimensions', () => {
			const dims: ProductDimensions = {};
			expect(dims).toBeDefined();
		});
	});
});

// =============================================================================
// Request/Response Type Tests
// =============================================================================

describe('API Request/Response types', () => {
	describe('ProductListParams', () => {
		it('should accept empty params', () => {
			const params: ProductListParams = {};
			expect(params).toBeDefined();
		});

		it('should accept full params', () => {
			const params: ProductListParams = {
				page: 1,
				limit: 20,
				search: 'laptop',
				productType: 'goods',
				isActive: true,
				trackingMethod: 'serial',
				sortBy: 'name',
				sortOrder: 'asc'
			};
			expect(params.page).toBe(1);
			expect(params.sortBy).toBe('name');
		});
	});

	describe('ProductListResponse', () => {
		it('should accept valid response', () => {
			const response: ProductListResponse = {
				data: mockProducts,
				meta: {
					page: 1,
					limit: 20,
					total: 100,
					totalPages: 5
				}
			};
			expect(response.data.length).toBeGreaterThan(0);
			expect(response.meta.totalPages).toBe(5);
		});
	});

	describe('CreateProductRequest', () => {
		it('should accept minimal create request', () => {
			const request: CreateProductRequest = {
				sku: 'NEW-SKU',
				name: 'New Product',
				productType: 'goods',
				salePrice: 100000
			};
			expect(request).toBeDefined();
		});

		it('should accept full create request', () => {
			const request: CreateProductRequest = {
				sku: 'NEW-SKU-FULL',
				name: 'New Full Product',
				description: 'Full description',
				productType: 'goods',
				trackInventory: true,
				trackingMethod: 'lot',
				defaultUomId: 'uom-1',
				salePrice: 200000,
				costPrice: 100000,
				currencyCode: 'VND',
				weightGrams: 500,
				dimensions: { lengthMm: 100, widthMm: 50, heightMm: 25 },
				attributes: { color: 'blue' },
				isActive: true,
				isSellable: true,
				isPurchaseable: true
			};
			expect(request.description).toBe('Full description');
		});
	});

	describe('UpdateProductRequest', () => {
		it('should accept partial update', () => {
			const request: UpdateProductRequest = {
				name: 'Updated Name'
			};
			expect(request.name).toBe('Updated Name');
		});

		it('should accept full update', () => {
			const request: UpdateProductRequest = {
				sku: 'UPDATED-SKU',
				name: 'Updated Product',
				salePrice: 300000,
				isActive: false
			};
			expect(request).toBeDefined();
		});
	});

	describe('CreateVariantRequest', () => {
		it('should accept minimal variant request', () => {
			const request: CreateVariantRequest = {
				sku: 'VAR-NEW',
				variantAttributes: { size: 'S' }
			};
			expect(request).toBeDefined();
		});

		it('should accept full variant request', () => {
			const request: CreateVariantRequest = {
				sku: 'VAR-NEW-FULL',
				barcode: '1234567890',
				variantAttributes: { color: 'red', size: 'M' },
				priceDifference: 50000,
				isActive: true
			};
			expect(request.barcode).toBe('1234567890');
		});
	});

	describe('UpdateVariantRequest', () => {
		it('should accept partial variant update', () => {
			const request: UpdateVariantRequest = {
				isActive: false
			};
			expect(request.isActive).toBe(false);
		});
	});
});

// =============================================================================
// Form Data Type Tests
// =============================================================================

describe('Form data types', () => {
	describe('ProductFormData', () => {
		it('should accept valid form data', () => {
			const formData: ProductFormData = {
				sku: 'FORM-SKU',
				name: 'Form Product',
				description: 'Form description',
				productType: 'goods',
				barcode: '',
				barcodeType: '',
				categoryId: '',
				trackInventory: true,
				trackingMethod: 'none',
				defaultUomId: 'uom-1',
				salePrice: 100000,
				costPrice: 50000,
				currencyCode: 'VND',
				weightGrams: 500,
				lengthMm: 100,
				widthMm: 50,
				heightMm: 25,
				isActive: true,
				isSellable: true,
				isPurchaseable: true
			};
			expect(formData).toBeDefined();
			expect(formData.sku).toBe('FORM-SKU');
		});
	});

	describe('VariantFormData', () => {
		it('should accept valid variant form data', () => {
			const formData: VariantFormData = {
				sku: 'VAR-FORM-SKU',
				barcode: '1234567890',
				priceDifference: 25000,
				isActive: true,
				attributes: [
					{ key: 'color', value: 'red' },
					{ key: 'size', value: 'M' }
				]
			};
			expect(formData).toBeDefined();
			expect(formData.attributes.length).toBe(2);
		});

		it('should accept variant form data with empty attributes', () => {
			const formData: VariantFormData = {
				sku: 'VAR-SIMPLE',
				barcode: '',
				priceDifference: 0,
				isActive: true,
				attributes: []
			};
			expect(formData.attributes.length).toBe(0);
		});
	});
});

// =============================================================================
// Type Enum Tests
// =============================================================================

describe('Type enums', () => {
	it('ProductType should have valid values', () => {
		const validTypes: ProductType[] = ['goods', 'service', 'consumable'];
		expect(validTypes.length).toBe(3);
	});

	it('TrackingMethod should have valid values', () => {
		const validMethods: TrackingMethod[] = ['none', 'lot', 'serial'];
		expect(validMethods.length).toBe(3);
	});

	it('ProductStatus should have valid values', () => {
		const validStatuses: ProductStatus[] = ['active', 'inactive', 'archived'];
		expect(validStatuses.length).toBe(3);
	});

	it('UomType should have valid values', () => {
		const validTypes: UomType[] = ['unit', 'weight', 'length', 'volume', 'time'];
		expect(validTypes.length).toBe(5);
	});
});
