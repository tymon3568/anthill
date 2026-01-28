<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { productApi } from '$lib/api/inventory/products';
	import { categoryApi } from '$lib/api/inventory/categories';
	import type { ProductResponse, ProductListParams, CategoryResponse } from '$lib/types/inventory';
	import { SvelteSet } from 'svelte/reactivity';

	// State
	let searchQuery = $state('');
	let searchInput = $state('');
	let selectedType = $state<string>('');
	let selectedCategory = $state<string>('');
	let selectedStatus = $state<'active' | 'inactive' | ''>('');
	let selectedProducts = $state(new SvelteSet<string>());
	let sortBy = $state<'name' | 'sku' | 'salePrice' | 'createdAt' | 'updatedAt'>('name');
	let sortOrder = $state<'asc' | 'desc'>('asc');

	// Pagination state
	let currentPage = $state(1);
	let itemsPerPage = $state(10);

	// API state
	let products = $state<ProductResponse[]>([]);
	let categories = $state<CategoryResponse[]>([]);
	let totalItems = $state(0);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	// Category lookup map for efficient name retrieval
	const categoryMap = $derived(new Map(categories.map((c) => [c.categoryId, c])));

	// Fetch categories on mount
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

	// Debounce search
	let debounceTimer: ReturnType<typeof setTimeout>;
	function handleSearchInput(e: Event) {
		const target = e.target as HTMLInputElement;
		searchInput = target.value;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			searchQuery = searchInput;
			currentPage = 1; // Reset to first page on search
		}, 300);
	}

	// Fetch products from API
	async function fetchProducts() {
		isLoading = true;
		error = null;

		const params: ProductListParams = {
			page: currentPage,
			pageSize: itemsPerPage,
			sortBy: sortBy,
			sortDir: sortOrder
		};

		if (searchQuery) {
			params.search = searchQuery;
		}

		if (selectedType) {
			params.productType = selectedType;
		}

		if (selectedCategory) {
			params.categoryId = selectedCategory;
		}

		if (selectedStatus) {
			params.isActive = selectedStatus === 'active';
		}

		const result = await productApi.list(params);

		if (result.success && result.data) {
			products = result.data.products;
			totalItems = result.data.pagination.totalItems;
		} else {
			error = result.error || 'Failed to load products';
			products = [];
			totalItems = 0;
		}

		isLoading = false;
	}

	// Reactive effect to fetch products when filters change
	$effect(() => {
		// Track dependencies
		const _ = {
			searchQuery,
			selectedType,
			selectedCategory,
			selectedStatus,
			sortBy,
			sortOrder,
			currentPage,
			itemsPerPage
		};
		fetchProducts();
	});

	// Pagination derived values
	const totalPages = $derived(Math.ceil(totalItems / itemsPerPage));
	const showingFrom = $derived(totalItems > 0 ? (currentPage - 1) * itemsPerPage + 1 : 0);
	const showingTo = $derived(Math.min(currentPage * itemsPerPage, totalItems));

	// Helpers
	function toggleProductSelection(id: string) {
		if (selectedProducts.has(id)) {
			selectedProducts.delete(id);
		} else {
			selectedProducts.add(id);
		}
	}

	function toggleAllProducts() {
		if (selectedProducts.size === products.length) {
			selectedProducts.clear();
		} else {
			selectedProducts.clear();
			products.forEach((p) => selectedProducts.add(p.productId));
		}
	}

	function getStatusBadgeVariant(isActive: boolean): 'default' | 'secondary' | 'destructive' {
		return isActive ? 'default' : 'secondary';
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

	function handleSort(column: 'name' | 'sku' | 'salePrice' | 'createdAt' | 'updatedAt') {
		if (sortBy === column) {
			sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
		} else {
			sortBy = column;
			sortOrder = 'asc';
		}
	}

	function formatCurrency(value: number | null | undefined, currency: string = 'VND'): string {
		if (value == null) return '-';
		return new Intl.NumberFormat('vi-VN', {
			style: 'currency',
			currency: currency,
			minimumFractionDigits: 0
		}).format(value);
	}

	function goToPage(page: number) {
		if (page >= 1 && page <= totalPages) {
			currentPage = page;
			selectedProducts.clear(); // Clear selection on page change
		}
	}

	function clearFilters() {
		searchQuery = '';
		searchInput = '';
		selectedType = '';
		selectedCategory = '';
		selectedStatus = '';
		currentPage = 1;
	}

	async function handleExportSelected() {
		// TODO: Implement CSV export
		const selected = products.filter((p) => selectedProducts.has(p.productId));
		console.log('Exporting:', selected);
		alert(`Exporting ${selected.length} products...`);
	}

	async function handleDeleteSelected() {
		if (confirm(`Delete ${selectedProducts.size} selected products?`)) {
			const result = await productApi.bulkDelete(Array.from(selectedProducts));
			if (result.success) {
				selectedProducts.clear();
				await fetchProducts(); // Refresh list
			} else {
				alert('Failed to delete products: ' + result.error);
			}
		}
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
			<p class="text-muted-foreground">Manage your product catalog</p>
		</div>
		<Button href="/inventory/products/new">Add Product</Button>
	</div>

	<!-- Filters -->
	<Card>
		<CardContent class="pt-6">
			<div class="flex flex-wrap items-end gap-4">
				<div class="min-w-[200px] flex-1">
					<Input
						type="search"
						placeholder="Search by SKU or name..."
						value={searchInput}
						oninput={handleSearchInput}
						class="max-w-sm"
					/>
				</div>
				<div class="w-[150px]">
					<select
						bind:value={selectedType}
						class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						onchange={() => (currentPage = 1)}
					>
						<option value="">All Types</option>
						<option value="goods">Goods</option>
						<option value="service">Service</option>
						<option value="consumable">Consumable</option>
					</select>
				</div>
				<div class="w-[180px]">
					<select
						bind:value={selectedCategory}
						class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						onchange={() => (currentPage = 1)}
					>
						<option value="">All Categories</option>
						{#each categories as category (category.categoryId)}
							<option value={category.categoryId}>
								{category.level > 0 ? '—'.repeat(category.level) + ' ' : ''}{category.name}
							</option>
						{/each}
					</select>
				</div>
				<div class="w-[150px]">
					<select
						bind:value={selectedStatus}
						class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						onchange={() => (currentPage = 1)}
					>
						<option value="">All Status</option>
						<option value="active">Active</option>
						<option value="inactive">Inactive</option>
					</select>
				</div>
				{#if searchQuery || selectedType || selectedCategory || selectedStatus}
					<Button variant="ghost" size="sm" onclick={clearFilters}>Clear</Button>
				{/if}
			</div>
		</CardContent>
	</Card>

	<!-- Bulk Actions -->
	{#if selectedProducts.size > 0}
		<div class="flex items-center gap-4 rounded-lg bg-muted p-4">
			<span class="text-sm font-medium">{selectedProducts.size} selected</span>
			<Button variant="outline" size="sm" onclick={handleExportSelected}>Export Selected</Button>
			<Button variant="destructive" size="sm" onclick={handleDeleteSelected}>Delete Selected</Button
			>
		</div>
	{/if}

	<!-- Products Table -->
	<Card>
		<CardHeader>
			<CardTitle>Product List ({totalItems})</CardTitle>
		</CardHeader>
		<CardContent>
			{#if error}
				<div class="rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-center">
					<p class="text-destructive">{error}</p>
					<Button variant="outline" size="sm" class="mt-2" onclick={fetchProducts}>
						Try Again
					</Button>
				</div>
			{:else if isLoading}
				<div class="flex items-center justify-center py-12">
					<div
						class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"
					></div>
					<span class="ml-3 text-muted-foreground">Loading products...</span>
				</div>
			{:else}
				<div class="overflow-x-auto">
					<table class="w-full">
						<thead>
							<tr class="border-b text-left text-sm text-muted-foreground">
								<th class="w-10 p-3">
									<input
										type="checkbox"
										checked={selectedProducts.size === products.length && products.length > 0}
										onchange={toggleAllProducts}
										class="rounded"
									/>
								</th>
								<th
									class="cursor-pointer p-3 hover:text-foreground"
									onclick={() => handleSort('sku')}
								>
									SKU {sortBy === 'sku' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
								</th>
								<th
									class="cursor-pointer p-3 hover:text-foreground"
									onclick={() => handleSort('name')}
								>
									Name {sortBy === 'name' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
								</th>
								<th class="p-3">Type</th>
								<th class="p-3">Category</th>
								<th
									class="cursor-pointer p-3 text-right hover:text-foreground"
									onclick={() => handleSort('salePrice')}
								>
									Price {sortBy === 'salePrice' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
								</th>
								<th class="p-3">Tracking</th>
								<th class="p-3">Status</th>
								<th class="w-24 p-3">Actions</th>
							</tr>
						</thead>
						<tbody>
							{#each products as product (product.productId)}
								<tr class="border-b hover:bg-muted/50">
									<td class="p-3">
										<input
											type="checkbox"
											checked={selectedProducts.has(product.productId)}
											onchange={() => toggleProductSelection(product.productId)}
											class="rounded"
										/>
									</td>
									<td class="p-3 font-mono text-sm">{product.sku}</td>
									<td class="p-3">
										<div>
											<a
												href="/inventory/products/{product.productId}"
												class="font-medium hover:underline"
											>
												{product.name}
											</a>
											{#if product.description}
												<p class="line-clamp-1 text-sm text-muted-foreground">
													{product.description}
												</p>
											{/if}
										</div>
									</td>
									<td class="p-3">
										<span
											class="rounded-full px-2 py-1 text-xs font-medium {getTypeBadgeClass(
												product.productType
											)}"
										>
											{product.productType}
										</span>
									</td>
									<td class="p-3">
										{#if product.categoryId && categoryMap.get(product.categoryId)}
											<span class="text-sm">{categoryMap.get(product.categoryId)?.name}</span>
										{:else}
											<span class="text-sm text-muted-foreground">—</span>
										{/if}
									</td>
									<td class="p-3 text-right font-medium">
										{formatCurrency(product.salePrice, product.currencyCode)}
									</td>
									<td class="p-3">
										{#if product.trackInventory}
											<Badge variant="outline" class="text-xs">{product.trackingMethod}</Badge>
										{:else}
											<span class="text-xs text-muted-foreground">-</span>
										{/if}
									</td>
									<td class="p-3">
										<Badge variant={getStatusBadgeVariant(product.isActive)}>
											{product.isActive ? 'Active' : 'Inactive'}
										</Badge>
									</td>
									<td class="p-3">
										<div class="flex gap-1">
											<Button
												variant="ghost"
												size="sm"
												href="/inventory/products/{product.productId}"
											>
												View
											</Button>
										</div>
									</td>
								</tr>
							{:else}
								<tr>
									<td colspan="9" class="p-8 text-center text-muted-foreground">
										{#if searchQuery || selectedType || selectedCategory || selectedStatus}
											No products match your filters.
											<button class="text-primary hover:underline ml-1" onclick={clearFilters}>
												Clear filters
											</button>
										{:else}
											No products found. <a
												href="/inventory/products/new"
												class="text-primary hover:underline">Add your first product</a
											>
										{/if}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>

				<!-- Pagination -->
				{#if totalItems > 0}
					<div class="mt-4 flex items-center justify-between border-t pt-4">
						<div class="flex items-center gap-2 text-sm text-muted-foreground">
							<span>Show</span>
							<select
								bind:value={itemsPerPage}
								class="rounded border border-input bg-background px-2 py-1 text-sm"
								onchange={() => (currentPage = 1)}
							>
								<option value={10}>10</option>
								<option value={25}>25</option>
								<option value={50}>50</option>
							</select>
							<span>per page</span>
						</div>

						<div class="text-sm text-muted-foreground">
							Showing {showingFrom} - {showingTo} of {totalItems}
						</div>

						<div class="flex items-center gap-1">
							<Button
								variant="outline"
								size="sm"
								disabled={currentPage === 1}
								onclick={() => goToPage(1)}
							>
								«
							</Button>
							<Button
								variant="outline"
								size="sm"
								disabled={currentPage === 1}
								onclick={() => goToPage(currentPage - 1)}
							>
								‹
							</Button>

							{#each Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
								const start = Math.max(1, currentPage - 2);
								const end = Math.min(totalPages, start + 4);
								const adjustedStart = Math.max(1, end - 4);
								return adjustedStart + i;
							}).filter((p) => p <= totalPages) as page (page)}
								<Button
									variant={currentPage === page ? 'default' : 'outline'}
									size="sm"
									onclick={() => goToPage(page)}
								>
									{page}
								</Button>
							{/each}

							<Button
								variant="outline"
								size="sm"
								disabled={currentPage === totalPages}
								onclick={() => goToPage(currentPage + 1)}
							>
								›
							</Button>
							<Button
								variant="outline"
								size="sm"
								disabled={currentPage === totalPages}
								onclick={() => goToPage(totalPages)}
							>
								»
							</Button>
						</div>
					</div>
				{/if}
			{/if}
		</CardContent>
	</Card>
</div>
