<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { mockPriceLists, type PriceList, type PriceListType } from '$lib/types/pricing';

	// State
	let searchQuery = $state('');
	let searchInput = $state('');
	let selectedType = $state<PriceListType | ''>('');
	let selectedStatus = $state<'active' | 'inactive' | ''>('');
	let selectedPriceLists = $state<Set<string>>(new Set());
	let sortBy = $state<'name' | 'code' | 'priority' | 'createdAt'>('name');
	let sortOrder = $state<'asc' | 'desc'>('asc');

	// Pagination state
	let currentPage = $state(1);
	let itemsPerPage = $state(10);

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

	// Derived filtered price lists
	const filteredPriceLists = $derived.by(() => {
		let result = [...mockPriceLists];

		// Search filter
		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			result = result.filter(
				(pl) =>
					pl.name.toLowerCase().includes(query) ||
					pl.code.toLowerCase().includes(query) ||
					(pl.description?.toLowerCase().includes(query) ?? false)
			);
		}

		// Type filter
		if (selectedType) {
			result = result.filter((pl) => pl.priceListType === selectedType);
		}

		// Status filter
		if (selectedStatus) {
			const isActive = selectedStatus === 'active';
			result = result.filter((pl) => pl.isActive === isActive);
		}

		// Sort
		result.sort((a, b) => {
			let aVal: string | number;
			let bVal: string | number;

			switch (sortBy) {
				case 'code':
					aVal = a.code;
					bVal = b.code;
					break;
				case 'priority':
					aVal = a.priority;
					bVal = b.priority;
					break;
				case 'createdAt':
					aVal = a.createdAt.getTime();
					bVal = b.createdAt.getTime();
					break;
				default:
					aVal = a.name;
					bVal = b.name;
			}

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

	// Pagination derived values
	const totalItems = $derived(filteredPriceLists.length);
	const totalPages = $derived(Math.ceil(totalItems / itemsPerPage));
	const paginatedPriceLists = $derived(
		filteredPriceLists.slice((currentPage - 1) * itemsPerPage, currentPage * itemsPerPage)
	);
	const showingFrom = $derived((currentPage - 1) * itemsPerPage + 1);
	const showingTo = $derived(Math.min(currentPage * itemsPerPage, totalItems));

	// Helpers
	function toggleSelection(id: string) {
		const newSet = new Set(selectedPriceLists);
		if (newSet.has(id)) {
			newSet.delete(id);
		} else {
			newSet.add(id);
		}
		selectedPriceLists = newSet;
	}

	function toggleAll() {
		if (selectedPriceLists.size === paginatedPriceLists.length) {
			selectedPriceLists = new Set();
		} else {
			selectedPriceLists = new Set(paginatedPriceLists.map((pl) => pl.priceListId));
		}
	}

	function getStatusBadgeVariant(isActive: boolean): 'default' | 'secondary' {
		return isActive ? 'default' : 'secondary';
	}

	function getTypeBadgeClass(type: PriceListType): string {
		return type === 'sale'
			? 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200'
			: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200';
	}

	function handleSort(column: typeof sortBy) {
		if (sortBy === column) {
			sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
		} else {
			sortBy = column;
			sortOrder = 'asc';
		}
	}

	function formatValidity(pl: PriceList): string {
		if (!pl.validFrom && !pl.validTo) return 'Always';
		const from = pl.validFrom ? pl.validFrom.toLocaleDateString('vi-VN') : '';
		const to = pl.validTo ? pl.validTo.toLocaleDateString('vi-VN') : '';
		if (from && to) return `${from} - ${to}`;
		if (from) return `From ${from}`;
		if (to) return `Until ${to}`;
		return 'Always';
	}

	function getValidityStatus(pl: PriceList): 'active' | 'scheduled' | 'expired' | null {
		if (!pl.validFrom && !pl.validTo) return null;
		const now = new Date();
		if (pl.validFrom && now < pl.validFrom) return 'scheduled';
		if (pl.validTo && now > pl.validTo) return 'expired';
		return 'active';
	}

	function goToPage(page: number) {
		if (page >= 1 && page <= totalPages) {
			currentPage = page;
			selectedPriceLists = new Set();
		}
	}

	function clearFilters() {
		searchQuery = '';
		searchInput = '';
		selectedType = '';
		selectedStatus = '';
		currentPage = 1;
	}

	function handleDeleteSelected() {
		if (confirm(`Delete ${selectedPriceLists.size} selected price lists?`)) {
			console.log('Deleting:', Array.from(selectedPriceLists));
			selectedPriceLists = new Set();
		}
	}

	function handleBulkActivate() {
		console.log('Activating:', Array.from(selectedPriceLists));
		selectedPriceLists = new Set();
	}

	function handleBulkDeactivate() {
		console.log('Deactivating:', Array.from(selectedPriceLists));
		selectedPriceLists = new Set();
	}
</script>

<svelte:head>
	<title>Price Lists - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Price Lists</h1>
			<p class="text-muted-foreground">Manage pricing for different customer groups</p>
		</div>
		<Button href="/inventory/pricing/price-lists/new">New Price List</Button>
	</div>

	<!-- Filters -->
	<Card>
		<CardContent class="pt-6">
			<div class="flex flex-wrap items-end gap-4">
				<div class="min-w-[200px] flex-1">
					<Input
						type="search"
						placeholder="Search by name or code..."
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
						<option value="sale">Sale</option>
						<option value="purchase">Purchase</option>
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
				{#if searchQuery || selectedType || selectedStatus}
					<Button variant="ghost" size="sm" onclick={clearFilters}>Clear</Button>
				{/if}
			</div>
		</CardContent>
	</Card>

	<!-- Bulk Actions -->
	{#if selectedPriceLists.size > 0}
		<div class="flex items-center gap-4 rounded-lg bg-muted p-4">
			<span class="text-sm font-medium">{selectedPriceLists.size} selected</span>
			<Button variant="outline" size="sm" onclick={handleBulkActivate}>Activate</Button>
			<Button variant="outline" size="sm" onclick={handleBulkDeactivate}>Deactivate</Button>
			<Button variant="destructive" size="sm" onclick={handleDeleteSelected}>Delete</Button>
		</div>
	{/if}

	<!-- Price Lists Table -->
	<Card>
		<CardHeader>
			<CardTitle>Price Lists ({totalItems})</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead>
						<tr class="border-b text-left text-sm text-muted-foreground">
							<th class="w-10 p-3">
								<input
									type="checkbox"
									checked={selectedPriceLists.size === paginatedPriceLists.length &&
										paginatedPriceLists.length > 0}
									onchange={toggleAll}
									class="rounded"
								/>
							</th>
							<th
								class="cursor-pointer p-3 hover:text-foreground"
								onclick={() => handleSort('name')}
							>
								Name {sortBy === 'name' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
							</th>
							<th
								class="cursor-pointer p-3 hover:text-foreground"
								onclick={() => handleSort('code')}
							>
								Code {sortBy === 'code' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
							</th>
							<th class="p-3">Type</th>
							<th class="p-3 text-right">Items</th>
							<th class="p-3">Validity</th>
							<th
								class="cursor-pointer p-3 hover:text-foreground"
								onclick={() => handleSort('priority')}
							>
								Priority {sortBy === 'priority' ? (sortOrder === 'asc' ? '↑' : '↓') : ''}
							</th>
							<th class="p-3">Status</th>
							<th class="w-24 p-3">Actions</th>
						</tr>
					</thead>
					<tbody>
						{#each paginatedPriceLists as priceList (priceList.priceListId)}
							<tr class="border-b hover:bg-muted/50">
								<td class="p-3">
									<input
										type="checkbox"
										checked={selectedPriceLists.has(priceList.priceListId)}
										onchange={() => toggleSelection(priceList.priceListId)}
										class="rounded"
									/>
								</td>
								<td class="p-3">
									<div>
										<a
											href="/inventory/pricing/price-lists/{priceList.priceListId}"
											class="font-medium hover:underline"
										>
											{priceList.name}
										</a>
										{#if priceList.isDefault}
											<Badge variant="outline" class="ml-2 text-xs">Default</Badge>
										{/if}
										{#if priceList.description}
											<p class="line-clamp-1 text-sm text-muted-foreground">
												{priceList.description}
											</p>
										{/if}
										{#if priceList.basedOn === 'base_price' && priceList.defaultPercentage !== 0}
											<span class="text-xs text-muted-foreground">
												{priceList.defaultPercentage > 0 ? '+' : ''}{priceList.defaultPercentage}%
												from base
											</span>
										{/if}
									</div>
								</td>
								<td class="p-3 font-mono text-sm">{priceList.code}</td>
								<td class="p-3">
									<span
										class="rounded-full px-2 py-1 text-xs font-medium {getTypeBadgeClass(
											priceList.priceListType
										)}"
									>
										{priceList.priceListType}
									</span>
								</td>
								<td class="p-3 text-right">{priceList.itemCount ?? 0}</td>
								<td class="p-3">
									<div class="flex items-center gap-2">
										<span class="text-sm">{formatValidity(priceList)}</span>
										{#if getValidityStatus(priceList) === 'active'}
											<span class="h-2 w-2 rounded-full bg-green-500"></span>
										{:else if getValidityStatus(priceList) === 'scheduled'}
											<span class="h-2 w-2 rounded-full bg-yellow-500"></span>
										{:else if getValidityStatus(priceList) === 'expired'}
											<span class="h-2 w-2 rounded-full bg-red-500"></span>
										{/if}
									</div>
								</td>
								<td class="p-3 text-center">{priceList.priority}</td>
								<td class="p-3">
									<Badge variant={getStatusBadgeVariant(priceList.isActive)}>
										{priceList.isActive ? 'Active' : 'Inactive'}
									</Badge>
								</td>
								<td class="p-3">
									<div class="flex gap-1">
										<Button
											variant="ghost"
											size="sm"
											href="/inventory/pricing/price-lists/{priceList.priceListId}"
										>
											View
										</Button>
									</div>
								</td>
							</tr>
						{:else}
							<tr>
								<td colspan="9" class="p-8 text-center text-muted-foreground">
									{#if searchQuery || selectedType || selectedStatus}
										No price lists match your filters.
										<button class="ml-1 text-primary hover:underline" onclick={clearFilters}>
											Clear filters
										</button>
									{:else}
										No price lists found.
										<a
											href="/inventory/pricing/price-lists/new"
											class="text-primary hover:underline"
										>
											Create your first price list
										</a>
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
						}).filter((p) => p <= totalPages) as page}
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
		</CardContent>
	</Card>
</div>
