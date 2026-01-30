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
	import { Badge } from '$lib/components/ui/badge';
	import * as Tabs from '$lib/components/ui/tabs';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { productApi } from '$lib/api/inventory/products';
	import { categoryApi } from '$lib/api/inventory/categories';
	import { variantsApi } from '$lib/api/products';
	import type { ProductResponse, CategoryResponse } from '$lib/types/inventory';
	import type { ProductVariant } from '$lib/types/products';
	import ProductImageGallery from '$lib/components/inventory/ProductImageGallery.svelte';

	// Get product ID from URL
	const productId = $derived($page.params.id ?? '');

	// Product data from API
	let product = $state<ProductResponse | null>(null);
	let isLoading = $state(true);
	let loadError = $state('');

	// Variants from API
	let variants = $state<ProductVariant[]>([]);
	let isLoadingVariants = $state(false);
	let variantsError = $state('');

	// UOM options - static list (UOM API endpoint not implemented yet)
	// Using same static list as new/edit pages for consistency
	const uomOptions = [
		{ uomId: '', uomName: 'None', uomCode: '-' },
		{ uomId: 'piece', uomName: 'Piece', uomCode: 'PC' },
		{ uomId: 'box', uomName: 'Box', uomCode: 'BOX' },
		{ uomId: 'kg', uomName: 'Kilogram', uomCode: 'KG' },
		{ uomId: 'meter', uomName: 'Meter', uomCode: 'M' }
	];

	// Categories from API
	let categories = $state<CategoryResponse[]>([]);

	// UI state
	let activeTab = $state('details');
	let deleteDialogOpen = $state(false);
	let isDeleting = $state(false);
	let deleteError = $state('');

	// Variant form state
	let variantDialogOpen = $state(false);
	let editingVariant = $state<ProductVariant | null>(null);
	let isSavingVariant = $state(false);
	let variantSaveError = $state('');
	let variantForm = $state({
		sku: '',
		barcode: '',
		priceDifference: 0,
		isActive: true,
		attributes: [{ key: '', value: '' }]
	});

	// Delete variant state
	let deleteVariantDialogOpen = $state(false);
	let deletingVariant = $state<ProductVariant | null>(null);
	let isDeletingVariant = $state(false);
	let deleteVariantError = $state('');

	// Fetch product on mount
	$effect(() => {
		if (productId) {
			loadProduct();
			loadVariants();
			loadCategories();
		}
	});

	async function loadProduct() {
		isLoading = true;
		loadError = '';
		try {
			const result = await productApi.get(productId);
			if (result.success && result.data) {
				product = result.data;
			} else {
				loadError = result.error || 'Failed to load product';
			}
		} catch (error) {
			loadError = error instanceof Error ? error.message : 'Failed to load product';
		} finally {
			isLoading = false;
		}
	}

	async function loadVariants() {
		isLoadingVariants = true;
		variantsError = '';
		try {
			const result = await variantsApi.list(productId);
			if (result.success && result.data) {
				// Handle both array and paginated response formats
				const data = result.data as unknown;
				if (Array.isArray(data)) {
					variants = data;
				} else if (data && typeof data === 'object' && 'variants' in data) {
					variants = (data as { variants: ProductVariant[] }).variants;
				} else {
					variants = [];
				}
			} else {
				variantsError = result.error || 'Failed to load variants';
			}
		} catch (error) {
			variantsError = error instanceof Error ? error.message : 'Failed to load variants';
		} finally {
			isLoadingVariants = false;
		}
	}

	async function loadCategories() {
		try {
			// Use getTree() to fetch all categories - list() has a max pageSize of 100
			const result = await categoryApi.getTree();
			if (result.success && result.data) {
				categories = result.data;
			}
		} catch {
			// Silently fail - categories are optional display data
		}
	}

	function formatCurrency(cents: number, currency: string = 'VND'): string {
		const value = cents / 100;
		return new Intl.NumberFormat('vi-VN', {
			style: 'currency',
			currency: currency,
			minimumFractionDigits: 0
		}).format(value);
	}

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleDateString('vi-VN', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function getUomName(uomId?: string | null): string {
		if (!uomId) return '-';
		const uom = uomOptions.find((u) => u.uomId === uomId);
		return uom ? `${uom.uomName} (${uom.uomCode})` : uomId;
	}

	function getUomDisplayText(): string {
		return getUomName(product?.defaultUomId);
	}

	function getCategoryName(categoryId?: string | null): string {
		if (!categoryId) return '-';
		const category = categories.find((c) => c.categoryId === categoryId);
		return category ? category.name : '-';
	}

	function getTypeBadgeClass(type: string): string {
		switch (type) {
			case 'goods':
				return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
			case 'service':
				return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
			case 'consumable':
				return 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200';
			default:
				return '';
		}
	}

	async function handleDelete() {
		isDeleting = true;
		deleteError = '';
		try {
			const result = await productApi.delete(productId);
			if (result.success) {
				goto('/inventory/products');
			} else {
				deleteError = result.error || 'Failed to delete product';
			}
		} catch (error) {
			deleteError = error instanceof Error ? error.message : 'Failed to delete product';
		} finally {
			isDeleting = false;
		}
	}

	function openVariantDialog(variant?: ProductVariant) {
		variantSaveError = '';
		if (variant) {
			editingVariant = variant;
			variantForm = {
				sku: variant.sku,
				barcode: variant.barcode || '',
				priceDifference: variant.priceDifference / 100,
				isActive: variant.isActive,
				attributes: Object.entries(variant.variantAttributes).map(([key, value]) => ({
					key,
					value
				}))
			};
		} else {
			editingVariant = null;
			variantForm = {
				sku: product ? `${product.sku}-` : '',
				barcode: '',
				priceDifference: 0,
				isActive: true,
				attributes: [{ key: '', value: '' }]
			};
		}
		variantDialogOpen = true;
	}

	function addAttributeRow() {
		variantForm.attributes = [...variantForm.attributes, { key: '', value: '' }];
	}

	function removeAttributeRow(index: number) {
		variantForm.attributes = variantForm.attributes.filter((_, i) => i !== index);
	}

	async function handleSaveVariant() {
		isSavingVariant = true;
		variantSaveError = '';

		try {
			// Build variant attributes from form
			const variantAttributes: Record<string, string> = {};
			for (const attr of variantForm.attributes) {
				if (attr.key.trim() && attr.value.trim()) {
					variantAttributes[attr.key.trim()] = attr.value.trim();
				}
			}

			const variantData = {
				sku: variantForm.sku,
				barcode: variantForm.barcode || undefined,
				priceDifference: Math.round(variantForm.priceDifference * 100), // Convert to cents
				isActive: variantForm.isActive,
				variantAttributes
			};

			if (editingVariant) {
				// Update existing variant
				const result = await variantsApi.update(productId, editingVariant.variantId, variantData);
				if (result.success && result.data) {
					// Update in local state
					variants = variants.map((v) =>
						v.variantId === editingVariant!.variantId ? result.data! : v
					);
					variantDialogOpen = false;
				} else {
					variantSaveError = result.error || 'Failed to update variant';
				}
			} else {
				// Create new variant
				const result = await variantsApi.create(productId, variantData);
				if (result.success && result.data) {
					variants = [...variants, result.data];
					variantDialogOpen = false;
				} else {
					variantSaveError = result.error || 'Failed to create variant';
				}
			}
		} catch (error) {
			variantSaveError = error instanceof Error ? error.message : 'Failed to save variant';
		} finally {
			isSavingVariant = false;
		}
	}

	function openDeleteVariantDialog(variant: ProductVariant) {
		deletingVariant = variant;
		deleteVariantError = '';
		deleteVariantDialogOpen = true;
	}

	async function handleDeleteVariant() {
		if (!deletingVariant) return;

		isDeletingVariant = true;
		deleteVariantError = '';

		try {
			const result = await variantsApi.delete(productId, deletingVariant.variantId);
			if (result.success) {
				variants = variants.filter((v) => v.variantId !== deletingVariant!.variantId);
				deleteVariantDialogOpen = false;
				deletingVariant = null;
			} else {
				deleteVariantError = result.error || 'Failed to delete variant';
			}
		} catch (error) {
			deleteVariantError = error instanceof Error ? error.message : 'Failed to delete variant';
		} finally {
			isDeletingVariant = false;
		}
	}
</script>

<svelte:head>
	<title>{product?.name || 'Product'} - Anthill</title>
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
{:else if !product}
	<div class="flex min-h-[50vh] items-center justify-center">
		<Card class="w-full max-w-md">
			<CardContent class="pt-6 text-center">
				<p class="text-muted-foreground">Product not found</p>
				<Button href="/inventory/products" class="mt-4">Back to Products</Button>
			</CardContent>
		</Card>
	</div>
{:else}
	<div class="space-y-6">
		<!-- Header -->
		<div class="flex items-start justify-between">
			<div class="space-y-1">
				<div class="flex items-center gap-2">
					<Button variant="ghost" size="sm" href="/inventory/products">← Back</Button>
				</div>
				<h1 class="text-2xl font-bold">{product.name}</h1>
				<div class="flex items-center gap-3">
					<span class="font-mono text-sm text-muted-foreground">{product.sku}</span>
					<Badge variant={product.isActive ? 'default' : 'secondary'}>
						{product.isActive ? 'Active' : 'Inactive'}
					</Badge>
					<span
						class="rounded-full px-2 py-0.5 text-xs font-medium {getTypeBadgeClass(
							product.productType
						)}"
					>
						{product.productType}
					</span>
				</div>
			</div>
			<div class="flex gap-2">
				<Button variant="outline" href="/inventory/products/{product.productId}/edit">Edit</Button>
				<Button variant="destructive" onclick={() => (deleteDialogOpen = true)}>Delete</Button>
			</div>
		</div>

		<!-- Tabs -->
		<Tabs.Root bind:value={activeTab} class="w-full">
			<Tabs.List>
				<Tabs.Trigger value="details">Details</Tabs.Trigger>
				<Tabs.Trigger value="images">Images</Tabs.Trigger>
				<Tabs.Trigger value="pricing">Pricing</Tabs.Trigger>
				<Tabs.Trigger value="inventory">Inventory</Tabs.Trigger>
				<Tabs.Trigger value="variants">
					Variants
					{#if variants.length > 0}
						<Badge variant="secondary" class="ml-2">{variants.length}</Badge>
					{/if}
				</Tabs.Trigger>
			</Tabs.List>

			<!-- Details Tab -->
			<Tabs.Content value="details" class="mt-6 space-y-6">
				<Card>
					<CardHeader>
						<CardTitle>Basic Information</CardTitle>
					</CardHeader>
					<CardContent>
						<dl class="grid grid-cols-2 gap-4">
							<div>
								<dt class="text-sm text-muted-foreground">Name</dt>
								<dd class="font-medium">{product.name}</dd>
							</div>
							<div>
								<dt class="text-sm text-muted-foreground">SKU</dt>
								<dd class="font-mono">{product.sku}</dd>
							</div>
							<div class="col-span-2">
								<dt class="text-sm text-muted-foreground">Description</dt>
								<dd>{product.description || '-'}</dd>
							</div>
							<div>
								<dt class="text-sm text-muted-foreground">Product Type</dt>
								<dd class="capitalize">{product.productType}</dd>
							</div>
							<div>
								<dt class="text-sm text-muted-foreground">Category</dt>
								<dd>{getCategoryName(product.categoryId)}</dd>
							</div>
							<div>
								<dt class="text-sm text-muted-foreground">Default UOM</dt>
								<dd>{getUomDisplayText()}</dd>
							</div>
						</dl>
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle>Physical Attributes</CardTitle>
					</CardHeader>
					<CardContent>
						<dl class="grid grid-cols-4 gap-4">
							<div>
								<dt class="text-sm text-muted-foreground">Weight</dt>
								<dd>{product.weightGrams ? `${product.weightGrams}g` : '-'}</dd>
							</div>
							<div>
								<dt class="text-sm text-muted-foreground">Length</dt>
								<dd>{product.dimensions?.length_mm ? `${product.dimensions.length_mm}mm` : '-'}</dd>
							</div>
							<div>
								<dt class="text-sm text-muted-foreground">Width</dt>
								<dd>{product.dimensions?.width_mm ? `${product.dimensions.width_mm}mm` : '-'}</dd>
							</div>
							<div>
								<dt class="text-sm text-muted-foreground">Height</dt>
								<dd>{product.dimensions?.height_mm ? `${product.dimensions.height_mm}mm` : '-'}</dd>
							</div>
						</dl>
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle>Status & Flags</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="flex gap-6">
							<div class="flex items-center gap-2">
								<span
									class="h-2 w-2 rounded-full {product.isActive ? 'bg-green-500' : 'bg-gray-400'}"
								></span>
								<span>Active</span>
							</div>
							<div class="flex items-center gap-2">
								<span
									class="h-2 w-2 rounded-full {product.isSellable ? 'bg-green-500' : 'bg-gray-400'}"
								></span>
								<span>Sellable</span>
							</div>
							<div class="flex items-center gap-2">
								<span
									class="h-2 w-2 rounded-full {product.isPurchaseable
										? 'bg-green-500'
										: 'bg-gray-400'}"
								></span>
								<span>Purchaseable</span>
							</div>
						</div>
					</CardContent>
				</Card>

				<div class="text-sm text-muted-foreground">
					Created: {formatDate(product.createdAt)} • Updated: {formatDate(product.updatedAt)}
				</div>
			</Tabs.Content>

			<!-- Images Tab -->
			<Tabs.Content value="images" class="mt-6">
				<ProductImageGallery {productId} />
			</Tabs.Content>

			<!-- Pricing Tab -->
			<Tabs.Content value="pricing" class="mt-6">
				<Card>
					<CardHeader>
						<CardTitle>Pricing Information</CardTitle>
					</CardHeader>
					<CardContent>
						<dl class="grid grid-cols-3 gap-6">
							<div>
								<dt class="text-sm text-muted-foreground">Sale Price</dt>
								<dd class="text-2xl font-bold">
									{product.salePrice != null
										? formatCurrency(product.salePrice, product.currencyCode)
										: '-'}
								</dd>
							</div>
							<div>
								<dt class="text-sm text-muted-foreground">Cost Price</dt>
								<dd class="text-2xl font-medium">
									{product.costPrice != null
										? formatCurrency(product.costPrice, product.currencyCode)
										: '-'}
								</dd>
							</div>
							{#if product.costPrice != null && product.salePrice != null && product.salePrice > 0}
								{@const margin =
									((product.salePrice - product.costPrice) / product.salePrice) * 100}
								<div>
									<dt class="text-sm text-muted-foreground">Profit Margin</dt>
									<dd
										class="text-2xl font-medium"
										class:text-green-600={margin > 0}
										class:text-red-600={margin < 0}
									>
										{margin.toFixed(1)}%
									</dd>
								</div>
							{/if}
						</dl>
					</CardContent>
				</Card>
			</Tabs.Content>

			<!-- Inventory Tab -->
			<Tabs.Content value="inventory" class="mt-6">
				<Card>
					<CardHeader>
						<CardTitle>Inventory Settings</CardTitle>
					</CardHeader>
					<CardContent>
						<dl class="grid grid-cols-2 gap-6">
							<div>
								<dt class="text-sm text-muted-foreground">Track Inventory</dt>
								<dd>
									<Badge variant={product.trackInventory ? 'default' : 'secondary'}>
										{product.trackInventory ? 'Yes' : 'No'}
									</Badge>
								</dd>
							</div>
							{#if product.trackInventory}
								<div>
									<dt class="text-sm text-muted-foreground">Tracking Method</dt>
									<dd class="capitalize">{product.trackingMethod}</dd>
								</div>
							{/if}
						</dl>
					</CardContent>
				</Card>

				<div class="mt-4 rounded-lg border border-dashed p-6 text-center text-muted-foreground">
					<p>Stock levels are managed in the Inventory module.</p>
					<Button variant="link" href="/inventory">Go to Inventory →</Button>
				</div>
			</Tabs.Content>

			<!-- Variants Tab -->
			<Tabs.Content value="variants" class="mt-6">
				<Card>
					<CardHeader class="flex flex-row items-center justify-between">
						<div>
							<CardTitle>Product Variants</CardTitle>
							<CardDescription>Manage variations like size, color, etc.</CardDescription>
						</div>
						<Button onclick={() => openVariantDialog()}>Add Variant</Button>
					</CardHeader>
					<CardContent>
						{#if isLoadingVariants}
							<div class="flex items-center justify-center py-8">
								<div
									class="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"
								></div>
								<span class="ml-2 text-muted-foreground">Loading variants...</span>
							</div>
						{:else if variantsError}
							<div class="py-4 text-center">
								<p class="text-destructive">{variantsError}</p>
								<Button variant="link" onclick={loadVariants}>Retry</Button>
							</div>
						{:else if variants.length === 0}
							<div class="py-8 text-center text-muted-foreground">
								<p>No variants defined for this product.</p>
								<Button variant="link" onclick={() => openVariantDialog()}
									>Add your first variant</Button
								>
							</div>
						{:else}
							<table class="w-full">
								<thead>
									<tr class="border-b text-left text-sm text-muted-foreground">
										<th class="p-3">SKU</th>
										<th class="p-3">Attributes</th>
										<th class="p-3 text-right">Price Difference</th>
										<th class="p-3">Status</th>
										<th class="p-3">Actions</th>
									</tr>
								</thead>
								<tbody>
									{#each variants as variant (variant.variantId)}
										<tr class="border-b">
											<td class="p-3 font-mono text-sm">{variant.sku}</td>
											<td class="p-3">
												<div class="flex flex-wrap gap-1">
													{#each Object.entries(variant.variantAttributes) as [key, value]}
														<Badge variant="outline">{key}: {value}</Badge>
													{/each}
												</div>
											</td>
											<td class="p-3 text-right">
												{#if variant.priceDifference === 0}
													<span class="text-muted-foreground">-</span>
												{:else if variant.priceDifference > 0}
													<span class="text-green-600"
														>+{formatCurrency(variant.priceDifference, product.currencyCode)}</span
													>
												{:else}
													<span class="text-red-600"
														>{formatCurrency(variant.priceDifference, product.currencyCode)}</span
													>
												{/if}
											</td>
											<td class="p-3">
												<Badge variant={variant.isActive ? 'default' : 'secondary'}>
													{variant.isActive ? 'Active' : 'Inactive'}
												</Badge>
											</td>
											<td class="p-3">
												<div class="flex gap-1">
													<Button
														variant="ghost"
														size="sm"
														onclick={() => openVariantDialog(variant)}
													>
														Edit
													</Button>
													<Button
														variant="ghost"
														size="sm"
														class="text-destructive hover:text-destructive"
														onclick={() => openDeleteVariantDialog(variant)}
													>
														Delete
													</Button>
												</div>
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						{/if}
					</CardContent>
				</Card>
			</Tabs.Content>
		</Tabs.Root>
	</div>

	<!-- Delete Confirmation Dialog -->
	<Dialog.Root bind:open={deleteDialogOpen}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Delete Product</Dialog.Title>
				<Dialog.Description>
					Are you sure you want to delete "{product.name}"? This action cannot be undone.
				</Dialog.Description>
			</Dialog.Header>
			{#if deleteError}
				<div class="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
					{deleteError}
				</div>
			{/if}
			<Dialog.Footer>
				<Button variant="outline" onclick={() => (deleteDialogOpen = false)}>Cancel</Button>
				<Button variant="destructive" onclick={handleDelete} disabled={isDeleting}>
					{isDeleting ? 'Deleting...' : 'Delete'}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<!-- Variant Dialog -->
	<Dialog.Root bind:open={variantDialogOpen}>
		<Dialog.Content class="max-w-lg">
			<Dialog.Header>
				<Dialog.Title>{editingVariant ? 'Edit Variant' : 'Add Variant'}</Dialog.Title>
			</Dialog.Header>
			<div class="space-y-4 py-4">
				{#if variantSaveError}
					<div class="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
						{variantSaveError}
					</div>
				{/if}
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="variantSku">SKU *</Label>
						<Input id="variantSku" bind:value={variantForm.sku} disabled={isSavingVariant} />
					</div>
					<div class="space-y-2">
						<Label for="variantBarcode">Barcode</Label>
						<Input
							id="variantBarcode"
							bind:value={variantForm.barcode}
							disabled={isSavingVariant}
						/>
					</div>
				</div>

				<div class="space-y-2">
					<Label for="priceDiff">Price Difference ({product.currencyCode})</Label>
					<Input
						id="priceDiff"
						type="number"
						bind:value={variantForm.priceDifference}
						disabled={isSavingVariant}
					/>
					<p class="text-xs text-muted-foreground">
						Final price: {formatCurrency(
							(product.salePrice ?? 0) + variantForm.priceDifference * 100,
							product.currencyCode
						)}
					</p>
				</div>

				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<Label>Variant Attributes</Label>
						<Button variant="ghost" size="sm" onclick={addAttributeRow} disabled={isSavingVariant}
							>+ Add</Button
						>
					</div>
					{#each variantForm.attributes as attr, i}
						<div class="flex gap-2">
							<Input
								placeholder="Key (e.g., Color)"
								bind:value={attr.key}
								disabled={isSavingVariant}
							/>
							<Input
								placeholder="Value (e.g., Red)"
								bind:value={attr.value}
								disabled={isSavingVariant}
							/>
							{#if variantForm.attributes.length > 1}
								<Button
									variant="ghost"
									size="sm"
									onclick={() => removeAttributeRow(i)}
									disabled={isSavingVariant}>×</Button
								>
							{/if}
						</div>
					{/each}
				</div>

				<label class="flex items-center gap-2">
					<input
						type="checkbox"
						bind:checked={variantForm.isActive}
						class="rounded"
						disabled={isSavingVariant}
					/>
					<span>Active</span>
				</label>
			</div>
			<Dialog.Footer>
				<Button
					variant="outline"
					onclick={() => (variantDialogOpen = false)}
					disabled={isSavingVariant}>Cancel</Button
				>
				<Button onclick={handleSaveVariant} disabled={isSavingVariant}>
					{#if isSavingVariant}
						Saving...
					{:else}
						{editingVariant ? 'Save Changes' : 'Add Variant'}
					{/if}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<!-- Delete Variant Confirmation Dialog -->
	<Dialog.Root bind:open={deleteVariantDialogOpen}>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Delete Variant</Dialog.Title>
				<Dialog.Description>
					Are you sure you want to delete variant "{deletingVariant?.sku}"? This action cannot be
					undone.
				</Dialog.Description>
			</Dialog.Header>
			{#if deleteVariantError}
				<div class="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
					{deleteVariantError}
				</div>
			{/if}
			<Dialog.Footer>
				<Button
					variant="outline"
					onclick={() => (deleteVariantDialogOpen = false)}
					disabled={isDeletingVariant}>Cancel</Button
				>
				<Button variant="destructive" onclick={handleDeleteVariant} disabled={isDeletingVariant}>
					{isDeletingVariant ? 'Deleting...' : 'Delete'}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
{/if}
