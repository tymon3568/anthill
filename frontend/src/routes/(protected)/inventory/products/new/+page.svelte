<script lang="ts">
	import {
		Card,
		CardContent,
		CardHeader,
		CardTitle,
		CardDescription
	} from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { productApi } from '$lib/api/inventory/products';
	import { categoryApi } from '$lib/api/inventory/categories';
	import type { ProductCreateRequest, CategoryResponse } from '$lib/types/inventory';
	import { goto } from '$app/navigation';

	// UOM options - will be fetched from API when UOM endpoint is available
	// For now, use static list matching common use cases
	const uomOptions = [
		{ uomId: '', uomName: 'None', uomCode: '-' },
		{ uomId: 'piece', uomName: 'Piece', uomCode: 'PC' },
		{ uomId: 'box', uomName: 'Box', uomCode: 'BOX' },
		{ uomId: 'kg', uomName: 'Kilogram', uomCode: 'KG' },
		{ uomId: 'meter', uomName: 'Meter', uomCode: 'M' }
	];

	// Form state based on database schema
	let sku = $state('');
	let name = $state('');
	let description = $state('');
	let productType = $state<'goods' | 'service' | 'consumable'>('goods');
	let categoryId = $state('');
	let trackInventory = $state(true);
	let trackingMethod = $state<'none' | 'lot' | 'serial'>('none');
	let defaultUomId = $state('');
	let salePrice = $state(0);
	let costPrice = $state(0);
	let currencyCode = $state('VND');
	let weightGrams = $state(0);
	let isActive = $state(true);
	let isSellable = $state(true);
	let isPurchaseable = $state(true);

	// UI state
	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});
	let submitError = $state('');

	// Categories state
	let categories = $state<CategoryResponse[]>([]);

	$effect(() => {
		fetchCategories();
	});

	async function fetchCategories() {
		// Use getTree() to fetch all categories - list() has a max pageSize of 100
		const result = await categoryApi.getTree();
		if (result.success && result.data) {
			categories = result.data;
		}
	}

	// For dimensions binding
	let lengthMm = $state(0);
	let widthMm = $state(0);
	let heightMm = $state(0);

	// Auto-generate SKU from name
	let autoGenerateSku = $state(true);
	$effect(() => {
		if (autoGenerateSku && name) {
			sku = name
				.toUpperCase()
				.replace(/[^A-Z0-9]/g, '-')
				.replace(/-+/g, '-')
				.slice(0, 20);
		}
	});

	function validateForm(): boolean {
		errors = {};

		if (!sku.trim()) {
			errors.sku = 'SKU is required';
		} else if (sku.length > 100) {
			errors.sku = 'SKU must be 100 characters or less';
		}

		if (!name.trim()) {
			errors.name = 'Name is required';
		} else if (name.length > 255) {
			errors.name = 'Name must be 255 characters or less';
		}

		if (salePrice < 0) {
			errors.salePrice = 'Price cannot be negative';
		}

		if (costPrice < 0) {
			errors.costPrice = 'Cost price cannot be negative';
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();

		if (!validateForm()) return;

		isSubmitting = true;
		submitError = '';

		try {
			const payload: ProductCreateRequest = {
				sku: sku.trim(),
				name: name.trim(),
				description: description.trim() || undefined,
				productType,
				categoryId: categoryId || undefined,
				trackInventory,
				trackingMethod,
				defaultUomId: defaultUomId || undefined,
				salePrice: Math.round(salePrice * 100), // Convert to cents
				costPrice: costPrice > 0 ? Math.round(costPrice * 100) : undefined,
				currencyCode,
				weightGrams: weightGrams > 0 ? weightGrams : undefined,
				dimensions:
					lengthMm > 0 || widthMm > 0 || heightMm > 0
						? {
								lengthMm: lengthMm || undefined,
								widthMm: widthMm || undefined,
								heightMm: heightMm || undefined
							}
						: undefined,
				isActive,
				isSellable,
				isPurchaseable
			};

			const result = await productApi.create(payload);

			if (result.success && result.data) {
				goto('/inventory/products');
			} else {
				submitError = result.error || 'Failed to create product';
			}
		} catch (error) {
			submitError = error instanceof Error ? error.message : 'Failed to create product';
		} finally {
			isSubmitting = false;
		}
	}

	function handleSkuInput() {
		autoGenerateSku = false;
	}
</script>

<svelte:head>
	<title>New Product - Anthill</title>
</svelte:head>

<div class="mx-auto max-w-3xl space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">New Product</h1>
			<p class="text-muted-foreground">Add a new product to your catalog</p>
		</div>
		<Button variant="outline" href="/inventory/products">Cancel</Button>
	</div>

	<form onsubmit={handleSubmit} class="space-y-6">
		<!-- Basic Information -->
		<Card>
			<CardHeader>
				<CardTitle>Basic Information</CardTitle>
				<CardDescription>Enter the product's basic details</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="name">Name *</Label>
						<Input
							id="name"
							bind:value={name}
							placeholder="Product name"
							class={errors.name ? 'border-destructive' : ''}
						/>
						{#if errors.name}
							<p class="text-sm text-destructive">{errors.name}</p>
						{/if}
					</div>
					<div class="space-y-2">
						<div class="flex items-center justify-between">
							<Label for="sku">SKU *</Label>
							<label class="flex items-center gap-2 text-xs text-muted-foreground">
								<input type="checkbox" bind:checked={autoGenerateSku} class="rounded" />
								Auto-generate
							</label>
						</div>
						<Input
							id="sku"
							bind:value={sku}
							placeholder="PROD-001"
							oninput={handleSkuInput}
							class={errors.sku ? 'border-destructive' : ''}
						/>
						{#if errors.sku}
							<p class="text-sm text-destructive">{errors.sku}</p>
						{/if}
					</div>
				</div>

				<div class="space-y-2">
					<Label for="description">Description</Label>
					<textarea
						id="description"
						bind:value={description}
						placeholder="Product description"
						rows="3"
						class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
					></textarea>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="productType">Product Type *</Label>
						<select
							id="productType"
							bind:value={productType}
							class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						>
							<option value="goods">Goods</option>
							<option value="service">Service</option>
							<option value="consumable">Consumable</option>
						</select>
						<p class="text-xs text-muted-foreground">
							{#if productType === 'goods'}
								Physical products that are tracked in inventory
							{:else if productType === 'service'}
								Intangible services (no inventory tracking)
							{:else}
								Consumable items used in operations
							{/if}
						</p>
					</div>
					<div class="space-y-2">
						<Label for="defaultUomId">Default Unit of Measure</Label>
						<select
							id="defaultUomId"
							bind:value={defaultUomId}
							class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						>
							{#each uomOptions as uom}
								<option value={uom.uomId}>{uom.uomName} ({uom.uomCode})</option>
							{/each}
						</select>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="categoryId">Category</Label>
					<select
						id="categoryId"
						bind:value={categoryId}
						class="w-full max-w-xs rounded-md border border-input bg-background px-3 py-2 text-sm"
					>
						<option value="">No category</option>
						{#each categories as category}
							<option value={category.categoryId}>
								{'—'.repeat(category.level)}{category.level > 0 ? ' ' : ''}{category.name}
							</option>
						{/each}
					</select>
				</div>
			</CardContent>
		</Card>

		<!-- Pricing -->
		<Card>
			<CardHeader>
				<CardTitle>Pricing</CardTitle>
				<CardDescription>Set the product's pricing information</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="grid grid-cols-3 gap-4">
					<div class="space-y-2">
						<Label for="salePrice">Sale Price *</Label>
						<div class="relative">
							<Input
								id="salePrice"
								type="number"
								step="1000"
								min="0"
								bind:value={salePrice}
								class={errors.salePrice ? 'border-destructive pr-16' : 'pr-16'}
							/>
							<span class="absolute top-1/2 right-3 -translate-y-1/2 text-sm text-muted-foreground">
								{currencyCode}
							</span>
						</div>
						{#if errors.salePrice}
							<p class="text-sm text-destructive">{errors.salePrice}</p>
						{/if}
					</div>
					<div class="space-y-2">
						<Label for="costPrice">Cost Price</Label>
						<div class="relative">
							<Input
								id="costPrice"
								type="number"
								step="1000"
								min="0"
								bind:value={costPrice}
								class={errors.costPrice ? 'border-destructive pr-16' : 'pr-16'}
							/>
							<span class="absolute top-1/2 right-3 -translate-y-1/2 text-sm text-muted-foreground">
								{currencyCode}
							</span>
						</div>
						{#if errors.costPrice}
							<p class="text-sm text-destructive">{errors.costPrice}</p>
						{/if}
					</div>
					<div class="space-y-2">
						<Label for="currencyCode">Currency</Label>
						<select
							id="currencyCode"
							bind:value={currencyCode}
							class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						>
							<option value="VND">VND - Vietnamese Dong</option>
							<option value="USD">USD - US Dollar</option>
						</select>
					</div>
				</div>

				{#if costPrice && salePrice > 0}
					{@const margin = ((salePrice - costPrice) / salePrice) * 100}
					<div class="rounded-lg bg-muted p-3">
						<span class="text-sm">
							Profit Margin:
							<span
								class="font-medium"
								class:text-green-600={margin > 0}
								class:text-red-600={margin < 0}
							>
								{margin.toFixed(1)}%
							</span>
						</span>
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Inventory Settings -->
		<Card>
			<CardHeader>
				<CardTitle>Inventory Settings</CardTitle>
				<CardDescription>Configure how this product is tracked in inventory</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="flex items-center gap-2">
					<input
						type="checkbox"
						id="trackInventory"
						bind:checked={trackInventory}
						class="rounded"
						disabled={productType === 'service'}
					/>
					<Label for="trackInventory" class="font-normal">Track inventory for this product</Label>
				</div>

				{#if trackInventory}
					<div class="space-y-2">
						<Label for="trackingMethod">Tracking Method</Label>
						<select
							id="trackingMethod"
							bind:value={trackingMethod}
							class="w-full max-w-xs rounded-md border border-input bg-background px-3 py-2 text-sm"
						>
							<option value="none">No tracking (quantity only)</option>
							<option value="lot">Lot/Batch tracking</option>
							<option value="serial">Serial number tracking</option>
						</select>
						<p class="text-xs text-muted-foreground">
							{#if trackingMethod === 'none'}
								Track only quantity, no additional traceability
							{:else if trackingMethod === 'lot'}
								Track by lot/batch number with expiry dates
							{:else}
								Track individual items by unique serial numbers
							{/if}
						</p>
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Physical Attributes -->
		<Card>
			<CardHeader>
				<CardTitle>Physical Attributes</CardTitle>
				<CardDescription>Optional physical specifications</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="grid grid-cols-4 gap-4">
					<div class="space-y-2">
						<Label for="weightGrams">Weight (grams)</Label>
						<Input id="weightGrams" type="number" min="0" bind:value={weightGrams} />
					</div>
					<div class="space-y-2">
						<Label for="lengthMm">Length (mm)</Label>
						<Input id="lengthMm" type="number" min="0" bind:value={lengthMm} />
					</div>
					<div class="space-y-2">
						<Label for="widthMm">Width (mm)</Label>
						<Input id="widthMm" type="number" min="0" bind:value={widthMm} />
					</div>
					<div class="space-y-2">
						<Label for="heightMm">Height (mm)</Label>
						<Input id="heightMm" type="number" min="0" bind:value={heightMm} />
					</div>
				</div>
			</CardContent>
		</Card>

		<!-- Status -->
		<Card>
			<CardHeader>
				<CardTitle>Status</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="flex flex-wrap gap-6">
					<label class="flex items-center gap-2">
						<input type="checkbox" bind:checked={isActive} class="rounded" />
						<span>Active</span>
					</label>
					<label class="flex items-center gap-2">
						<input type="checkbox" bind:checked={isSellable} class="rounded" />
						<span>Can be sold</span>
					</label>
					<label class="flex items-center gap-2">
						<input type="checkbox" bind:checked={isPurchaseable} class="rounded" />
						<span>Can be purchased</span>
					</label>
				</div>
			</CardContent>
		</Card>

		<!-- Submit -->
		<div class="flex flex-col gap-4">
			{#if submitError}
				<div class="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
					{submitError}
				</div>
			{/if}
			<div class="flex justify-end gap-4">
				<Button type="button" variant="outline" href="/inventory/products">Cancel</Button>
				<Button type="submit" disabled={isSubmitting}>
					{#if isSubmitting}
						Creating...
					{:else}
						Create Product
					{/if}
				</Button>
			</div>
		</div>
	</form>
</div>
