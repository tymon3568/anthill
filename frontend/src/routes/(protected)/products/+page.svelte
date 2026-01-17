<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { mockProducts, mockCategories, type Product } from '$lib/api/products';

	// State
	let searchQuery = $state('');
	let selectedCategory = $state('');
	let selectedStatus = $state('');
	let selectedProducts = $state<Set<string>>(new Set());
	let sortBy = $state('name');
	let sortOrder = $state<'asc' | 'desc'>('asc');

	// Derived filtered products
	const filteredProducts = $derived.by(() => {
		let result = [...mockProducts];

		// Search filter
		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			result = result.filter(
				(p) =>
					p.name.toLowerCase().includes(query) ||
					p.sku.toLowerCase().includes(query) ||
					p.description?.toLowerCase().includes(query)
			);
		}

		// Category filter
		if (selectedCategory) {
			result = result.filter((p) => p.categoryId === selectedCategory);
		}

		// Status filter
		if (selectedStatus) {
			result = result.filter((p) => p.status === selectedStatus);
		}

		// Sort
		result.sort((a, b) => {
			const aVal = a[sortBy as keyof Product] ?? '';
			const bVal = b[sortBy as keyof Product] ?? '';

			if (typeof aVal === 'string' && typeof bVal === 'string') {
				return sortOrder === 'asc' ? aVal.localeCompare(bVal) : bVal.localeCompare(aVal);
			}
			if (typeof aVal === 'number' && typeof bVal === 'number') {
				return sortOrder === 'asc' ? aVal - bVal : bVal - aVal;
			}
			return 0;
		});

		return result;
	});

	// Helpers
	function toggleProductSelection(id: string) {
		const newSet = new Set(selectedProducts);
		if (newSet.has(id)) {
			newSet.delete(id);
		} else {
			newSet.add(id);
		}
		selectedProducts = newSet;
	}

	function toggleAllProducts() {
		if (selectedProducts.size === filteredProducts.length) {
			selectedProducts = new Set();
		} else {
			selectedProducts = new Set(filteredProducts.map((p) => p.id));
		}
	}

	function getStatusBadgeVariant(status: string) {
		switch (status) {
			case 'active':
				return 'default';
			case 'inactive':
				return 'secondary';
			case 'discontinued':
				return 'destructive';
			default:
				return 'outline';
		}
	}

	function isLowStock(product: Product): boolean {
		return product.quantity <= product.minQuantity;
	}

	function handleSort(column: string) {
		if (sortBy === column) {
			sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
		} else {
			sortBy = column;
			sortOrder = 'asc';
		}
	}

	function formatCurrency(value: number): string {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD'
		}).format(value);
	}
</script>

<svelte:head>
	<title>Products - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Products</h1>
			<p class="text-muted-foreground">Manage your product inventory</p>
		</div>
		<Button href="/products/new">Add Product</Button>
	</div>

	<!-- Filters -->
	<Card>
		<CardContent class="pt-6">
			<div class="flex flex-wrap gap-4">
				<div class="flex-1">
					<Input
						type="search"
						placeholder="Search products..."
						bind:value={searchQuery}
						class="max-w-sm"
					/>
				</div>
				<select
					bind:value={selectedCategory}
					class="rounded-md border border-input bg-background px-3 py-2 text-sm"
				>
					<option value="">All Categories</option>
					{#each mockCategories as category}
						<option value={category.id}>{category.name}</option>
					{/each}
				</select>
				<select
					bind:value={selectedStatus}
					class="rounded-md border border-input bg-background px-3 py-2 text-sm"
				>
					<option value="">All Status</option>
					<option value="active">Active</option>
					<option value="inactive">Inactive</option>
					<option value="discontinued">Discontinued</option>
				</select>
			</div>
		</CardContent>
	</Card>

	<!-- Bulk Actions -->
	{#if selectedProducts.size > 0}
		<div class="flex items-center gap-4 rounded-lg bg-muted p-4">
			<span class="text-sm font-medium">{selectedProducts.size} selected</span>
			<Button variant="outline" size="sm">Export Selected</Button>
			<Button variant="destructive" size="sm">Delete Selected</Button>
		</div>
	{/if}

	<!-- Products Table -->
	<Card>
		<CardHeader>
			<CardTitle>Product List ({filteredProducts.length})</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead>
						<tr class="border-b text-left text-sm text-muted-foreground">
							<th class="p-3">
								<input
									type="checkbox"
									checked={selectedProducts.size === filteredProducts.length &&
										filteredProducts.length > 0}
									onchange={toggleAllProducts}
									class="rounded"
								/>
							</th>
							<th class="cursor-pointer p-3 hover:text-foreground" onclick={() => handleSort('sku')}>
								SKU {sortBy === 'sku' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
							</th>
							<th
								class="cursor-pointer p-3 hover:text-foreground"
								onclick={() => handleSort('name')}
							>
								Name {sortBy === 'name' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
							</th>
							<th class="p-3">Category</th>
							<th
								class="cursor-pointer p-3 hover:text-foreground"
								onclick={() => handleSort('price')}
							>
								Price {sortBy === 'price' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
							</th>
							<th
								class="cursor-pointer p-3 hover:text-foreground"
								onclick={() => handleSort('quantity')}
							>
								Stock {sortBy === 'quantity' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
							</th>
							<th class="p-3">Status</th>
							<th class="p-3">Actions</th>
						</tr>
					</thead>
					<tbody>
						{#each filteredProducts as product}
							<tr class="border-b hover:bg-muted/50">
								<td class="p-3">
									<input
										type="checkbox"
										checked={selectedProducts.has(product.id)}
										onchange={() => toggleProductSelection(product.id)}
										class="rounded"
									/>
								</td>
								<td class="p-3 font-mono text-sm">{product.sku}</td>
								<td class="p-3">
									<div>
										<p class="font-medium">{product.name}</p>
										{#if product.description}
											<p class="text-sm text-muted-foreground">{product.description}</p>
										{/if}
									</div>
								</td>
								<td class="p-3">{product.categoryName}</td>
								<td class="p-3">{formatCurrency(product.price)}</td>
								<td class="p-3">
									<div class="flex items-center gap-2">
										<span class={isLowStock(product) ? 'text-destructive font-medium' : ''}>
											{product.quantity}
										</span>
										{#if isLowStock(product)}
											<Badge variant="destructive" class="text-xs">Low</Badge>
										{/if}
									</div>
								</td>
								<td class="p-3">
									<Badge variant={getStatusBadgeVariant(product.status)}>
										{product.status}
									</Badge>
								</td>
								<td class="p-3">
									<div class="flex gap-2">
										<Button variant="ghost" size="sm" href="/products/{product.id}">Edit</Button>
										<Button variant="ghost" size="sm">Delete</Button>
									</div>
								</td>
							</tr>
						{:else}
							<tr>
								<td colspan="8" class="p-8 text-center text-muted-foreground">
									No products found
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</CardContent>
	</Card>
</div>
