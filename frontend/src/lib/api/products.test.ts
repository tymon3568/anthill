import { describe, it, expect, vi, beforeEach } from 'vitest';
import { productsApi, variantsApi, uomApi } from '$lib/api/products';
import { apiClient } from '$lib/api/client';
import type {
	Product,
	ProductVariant,
	UnitOfMeasure,
	ProductListResponse,
	CreateProductRequest,
	UpdateProductRequest,
	CreateVariantRequest,
	UpdateVariantRequest
} from '$lib/types/products';

// Mock the apiClient
vi.mock('$lib/api/client', () => ({
	apiClient: {
		get: vi.fn(),
		post: vi.fn(),
		put: vi.fn(),
		delete: vi.fn()
	}
}));

// ============================================================
// TEST DATA
// ============================================================

const mockProduct: Product = {
	productId: 'prod-1',
	tenantId: 'tenant-1',
	sku: 'TEST-001',
	name: 'Test Product',
	description: 'A test product',
	productType: 'goods',
	trackInventory: true,
	trackingMethod: 'none',
	defaultUomId: 'uom-1',
	salePrice: 10000000,
	costPrice: 5000000,
	currencyCode: 'VND',
	weightGrams: 500,
	dimensions: { lengthMm: 100, widthMm: 50, heightMm: 25 },
	isActive: true,
	isSellable: true,
	isPurchaseable: true,
	createdAt: '2026-01-01T00:00:00Z',
	updatedAt: '2026-01-01T00:00:00Z',
	variantCount: 0
};

const mockProductListResponse: ProductListResponse = {
	data: [mockProduct],
	meta: {
		page: 1,
		limit: 10,
		total: 1,
		totalPages: 1
	}
};

const mockVariant: ProductVariant = {
	variantId: 'var-1',
	tenantId: 'tenant-1',
	parentProductId: 'prod-1',
	sku: 'TEST-001-RED',
	variantAttributes: { color: 'Red' },
	priceDifference: 50000,
	isActive: true,
	createdAt: '2026-01-01T00:00:00Z',
	updatedAt: '2026-01-01T00:00:00Z'
};

const mockUom: UnitOfMeasure = {
	uomId: 'uom-1',
	tenantId: 'tenant-1',
	uomCode: 'PC',
	uomName: 'Piece',
	uomType: 'unit',
	isBaseUnit: true,
	isActive: true,
	createdAt: '2026-01-01T00:00:00Z',
	updatedAt: '2026-01-01T00:00:00Z'
};

// ============================================================
// PRODUCTS API TESTS
// ============================================================

describe('Products API Client', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('productsApi.list', () => {
		it('should list products without params', async () => {
			const mockResponse = { success: true, data: mockProductListResponse };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productsApi.list();

			expect(apiClient.get).toHaveBeenCalledWith('/products');
			expect(result).toEqual(mockResponse);
		});

		it('should list products with all filter params', async () => {
			const mockResponse = { success: true, data: mockProductListResponse };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productsApi.list({
				page: 2,
				limit: 25,
				search: 'laptop',
				productType: 'goods',
				isActive: true,
				trackingMethod: 'serial',
				sortBy: 'name',
				sortOrder: 'asc'
			});

			expect(apiClient.get).toHaveBeenCalledWith(
				'/products?page=2&limit=25&search=laptop&product_type=goods&is_active=true&tracking_method=serial&sort_by=name&sort_order=asc'
			);
			expect(result).toEqual(mockResponse);
		});

		it('should list products with partial params', async () => {
			const mockResponse = { success: true, data: mockProductListResponse };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productsApi.list({
				search: 'mouse',
				isActive: false
			});

			expect(apiClient.get).toHaveBeenCalledWith('/products?search=mouse&is_active=false');
			expect(result).toEqual(mockResponse);
		});

		it('should handle list error', async () => {
			const mockResponse = { success: false, error: 'Network error' };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productsApi.list();

			expect(result.success).toBe(false);
			expect(result.error).toBe('Network error');
		});
	});

	describe('productsApi.get', () => {
		it('should get a single product by ID', async () => {
			const mockResponse = { success: true, data: mockProduct };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productsApi.get('prod-1');

			expect(apiClient.get).toHaveBeenCalledWith('/products/prod-1');
			expect(result).toEqual(mockResponse);
		});

		it('should handle product not found', async () => {
			const mockResponse = { success: false, error: 'Product not found' };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productsApi.get('invalid-id');

			expect(result.success).toBe(false);
			expect(result.error).toBe('Product not found');
		});
	});

	describe('productsApi.create', () => {
		it('should create a new product', async () => {
			const mockResponse = { success: true, data: mockProduct };
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createData: CreateProductRequest = {
				sku: 'TEST-001',
				name: 'Test Product',
				productType: 'goods',
				salePrice: 10000000
			};

			const result = await productsApi.create(createData);

			expect(apiClient.post).toHaveBeenCalledWith('/products', createData);
			expect(result).toEqual(mockResponse);
		});

		it('should create product with all fields', async () => {
			const mockResponse = { success: true, data: mockProduct };
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createData: CreateProductRequest = {
				sku: 'TEST-002',
				name: 'Full Product',
				description: 'Complete product data',
				productType: 'goods',
				trackInventory: true,
				trackingMethod: 'lot',
				defaultUomId: 'uom-1',
				salePrice: 20000000,
				costPrice: 10000000,
				currencyCode: 'VND',
				weightGrams: 1000,
				dimensions: { lengthMm: 200, widthMm: 100, heightMm: 50 },
				isActive: true,
				isSellable: true,
				isPurchaseable: true
			};

			const result = await productsApi.create(createData);

			expect(apiClient.post).toHaveBeenCalledWith('/products', createData);
			expect(result.success).toBe(true);
		});

		it('should handle duplicate SKU error', async () => {
			const mockResponse = { success: false, error: 'SKU already exists' };
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createData: CreateProductRequest = {
				sku: 'DUPLICATE',
				name: 'Duplicate',
				productType: 'goods',
				salePrice: 1000
			};

			const result = await productsApi.create(createData);

			expect(result.success).toBe(false);
			expect(result.error).toBe('SKU already exists');
		});
	});

	describe('productsApi.update', () => {
		it('should update a product', async () => {
			const updatedProduct = { ...mockProduct, name: 'Updated Product' };
			const mockResponse = { success: true, data: updatedProduct };
			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const updateData: UpdateProductRequest = {
				name: 'Updated Product'
			};

			const result = await productsApi.update('prod-1', updateData);

			expect(apiClient.put).toHaveBeenCalledWith('/products/prod-1', updateData);
			expect(result).toEqual(mockResponse);
		});

		it('should update multiple fields', async () => {
			const updatedProduct = {
				...mockProduct,
				name: 'New Name',
				salePrice: 15000000,
				isActive: false
			};
			const mockResponse = { success: true, data: updatedProduct };
			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const updateData: UpdateProductRequest = {
				name: 'New Name',
				salePrice: 15000000,
				isActive: false
			};

			const result = await productsApi.update('prod-1', updateData);

			expect(result.success).toBe(true);
			expect(result.data?.name).toBe('New Name');
		});
	});

	describe('productsApi.delete', () => {
		it('should delete a product', async () => {
			const mockResponse = { success: true };
			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await productsApi.delete('prod-1');

			expect(apiClient.delete).toHaveBeenCalledWith('/products/prod-1');
			expect(result.success).toBe(true);
		});

		it('should handle delete not found', async () => {
			const mockResponse = { success: false, error: 'Product not found' };
			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await productsApi.delete('invalid-id');

			expect(result.success).toBe(false);
		});
	});

	describe('productsApi.bulkDelete', () => {
		it('should bulk delete products', async () => {
			const mockResponse = { success: true, data: { deleted: 3 } };
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await productsApi.bulkDelete(['prod-1', 'prod-2', 'prod-3']);

			expect(apiClient.post).toHaveBeenCalledWith('/products/bulk-delete', {
				ids: ['prod-1', 'prod-2', 'prod-3']
			});
			expect(result.success).toBe(true);
			expect(result.data?.deleted).toBe(3);
		});

		it('should handle partial bulk delete', async () => {
			const mockResponse = { success: true, data: { deleted: 2 } };
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const result = await productsApi.bulkDelete(['prod-1', 'prod-2', 'invalid']);

			expect(result.data?.deleted).toBe(2);
		});
	});

	describe('productsApi.export', () => {
		it('should export products without params', async () => {
			const mockBlob = new Blob(['csv data'], { type: 'text/csv' });
			const mockResponse = { success: true, data: mockBlob };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productsApi.export();

			expect(apiClient.get).toHaveBeenCalledWith('/products/export');
			expect(result.success).toBe(true);
		});

		it('should export products with filter params', async () => {
			const mockBlob = new Blob(['csv data'], { type: 'text/csv' });
			const mockResponse = { success: true, data: mockBlob };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await productsApi.export({
				search: 'laptop',
				productType: 'goods',
				isActive: true
			});

			expect(apiClient.get).toHaveBeenCalledWith(
				'/products/export?search=laptop&product_type=goods&is_active=true'
			);
			expect(result.success).toBe(true);
		});
	});
});

// ============================================================
// VARIANTS API TESTS
// ============================================================

describe('Variants API Client', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('variantsApi.list', () => {
		it('should list variants for a product', async () => {
			const mockResponse = { success: true, data: [mockVariant] };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantsApi.list('prod-1');

			expect(apiClient.get).toHaveBeenCalledWith('/products/prod-1/variants');
			expect(result).toEqual(mockResponse);
		});

		it('should return empty array when no variants', async () => {
			const mockResponse = { success: true, data: [] };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantsApi.list('prod-no-variants');

			expect(result.success).toBe(true);
			expect(result.data).toEqual([]);
		});
	});

	describe('variantsApi.get', () => {
		it('should get a single variant', async () => {
			const mockResponse = { success: true, data: mockVariant };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await variantsApi.get('prod-1', 'var-1');

			expect(apiClient.get).toHaveBeenCalledWith('/products/prod-1/variants/var-1');
			expect(result).toEqual(mockResponse);
		});
	});

	describe('variantsApi.create', () => {
		it('should create a new variant', async () => {
			const mockResponse = { success: true, data: mockVariant };
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createData: CreateVariantRequest = {
				sku: 'TEST-001-RED',
				variantAttributes: { color: 'Red' },
				priceDifference: 50000,
				isActive: true
			};

			const result = await variantsApi.create('prod-1', createData);

			expect(apiClient.post).toHaveBeenCalledWith('/products/prod-1/variants', createData);
			expect(result).toEqual(mockResponse);
		});

		it('should create variant with barcode', async () => {
			const variantWithBarcode = { ...mockVariant, barcode: '1234567890' };
			const mockResponse = { success: true, data: variantWithBarcode };
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createData: CreateVariantRequest = {
				sku: 'TEST-001-BLUE',
				barcode: '1234567890',
				variantAttributes: { color: 'Blue' }
			};

			const result = await variantsApi.create('prod-1', createData);

			expect(result.success).toBe(true);
			expect(result.data?.barcode).toBe('1234567890');
		});
	});

	describe('variantsApi.update', () => {
		it('should update a variant', async () => {
			const updatedVariant = { ...mockVariant, priceDifference: 100000 };
			const mockResponse = { success: true, data: updatedVariant };
			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const updateData: UpdateVariantRequest = {
				priceDifference: 100000
			};

			const result = await variantsApi.update('prod-1', 'var-1', updateData);

			expect(apiClient.put).toHaveBeenCalledWith('/products/prod-1/variants/var-1', updateData);
			expect(result).toEqual(mockResponse);
		});
	});

	describe('variantsApi.delete', () => {
		it('should delete a variant', async () => {
			const mockResponse = { success: true };
			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await variantsApi.delete('prod-1', 'var-1');

			expect(apiClient.delete).toHaveBeenCalledWith('/products/prod-1/variants/var-1');
			expect(result.success).toBe(true);
		});
	});
});

// ============================================================
// UOM API TESTS
// ============================================================

describe('UOM API Client', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('uomApi.list', () => {
		it('should list all UOMs', async () => {
			const mockResponse = { success: true, data: [mockUom] };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await uomApi.list();

			expect(apiClient.get).toHaveBeenCalledWith('/uom');
			expect(result).toEqual(mockResponse);
		});
	});

	describe('uomApi.get', () => {
		it('should get a single UOM', async () => {
			const mockResponse = { success: true, data: mockUom };
			vi.mocked(apiClient.get).mockResolvedValue(mockResponse);

			const result = await uomApi.get('uom-1');

			expect(apiClient.get).toHaveBeenCalledWith('/uom/uom-1');
			expect(result).toEqual(mockResponse);
		});
	});

	describe('uomApi.create', () => {
		it('should create a new UOM', async () => {
			const mockResponse = { success: true, data: mockUom };
			vi.mocked(apiClient.post).mockResolvedValue(mockResponse);

			const createData = {
				uomCode: 'BOX',
				uomName: 'Box',
				uomType: 'unit' as const,
				isBaseUnit: false,
				baseUomId: 'uom-1',
				conversionFactor: 12,
				isActive: true
			};

			const result = await uomApi.create(createData);

			expect(apiClient.post).toHaveBeenCalledWith('/uom', createData);
			expect(result.success).toBe(true);
		});
	});

	describe('uomApi.update', () => {
		it('should update a UOM', async () => {
			const updatedUom = { ...mockUom, uomName: 'Updated Piece' };
			const mockResponse = { success: true, data: updatedUom };
			vi.mocked(apiClient.put).mockResolvedValue(mockResponse);

			const result = await uomApi.update('uom-1', { uomName: 'Updated Piece' });

			expect(apiClient.put).toHaveBeenCalledWith('/uom/uom-1', { uomName: 'Updated Piece' });
			expect(result).toEqual(mockResponse);
		});
	});

	describe('uomApi.delete', () => {
		it('should delete a UOM', async () => {
			const mockResponse = { success: true };
			vi.mocked(apiClient.delete).mockResolvedValue(mockResponse);

			const result = await uomApi.delete('uom-1');

			expect(apiClient.delete).toHaveBeenCalledWith('/uom/uom-1');
			expect(result.success).toBe(true);
		});
	});
});
