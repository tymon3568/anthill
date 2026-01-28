<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { stockLevelApi } from '$lib/api/inventory/stock-levels';
	import { warehouseApi } from '$lib/api/inventory/warehouses';
	import type {
		StockLevelResponse,
		StockLevelSummary,
		StockLevelListParams,
		StockStatus,
		WarehouseResponse
	} from '$lib/types/inventory';

	// State
	let searchQuery = $state('');
	let searchInput = $state('');
	let selectedWarehouse = $state<string>('');
	let selectedStatus = $state<'all' | 'in_stock' | 'low_stock' | 'out_of_stock'>('all');
	let sortBy = $state<string>('product_name');
	let sortOrder = $state<'asc' | 'desc'>('asc');

	// Pagination state
	let currentPage = $state(1);
	let itemsPerPage = $state(20);

	// API state
	let stockLevels = $state<StockLevelResponse[]>([]);
	let summary = $state<StockLevelSummary | null>(null);
	let totalItems = $state(0);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	// Warehouses for filter dropdown
	let warehouses = $state<WarehouseResponse[]>([]);
	let isLoadingWarehouses = $state(true);

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

	// Fetch warehouses for filter dropdown
	async function fetchWarehouses() {
		isLoadingWarehouses = true;
		const result = await warehouseApi.list();
		if (result.success && result.data) {
			warehouses = result.data.warehouses;
		}
		isLoadingWarehouses = false;
	}

	// Fetch stock levels from API
	async function fetchStockLevels() {
		isLoading = true;
		error = null;

		const params: StockLevelListParams = {
			page: currentPage,
			pageSize: itemsPerPage,
			sortBy: sortBy,
			sortDir: sortOrder
		};

		if (searchQuery) {
			params.search = searchQuery;
		}

		if (selectedWarehouse) {
			params.warehouseId = selectedWarehouse;
		}

		if (selectedStatus === 'out_of_stock') {
			params.outOfStockOnly = true;
		} else if (selectedStatus === 'low_stock') {
			params.lowStockOnly = true;
		}

		const result = await stockLevelApi.list(params);

		if (result.success && result.data) {
			stockLevels = result.data.items;
			summary = result.data.summary;
			totalItems = result.data.pagination.totalItems;
		} else {
			error = result.error || 'Failed to load stock levels';
			stockLevels = [];
			summary = null;
			totalItems = 0;
		}

		isLoading = false;
	}

	// Load warehouses on mount
	$effect(() => {
		fetchWarehouses();
	});

	// Reactive effect to fetch stock levels when filters change
	$effect(() => {
		const _ = {
			searchQuery,
			selectedWarehouse,
			selectedStatus,
			sortBy,
			sortOrder,
			currentPage,
			itemsPerPage
		};
		fetchStockLevels();
	});

	// Pagination derived values
	const totalPages = $derived(Math.ceil(totalItems / itemsPerPage));
	const showingFrom = $derived(totalItems > 0 ? (currentPage - 1) * itemsPerPage + 1 : 0);
	const showingTo = $derived(Math.min(currentPage * itemsPerPage, totalItems));

	// Helpers
	function getStatusBadgeVariant(status: StockStatus): 'default' | 'secondary' | 'destructive' {
		switch (status) {
			case 'in_stock':
				return 'default';
			case 'low_stock':
				return 'secondary';
			case 'out_of_stock':
				return 'destructive';
			default:
				return 'default';
		}
	}

	function getStatusLabel(status: StockStatus): string {
		switch (status) {
			case 'in_stock':
				return 'In Stock';
			case 'low_stock':
				return 'Low Stock';
			case 'out_of_stock':
				return 'Out of Stock';
			default:
				return status;
		}
	}

	function handleSort(column: string) {
		if (sortBy === column) {
			sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
		} else {
			sortBy = column;
			sortOrder = 'asc';
		}
	}

	function formatNumber(value: number): string {
		return new Intl.NumberFormat('vi-VN').format(value);
	}

	function goToPage(page: number) {
		if (page >= 1 && page <= totalPages) {
			currentPage = page;
		}
	}

	function clearFilters() {
		searchQuery = '';
		searchInput = '';
		selectedWarehouse = '';
		selectedStatus = 'all';
		currentPage = 1;
	}
</script>

<svelte:head>
	<title>Stock Levels - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Stock Levels</h1>
			<p class="text-muted-foreground">View inventory quantities across warehouses</p>
		</div>
		<Button variant="outline" onclick={fetchStockLevels}>Refresh</Button>
	</div>

	<!-- Summary Cards -->
	{#if summary}
		<div class="grid grid-cols-1 gap-4 md:grid-cols-5">
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold">{formatNumber(summary.totalProducts)}</div>
					<p class="text-sm text-muted-foreground">Total Products</p>
				</CardContent>
			</Card>
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold text-green-600">
						{formatNumber(summary.totalAvailableQuantity)}
					</div>
					<p class="text-sm text-muted-foreground">Available Quantity</p>
				</CardContent>
			</Card>
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold text-blue-600">
						{formatNumber(summary.totalReservedQuantity)}
					</div>
					<p class="text-sm text-muted-foreground">Reserved Quantity</p>
				</CardContent>
			</Card>
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold text-yellow-600">
						{formatNumber(summary.lowStockCount)}
					</div>
					<p class="text-sm text-muted-foreground">Low Stock Items</p>
				</CardContent>
			</Card>
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold text-red-600">
						{formatNumber(summary.outOfStockCount)}
					</div>
					<p class="text-sm text-muted-foreground">Out of Stock</p>
				</CardContent>
			</Card>
		</div>
	{/if}

	<!-- Filters -->
	<Card>
		<CardContent class="pt-6">
			<div class="flex flex-wrap items-end gap-4">
				<div class="min-w-[200px] flex-1">
					<Input
						type="search"
						placeholder="Search by SKU or product name..."
						value={searchInput}
						oninput={handleSearchInput}
						class="max-w-sm"
					/>
				</div>
				<div class="w-[180px]">
					<select
						bind:value={selectedWarehouse}
						class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						onchange={() => (currentPage = 1)}
						disabled={isLoadingWarehouses}
					>
						<option value="">All Warehouses</option>
						{#each warehouses as warehouse (warehouse.warehouseId)}
							<option value={warehouse.warehouseId}>{warehouse.warehouseName}</option>
						{/each}
					</select>
				</div>
				<div class="w-[150px]">
					<select
						bind:value={selectedStatus}
						class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
						onchange={() => (currentPage = 1)}
					>
						<option value="all">All Status</option>
						<option value="in_stock">In Stock</option>
						<option value="low_stock">Low Stock</option>
						<option value="out_of_stock">Out of Stock</option>
					</select>
				</div>
				{#if searchQuery || selectedWarehouse || selectedStatus !== 'all'}
					<Button variant="ghost" size="sm" onclick={clearFilters}>Clear</Button>
				{/if}
			</div>
		</CardContent>
	</Card>

	<!-- Stock Levels Table -->
	<Card>
		<CardHeader>
			<CardTitle>Stock Level List ({totalItems})</CardTitle>
		</CardHeader>
		<CardContent>
			{#if error}
				<div class="rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-center">
					<p class="text-destructive">{error}</p>
					<Button variant="outline" size="sm" class="mt-2" onclick={fetchStockLevels}>
						Try Again
					</Button>
				</div>
			{:else if isLoading}
				<div class="flex items-center justify-center py-12">
					<div
						class="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"
					></div>
					<span class="ml-3 text-muted-foreground">Loading stock levels...</span>
				</div>
			{:else}
				<div class="overflow-x-auto">
					<table class="w-full">
						<thead>
							<tr class="border-b text-left text-sm text-muted-foreground">
								<th
									class="cursor-pointer p-3 hover:text-foreground"
									onclick={() => handleSort('product_sku')}
								>
									SKU {sortBy === 'product_sku' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
								</th>
								<th
									class="cursor-pointer p-3 hover:text-foreground"
									onclick={() => handleSort('product_name')}
								>
									Product {sortBy === 'product_name' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
								</th>
								<th
									class="cursor-pointer p-3 hover:text-foreground"
									onclick={() => handleSort('warehouse_name')}
								>
									Warehouse {sortBy === 'warehouse_name'
										? sortOrder === 'asc'
											? '↑'
											: '↓'
										: ''}
								</th>
								<th
									class="cursor-pointer p-3 text-right hover:text-foreground"
									onclick={() => handleSort('available_quantity')}
								>
									Available {sortBy === 'available_quantity'
										? sortOrder === 'asc'
											? '↑'
											: '↓'
										: ''}
								</th>
								<th class="p-3 text-right">Reserved</th>
								<th class="p-3 text-right">Total</th>
								<th class="p-3">Status</th>
								<th class="p-3">Updated</th>
							</tr>
						</thead>
						<tbody>
							{#each stockLevels as item (item.inventoryId)}
								<tr class="border-b hover:bg-muted/50">
									<td class="p-3 font-mono text-sm">{item.productSku}</td>
									<td class="p-3">
										<a
											href="/inventory/products/{item.productId}"
											class="font-medium hover:underline"
										>
											{item.productName}
										</a>
									</td>
									<td class="p-3">
										<div>
											<span class="font-medium">{item.warehouseCode}</span>
											<p class="text-sm text-muted-foreground">{item.warehouseName}</p>
										</div>
									</td>
									<td class="p-3 text-right font-medium">
										{formatNumber(item.availableQuantity)}
									</td>
									<td class="p-3 text-right text-muted-foreground">
										{formatNumber(item.reservedQuantity)}
									</td>
									<td class="p-3 text-right font-medium">
										{formatNumber(item.totalQuantity)}
									</td>
									<td class="p-3">
										<Badge variant={getStatusBadgeVariant(item.status)}>
											{getStatusLabel(item.status)}
										</Badge>
									</td>
									<td class="p-3 text-sm text-muted-foreground">
										{new Date(item.updatedAt).toLocaleDateString('vi-VN')}
									</td>
								</tr>
							{:else}
								<tr>
									<td colspan="8" class="p-8 text-center text-muted-foreground">
										{#if searchQuery || selectedWarehouse || selectedStatus !== 'all'}
											No stock levels match your filters.
											<button class="text-primary hover:underline ml-1" onclick={clearFilters}>
												Clear filters
											</button>
										{:else}
											No stock levels found.
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
								<option value={20}>20</option>
								<option value={50}>50</option>
								<option value={100}>100</option>
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
