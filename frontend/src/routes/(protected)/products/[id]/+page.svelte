<script lang="ts">
	import { page } from '$app/stores';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import {
		mockProducts,
		mockCategories,
		type Product,
		type UpdateProductRequest
	} from '$lib/api/products';
	import { goto } from '$app/navigation';

	// Get product ID from URL
	const productId = $derived($page.params.id);

	// Find product from mock data
	const existingProduct = $derived(mockProducts.find((p) => p.id === productId));

	// Form state initialized from existing product
	let formData = $state<UpdateProductRequest>({
		sku: '',
		name: '',
		description: '',
		categoryId: '',
		price: 0,
		costPrice: 0,
		quantity: 0,
		minQuantity: 10,
		unit: 'pcs',
		status: 'active'
	});

	// Initialize form when product is found
	$effect(() => {
		if (existingProduct) {
			formData = {
				sku: existingProduct.sku,
				name: existingProduct.name,
				description: existingProduct.description || '',
				categoryId: existingProduct.categoryId,
				price: existingProduct.price,
				costPrice: existingProduct.costPrice || 0,
				quantity: existingProduct.quantity,
				minQuantity: existingProduct.minQuantity,
				unit: existingProduct.unit,
				status: existingProduct.status
			};
		}
	});

	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});

	function validateForm(): boolean {
		errors = {};

		if (!formData.sku?.trim()) {
			errors.sku = 'SKU is required';
		}
		if (!formData.name?.trim()) {
			errors.name = 'Name is required';
		}
		if (!formData.categoryId) {
			errors.categoryId = 'Category is required';
		}
		if ((formData.price ?? 0) <= 0) {
			errors.price = 'Price must be greater than 0';
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();

		if (!validateForm()) return;

		isSubmitting = true;

		try {
			// TODO: Call API to update product
			console.log('Updating product:', productId, formData);

			// Simulate API call
			await new Promise((resolve) => setTimeout(resolve, 500));

			goto('/products');
		} catch (error) {
			console.error('Failed to update product:', error);
		} finally {
			isSubmitting = false;
		}
	}

	async function handleDelete() {
		if (!confirm('Are you sure you want to delete this product?')) return;

		try {
			// TODO: Call API to delete product
			console.log('Deleting product:', productId);

			await new Promise((resolve) => setTimeout(resolve, 500));

			goto('/products');
		} catch (error) {
			console.error('Failed to delete product:', error);
		}
	}
</script>

<svelte:head>
	<title>{existingProduct?.name || 'Edit Product'} - Anthill</title>
</svelte:head>

{#if !existingProduct}
	<div class="flex min-h-[50vh] items-center justify-center">
		<Card class="w-full max-w-md">
			<CardContent class="pt-6 text-center">
				<p class="text-muted-foreground">Product not found</p>
				<Button href="/products" class="mt-4">Back to Products</Button>
			</CardContent>
		</Card>
	</div>
{:else}
	<div class="mx-auto max-w-2xl space-y-6">
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-2xl font-bold">Edit Product</h1>
				<p class="text-muted-foreground">SKU: {existingProduct.sku}</p>
			</div>
			<div class="flex gap-2">
				<Button variant="destructive" onclick={handleDelete}>Delete</Button>
				<Button variant="outline" href="/products">Cancel</Button>
			</div>
		</div>

		<form onsubmit={handleSubmit}>
			<Card>
				<CardHeader>
					<div class="flex items-center justify-between">
						<CardTitle>Product Details</CardTitle>
						<Badge variant={formData.status === 'active' ? 'default' : 'secondary'}>
							{formData.status}
						</Badge>
					</div>
				</CardHeader>
				<CardContent class="space-y-4">
					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label for="sku">SKU *</Label>
							<Input
								id="sku"
								bind:value={formData.sku}
								class={errors.sku ? 'border-destructive' : ''}
							/>
							{#if errors.sku}
								<p class="text-sm text-destructive">{errors.sku}</p>
							{/if}
						</div>
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

					<div class="grid grid-cols-3 gap-4">
						<div class="space-y-2">
							<Label for="category">Category *</Label>
							<select
								id="category"
								bind:value={formData.categoryId}
								class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm {errors.categoryId
									? 'border-destructive'
									: ''}"
							>
								{#each mockCategories as category}
									<option value={category.id}>{category.name}</option>
								{/each}
							</select>
							{#if errors.categoryId}
								<p class="text-sm text-destructive">{errors.categoryId}</p>
							{/if}
						</div>
						<div class="space-y-2">
							<Label for="unit">Unit</Label>
							<select
								id="unit"
								bind:value={formData.unit}
								class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
							>
								<option value="pcs">Pieces</option>
								<option value="kg">Kilograms</option>
								<option value="m">Meters</option>
								<option value="l">Liters</option>
								<option value="box">Boxes</option>
							</select>
						</div>
						<div class="space-y-2">
							<Label for="status">Status</Label>
							<select
								id="status"
								bind:value={formData.status}
								class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
							>
								<option value="active">Active</option>
								<option value="inactive">Inactive</option>
								<option value="discontinued">Discontinued</option>
							</select>
						</div>
					</div>

					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label for="price">Price *</Label>
							<Input
								id="price"
								type="number"
								step="0.01"
								min="0"
								bind:value={formData.price}
								class={errors.price ? 'border-destructive' : ''}
							/>
							{#if errors.price}
								<p class="text-sm text-destructive">{errors.price}</p>
							{/if}
						</div>
						<div class="space-y-2">
							<Label for="costPrice">Cost Price</Label>
							<Input
								id="costPrice"
								type="number"
								step="0.01"
								min="0"
								bind:value={formData.costPrice}
							/>
						</div>
					</div>

					<div class="grid grid-cols-2 gap-4">
						<div class="space-y-2">
							<Label for="quantity">Current Stock</Label>
							<Input id="quantity" type="number" min="0" bind:value={formData.quantity} />
						</div>
						<div class="space-y-2">
							<Label for="minQuantity">Minimum Stock Level</Label>
							<Input id="minQuantity" type="number" min="0" bind:value={formData.minQuantity} />
						</div>
					</div>
				</CardContent>
			</Card>

			<div class="mt-6 flex justify-end gap-4">
				<Button type="button" variant="outline" href="/products">Cancel</Button>
				<Button type="submit" disabled={isSubmitting}>
					{#if isSubmitting}
						Saving...
					{:else}
						Save Changes
					{/if}
				</Button>
			</div>
		</form>
	</div>
{/if}
