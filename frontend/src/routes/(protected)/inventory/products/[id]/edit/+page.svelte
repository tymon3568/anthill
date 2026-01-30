<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
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
	import { productApi } from '$lib/api/inventory/products';
	import { categoryApi } from '$lib/api/inventory/categories';
	import type {
		ProductResponse,
		ProductUpdateRequest,
		ProductTrackingMethod,
		CategoryResponse,
		BarcodeType
	} from '$lib/types/inventory';
	import ProductImageGallery from '$lib/components/inventory/ProductImageGallery.svelte';

	// Get product ID from URL
	const productId = $derived($page.params.id ?? '');

	// Product data from API
	let existingProduct = $state<ProductResponse | null>(null);
	let isLoading = $state(true);
	let loadError = $state('');

	// UOM options - will be fetched from API when available
	const uomOptions = [
		{ uomId: '', uomName: 'None', uomCode: '-' },
		{ uomId: 'piece', uomName: 'Piece', uomCode: 'PC' },
		{ uomId: 'box', uomName: 'Box', uomCode: 'BOX' },
		{ uomId: 'kg', uomName: 'Kilogram', uomCode: 'KG' },
		{ uomId: 'meter', uomName: 'Meter', uomCode: 'M' }
	];

	// Fetch product on mount
	$effect(() => {
		if (productId) {
			loadProduct();
		}
	});

	async function loadProduct() {
		isLoading = true;
		loadError = '';
		try {
			const result = await productApi.get(productId);
			if (result.success && result.data) {
				existingProduct = result.data;
				initializeForm(result.data);
			} else {
				loadError = result.error || 'Failed to load product';
			}
		} catch (error) {
			loadError = error instanceof Error ? error.message : 'Failed to load product';
		} finally {
			isLoading = false;
		}
	}

	function initializeForm(product: ProductResponse) {
		formData = {
			name: product.name,
			description: product.description || '',
			productType: product.productType,
			barcode: product.barcode || '',
			barcodeType: product.barcodeType || '',
			categoryId: product.categoryId || '',
			trackInventory: product.trackInventory,
			trackingMethod: product.trackingMethod,
			defaultUomId: product.defaultUomId,
			salePrice: product.salePrice != null ? product.salePrice / 100 : 0, // Convert from cents
			costPrice: product.costPrice != null ? product.costPrice / 100 : 0,
			currencyCode: product.currencyCode,
			weightGrams: product.weightGrams || 0,
			isActive: product.isActive,
			isSellable: product.isSellable,
			isPurchaseable: product.isPurchaseable
		};
		sku = product.sku;
		lengthMm = (product.dimensions?.lengthMm as number) || 0;
		widthMm = (product.dimensions?.widthMm as number) || 0;
		heightMm = (product.dimensions?.heightMm as number) || 0;
	}

	// Form state - separate from API types for UI binding
	interface FormData {
		name: string;
		description: string;
		productType: string;
		barcode: string;
		barcodeType: BarcodeType | '';
		categoryId: string;
		trackInventory: boolean;
		trackingMethod: ProductTrackingMethod;
		defaultUomId?: string | null;
		salePrice: number;
		costPrice: number;
		currencyCode: string;
		weightGrams: number;
		isActive: boolean;
		isSellable: boolean;
		isPurchaseable: boolean;
	}
	let formData = $state<FormData>({
		name: '',
		description: '',
		productType: 'goods',
		barcode: '',
		barcodeType: '',
		categoryId: '',
		trackInventory: true,
		trackingMethod: 'none',
		defaultUomId: null,
		salePrice: 0,
		costPrice: 0,
		currencyCode: 'VND',
		weightGrams: 0,
		isActive: true,
		isSellable: true,
		isPurchaseable: true
	});
	let sku = $state('');
	let lengthMm = $state(0);
	let widthMm = $state(0);
	let heightMm = $state(0);

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

	function validateForm(): boolean {
		errors = {};

		if (!sku.trim()) {
			errors.sku = 'SKU is required';
		}
		if (!formData.name?.trim()) {
			errors.name = 'Name is required';
		}
		if (formData.salePrice < 0) {
			errors.salePrice = 'Price cannot be negative';
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();

		if (!validateForm()) return;

		isSubmitting = true;
		submitError = '';

		try {
			// Build update payload with camelCase dimensions (backend uses serde rename_all = "camelCase")
			const payload: ProductUpdateRequest = {
				name: formData.name.trim(),
				description: formData.description.trim() || null,
				productType: formData.productType,
				barcode: formData.barcode.trim() || null,
				barcodeType: formData.barcodeType || null,
				categoryId: formData.categoryId || null,
				trackInventory: formData.trackInventory,
				trackingMethod: formData.trackingMethod,
				defaultUomId: formData.defaultUomId || null,
				salePrice: Math.round(formData.salePrice * 100), // Convert to cents
				costPrice: formData.costPrice ? Math.round(formData.costPrice * 100) : null,
				currencyCode: formData.currencyCode,
				weightGrams: formData.weightGrams || null,
				dimensions: {
					lengthMm: lengthMm || undefined,
					widthMm: widthMm || undefined,
					heightMm: heightMm || undefined
				},
				isActive: formData.isActive,
				isSellable: formData.isSellable,
				isPurchaseable: formData.isPurchaseable
			};

			const result = await productApi.update(productId, payload);

			if (result.success) {
				goto(`/inventory/products/${productId}`);
			} else {
				submitError = result.error || 'Failed to update product';
			}
		} catch (error) {
			submitError = error instanceof Error ? error.message : 'Failed to update product';
		} finally {
			isSubmitting = false;
		}
	}
</script>

<svelte:head>
	<title>Edit {existingProduct?.name || 'Product'} - Anthill</title>
</svelte:head>

{#if isLoading}
	<div class="flex min-h-[50vh] items-center justify-center">
		<div class="text-center">
			<div
				class="mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"
			></div>
			<p class="text-muted-foreground">Loading product...</p>
		</div>
	</div>
{:else if loadError}
	<div class="flex min-h-[50vh] items-center justify-center">
		<Card class="w-full max-w-md">
			<CardContent class="pt-6 text-center">
				<p class="mb-2 text-destructive">Error loading product</p>
				<p class="text-sm text-muted-foreground">{loadError}</p>
				<Button href="/inventory/products" class="mt-4">Back to Products</Button>
			</CardContent>
		</Card>
	</div>
{:else if !existingProduct}
	<div class="flex min-h-[50vh] items-center justify-center">
		<Card class="w-full max-w-md">
			<CardContent class="pt-6 text-center">
				<p class="text-muted-foreground">Product not found</p>
				<Button href="/inventory/products" class="mt-4">Back to Products</Button>
			</CardContent>
		</Card>
	</div>
{:else}
	<div class="mx-auto max-w-3xl space-y-6">
		<div class="flex items-center justify-between">
			<div>
				<div class="flex items-center gap-2">
					<Button variant="ghost" size="sm" href="/inventory/products/{existingProduct.productId}"
						>← Back</Button
					>
				</div>
				<h1 class="text-2xl font-bold">Edit Product</h1>
				<p class="font-mono text-muted-foreground">{existingProduct.sku}</p>
			</div>
			<Button variant="outline" href="/inventory/products/{existingProduct.productId}"
				>Cancel</Button
			>
		</div>

		<form onsubmit={handleSubmit} class="space-y-6">
			<!-- Basic Information -->
			<Card>
				<CardHeader>
					<CardTitle>Basic Information</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4">
					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label for="name">Name *</Label>
							<Input
								id="name"
								bind:value={formData.name}
								class={errors.name ? 'border-destructive' : ''}
							/>
							{#if errors.name}
								<p class="text-sm text-destructive">{errors.name}</p>
							{/if}
						</div>
						<div class="space-y-2">
							<Label for="sku">SKU *</Label>
							<Input
								id="sku"
								bind:value={sku}
								disabled
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
							bind:value={formData.description}
							rows="3"
							class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						></textarea>
					</div>

					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label for="productType">Product Type</Label>
							<select
								id="productType"
								bind:value={formData.productType}
								class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
							>
								<option value="goods">Goods</option>
								<option value="service">Service</option>
								<option value="consumable">Consumable</option>
							</select>
						</div>
						<div class="space-y-2">
							<Label for="defaultUomId">Default UOM</Label>
							<select
								id="defaultUomId"
								bind:value={formData.defaultUomId}
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
							bind:value={formData.categoryId}
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

					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label for="barcode">Barcode</Label>
							<Input id="barcode" bind:value={formData.barcode} placeholder="e.g., 5901234123457" />
						</div>
						<div class="space-y-2">
							<Label for="barcodeType">Barcode Type</Label>
							<select
								id="barcodeType"
								bind:value={formData.barcodeType}
								class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
							>
								<option value="">Select type...</option>
								<option value="ean13">EAN-13</option>
								<option value="upc_a">UPC-A</option>
								<option value="isbn">ISBN</option>
								<option value="custom">Custom</option>
							</select>
							<p class="text-xs text-muted-foreground">
								{#if formData.barcodeType === 'ean13'}
									European Article Number (13 digits)
								{:else if formData.barcodeType === 'upc_a'}
									Universal Product Code (12 digits)
								{:else if formData.barcodeType === 'isbn'}
									International Standard Book Number
								{:else if formData.barcodeType === 'custom'}
									Custom barcode format
								{:else}
									Select a barcode type for scanning
								{/if}
							</p>
						</div>
					</div>
				</CardContent>
			</Card>

			<!-- Product Images -->
			<ProductImageGallery {productId} />

			<!-- Pricing -->
			<Card>
				<CardHeader>
					<CardTitle>Pricing</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4">
					<div class="grid grid-cols-3 gap-4">
						<div class="space-y-2">
							<Label for="salePrice">Sale Price</Label>
							<div class="relative">
								<Input
									id="salePrice"
									type="number"
									step="1000"
									min="0"
									bind:value={formData.salePrice}
									class="pr-16"
								/>
								<span
									class="absolute top-1/2 right-3 -translate-y-1/2 text-sm text-muted-foreground"
								>
									{formData.currencyCode}
								</span>
							</div>
						</div>
						<div class="space-y-2">
							<Label for="costPrice">Cost Price</Label>
							<div class="relative">
								<Input
									id="costPrice"
									type="number"
									step="1000"
									min="0"
									bind:value={formData.costPrice}
									class="pr-16"
								/>
								<span
									class="absolute top-1/2 right-3 -translate-y-1/2 text-sm text-muted-foreground"
								>
									{formData.currencyCode}
								</span>
							</div>
						</div>
						<div class="space-y-2">
							<Label for="currencyCode">Currency</Label>
							<select
								id="currencyCode"
								bind:value={formData.currencyCode}
								class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
							>
								<option value="VND">VND</option>
								<option value="USD">USD</option>
							</select>
						</div>
					</div>
				</CardContent>
			</Card>

			<!-- Inventory Settings -->
			<Card>
				<CardHeader>
					<CardTitle>Inventory Settings</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4">
					<label class="flex items-center gap-2">
						<input
							type="checkbox"
							bind:checked={formData.trackInventory}
							class="rounded"
							disabled={formData.productType === 'service'}
						/>
						<span>Track inventory for this product</span>
					</label>

					{#if formData.trackInventory}
						<div class="space-y-2">
							<Label for="trackingMethod">Tracking Method</Label>
							<select
								id="trackingMethod"
								bind:value={formData.trackingMethod}
								class="w-full max-w-xs rounded-md border border-input bg-background px-3 py-2 text-sm"
							>
								<option value="none">No tracking</option>
								<option value="lot">Lot/Batch tracking</option>
								<option value="serial">Serial number tracking</option>
							</select>
						</div>
					{/if}
				</CardContent>
			</Card>

			<!-- Physical Attributes -->
			<Card>
				<CardHeader>
					<CardTitle>Physical Attributes</CardTitle>
				</CardHeader>
				<CardContent>
					<div class="grid grid-cols-4 gap-4">
						<div class="space-y-2">
							<Label for="weightGrams">Weight (g)</Label>
							<Input id="weightGrams" type="number" min="0" bind:value={formData.weightGrams} />
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
							<input type="checkbox" bind:checked={formData.isActive} class="rounded" />
							<span>Active</span>
						</label>
						<label class="flex items-center gap-2">
							<input type="checkbox" bind:checked={formData.isSellable} class="rounded" />
							<span>Can be sold</span>
						</label>
						<label class="flex items-center gap-2">
							<input type="checkbox" bind:checked={formData.isPurchaseable} class="rounded" />
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
					<Button
						type="button"
						variant="outline"
						href="/inventory/products/{existingProduct.productId}"
					>
						Cancel
					</Button>
					<Button type="submit" disabled={isSubmitting}>
						{#if isSubmitting}
							Saving...
						{:else}
							Save Changes
						{/if}
					</Button>
				</div>
			</div>
		</form>
	</div>
{/if}
