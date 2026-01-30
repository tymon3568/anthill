<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Label } from '$lib/components/ui/label';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import { WarehouseCard, WarehouseForm } from '$lib/components/inventory';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import type {
		WarehouseResponse,
		CreateWarehouseRequest,
		UpdateWarehouseRequest
	} from '$lib/types/inventory';

	// Page state
	let searchQuery = $state('');
	let showInactive = $state(false);
	let selectedType = $state<string | null>(null);
	let viewMode = $state<'grid' | 'list'>('grid');
	let isFormOpen = $state(false);
	let editingWarehouse = $state<WarehouseResponse | null>(null);
	let isSubmitting = $state(false);

	// Delete confirmation state
	let deleteDialogOpen = $state(false);
	let warehouseToDelete = $state<WarehouseResponse | null>(null);
	let isDeleting = $state(false);

	const warehouseTypes = [
		{ value: 'main', label: 'Main Warehouse' },
		{ value: 'satellite', label: 'Satellite' },
		{ value: 'distribution', label: 'Distribution Center' },
		{ value: 'storage', label: 'Storage Facility' }
	];

	// Computed values
	const filteredWarehouses = $derived.by(() => {
		let result = showInactive
			? warehouseState.items
			: warehouseState.items.filter((w) => w.isActive);

		// Filter by search query
		if (searchQuery.trim()) {
			const query = searchQuery.toLowerCase();
			result = result.filter(
				(w) =>
					w.warehouseName.toLowerCase().includes(query) ||
					w.warehouseCode.toLowerCase().includes(query) ||
					w.description?.toLowerCase().includes(query)
			);
		}

		// Filter by type
		if (selectedType) {
			result = result.filter((w) => w.warehouseType === selectedType);
		}

		return result;
	});

	const stats = $derived.by(() => {
		const total = warehouseState.items.length;
		const active = warehouseState.items.filter((w) => w.isActive).length;
		const inactive = total - active;
		const byType = warehouseTypes.reduce(
			(acc, type) => {
				acc[type.value] = warehouseState.items.filter((w) => w.warehouseType === type.value).length;
				return acc;
			},
			{} as Record<string, number>
		);
		return { total, active, inactive, byType };
	});

	async function loadWarehouses() {
		await warehouseStore.load();
	}

	function handleAddWarehouse() {
		editingWarehouse = null;
		isFormOpen = true;
	}

	function handleView(warehouse: WarehouseResponse) {
		goto(`/inventory/warehouses/${warehouse.warehouseId}`);
	}

	function handleEdit(warehouse: WarehouseResponse) {
		editingWarehouse = warehouse;
		isFormOpen = true;
	}

	function handleDeleteClick(warehouse: WarehouseResponse) {
		warehouseToDelete = warehouse;
		deleteDialogOpen = true;
	}

	async function handleDelete() {
		if (!warehouseToDelete) return;

		isDeleting = true;
		const success = await warehouseStore.delete(warehouseToDelete.warehouseId);

		if (success) {
			deleteDialogOpen = false;
			warehouseToDelete = null;
		}
		isDeleting = false;
	}

	function handleFormClose() {
		isFormOpen = false;
		editingWarehouse = null;
	}

	async function handleFormSubmit(data: CreateWarehouseRequest | UpdateWarehouseRequest) {
		isSubmitting = true;

		if (editingWarehouse) {
			const result = await warehouseStore.update(editingWarehouse.warehouseId, data);
			if (result) {
				handleFormClose();
			}
		} else {
			const result = await warehouseStore.create(data as CreateWarehouseRequest);
			if (result) {
				handleFormClose();
			}
		}

		isSubmitting = false;
	}

	function handleTypeChange(value: string | undefined) {
		selectedType = value ?? null;
	}

	function clearFilters() {
		searchQuery = '';
		selectedType = null;
		showInactive = false;
	}

	onMount(() => {
		loadWarehouses();
	});
</script>

<svelte:head>
	<title>Warehouses | Inventory</title>
</svelte:head>

<div class="container mx-auto space-y-6 py-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Warehouses</h1>
			<p class="text-muted-foreground">Manage your warehouse locations and storage facilities</p>
		</div>
		<Button onclick={handleAddWarehouse}>+ Add Warehouse</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
		<Card>
			<CardContent class="pt-6">
				<div class="text-2xl font-bold">{stats.total}</div>
				<p class="text-sm text-muted-foreground">Total Warehouses</p>
			</CardContent>
		</Card>
		<Card>
			<CardContent class="pt-6">
				<div class="text-2xl font-bold text-green-600">{stats.active}</div>
				<p class="text-sm text-muted-foreground">Active</p>
			</CardContent>
		</Card>
		<Card>
			<CardContent class="pt-6">
				<div class="text-2xl font-bold text-orange-600">{stats.inactive}</div>
				<p class="text-sm text-muted-foreground">Inactive</p>
			</CardContent>
		</Card>
		<Card>
			<CardContent class="pt-6">
				<div class="text-2xl font-bold">{stats.byType['main'] || 0}</div>
				<p class="text-sm text-muted-foreground">Main Warehouses</p>
			</CardContent>
		</Card>
	</div>

	<!-- Filters -->
	<div class="flex flex-wrap items-center gap-4">
		<div class="flex-1">
			<Input placeholder="Search warehouses..." bind:value={searchQuery} class="max-w-sm" />
		</div>

		<Select.Root type="single" value={selectedType ?? undefined} onValueChange={handleTypeChange}>
			<Select.Trigger class="w-48">
				{selectedType ? warehouseTypes.find((t) => t.value === selectedType)?.label : 'All Types'}
			</Select.Trigger>
			<Select.Content>
				<Select.Item value="">All Types</Select.Item>
				{#each warehouseTypes as type (type.value)}
					<Select.Item value={type.value}>{type.label}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>

		<div class="flex items-center gap-2">
			<Checkbox id="showInactive" bind:checked={showInactive} />
			<Label for="showInactive" class="font-normal">Show inactive</Label>
		</div>

		<!-- View Mode Toggle -->
		<div class="flex items-center gap-1 rounded-md border p-1">
			<Button
				variant={viewMode === 'grid' ? 'secondary' : 'ghost'}
				size="sm"
				onclick={() => (viewMode = 'grid')}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<rect x="3" y="3" width="7" height="7" />
					<rect x="14" y="3" width="7" height="7" />
					<rect x="3" y="14" width="7" height="7" />
					<rect x="14" y="14" width="7" height="7" />
				</svg>
			</Button>
			<Button
				variant={viewMode === 'list' ? 'secondary' : 'ghost'}
				size="sm"
				onclick={() => (viewMode = 'list')}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<line x1="8" y1="6" x2="21" y2="6" />
					<line x1="8" y1="12" x2="21" y2="12" />
					<line x1="8" y1="18" x2="21" y2="18" />
					<line x1="3" y1="6" x2="3.01" y2="6" />
					<line x1="3" y1="12" x2="3.01" y2="12" />
					<line x1="3" y1="18" x2="3.01" y2="18" />
				</svg>
			</Button>
		</div>

		{#if searchQuery || selectedType || showInactive}
			<Button variant="ghost" size="sm" onclick={clearFilters}>Clear filters</Button>
		{/if}
	</div>

	<!-- Error State -->
	{#if warehouseState.error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{warehouseState.error}</p>
				<Button variant="outline" size="sm" class="mt-2" onclick={loadWarehouses}>Retry</Button>
			</CardContent>
		</Card>
	{:else if warehouseState.isLoading}
		<!-- Loading State -->
		<div class="grid gap-4 {viewMode === 'grid' ? 'sm:grid-cols-2 lg:grid-cols-3' : ''}">
			{#each Array(6) as _, i (i)}
				<Card class="animate-pulse">
					<CardContent class="pt-6">
						<div class="h-4 w-3/4 rounded bg-muted"></div>
						<div class="mt-2 h-3 w-1/2 rounded bg-muted"></div>
						<div class="mt-4 grid grid-cols-2 gap-4">
							<div class="h-12 rounded bg-muted"></div>
							<div class="h-12 rounded bg-muted"></div>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{:else if filteredWarehouses.length === 0}
		<!-- Empty State -->
		<Card>
			<CardContent class="py-12 text-center">
				{#if warehouseState.items.length === 0}
					<p class="text-muted-foreground">No warehouses found</p>
					<p class="mt-1 text-sm text-muted-foreground">
						Get started by adding your first warehouse
					</p>
					<Button class="mt-4" onclick={handleAddWarehouse}>Add Warehouse</Button>
				{:else}
					<p class="text-muted-foreground">No warehouses match your filters</p>
					<Button variant="outline" class="mt-4" onclick={clearFilters}>Clear filters</Button>
				{/if}
			</CardContent>
		</Card>
	{:else}
		<!-- Warehouse Grid/List -->
		<div class="grid gap-4 {viewMode === 'grid' ? 'sm:grid-cols-2 lg:grid-cols-3' : ''}">
			{#each filteredWarehouses as warehouse (warehouse.warehouseId)}
				<WarehouseCard
					{warehouse}
					onView={handleView}
					onEdit={handleEdit}
					onDelete={handleDeleteClick}
				/>
			{/each}
		</div>
	{/if}
</div>

<!-- Warehouse Form Modal -->
<WarehouseForm
	open={isFormOpen}
	warehouse={editingWarehouse}
	{isSubmitting}
	onClose={handleFormClose}
	onSubmit={handleFormSubmit}
/>

<!-- Delete Confirmation Dialog -->
<Dialog.Root bind:open={deleteDialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Delete Warehouse</Dialog.Title>
			<Dialog.Description>
				{#if warehouseToDelete}
					Are you sure you want to delete "{warehouseToDelete.warehouseName}"?
					<p class="mt-2 font-medium text-destructive">
						This action cannot be undone. All zones and locations within this warehouse will also be
						deleted.
					</p>
				{/if}
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" disabled={isDeleting} onclick={() => (deleteDialogOpen = false)}>
				Cancel
			</Button>
			<Button variant="destructive" onclick={handleDelete} disabled={isDeleting}>
				{isDeleting ? 'Deleting...' : 'Delete'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
