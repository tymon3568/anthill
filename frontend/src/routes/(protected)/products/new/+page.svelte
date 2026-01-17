<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { mockCategories, type CreateProductRequest } from '$lib/api/products';
	import { goto } from '$app/navigation';

	// Form state
	let formData = $state<CreateProductRequest>({
		sku: '',
		name: '',
		description: '',
		categoryId: '',
		price: 0,
		costPrice: 0,
		quantity: 0,
		minQuantity: 10,
		unit: 'pcs',
		tags: []
	});

	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});

	function validateForm(): boolean {
		errors = {};

		if (!formData.sku.trim()) {
			errors.sku = 'SKU is required';
		}
		if (!formData.name.trim()) {
			errors.name = 'Name is required';
		}
		if (!formData.categoryId) {
			errors.categoryId = 'Category is required';
		}
		if (formData.price <= 0) {
			errors.price = 'Price must be greater than 0';
		}
		if (formData.quantity < 0) {
			errors.quantity = 'Quantity cannot be negative';
		}

		return Object.keys(errors).length === 0;
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();

		if (!validateForm()) return;

		isSubmitting = true;

		try {
			// TODO: Call API to create product
			console.log('Creating product:', formData);

			// Simulate API call
			await new Promise((resolve) => setTimeout(resolve, 500));

			goto('/products');
		} catch (error) {
			console.error('Failed to create product:', error);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<svelte:head>
	<title>New Product - Anthill</title>
</svelte:head>

<div class="mx-auto max-w-2xl space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">New Product</h1>
			<p class="text-muted-foreground">Add a new product to your inventory</p>
		</div>
		<Button variant="outline" href="/products">Cancel</Button>
	</div>

	<form onsubmit={handleSubmit}>
		<Card>
			<CardHeader>
				<CardTitle>Product Details</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="sku">SKU *</Label>
						<Input
							id="sku"
							bind:value={formData.sku}
							placeholder="PROD-001"
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
							placeholder="Product name"
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
						placeholder="Product description"
						rows="3"
						class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
					></textarea>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="category">Category *</Label>
						<select
							id="category"
							bind:value={formData.categoryId}
							class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm {errors.categoryId
								? 'border-destructive'
								: ''}"
						>
							<option value="">Select category</option>
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
						<Input id="costPrice" type="number" step="0.01" min="0" bind:value={formData.costPrice} />
					</div>
				</div>

				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="quantity">Initial Quantity</Label>
						<Input
							id="quantity"
							type="number"
							min="0"
							bind:value={formData.quantity}
							class={errors.quantity ? 'border-destructive' : ''}
						/>
						{#if errors.quantity}
							<p class="text-sm text-destructive">{errors.quantity}</p>
						{/if}
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
					Creating...
				{:else}
					Create Product
				{/if}
			</Button>
		</div>
	</form>
</div>
