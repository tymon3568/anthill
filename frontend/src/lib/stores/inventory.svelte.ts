// ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
//    - Use $state (NOT writable stores)
//    - Always consult MCP documentation before changes
//    - See .svelte-instructions.md for guidelines
import type { InventoryStore, Product, Category } from '$lib/types';

// Inventory state using Svelte 5 runes
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
