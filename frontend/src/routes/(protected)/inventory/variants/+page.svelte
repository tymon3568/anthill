<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { variantApi } from '$lib/api/inventory/variants';
	import type { VariantResponse, VariantListParams } from '$lib/types/inventory';
	import { SvelteSet } from 'svelte/reactivity';

	// State
	let searchQuery = $state('');
	let searchInput = $state('');
	let selectedStatus = $state<'active' | 'inactive' | ''>('');
	let selectedVariants = new SvelteSet<string>();
	let sortBy = $state<'sku' | 'createdAt' | 'updatedAt'>('sku');
	let sortOrder = $state<'asc' | 'desc'>('asc');

	// Pagination state
	let currentPage = $state(1);
	let itemsPerPage = $state(10);

	// API state
	let variants = $state<VariantResponse[]>([]);
	let totalItems = $state(0);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	// Debounce search
	let debounceTimer: ReturnType<typeof setTimeout>;
	function handleSearchInput(e: Event) {
		const target = e.target as HTMLInputElement;
		searchInput = target.value;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => {
			searchQuery = searchInput;
			currentPage = 1;
		}, 300);
	}

	// Fetch variants from API
	async function fetchVariants() {
		isLoading = true;
		error = null;

		const params: VariantListParams = {
			page: currentPage,
			pageSize: itemsPerPage,
			sortBy: sortBy,
			sortDir: sortOrder
		};

		if (searchQuery) {
			params.search = searchQuery;
		}

		if (selectedStatus) {
			params.isActive = selectedStatus === 'active';
		}

		const result = await variantApi.list(params);

		if (result.success && result.data) {
			variants = result.data.variants;
			totalItems = result.data.pagination.totalItems;
		} else {
			error = result.error || 'Failed to load variants';
			variants = [];
			totalItems = 0;
		}

		isLoading = false;
	}

	// Reactive effect to fetch variants when filters change
	$effect(() => {
		// Track dependencies explicitly for reactivity
		void searchQuery;
		void selectedStatus;
		void sortBy;
		void sortOrder;
		void currentPage;
		void itemsPerPage;
		fetchVariants();
	});

	// Pagination derived values
	const totalPages = $derived(Math.ceil(totalItems / itemsPerPage));
	const showingFrom = $derived(totalItems > 0 ? (currentPage - 1) * itemsPerPage + 1 : 0);
	const showingTo = $derived(Math.min(currentPage * itemsPerPage, totalItems));

	// Helpers
	function toggleVariantSelection(id: string) {
		if (selectedVariants.has(id)) {
			selectedVariants.delete(id);
		} else {
			selectedVariants.add(id);
		}
	}

	function toggleAllVariants() {
		if (selectedVariants.size === variants.length) {
			selectedVariants.clear();
		} else {
			selectedVariants.clear();
			variants.forEach((v) => selectedVariants.add(v.variantId));
		}
	}

	function getStatusBadgeVariant(isActive: boolean): 'default' | 'secondary' | 'destructive' {
		return isActive ? 'default' : 'secondary';
	}

	function handleSort(column: 'sku' | 'createdAt' | 'updatedAt') {
		if (sortBy === column) {
			sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
		} else {
			sortBy = column;
			sortOrder = 'asc';
		}
	}

	function formatCurrency(value: number | null | undefined, currency: string = 'VND'): string {
		if (value == null) return '-';
		const sign = value >= 0 ? '+' : '';
		return (
			sign +
			new Intl.NumberFormat('vi-VN', {
				style: 'currency',
				currency: currency,
				minimumFractionDigits: 0
			}).format(value)
		);
	}

	function goToPage(page: number) {
		if (page >= 1 && page <= totalPages) {
			currentPage = page;
			selectedVariants.clear();
		}
	}

	function clearFilters() {
		searchQuery = '';
		searchInput = '';
		selectedStatus = '';
		currentPage = 1;
	}

	async function handleBulkActivate() {
		const result = await variantApi.bulkActivate(Array.from(selectedVariants));
		if (result.success) {
			selectedVariants.clear();
			await fetchVariants();
		} else {
			alert('Failed to activate variants: ' + result.error);
		}
	}

	async function handleBulkDeactivate() {
		const result = await variantApi.bulkDeactivate(Array.from(selectedVariants));
		if (result.success) {
			selectedVariants.clear();
			await fetchVariants();
		} else {
			alert('Failed to deactivate variants: ' + result.error);
		}
	}

	async function handleDeleteSelected() {
		if (confirm(`Delete ${selectedVariants.size} selected variants?`)) {
			const result = await variantApi.bulkDelete(Array.from(selectedVariants));
			if (result.success) {
				selectedVariants.clear();
				await fetchVariants();
			} else {
				alert('Failed to delete variants: ' + result.error);
			}
		}
	}
</script>

<svelte:head>
	<title>Variants - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Product Variants</h1>
			<p class="text-muted-foreground">View and search variants across all products</p>
		</div>
	</div>

	<!-- Filters -->
	<Card>
		<CardContent class="pt-6">
			<div class="flex flex-wrap items-end gap-4">
				<div class="min-w-[200px] flex-1">
					<Input
						type="search"
						placeholder="Search by SKU, barcode, or product name..."
						value={searchInput}
						oninput={handleSearchInput}
						class="max-w-sm"
					/>
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
				{#if searchQuery || selectedStatus}
					<Button variant="ghost" size="sm" onclick={clearFilters}>Clear</Button>
				{/if}
			</div>
		</CardContent>
	</Card>

	<!-- Bulk Actions -->
	{#if selectedVariants.size > 0}
		<div class="flex items-center gap-4 rounded-lg bg-muted p-4">
			<span class="text-sm font-medium">{selectedVariants.size} selected</span>
			<Button variant="outline" size="sm" onclick={handleBulkActivate}>Activate</Button>
			<Button variant="outline" size="sm" onclick={handleBulkDeactivate}>Deactivate</Button>
			<Button variant="destructive" size="sm" onclick={handleDeleteSelected}>Delete</Button>
		</div>
	{/if}

	<!-- Variants Table -->
	<Card>
		<CardHeader>
			<CardTitle>Variant List ({totalItems})</CardTitle>
		</CardHeader>
		<CardContent>
			{#if error}
				<div class="rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-center">
					<p class="text-destructive">{error}</p>
					<Button variant="outline" size="sm" class="mt-2" onclick={fetchVariants}>
						Try Again
					</Button>
				</div>
			{:else if isLoading}
				<div class="flex items-center justify-center py-12">
					<div
						class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"
					></div>
					<span class="ml-3 text-muted-foreground">Loading variants...</span>
				</div>
			{:else}
				<div class="overflow-x-auto">
					<table class="w-full">
						<thead>
							<tr class="border-b text-left text-sm text-muted-foreground">
								<th class="w-10 p-3">
									<input
										type="checkbox"
										checked={selectedVariants.size === variants.length && variants.length > 0}
										onchange={toggleAllVariants}
										class="rounded"
									/>
								</th>
								<th
									class="cursor-pointer p-3 hover:text-foreground"
									onclick={() => handleSort('sku')}
								>
									SKU {sortBy === 'sku' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
								</th>
								<th class="p-3">Parent Product</th>
								<th class="p-3">Attributes</th>
								<th class="p-3">Barcode</th>
								<th class="p-3 text-right">Price Diff</th>
								<th class="p-3">Status</th>
								<th class="w-24 p-3">Actions</th>
							</tr>
						</thead>
						<tbody>
							{#each variants as variant (variant.variantId)}
								<tr class="border-b hover:bg-muted/50">
									<td class="p-3">
										<input
											type="checkbox"
											checked={selectedVariants.has(variant.variantId)}
											onchange={() => toggleVariantSelection(variant.variantId)}
											class="rounded"
										/>
									</td>
									<td class="p-3 font-mono text-sm">{variant.sku}</td>
									<td class="p-3">
										<a
											href="/inventory/products/{variant.parentProductId}"
											class="font-medium hover:underline"
										>
											{variant.parentProductName || variant.parentProductSku || 'View Product'}
										</a>
									</td>
									<td class="p-3">
										<div class="flex flex-wrap gap-1">
											{#each Object.entries(variant.variantAttributes) as [key, value] (key)}
												<Badge variant="outline" class="text-xs">
													{key}: {value}
												</Badge>
											{/each}
										</div>
									</td>
									<td class="p-3 font-mono text-sm text-muted-foreground">
										{variant.barcode || '-'}
									</td>
									<td class="p-3 text-right">
										{#if variant.priceDifference !== 0}
											<span class={variant.priceDifference > 0 ? 'text-green-600' : 'text-red-600'}>
												{formatCurrency(variant.priceDifference)}
											</span>
										{:else}
											<span class="text-muted-foreground">-</span>
										{/if}
									</td>
									<td class="p-3">
										<Badge variant={getStatusBadgeVariant(variant.isActive)}>
											{variant.isActive ? 'Active' : 'Inactive'}
										</Badge>
									</td>
									<td class="p-3">
										<div class="flex gap-1">
											<Button
												variant="ghost"
												size="sm"
												href="/inventory/products/{variant.parentProductId}"
											>
												View
											</Button>
										</div>
									</td>
								</tr>
							{:else}
								<tr>
									<td colspan="8" class="p-8 text-center text-muted-foreground">
										{#if searchQuery || selectedStatus}
											No variants match your filters.
											<button class="text-primary hover:underline ml-1" onclick={clearFilters}>
												Clear filters
											</button>
										{:else}
											No variants found. Variants are created within product details.
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
