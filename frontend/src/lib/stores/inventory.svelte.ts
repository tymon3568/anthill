// =============================================================================
// Inventory Store - Svelte 5 Runes-based State Management
// Comprehensive inventory state management for the frontend
// =============================================================================
// ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
//    - Use $state (NOT writable stores)
//    - Always consult MCP documentation before changes
//    - See .svelte-instructions.md for guidelines

import type {
	CategoryResponse,
	ProductResponse,
	VariantResponse,
	WarehouseResponse,
	StockLevelResponse,
	StockLevelSummary,
	PaginationInfo,
	CategoryListParams,
	ProductListParams,
	VariantListParams,
	StockLevelListParams
} from '$lib/types/inventory';
import { categoryApi } from '$lib/api/inventory/categories';
import { productApi } from '$lib/api/inventory/products';
import { variantApi } from '$lib/api/inventory/variants';
import { warehouseApi } from '$lib/api/inventory/warehouses';
import { stockLevelApi } from '$lib/api/inventory/stock-levels';

// =============================================================================
// State Types
// =============================================================================

interface CategoryState {
	items: CategoryResponse[];
	tree: CategoryResponse[];
	selected: CategoryResponse | null;
	pagination: PaginationInfo | null;
	isLoading: boolean;
	error: string | null;
}

interface ProductState {
	items: ProductResponse[];
	selected: ProductResponse | null;
	pagination: PaginationInfo | null;
	isLoading: boolean;
	error: string | null;
}

interface VariantState {
	items: VariantResponse[];
	selected: VariantResponse | null;
	pagination: PaginationInfo | null;
	isLoading: boolean;
	error: string | null;
}

interface WarehouseState {
	items: WarehouseResponse[];
	selected: WarehouseResponse | null;
	isLoading: boolean;
	error: string | null;
}

interface StockLevelState {
	items: StockLevelResponse[];
	summary: StockLevelSummary | null;
	pagination: PaginationInfo | null;
	isLoading: boolean;
	error: string | null;
}

interface InventoryDashboardState {
	lowStockCount: number;
	totalProducts: number;
	totalCategories: number;
	totalWarehouses: number;
	recentActivity: unknown[];
	isLoading: boolean;
	error: string | null;
}

// =============================================================================
// Category State & Store
// =============================================================================

export const categoryState = $state<CategoryState>({
	items: [],
	tree: [],
	selected: null,
	pagination: null,
	isLoading: false,
	error: null
});

export const categoryStore = {
	/**
	 * Load categories with optional filtering
	 */
	async load(params: CategoryListParams = {}): Promise<void> {
		categoryState.isLoading = true;
		categoryState.error = null;

		const response = await categoryApi.list(params);

		if (response.success && response.data) {
			categoryState.items = response.data.categories;
			categoryState.pagination = response.data.pagination;
		} else {
			categoryState.error = response.error || 'Failed to load categories';
		}

		categoryState.isLoading = false;
	},

	/**
	 * Load category tree structure
	 */
	async loadTree(rootId?: string): Promise<void> {
		categoryState.isLoading = true;
		categoryState.error = null;

		const response = await categoryApi.getTree(rootId);

		if (response.success && response.data) {
			categoryState.tree = response.data;
		} else {
			categoryState.error = response.error || 'Failed to load category tree';
		}

		categoryState.isLoading = false;
	},

	/**
	 * Get a single category by ID
	 */
	async get(categoryId: string): Promise<CategoryResponse | null> {
		categoryState.isLoading = true;
		categoryState.error = null;

		const response = await categoryApi.get(categoryId);

		if (response.success && response.data) {
			categoryState.selected = response.data;
			categoryState.isLoading = false;
			return response.data;
		} else {
			categoryState.error = response.error || 'Failed to load category';
			categoryState.isLoading = false;
			return null;
		}
	},

	/**
	 * Create a new category
	 */
	async create(data: Parameters<typeof categoryApi.create>[0]): Promise<CategoryResponse | null> {
		categoryState.isLoading = true;
		categoryState.error = null;

		const response = await categoryApi.create(data);

		if (response.success && response.data) {
			categoryState.items = [...categoryState.items, response.data];
			categoryState.isLoading = false;
			return response.data;
		} else {
			categoryState.error = response.error || 'Failed to create category';
			categoryState.isLoading = false;
			return null;
		}
	},

	/**
	 * Update an existing category
	 */
	async update(
		categoryId: string,
		data: Parameters<typeof categoryApi.update>[1]
	): Promise<CategoryResponse | null> {
		categoryState.isLoading = true;
		categoryState.error = null;

		const response = await categoryApi.update(categoryId, data);

		if (response.success && response.data) {
			categoryState.items = categoryState.items.map((cat) =>
				cat.categoryId === categoryId ? response.data! : cat
			);
			if (categoryState.selected?.categoryId === categoryId) {
				categoryState.selected = response.data;
			}
			categoryState.isLoading = false;
			return response.data;
		} else {
			categoryState.error = response.error || 'Failed to update category';
			categoryState.isLoading = false;
			return null;
		}
	},

	/**
	 * Delete a category
	 */
	async delete(categoryId: string): Promise<boolean> {
		categoryState.isLoading = true;
		categoryState.error = null;

		const response = await categoryApi.delete(categoryId);

		if (response.success) {
			categoryState.items = categoryState.items.filter((cat) => cat.categoryId !== categoryId);
			if (categoryState.selected?.categoryId === categoryId) {
				categoryState.selected = null;
			}
			categoryState.isLoading = false;
			return true;
		} else {
			categoryState.error = response.error || 'Failed to delete category';
			categoryState.isLoading = false;
			return false;
		}
	},

	/**
	 * Select a category
	 */
	select(category: CategoryResponse | null): void {
		categoryState.selected = category;
	},

	/**
	 * Clear category state
	 */
	clear(): void {
		categoryState.items = [];
		categoryState.tree = [];
		categoryState.selected = null;
		categoryState.pagination = null;
		categoryState.isLoading = false;
		categoryState.error = null;
	}
};

// =============================================================================
// Product State & Store
// =============================================================================

export const productState = $state<ProductState>({
	items: [],
	selected: null,
	pagination: null,
	isLoading: false,
	error: null
});

export const productStore = {
	/**
	 * Load products with optional filtering
	 */
	async load(params: ProductListParams = {}): Promise<void> {
		productState.isLoading = true;
		productState.error = null;

		const response = await productApi.list(params);

		if (response.success && response.data) {
			productState.items = response.data.products;
			productState.pagination = response.data.pagination;
		} else {
			productState.error = response.error || 'Failed to load products';
		}

		productState.isLoading = false;
	},

	/**
	 * Get a single product by ID
	 */
	async get(productId: string): Promise<ProductResponse | null> {
		productState.isLoading = true;
		productState.error = null;

		const response = await productApi.get(productId);

		if (response.success && response.data) {
			productState.selected = response.data;
			productState.isLoading = false;
			return response.data;
		} else {
			productState.error = response.error || 'Failed to load product';
			productState.isLoading = false;
			return null;
		}
	},

	/**
	 * Get a product by SKU
	 */
	async getBySku(sku: string): Promise<ProductResponse | null> {
		productState.isLoading = true;
		productState.error = null;

		const response = await productApi.getBySku(sku);

		if (response.success && response.data) {
			productState.selected = response.data;
			productState.isLoading = false;
			return response.data;
		} else {
			productState.error = response.error || 'Failed to load product';
			productState.isLoading = false;
			return null;
		}
	},

	/**
	 * Create a new product
	 */
	async create(data: Parameters<typeof productApi.create>[0]): Promise<ProductResponse | null> {
		productState.isLoading = true;
		productState.error = null;

		const response = await productApi.create(data);

		if (response.success && response.data) {
			productState.items = [...productState.items, response.data];
			productState.isLoading = false;
			return response.data;
		} else {
			productState.error = response.error || 'Failed to create product';
			productState.isLoading = false;
			return null;
		}
	},

	/**
	 * Update an existing product
	 */
	async update(
		productId: string,
		data: Parameters<typeof productApi.update>[1]
	): Promise<ProductResponse | null> {
		productState.isLoading = true;
		productState.error = null;

		const response = await productApi.update(productId, data);

		if (response.success && response.data) {
			productState.items = productState.items.map((prod) =>
				prod.productId === productId ? response.data! : prod
			);
			if (productState.selected?.productId === productId) {
				productState.selected = response.data;
			}
			productState.isLoading = false;
			return response.data;
		} else {
			productState.error = response.error || 'Failed to update product';
			productState.isLoading = false;
			return null;
		}
	},

	/**
	 * Delete a product
	 */
	async delete(productId: string): Promise<boolean> {
		productState.isLoading = true;
		productState.error = null;

		const response = await productApi.delete(productId);

		if (response.success) {
			productState.items = productState.items.filter((prod) => prod.productId !== productId);
			if (productState.selected?.productId === productId) {
				productState.selected = null;
			}
			productState.isLoading = false;
			return true;
		} else {
			productState.error = response.error || 'Failed to delete product';
			productState.isLoading = false;
			return false;
		}
	},

	/**
	 * Select a product
	 */
	select(product: ProductResponse | null): void {
		productState.selected = product;
	},

	/**
	 * Clear product state
	 */
	clear(): void {
		productState.items = [];
		productState.selected = null;
		productState.pagination = null;
		productState.isLoading = false;
		productState.error = null;
	}
};

// =============================================================================
// Variant State & Store
// =============================================================================

export const variantState = $state<VariantState>({
	items: [],
	selected: null,
	pagination: null,
	isLoading: false,
	error: null
});

export const variantStore = {
	/**
	 * Load variants with optional filtering
	 */
	async load(params: VariantListParams = {}): Promise<void> {
		variantState.isLoading = true;
		variantState.error = null;

		const response = await variantApi.list(params);

		if (response.success && response.data) {
			variantState.items = response.data.variants;
			variantState.pagination = response.data.pagination;
		} else {
			variantState.error = response.error || 'Failed to load variants';
		}

		variantState.isLoading = false;
	},

	/**
	 * Load variants for a specific product
	 */
	async loadByProduct(
		parentProductId: string,
		params: Omit<VariantListParams, 'parentProductId'> = {}
	): Promise<void> {
		variantState.isLoading = true;
		variantState.error = null;

		const response = await variantApi.listByProduct(parentProductId, params);

		if (response.success && response.data) {
			variantState.items = response.data.variants;
			variantState.pagination = response.data.pagination;
		} else {
			variantState.error = response.error || 'Failed to load variants';
		}

		variantState.isLoading = false;
	},

	/**
	 * Get a single variant by ID
	 */
	async get(variantId: string): Promise<VariantResponse | null> {
		variantState.isLoading = true;
		variantState.error = null;

		const response = await variantApi.get(variantId);

		if (response.success && response.data) {
			variantState.selected = response.data;
			variantState.isLoading = false;
			return response.data;
		} else {
			variantState.error = response.error || 'Failed to load variant';
			variantState.isLoading = false;
			return null;
		}
	},

	/**
	 * Get a variant by SKU
	 */
	async getBySku(sku: string): Promise<VariantResponse | null> {
		variantState.isLoading = true;
		variantState.error = null;

		const response = await variantApi.getBySku(sku);

		if (response.success && response.data) {
			variantState.selected = response.data;
			variantState.isLoading = false;
			return response.data;
		} else {
			variantState.error = response.error || 'Failed to load variant';
			variantState.isLoading = false;
			return null;
		}
	},

	/**
	 * Get a variant by barcode
	 */
	async getByBarcode(barcode: string): Promise<VariantResponse | null> {
		variantState.isLoading = true;
		variantState.error = null;

		const response = await variantApi.getByBarcode(barcode);

		if (response.success && response.data) {
			variantState.selected = response.data;
			variantState.isLoading = false;
			return response.data;
		} else {
			variantState.error = response.error || 'Failed to load variant';
			variantState.isLoading = false;
			return null;
		}
	},

	/**
	 * Create a new variant
	 */
	async create(data: Parameters<typeof variantApi.create>[0]): Promise<VariantResponse | null> {
		variantState.isLoading = true;
		variantState.error = null;

		const response = await variantApi.create(data);

		if (response.success && response.data) {
			variantState.items = [...variantState.items, response.data];
			variantState.isLoading = false;
			return response.data;
		} else {
			variantState.error = response.error || 'Failed to create variant';
			variantState.isLoading = false;
			return null;
		}
	},

	/**
	 * Update an existing variant
	 */
	async update(
		variantId: string,
		data: Parameters<typeof variantApi.update>[1]
	): Promise<VariantResponse | null> {
		variantState.isLoading = true;
		variantState.error = null;

		const response = await variantApi.update(variantId, data);

		if (response.success && response.data) {
			variantState.items = variantState.items.map((v) =>
				v.variantId === variantId ? response.data! : v
			);
			if (variantState.selected?.variantId === variantId) {
				variantState.selected = response.data;
			}
			variantState.isLoading = false;
			return response.data;
		} else {
			variantState.error = response.error || 'Failed to update variant';
			variantState.isLoading = false;
			return null;
		}
	},

	/**
	 * Delete a variant
	 */
	async delete(variantId: string): Promise<boolean> {
		variantState.isLoading = true;
		variantState.error = null;

		const response = await variantApi.delete(variantId);

		if (response.success) {
			variantState.items = variantState.items.filter((v) => v.variantId !== variantId);
			if (variantState.selected?.variantId === variantId) {
				variantState.selected = null;
			}
			variantState.isLoading = false;
			return true;
		} else {
			variantState.error = response.error || 'Failed to delete variant';
			variantState.isLoading = false;
			return false;
		}
	},

	/**
	 * Select a variant
	 */
	select(variant: VariantResponse | null): void {
		variantState.selected = variant;
	},

	/**
	 * Clear variant state
	 */
	clear(): void {
		variantState.items = [];
		variantState.selected = null;
		variantState.pagination = null;
		variantState.isLoading = false;
		variantState.error = null;
	}
};

// =============================================================================
// Warehouse State & Store
// =============================================================================

export const warehouseState = $state<WarehouseState>({
	items: [],
	selected: null,
	isLoading: false,
	error: null
});

export const warehouseStore = {
	/**
	 * Load all warehouses
	 */
	async load(): Promise<void> {
		warehouseState.isLoading = true;
		warehouseState.error = null;

		const response = await warehouseApi.list();

		if (response.success && response.data) {
			warehouseState.items = response.data.warehouses;
		} else {
			warehouseState.error = response.error || 'Failed to load warehouses';
		}

		warehouseState.isLoading = false;
	},

	/**
	 * Get a single warehouse by ID
	 */
	async get(warehouseId: string): Promise<WarehouseResponse | null> {
		warehouseState.isLoading = true;
		warehouseState.error = null;

		const response = await warehouseApi.get(warehouseId);

		if (response.success && response.data) {
			warehouseState.selected = response.data;
			warehouseState.isLoading = false;
			return response.data;
		} else {
			warehouseState.error = response.error || 'Failed to load warehouse';
			warehouseState.isLoading = false;
			return null;
		}
	},

	/**
	 * Create a new warehouse
	 */
	async create(data: Parameters<typeof warehouseApi.create>[0]): Promise<WarehouseResponse | null> {
		warehouseState.isLoading = true;
		warehouseState.error = null;

		const response = await warehouseApi.create(data);

		if (response.success && response.data) {
			warehouseState.items = [...warehouseState.items, response.data];
			warehouseState.isLoading = false;
			return response.data;
		} else {
			warehouseState.error = response.error || 'Failed to create warehouse';
			warehouseState.isLoading = false;
			return null;
		}
	},

	/**
	 * Update an existing warehouse
	 */
	async update(
		warehouseId: string,
		data: Parameters<typeof warehouseApi.update>[1]
	): Promise<WarehouseResponse | null> {
		warehouseState.isLoading = true;
		warehouseState.error = null;

		const response = await warehouseApi.update(warehouseId, data);

		if (response.success && response.data) {
			warehouseState.items = warehouseState.items.map((wh) =>
				wh.warehouseId === warehouseId ? response.data! : wh
			);
			if (warehouseState.selected?.warehouseId === warehouseId) {
				warehouseState.selected = response.data;
			}
			warehouseState.isLoading = false;
			return response.data;
		} else {
			warehouseState.error = response.error || 'Failed to update warehouse';
			warehouseState.isLoading = false;
			return null;
		}
	},

	/**
	 * Delete a warehouse
	 */
	async delete(warehouseId: string): Promise<boolean> {
		warehouseState.isLoading = true;
		warehouseState.error = null;

		const response = await warehouseApi.delete(warehouseId);

		if (response.success) {
			warehouseState.items = warehouseState.items.filter((wh) => wh.warehouseId !== warehouseId);
			if (warehouseState.selected?.warehouseId === warehouseId) {
				warehouseState.selected = null;
			}
			warehouseState.isLoading = false;
			return true;
		} else {
			warehouseState.error = response.error || 'Failed to delete warehouse';
			warehouseState.isLoading = false;
			return false;
		}
	},

	/**
	 * Select a warehouse
	 */
	select(warehouse: WarehouseResponse | null): void {
		warehouseState.selected = warehouse;
	},

	/**
	 * Clear warehouse state
	 */
	clear(): void {
		warehouseState.items = [];
		warehouseState.selected = null;
		warehouseState.isLoading = false;
		warehouseState.error = null;
	}
};

// =============================================================================
// Stock Level State & Store
// =============================================================================

export const stockLevelState = $state<StockLevelState>({
	items: [],
	summary: null,
	pagination: null,
	isLoading: false,
	error: null
});

export const stockLevelStore = {
	/**
	 * Load stock levels with optional filtering and pagination
	 */
	async load(params: StockLevelListParams = {}): Promise<void> {
		stockLevelState.isLoading = true;
		stockLevelState.error = null;

		const response = await stockLevelApi.list(params);

		if (response.success && response.data) {
			stockLevelState.items = response.data.items;
			stockLevelState.summary = response.data.summary;
			stockLevelState.pagination = response.data.pagination;
		} else {
			stockLevelState.error = response.error || 'Failed to load stock levels';
		}

		stockLevelState.isLoading = false;
	},

	/**
	 * Refresh stock levels with current parameters
	 */
	async refresh(params: StockLevelListParams = {}): Promise<void> {
		await this.load(params);
	},

	/**
	 * Clear stock level state
	 */
	clear(): void {
		stockLevelState.items = [];
		stockLevelState.summary = null;
		stockLevelState.pagination = null;
		stockLevelState.isLoading = false;
		stockLevelState.error = null;
	}
};

// =============================================================================
// Dashboard State & Store
// =============================================================================

export const dashboardState = $state<InventoryDashboardState>({
	lowStockCount: 0,
	totalProducts: 0,
	totalCategories: 0,
	totalWarehouses: 0,
	recentActivity: [],
	isLoading: false,
	error: null
});

export const dashboardStore = {
	/**
	 * Load dashboard summary data
	 */
	async loadSummary(): Promise<void> {
		dashboardState.isLoading = true;
		dashboardState.error = null;

		try {
			// Load counts in parallel
			const [productsRes, categoriesRes, warehousesRes] = await Promise.all([
				productApi.list({ pageSize: 1 }),
				categoryApi.list({ pageSize: 1 }),
				warehouseApi.list({ pageSize: 1 })
			]);

			if (productsRes.success && productsRes.data) {
				dashboardState.totalProducts = productsRes.data.pagination.totalItems;
			}

			if (categoriesRes.success && categoriesRes.data) {
				dashboardState.totalCategories = categoriesRes.data.pagination.totalItems;
			}

			if (warehousesRes.success && warehousesRes.data) {
				dashboardState.totalWarehouses = warehousesRes.data.pagination.totalItems;
			}
		} catch (error) {
			dashboardState.error = error instanceof Error ? error.message : 'Failed to load dashboard';
		}

		dashboardState.isLoading = false;
	},

	/**
	 * Clear dashboard state
	 */
	clear(): void {
		dashboardState.lowStockCount = 0;
		dashboardState.totalProducts = 0;
		dashboardState.totalCategories = 0;
		dashboardState.totalWarehouses = 0;
		dashboardState.recentActivity = [];
		dashboardState.isLoading = false;
		dashboardState.error = null;
	}
};

// =============================================================================
// Legacy Compatibility Exports
// Maintains backward compatibility with existing code
// =============================================================================

import type { InventoryStore, Product, Category } from '$lib/types';

export const inventoryState = $state<InventoryStore>({
	products: [],
	categories: [],
	isLoading: false,
	error: null
});

export const inventoryStore = {
	setProducts: (products: Product[]) => {
		inventoryState.products = products;
	},

	setCategories: (categories: Category[]) => {
		inventoryState.categories = categories;
	},

	setLoading: (loading: boolean) => {
		inventoryState.isLoading = loading;
	},

	setError: (error: string | null) => {
		inventoryState.error = error;
	},

	addProduct: (product: Product) => {
		inventoryState.products = [...inventoryState.products, product];
	},

	updateProduct: (updatedProduct: Product) => {
		inventoryState.products = inventoryState.products.map((product) =>
			product.id === updatedProduct.id ? updatedProduct : product
		);
	},

	removeProduct: (productId: string) => {
		inventoryState.products = inventoryState.products.filter((product) => product.id !== productId);
	},

	addCategory: (category: Category) => {
		inventoryState.categories = [...inventoryState.categories, category];
	},

	updateCategory: (updatedCategory: Category) => {
		inventoryState.categories = inventoryState.categories.map((category) =>
			category.id === updatedCategory.id ? updatedCategory : category
		);
	},

	removeCategory: (categoryId: string) => {
		inventoryState.categories = inventoryState.categories.filter(
			(category) => category.id !== categoryId
		);
	},

	clearData: () => {
		inventoryState.products = [];
		inventoryState.categories = [];
		inventoryState.isLoading = false;
		inventoryState.error = null;
	}
};
