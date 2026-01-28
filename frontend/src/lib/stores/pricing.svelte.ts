// =============================================================================
// Pricing Store - Svelte 5 Runes-based State Management
// Comprehensive pricing state management for the frontend
// =============================================================================
// ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
//    - Use $state (NOT writable stores)
//    - Always consult MCP documentation before changes
//    - See .svelte-instructions.md for guidelines

import type {
	PriceList,
	PriceListItem,
	CustomerPriceList,
	PricingRule,
	PriceResult
} from '$lib/types/pricing';
import {
	priceListApi,
	priceListItemApi,
	customerPriceListApi,
	pricingRuleApi,
	priceCalculationApi
} from '$lib/api/pricing';
import type { ListParams, ItemListParams, RuleListParams } from '$lib/api/pricing';

// =============================================================================
// State Types
// =============================================================================

interface PaginationInfo {
	page: number;
	pageSize: number;
	totalItems: number;
	totalPages: number;
}

interface PriceListState {
	items: PriceList[];
	selected: PriceList | null;
	pagination: PaginationInfo | null;
	isLoading: boolean;
	error: string | null;
}

interface PriceListItemState {
	items: PriceListItem[];
	pagination: PaginationInfo | null;
	isLoading: boolean;
	error: string | null;
}

interface CustomerAssignmentState {
	items: CustomerPriceList[];
	isLoading: boolean;
	error: string | null;
}

interface PricingRuleState {
	items: PricingRule[];
	selected: PricingRule | null;
	pagination: PaginationInfo | null;
	isLoading: boolean;
	error: string | null;
}

interface PriceCalculationState {
	result: PriceResult | null;
	isCalculating: boolean;
	error: string | null;
}

// =============================================================================
// Price List State & Store
// =============================================================================

export const priceListState = $state<PriceListState>({
	items: [],
	selected: null,
	pagination: null,
	isLoading: false,
	error: null
});

export const priceListStore = {
	/**
	 * Load price lists with optional filtering
	 */
	async load(params: ListParams = {}): Promise<void> {
		priceListState.isLoading = true;
		priceListState.error = null;

		const response = await priceListApi.list(params);

		if (response.success && response.data) {
			priceListState.items = response.data.priceLists;
			priceListState.pagination = response.data.pagination;
		} else {
			priceListState.error = response.error || 'Failed to load price lists';
		}

		priceListState.isLoading = false;
	},

	/**
	 * Get a single price list by ID
	 */
	async get(priceListId: string): Promise<PriceList | null> {
		priceListState.isLoading = true;
		priceListState.error = null;

		const response = await priceListApi.get(priceListId);

		if (response.success && response.data) {
			priceListState.selected = response.data;
			priceListState.isLoading = false;
			return response.data;
		} else {
			priceListState.error = response.error || 'Failed to load price list';
			priceListState.isLoading = false;
			return null;
		}
	},

	/**
	 * Create a new price list
	 */
	async create(data: Parameters<typeof priceListApi.create>[0]): Promise<PriceList | null> {
		priceListState.isLoading = true;
		priceListState.error = null;

		const response = await priceListApi.create(data);

		if (response.success && response.data) {
			priceListState.items = [...priceListState.items, response.data];
			priceListState.isLoading = false;
			return response.data;
		} else {
			priceListState.error = response.error || 'Failed to create price list';
			priceListState.isLoading = false;
			return null;
		}
	},

	/**
	 * Update an existing price list
	 */
	async update(
		priceListId: string,
		data: Parameters<typeof priceListApi.update>[1]
	): Promise<PriceList | null> {
		priceListState.isLoading = true;
		priceListState.error = null;

		const response = await priceListApi.update(priceListId, data);

		if (response.success && response.data) {
			priceListState.items = priceListState.items.map((pl) =>
				pl.priceListId === priceListId ? response.data! : pl
			);
			if (priceListState.selected?.priceListId === priceListId) {
				priceListState.selected = response.data;
			}
			priceListState.isLoading = false;
			return response.data;
		} else {
			priceListState.error = response.error || 'Failed to update price list';
			priceListState.isLoading = false;
			return null;
		}
	},

	/**
	 * Delete a price list
	 */
	async delete(priceListId: string): Promise<boolean> {
		priceListState.isLoading = true;
		priceListState.error = null;

		const response = await priceListApi.delete(priceListId);

		if (response.success) {
			priceListState.items = priceListState.items.filter((pl) => pl.priceListId !== priceListId);
			if (priceListState.selected?.priceListId === priceListId) {
				priceListState.selected = null;
			}
			priceListState.isLoading = false;
			return true;
		} else {
			priceListState.error = response.error || 'Failed to delete price list';
			priceListState.isLoading = false;
			return false;
		}
	},

	/**
	 * Set a price list as default
	 */
	async setDefault(priceListId: string): Promise<boolean> {
		priceListState.isLoading = true;
		priceListState.error = null;

		const response = await priceListApi.setDefault(priceListId);

		if (response.success && response.data) {
			// Update all items to reflect new default
			priceListState.items = priceListState.items.map((pl) => ({
				...pl,
				isDefault: pl.priceListId === priceListId
			}));
			if (priceListState.selected?.priceListId === priceListId) {
				priceListState.selected = { ...priceListState.selected, isDefault: true };
			}
			priceListState.isLoading = false;
			return true;
		} else {
			priceListState.error = response.error || 'Failed to set default price list';
			priceListState.isLoading = false;
			return false;
		}
	},

	/**
	 * Select a price list
	 */
	select(priceList: PriceList | null): void {
		priceListState.selected = priceList;
	},

	/**
	 * Clear price list state
	 */
	clear(): void {
		priceListState.items = [];
		priceListState.selected = null;
		priceListState.pagination = null;
		priceListState.isLoading = false;
		priceListState.error = null;
	}
};

// =============================================================================
// Price List Items State & Store
// =============================================================================

export const priceListItemState = $state<PriceListItemState>({
	items: [],
	pagination: null,
	isLoading: false,
	error: null
});

export const priceListItemStore = {
	/**
	 * Load items for a price list
	 */
	async load(priceListId: string, params: ItemListParams = {}): Promise<void> {
		priceListItemState.isLoading = true;
		priceListItemState.error = null;

		const response = await priceListItemApi.list(priceListId, params);

		if (response.success && response.data) {
			priceListItemState.items = response.data.items;
			priceListItemState.pagination = response.data.pagination;
		} else {
			priceListItemState.error = response.error || 'Failed to load price list items';
		}

		priceListItemState.isLoading = false;
	},

	/**
	 * Create a new item
	 */
	async create(
		priceListId: string,
		data: Parameters<typeof priceListItemApi.create>[1]
	): Promise<PriceListItem | null> {
		priceListItemState.isLoading = true;
		priceListItemState.error = null;

		const response = await priceListItemApi.create(priceListId, data);

		if (response.success && response.data) {
			priceListItemState.items = [...priceListItemState.items, response.data];
			priceListItemState.isLoading = false;
			return response.data;
		} else {
			priceListItemState.error = response.error || 'Failed to create item';
			priceListItemState.isLoading = false;
			return null;
		}
	},

	/**
	 * Update an existing item
	 */
	async update(
		priceListId: string,
		itemId: string,
		data: Parameters<typeof priceListItemApi.update>[2]
	): Promise<PriceListItem | null> {
		priceListItemState.isLoading = true;
		priceListItemState.error = null;

		const response = await priceListItemApi.update(priceListId, itemId, data);

		if (response.success && response.data) {
			priceListItemState.items = priceListItemState.items.map((item) =>
				item.itemId === itemId ? response.data! : item
			);
			priceListItemState.isLoading = false;
			return response.data;
		} else {
			priceListItemState.error = response.error || 'Failed to update item';
			priceListItemState.isLoading = false;
			return null;
		}
	},

	/**
	 * Delete an item
	 */
	async delete(priceListId: string, itemId: string): Promise<boolean> {
		priceListItemState.isLoading = true;
		priceListItemState.error = null;

		const response = await priceListItemApi.delete(priceListId, itemId);

		if (response.success) {
			priceListItemState.items = priceListItemState.items.filter((item) => item.itemId !== itemId);
			priceListItemState.isLoading = false;
			return true;
		} else {
			priceListItemState.error = response.error || 'Failed to delete item';
			priceListItemState.isLoading = false;
			return false;
		}
	},

	/**
	 * Bulk create items
	 */
	async bulkCreate(
		priceListId: string,
		items: Parameters<typeof priceListItemApi.bulkCreate>[1]
	): Promise<PriceListItem[] | null> {
		priceListItemState.isLoading = true;
		priceListItemState.error = null;

		const response = await priceListItemApi.bulkCreate(priceListId, items);

		if (response.success && response.data) {
			priceListItemState.items = [...priceListItemState.items, ...response.data.items];
			priceListItemState.isLoading = false;
			return response.data.items;
		} else {
			priceListItemState.error = response.error || 'Failed to create items';
			priceListItemState.isLoading = false;
			return null;
		}
	},

	/**
	 * Clear items state
	 */
	clear(): void {
		priceListItemState.items = [];
		priceListItemState.pagination = null;
		priceListItemState.isLoading = false;
		priceListItemState.error = null;
	}
};

// =============================================================================
// Customer Assignment State & Store
// =============================================================================

export const customerAssignmentState = $state<CustomerAssignmentState>({
	items: [],
	isLoading: false,
	error: null
});

export const customerAssignmentStore = {
	/**
	 * Load customers assigned to a price list
	 */
	async load(priceListId: string): Promise<void> {
		customerAssignmentState.isLoading = true;
		customerAssignmentState.error = null;

		const response = await customerPriceListApi.list(priceListId);

		if (response.success && response.data) {
			customerAssignmentState.items = response.data;
		} else {
			customerAssignmentState.error = response.error || 'Failed to load customer assignments';
		}

		customerAssignmentState.isLoading = false;
	},

	/**
	 * Assign a customer to a price list
	 */
	async assign(
		priceListId: string,
		customerId: string,
		data: Parameters<typeof customerPriceListApi.assign>[2] = {}
	): Promise<CustomerPriceList | null> {
		customerAssignmentState.isLoading = true;
		customerAssignmentState.error = null;

		const response = await customerPriceListApi.assign(priceListId, customerId, data);

		if (response.success && response.data) {
			customerAssignmentState.items = [...customerAssignmentState.items, response.data];
			customerAssignmentState.isLoading = false;
			return response.data;
		} else {
			customerAssignmentState.error = response.error || 'Failed to assign customer';
			customerAssignmentState.isLoading = false;
			return null;
		}
	},

	/**
	 * Unassign a customer from a price list
	 */
	async unassign(priceListId: string, customerId: string): Promise<boolean> {
		customerAssignmentState.isLoading = true;
		customerAssignmentState.error = null;

		const response = await customerPriceListApi.unassign(priceListId, customerId);

		if (response.success) {
			customerAssignmentState.items = customerAssignmentState.items.filter(
				(cpl) => cpl.customerId !== customerId
			);
			customerAssignmentState.isLoading = false;
			return true;
		} else {
			customerAssignmentState.error = response.error || 'Failed to unassign customer';
			customerAssignmentState.isLoading = false;
			return false;
		}
	},

	/**
	 * Clear customer assignment state
	 */
	clear(): void {
		customerAssignmentState.items = [];
		customerAssignmentState.isLoading = false;
		customerAssignmentState.error = null;
	}
};

// =============================================================================
// Pricing Rules State & Store
// =============================================================================

export const pricingRuleState = $state<PricingRuleState>({
	items: [],
	selected: null,
	pagination: null,
	isLoading: false,
	error: null
});

export const pricingRuleStore = {
	/**
	 * Load pricing rules with optional filtering
	 */
	async load(params: RuleListParams = {}): Promise<void> {
		pricingRuleState.isLoading = true;
		pricingRuleState.error = null;

		const response = await pricingRuleApi.list(params);

		if (response.success && response.data) {
			pricingRuleState.items = response.data.rules;
			pricingRuleState.pagination = response.data.pagination;
		} else {
			pricingRuleState.error = response.error || 'Failed to load pricing rules';
		}

		pricingRuleState.isLoading = false;
	},

	/**
	 * Get a single pricing rule by ID
	 */
	async get(ruleId: string): Promise<PricingRule | null> {
		pricingRuleState.isLoading = true;
		pricingRuleState.error = null;

		const response = await pricingRuleApi.get(ruleId);

		if (response.success && response.data) {
			pricingRuleState.selected = response.data;
			pricingRuleState.isLoading = false;
			return response.data;
		} else {
			pricingRuleState.error = response.error || 'Failed to load pricing rule';
			pricingRuleState.isLoading = false;
			return null;
		}
	},

	/**
	 * Create a new pricing rule
	 */
	async create(data: Parameters<typeof pricingRuleApi.create>[0]): Promise<PricingRule | null> {
		pricingRuleState.isLoading = true;
		pricingRuleState.error = null;

		const response = await pricingRuleApi.create(data);

		if (response.success && response.data) {
			pricingRuleState.items = [...pricingRuleState.items, response.data];
			pricingRuleState.isLoading = false;
			return response.data;
		} else {
			pricingRuleState.error = response.error || 'Failed to create pricing rule';
			pricingRuleState.isLoading = false;
			return null;
		}
	},

	/**
	 * Update an existing pricing rule
	 */
	async update(
		ruleId: string,
		data: Parameters<typeof pricingRuleApi.update>[1]
	): Promise<PricingRule | null> {
		pricingRuleState.isLoading = true;
		pricingRuleState.error = null;

		const response = await pricingRuleApi.update(ruleId, data);

		if (response.success && response.data) {
			pricingRuleState.items = pricingRuleState.items.map((rule) =>
				rule.ruleId === ruleId ? response.data! : rule
			);
			if (pricingRuleState.selected?.ruleId === ruleId) {
				pricingRuleState.selected = response.data;
			}
			pricingRuleState.isLoading = false;
			return response.data;
		} else {
			pricingRuleState.error = response.error || 'Failed to update pricing rule';
			pricingRuleState.isLoading = false;
			return null;
		}
	},

	/**
	 * Delete a pricing rule
	 */
	async delete(ruleId: string): Promise<boolean> {
		pricingRuleState.isLoading = true;
		pricingRuleState.error = null;

		const response = await pricingRuleApi.delete(ruleId);

		if (response.success) {
			pricingRuleState.items = pricingRuleState.items.filter((rule) => rule.ruleId !== ruleId);
			if (pricingRuleState.selected?.ruleId === ruleId) {
				pricingRuleState.selected = null;
			}
			pricingRuleState.isLoading = false;
			return true;
		} else {
			pricingRuleState.error = response.error || 'Failed to delete pricing rule';
			pricingRuleState.isLoading = false;
			return false;
		}
	},

	/**
	 * Duplicate a pricing rule
	 */
	async duplicate(ruleId: string): Promise<PricingRule | null> {
		pricingRuleState.isLoading = true;
		pricingRuleState.error = null;

		const response = await pricingRuleApi.duplicate(ruleId);

		if (response.success && response.data) {
			pricingRuleState.items = [...pricingRuleState.items, response.data];
			pricingRuleState.isLoading = false;
			return response.data;
		} else {
			pricingRuleState.error = response.error || 'Failed to duplicate pricing rule';
			pricingRuleState.isLoading = false;
			return null;
		}
	},

	/**
	 * Toggle rule active status
	 */
	async toggleActive(ruleId: string, isActive: boolean): Promise<boolean> {
		pricingRuleState.isLoading = true;
		pricingRuleState.error = null;

		const response = isActive
			? await pricingRuleApi.activate(ruleId)
			: await pricingRuleApi.deactivate(ruleId);

		if (response.success && response.data) {
			pricingRuleState.items = pricingRuleState.items.map((rule) =>
				rule.ruleId === ruleId ? { ...rule, isActive } : rule
			);
			if (pricingRuleState.selected?.ruleId === ruleId) {
				pricingRuleState.selected = { ...pricingRuleState.selected, isActive };
			}
			pricingRuleState.isLoading = false;
			return true;
		} else {
			pricingRuleState.error = response.error || 'Failed to toggle rule status';
			pricingRuleState.isLoading = false;
			return false;
		}
	},

	/**
	 * Select a pricing rule
	 */
	select(rule: PricingRule | null): void {
		pricingRuleState.selected = rule;
	},

	/**
	 * Clear pricing rule state
	 */
	clear(): void {
		pricingRuleState.items = [];
		pricingRuleState.selected = null;
		pricingRuleState.pagination = null;
		pricingRuleState.isLoading = false;
		pricingRuleState.error = null;
	}
};

// =============================================================================
// Price Calculation State & Store
// =============================================================================

export const priceCalculationState = $state<PriceCalculationState>({
	result: null,
	isCalculating: false,
	error: null
});

export const priceCalculationStore = {
	/**
	 * Calculate price for a product
	 */
	async calculate(
		request: Parameters<typeof priceCalculationApi.calculate>[0]
	): Promise<PriceResult | null> {
		priceCalculationState.isCalculating = true;
		priceCalculationState.error = null;

		const response = await priceCalculationApi.calculate(request);

		if (response.success && response.data) {
			priceCalculationState.result = response.data;
			priceCalculationState.isCalculating = false;
			return response.data;
		} else {
			priceCalculationState.error = response.error || 'Failed to calculate price';
			priceCalculationState.isCalculating = false;
			return null;
		}
	},

	/**
	 * Clear calculation state
	 */
	clear(): void {
		priceCalculationState.result = null;
		priceCalculationState.isCalculating = false;
		priceCalculationState.error = null;
	}
};
