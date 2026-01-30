<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import { adjustmentStore } from '$lib/stores/adjustments.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import { productState, productStore } from '$lib/stores/inventory.svelte';
	import { stockLevelApi } from '$lib/api/inventory/stock-levels';
	import type { AdjustmentReasonCode, CreateAdjustmentItem } from '$lib/types/inventory';
	import {
		REASON_CODES,
		getReasonCodesByDirection,
		CATEGORY_LABELS
	} from '$lib/constants/adjustment-reasons';

	// UI Components
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Badge } from '$lib/components/ui/badge';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';
	import * as Table from '$lib/components/ui/table';

	// Icons
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import SaveIcon from '@lucide/svelte/icons/save';
	import LoaderIcon from '@lucide/svelte/icons/loader';

	// Form state
	let warehouseId = $state('');
	let notes = $state('');
	let isSubmitting = $state(false);
	let error = $state<string | null>(null);

	// Stock levels cache: Map<productId-warehouseId, availableQuantity>
	let stockLevels = $state<Map<string, number>>(new Map());
	let loadingStockLevels = $state<Set<string>>(new Set());

	// Line items
	interface LineItem {
		id: string;
		productId: string;
		productName: string;
		productSku: string;
		adjustmentType: 'increase' | 'decrease';
		quantity: number;
		reasonCode: AdjustmentReasonCode | '';
		notes: string;
	}

	let lineItems = $state<LineItem[]>([]);

	// Derived: available reason codes based on adjustment type
	function getAvailableReasons(type: 'increase' | 'decrease') {
		return getReasonCodesByDirection(type);
	}

	// Get stock level for a product in the selected warehouse
	function getStockLevel(productId: string): number | null {
		if (!warehouseId || !productId) return null;
		const key = `${productId}-${warehouseId}`;
		return stockLevels.get(key) ?? null;
	}

	// Check if stock level is loading
	function isStockLevelLoading(productId: string): boolean {
		if (!warehouseId || !productId) return false;
		const key = `${productId}-${warehouseId}`;
		return loadingStockLevels.has(key);
	}

	// Fetch stock level for a product
	async function fetchStockLevel(productId: string) {
		if (!warehouseId || !productId) return;

		const key = `${productId}-${warehouseId}`;
		if (stockLevels.has(key) || loadingStockLevels.has(key)) return;

		loadingStockLevels = new Set([...loadingStockLevels, key]);

		try {
			const response = await stockLevelApi.list({
				productId,
				warehouseId
			});

			if (response.success && response.data?.items && response.data.items.length > 0) {
				const level = response.data.items[0];
				stockLevels = new Map([...stockLevels, [key, level.availableQuantity]]);
			} else {
				// No stock level found, set to 0
				stockLevels = new Map([...stockLevels, [key, 0]]);
			}
		} catch {
			// On error, don't show stock level
		} finally {
			loadingStockLevels = new Set([...loadingStockLevels].filter((k) => k !== key));
		}
	}

	// Add new line item
	function addLineItem() {
		lineItems = [
			...lineItems,
			{
				id: crypto.randomUUID(),
				productId: '',
				productName: '',
				productSku: '',
				adjustmentType: 'decrease',
				quantity: 1,
				reasonCode: '',
				notes: ''
			}
		];
	}

	// Remove line item
	function removeLineItem(id: string) {
		lineItems = lineItems.filter((item) => item.id !== id);
	}

	// Update line item
	function updateLineItem(id: string, field: keyof LineItem, value: unknown) {
		lineItems = lineItems.map((item) => {
			if (item.id === id) {
				const updated = { ...item, [field]: value };
				// Reset reason code when adjustment type changes
				if (field === 'adjustmentType') {
					updated.reasonCode = '';
				}
				return updated;
			}
			return item;
		});
	}

	// Handle product selection
	function handleProductSelect(lineId: string, productId: string) {
		const product = productState.items.find((p) => p.productId === productId);
		if (product) {
			lineItems = lineItems.map((item) => {
				if (item.id === lineId) {
					return {
						...item,
						productId: product.productId,
						productName: product.name,
						productSku: product.sku
					};
				}
				return item;
			});
			// Fetch stock level for the selected product
			fetchStockLevel(product.productId);
		}
	}

	// Clear stock levels cache when warehouse changes
	function handleWarehouseChange(newWarehouseId: string) {
		warehouseId = newWarehouseId;
		// Clear cached stock levels as they depend on warehouse
		stockLevels = new Map();
		loadingStockLevels = new Set();
		// Fetch stock levels for all selected products
		lineItems.forEach((item) => {
			if (item.productId) {
				fetchStockLevel(item.productId);
			}
		});
	}

	// Form validation
	let isValid = $derived(() => {
		if (!warehouseId) return false;
		if (lineItems.length === 0) return false;

		return lineItems.every((item) => item.productId && item.quantity > 0 && item.reasonCode);
	});

	// Submit form
	async function handleSubmit() {
		if (!isValid) return;

		isSubmitting = true;
		error = null;

		const items: CreateAdjustmentItem[] = lineItems.map((item) => ({
			productId: item.productId,
			quantity: item.adjustmentType === 'decrease' ? -item.quantity : item.quantity,
			reasonCode: item.reasonCode as AdjustmentReasonCode,
			notes: item.notes || null
		}));

		const result = await adjustmentStore.create({
			warehouseId,
			items,
			notes: notes || null
		});

		isSubmitting = false;

		if (result) {
			toast.success(`Adjustment created successfully with ${lineItems.length} item(s)`);
			goto('/inventory/adjustments');
		} else {
			toast.error('Failed to create adjustment. Please try again.');
			error = 'Failed to create adjustment. Please try again.';
		}
	}

	onMount(async () => {
		await Promise.all([warehouseStore.load(), productStore.load()]);
		// Add initial line item
		addLineItem();
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Breadcrumbs -->
	<div class="flex items-center gap-2 text-sm text-muted-foreground">
		<a href="/inventory" class="hover:text-foreground">Inventory</a>
		<span>/</span>
		<a href="/inventory/adjustments" class="hover:text-foreground">Adjustments</a>
		<span>/</span>
		<span class="text-foreground">New</span>
	</div>

	<!-- Header -->
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="icon" href="/inventory/adjustments">
			<ArrowLeftIcon class="h-4 w-4" />
		</Button>
		<div>
			<h1 class="text-2xl font-bold">New Stock Adjustment</h1>
			<p class="text-muted-foreground">Record inventory changes</p>
		</div>
	</div>

	<!-- Error -->
	{#if error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<p class="text-destructive">{error}</p>
		</div>
	{/if}

	<!-- Form -->
	<form
		onsubmit={(e) => {
			e.preventDefault();
			handleSubmit();
		}}
	>
		<div class="grid gap-6">
			<!-- Header Section -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Adjustment Details</Card.Title>
				</Card.Header>
				<Card.Content class="grid gap-4">
					<div class="grid gap-2">
						<Label for="warehouse">Warehouse *</Label>
						<Select.Root type="single" onValueChange={(v) => handleWarehouseChange(v ?? '')}>
							<Select.Trigger id="warehouse" class="w-full">
								{warehouseState.items.find((w) => w.warehouseId === warehouseId)?.warehouseName ||
									'Select warehouse...'}
							</Select.Trigger>
							<Select.Content>
								{#each warehouseState.items.filter((w) => w.isActive) as warehouse}
									<Select.Item value={warehouse.warehouseId}>
										{warehouse.warehouseName} ({warehouse.warehouseCode})
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>

					<div class="grid gap-2">
						<Label for="notes">Notes (optional)</Label>
						<Textarea
							id="notes"
							placeholder="Add any notes about this adjustment..."
							bind:value={notes}
							rows={3}
						/>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Line Items Section -->
			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between">
					<Card.Title>Adjustment Items</Card.Title>
					<Button type="button" variant="outline" size="sm" onclick={addLineItem}>
						<PlusIcon class="mr-2 h-4 w-4" />
						Add Product
					</Button>
				</Card.Header>
				<Card.Content>
					{#if lineItems.length === 0}
						<div class="flex flex-col items-center justify-center py-8 text-center">
							<p class="text-muted-foreground">No items added yet</p>
							<Button type="button" variant="outline" class="mt-2" onclick={addLineItem}>
								<PlusIcon class="mr-2 h-4 w-4" />
								Add Product
							</Button>
						</div>
					{:else}
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head class="w-[250px]">Product</Table.Head>
									<Table.Head class="w-[80px]">Stock</Table.Head>
									<Table.Head class="w-[120px]">Type</Table.Head>
									<Table.Head class="w-[100px]">Quantity</Table.Head>
									<Table.Head class="w-[180px]">Reason</Table.Head>
									<Table.Head>Notes</Table.Head>
									<Table.Head class="w-[60px]"></Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each lineItems as item (item.id)}
									<Table.Row>
										<Table.Cell>
											<Select.Root
												type="single"
												onValueChange={(v) => handleProductSelect(item.id, v ?? '')}
											>
												<Select.Trigger class="w-full">
													{item.productSku
														? `${item.productSku} - ${item.productName}`
														: 'Select product...'}
												</Select.Trigger>
												<Select.Content>
													{#each productState.items.filter((p) => p.isActive) as product}
														<Select.Item value={product.productId}>
															{product.sku} - {product.name}
														</Select.Item>
													{/each}
												</Select.Content>
											</Select.Root>
										</Table.Cell>
										<Table.Cell>
											{#if !item.productId || !warehouseId}
												<span class="text-muted-foreground">-</span>
											{:else if isStockLevelLoading(item.productId)}
												<span class="text-muted-foreground">...</span>
											{:else}
												{@const stockLevel = getStockLevel(item.productId)}
												{#if stockLevel !== null}
													<Badge
														variant={stockLevel <= 0
															? 'destructive'
															: stockLevel <= 10
																? 'secondary'
																: 'outline'}
													>
														{stockLevel}
													</Badge>
												{:else}
													<span class="text-muted-foreground">-</span>
												{/if}
											{/if}
										</Table.Cell>
										<Table.Cell>
											<Select.Root
												type="single"
												value={item.adjustmentType}
												onValueChange={(v) =>
													updateLineItem(item.id, 'adjustmentType', v ?? 'decrease')}
											>
												<Select.Trigger class="w-full">
													{item.adjustmentType === 'increase' ? 'Increase' : 'Decrease'}
												</Select.Trigger>
												<Select.Content>
													<Select.Item value="increase">Increase (+)</Select.Item>
													<Select.Item value="decrease">Decrease (-)</Select.Item>
												</Select.Content>
											</Select.Root>
										</Table.Cell>
										<Table.Cell>
											<Input
												type="number"
												min="1"
												value={item.quantity}
												onchange={(e) =>
													updateLineItem(
														item.id,
														'quantity',
														parseInt((e.target as HTMLInputElement).value) || 1
													)}
											/>
										</Table.Cell>
										<Table.Cell>
											<Select.Root
												type="single"
												value={item.reasonCode}
												onValueChange={(v) => updateLineItem(item.id, 'reasonCode', v ?? '')}
											>
												<Select.Trigger class="w-full">
													{REASON_CODES.find((r) => r.code === item.reasonCode)?.label ||
														'Select reason...'}
												</Select.Trigger>
												<Select.Content>
													{#each getAvailableReasons(item.adjustmentType) as reason}
														<Select.Item value={reason.code}>{reason.label}</Select.Item>
													{/each}
												</Select.Content>
											</Select.Root>
										</Table.Cell>
										<Table.Cell>
											<Input
												type="text"
												placeholder="Optional notes..."
												value={item.notes}
												onchange={(e) =>
													updateLineItem(item.id, 'notes', (e.target as HTMLInputElement).value)}
											/>
										</Table.Cell>
										<Table.Cell>
											<Button
												type="button"
												variant="ghost"
												size="icon"
												onclick={() => removeLineItem(item.id)}
												disabled={lineItems.length === 1}
											>
												<TrashIcon class="h-4 w-4 text-muted-foreground" />
											</Button>
										</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					{/if}
				</Card.Content>
			</Card.Root>

			<!-- Actions -->
			<div class="flex justify-end gap-4">
				<Button type="button" variant="outline" href="/inventory/adjustments">Cancel</Button>
				<Button type="submit" disabled={!isValid || isSubmitting}>
					{#if isSubmitting}
						<LoaderIcon class="mr-2 h-4 w-4 animate-spin" />
						Saving...
					{:else}
						<SaveIcon class="mr-2 h-4 w-4" />
						Save Adjustment
					{/if}
				</Button>
			</div>
		</div>
	</form>
</div>
