<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Tabs from '$lib/components/ui/tabs';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import {
		WarehouseForm,
		WarehouseTreeView,
		LocationStockSummary
	} from '$lib/components/inventory';
	import { warehouseApi } from '$lib/api/inventory/warehouses';
	import type {
		WarehouseResponse,
		WarehouseZoneResponse,
		WarehouseLocationResponse,
		CreateWarehouseRequest,
		CreateWarehouseZoneRequest,
		CreateWarehouseLocationRequest
	} from '$lib/types/inventory';

	// Get warehouse ID from route params - early return if not available
	const warehouseId = $derived($page.params.id ?? '');

	// State
	let warehouse = $state<WarehouseResponse | null>(null);
	let zones = $state<WarehouseZoneResponse[]>([]);
	let locations = $state<WarehouseLocationResponse[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let activeTab = $state('overview');

	// Edit warehouse state
	let isWarehouseFormOpen = $state(false);
	let isSubmittingWarehouse = $state(false);

	// Zone form state
	let isZoneFormOpen = $state(false);
	let editingZone = $state<WarehouseZoneResponse | null>(null);
	let isSubmittingZone = $state(false);

	// Zone form fields
	let zoneCode = $state('');
	let zoneName = $state('');
	let zoneType = $state('storage');
	let zoneDescription = $state('');

	// Location form state
	let isLocationFormOpen = $state(false);
	let editingLocation = $state<WarehouseLocationResponse | null>(null);
	let isSubmittingLocation = $state(false);

	// Location form fields
	let locationCode = $state('');
	let locationName = $state('');
	let locationType = $state('shelf');
	let locationDescription = $state('');
	let locationZoneId = $state<string | null>(null);

	// Delete confirmation state
	let deleteDialogOpen = $state(false);
	let deleteType = $state<'zone' | 'location' | null>(null);
	let itemToDelete = $state<WarehouseZoneResponse | WarehouseLocationResponse | null>(null);
	let isDeleting = $state(false);

	// Stock view state
	let stockViewOpen = $state(false);
	let stockViewLocation = $state<WarehouseLocationResponse | null>(null);

	const zoneTypes = [
		{ value: 'receiving', label: 'Receiving' },
		{ value: 'storage', label: 'Storage' },
		{ value: 'shipping', label: 'Shipping' },
		{ value: 'quarantine', label: 'Quarantine' },
		{ value: 'returns', label: 'Returns' }
	];

	const locationTypes = [
		{ value: 'shelf', label: 'Shelf' },
		{ value: 'bin', label: 'Bin' },
		{ value: 'rack', label: 'Rack' },
		{ value: 'floor', label: 'Floor' },
		{ value: 'pallet', label: 'Pallet' }
	];

	const warehouseTypeIcons: Record<string, string> = {
		main: '🏭',
		satellite: '🏢',
		distribution: '📦',
		storage: '🗄️',
		default: '🏠'
	};

	const zoneTypeIcons: Record<string, string> = {
		receiving: '📥',
		storage: '📦',
		shipping: '📤',
		quarantine: '🔒',
		returns: '↩️',
		default: '📍'
	};

	// Stats
	const stats = $derived.by(() => {
		const activeZones = zones.filter((z) => z.isActive).length;
		const activeLocations = locations.filter((l) => l.isActive).length;
		const locationsWithZone = locations.filter((l) => l.zoneId).length;
		return {
			totalZones: zones.length,
			activeZones,
			totalLocations: locations.length,
			activeLocations,
			locationsWithZone
		};
	});

	async function loadWarehouse() {
		isLoading = true;
		error = null;

		const response = await warehouseApi.get(warehouseId);

		if (response.success && response.data) {
			warehouse = response.data;
		} else {
			error = response.error || 'Failed to load warehouse';
		}

		isLoading = false;
	}

	async function loadZones() {
		const response = await warehouseApi.listZones(warehouseId, { pageSize: 100 });
		if (response.success && response.data) {
			zones = response.data.zones;
		}
	}

	async function loadLocations() {
		const response = await warehouseApi.listLocations(warehouseId, { pageSize: 100 });
		if (response.success && response.data) {
			locations = response.data.locations;
		}
	}

	async function loadAll() {
		await Promise.all([loadWarehouse(), loadZones(), loadLocations()]);
	}

	// Warehouse handlers
	function handleEditWarehouse() {
		isWarehouseFormOpen = true;
	}

	async function handleWarehouseSubmit(data: CreateWarehouseRequest) {
		isSubmittingWarehouse = true;

		const response = await warehouseApi.update(warehouseId, data);
		if (response.success && response.data) {
			warehouse = response.data;
			isWarehouseFormOpen = false;
		}

		isSubmittingWarehouse = false;
	}

	// Zone handlers
	function handleAddZone() {
		editingZone = null;
		resetZoneForm();
		isZoneFormOpen = true;
	}

	function handleEditZone(zone: WarehouseZoneResponse) {
		editingZone = zone;
		zoneCode = zone.zoneCode;
		zoneName = zone.zoneName;
		zoneType = zone.zoneType;
		zoneDescription = zone.description || '';
		isZoneFormOpen = true;
	}

	function handleDeleteZoneClick(zone: WarehouseZoneResponse) {
		deleteType = 'zone';
		itemToDelete = zone;
		deleteDialogOpen = true;
	}

	function resetZoneForm() {
		zoneCode = '';
		zoneName = '';
		zoneType = 'storage';
		zoneDescription = '';
	}

	async function handleZoneSubmit(e: Event) {
		e.preventDefault();
		isSubmittingZone = true;

		const data: CreateWarehouseZoneRequest = {
			zoneCode,
			zoneName,
			zoneType,
			description: zoneDescription || undefined
		};

		if (editingZone) {
			const response = await warehouseApi.updateZone(warehouseId, editingZone.zoneId, data);
			if (response.success) {
				await loadZones();
				isZoneFormOpen = false;
				editingZone = null;
				resetZoneForm();
			}
		} else {
			const response = await warehouseApi.createZone(warehouseId, data);
			if (response.success) {
				await loadZones();
				isZoneFormOpen = false;
				resetZoneForm();
			}
		}

		isSubmittingZone = false;
	}

	// Location handlers
	function handleAddLocation(zoneId?: string) {
		editingLocation = null;
		resetLocationForm();
		if (zoneId) {
			locationZoneId = zoneId;
		}
		isLocationFormOpen = true;
	}

	function handleEditLocation(location: WarehouseLocationResponse) {
		editingLocation = location;
		locationCode = location.locationCode;
		locationName = location.locationName || '';
		locationType = location.locationType;
		locationDescription = location.description || '';
		locationZoneId = location.zoneId || null;
		isLocationFormOpen = true;
	}

	function handleDeleteLocationClick(location: WarehouseLocationResponse) {
		deleteType = 'location';
		itemToDelete = location;
		deleteDialogOpen = true;
	}

	function resetLocationForm() {
		locationCode = '';
		locationName = '';
		locationType = 'shelf';
		locationDescription = '';
		locationZoneId = null;
	}

	async function handleLocationSubmit(e: Event) {
		e.preventDefault();
		isSubmittingLocation = true;

		const data: CreateWarehouseLocationRequest = {
			locationCode,
			locationType,
			locationName: locationName || undefined,
			description: locationDescription || undefined,
			zoneId: locationZoneId || undefined
		};

		if (editingLocation) {
			const response = await warehouseApi.updateLocation(
				warehouseId,
				editingLocation.locationId,
				data
			);
			if (response.success) {
				await loadLocations();
				isLocationFormOpen = false;
				editingLocation = null;
				resetLocationForm();
			}
		} else {
			const response = await warehouseApi.createLocation(warehouseId, data);
			if (response.success) {
				await loadLocations();
				isLocationFormOpen = false;
				resetLocationForm();
			}
		}

		isSubmittingLocation = false;
	}

	// Delete handler
	async function handleDelete() {
		if (!itemToDelete) return;

		isDeleting = true;

		if (deleteType === 'zone') {
			const zone = itemToDelete as WarehouseZoneResponse;
			const response = await warehouseApi.deleteZone(warehouseId, zone.zoneId);
			if (response.success) {
				await loadZones();
				deleteDialogOpen = false;
				itemToDelete = null;
				deleteType = null;
			}
		} else if (deleteType === 'location') {
			const location = itemToDelete as WarehouseLocationResponse;
			const response = await warehouseApi.deleteLocation(warehouseId, location.locationId);
			if (response.success) {
				await loadLocations();
				deleteDialogOpen = false;
				itemToDelete = null;
				deleteType = null;
			}
		}

		isDeleting = false;
	}

	function getZoneName(zoneId: string | null | undefined): string {
		if (!zoneId) return 'Unassigned';
		const zone = zones.find((z) => z.zoneId === zoneId);
		return zone?.zoneName || 'Unknown';
	}

	function getLocationCountForZone(zoneId: string): number {
		return locations.filter((l) => l.zoneId === zoneId).length;
	}

	function handleViewStock(location: WarehouseLocationResponse) {
		stockViewLocation = location;
		stockViewOpen = true;
	}

	onMount(() => {
		loadAll();
	});
</script>

<svelte:head>
	<title>{warehouse?.warehouseName || 'Warehouse'} | Inventory</title>
</svelte:head>

<div class="container mx-auto space-y-6 py-6">
	<!-- Breadcrumb & Header -->
	<div class="flex items-center gap-2 text-sm text-muted-foreground">
		<a href="/inventory/warehouses" class="hover:text-foreground">Warehouses</a>
		<span>/</span>
		<span class="text-foreground">{warehouse?.warehouseName || 'Loading...'}</span>
	</div>

	{#if isLoading}
		<div class="space-y-4">
			<div class="h-8 w-48 animate-pulse rounded bg-muted"></div>
			<div class="h-4 w-96 animate-pulse rounded bg-muted"></div>
		</div>
	{:else if error}
		<Card class="border-destructive">
			<CardContent class="pt-6">
				<p class="text-destructive">{error}</p>
				<Button variant="outline" size="sm" class="mt-2" onclick={loadAll}>Retry</Button>
			</CardContent>
		</Card>
	{:else if warehouse}
		<!-- Header -->
		<div class="flex items-start justify-between">
			<div class="flex items-center gap-3">
				<span class="text-3xl"
					>{warehouseTypeIcons[warehouse.warehouseType] || warehouseTypeIcons.default}</span
				>
				<div>
					<div class="flex items-center gap-2">
						<h1 class="text-2xl font-bold">{warehouse.warehouseName}</h1>
						<Badge variant={warehouse.isActive ? 'default' : 'secondary'}>
							{warehouse.isActive ? 'Active' : 'Inactive'}
						</Badge>
					</div>
					<p class="text-muted-foreground">{warehouse.warehouseCode} • {warehouse.warehouseType}</p>
				</div>
			</div>
			<div class="flex gap-2">
				<Button variant="outline" onclick={handleEditWarehouse}>Edit Warehouse</Button>
				<Button variant="outline" onclick={() => goto('/inventory/warehouses')}>Back to List</Button
				>
			</div>
		</div>

		<!-- Stats Cards -->
		<div class="grid grid-cols-2 gap-4 sm:grid-cols-4">
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold">{stats.totalZones}</div>
					<p class="text-sm text-muted-foreground">Total Zones</p>
				</CardContent>
			</Card>
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold text-green-600">{stats.activeZones}</div>
					<p class="text-sm text-muted-foreground">Active Zones</p>
				</CardContent>
			</Card>
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold">{stats.totalLocations}</div>
					<p class="text-sm text-muted-foreground">Total Locations</p>
				</CardContent>
			</Card>
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold text-green-600">{stats.activeLocations}</div>
					<p class="text-sm text-muted-foreground">Active Locations</p>
				</CardContent>
			</Card>
		</div>

		<!-- Tabs -->
		<Tabs.Root bind:value={activeTab}>
			<Tabs.List>
				<Tabs.Trigger value="overview">Overview</Tabs.Trigger>
				<Tabs.Trigger value="tree">Tree View</Tabs.Trigger>
				<Tabs.Trigger value="zones">Zones ({stats.totalZones})</Tabs.Trigger>
				<Tabs.Trigger value="locations">Locations ({stats.totalLocations})</Tabs.Trigger>
			</Tabs.List>

			<!-- Overview Tab -->
			<Tabs.Content value="overview">
				<div class="grid gap-6 lg:grid-cols-2">
					<!-- Warehouse Info -->
					<Card>
						<CardHeader>
							<CardTitle>Warehouse Information</CardTitle>
						</CardHeader>
						<CardContent class="space-y-4">
							{#if warehouse.description}
								<div>
									<p class="text-sm font-medium text-muted-foreground">Description</p>
									<p>{warehouse.description}</p>
								</div>
							{/if}

							{#if warehouse.address && typeof warehouse.address === 'object'}
								{@const addr = warehouse.address as Record<string, string>}
								<div>
									<p class="text-sm font-medium text-muted-foreground">Address</p>
									<p>
										{addr.street ? `${addr.street}, ` : ''}
										{addr.city ? `${addr.city}, ` : ''}
										{addr.state ? `${addr.state} ` : ''}
										{addr.postalCode || ''}
									</p>
									{#if addr.country}
										<p>{addr.country}</p>
									{/if}
								</div>
							{/if}

							{#if warehouse.contactInfo && typeof warehouse.contactInfo === 'object'}
								{@const contact = warehouse.contactInfo as Record<string, string>}
								<div>
									<p class="text-sm font-medium text-muted-foreground">Contact</p>
									{#if contact.name}
										<p>{contact.name}</p>
									{/if}
									{#if contact.phone}
										<p class="text-sm">{contact.phone}</p>
									{/if}
									{#if contact.email}
										<p class="text-sm">{contact.email}</p>
									{/if}
								</div>
							{/if}

							<div class="grid grid-cols-2 gap-4 pt-2">
								<div>
									<p class="text-sm font-medium text-muted-foreground">Created</p>
									<p class="text-sm">{new Date(warehouse.createdAt).toLocaleDateString()}</p>
								</div>
								<div>
									<p class="text-sm font-medium text-muted-foreground">Updated</p>
									<p class="text-sm">{new Date(warehouse.updatedAt).toLocaleDateString()}</p>
								</div>
							</div>
						</CardContent>
					</Card>

					<!-- Quick Actions -->
					<Card>
						<CardHeader>
							<CardTitle>Quick Actions</CardTitle>
						</CardHeader>
						<CardContent class="space-y-3">
							<Button class="w-full justify-start" variant="outline" onclick={handleAddZone}>
								+ Add Zone
							</Button>
							<Button
								class="w-full justify-start"
								variant="outline"
								onclick={() => handleAddLocation()}
							>
								+ Add Location
							</Button>
							<Button
								class="w-full justify-start"
								variant="outline"
								onclick={() => (activeTab = 'zones')}
							>
								Manage Zones
							</Button>
							<Button
								class="w-full justify-start"
								variant="outline"
								onclick={() => (activeTab = 'locations')}
							>
								Manage Locations
							</Button>
						</CardContent>
					</Card>
				</div>
			</Tabs.Content>

			<!-- Tree View Tab -->
			<Tabs.Content value="tree">
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle>Warehouse Structure</CardTitle>
							<div class="flex gap-2">
								<Button variant="outline" size="sm" onclick={handleAddZone}>+ Add Zone</Button>
								<Button variant="outline" size="sm" onclick={() => handleAddLocation()}
									>+ Add Location</Button
								>
							</div>
						</div>
					</CardHeader>
					<CardContent>
						{#if zones.length === 0 && locations.length === 0}
							<div class="py-12 text-center">
								<p class="text-muted-foreground">No zones or locations found</p>
								<p class="mt-1 text-sm text-muted-foreground">
									Create zones and locations to build your warehouse structure
								</p>
								<div class="mt-4 flex justify-center gap-2">
									<Button onclick={handleAddZone}>Add Zone</Button>
									<Button variant="outline" onclick={() => handleAddLocation()}>Add Location</Button
									>
								</div>
							</div>
						{:else}
							<WarehouseTreeView
								{zones}
								{locations}
								onAddZone={handleAddZone}
								onAddLocation={handleAddLocation}
								onEditZone={handleEditZone}
								onEditLocation={handleEditLocation}
								onDeleteZone={handleDeleteZoneClick}
								onDeleteLocation={handleDeleteLocationClick}
								showActions={true}
							/>
						{/if}
					</CardContent>
				</Card>
			</Tabs.Content>

			<!-- Zones Tab -->
			<Tabs.Content value="zones">
				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<h3 class="text-lg font-medium">Zones</h3>
						<Button onclick={handleAddZone}>+ Add Zone</Button>
					</div>

					{#if zones.length === 0}
						<Card>
							<CardContent class="py-12 text-center">
								<p class="text-muted-foreground">No zones found</p>
								<p class="mt-1 text-sm text-muted-foreground">
									Create zones to organize your warehouse
								</p>
								<Button class="mt-4" onclick={handleAddZone}>Add Zone</Button>
							</CardContent>
						</Card>
					{:else}
						<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
							{#each zones as zone (zone.zoneId)}
								<Card class="relative">
									<CardHeader class="pb-2">
										<div class="flex items-start justify-between">
											<div class="flex items-center gap-2">
												<span class="text-xl"
													>{zoneTypeIcons[zone.zoneType] || zoneTypeIcons.default}</span
												>
												<div>
													<CardTitle class="text-base">{zone.zoneName}</CardTitle>
													<p class="text-sm text-muted-foreground">{zone.zoneCode}</p>
												</div>
											</div>
											<div class="flex items-center gap-2">
												<Badge variant={zone.isActive ? 'default' : 'secondary'}>
													{zone.isActive ? 'Active' : 'Inactive'}
												</Badge>
												<DropdownMenu.Root>
													<DropdownMenu.Trigger
														class="inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-accent focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none"
													>
														<span class="sr-only">Open menu</span>
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
															<circle cx="12" cy="12" r="1" />
															<circle cx="12" cy="5" r="1" />
															<circle cx="12" cy="19" r="1" />
														</svg>
													</DropdownMenu.Trigger>
													<DropdownMenu.Content align="end">
														<DropdownMenu.Item onclick={() => handleEditZone(zone)}>
															Edit
														</DropdownMenu.Item>
														<DropdownMenu.Item onclick={() => handleAddLocation(zone.zoneId)}>
															Add Location
														</DropdownMenu.Item>
														<DropdownMenu.Separator />
														<DropdownMenu.Item
															class="text-destructive"
															onclick={() => handleDeleteZoneClick(zone)}
														>
															Delete
														</DropdownMenu.Item>
													</DropdownMenu.Content>
												</DropdownMenu.Root>
											</div>
										</div>
									</CardHeader>
									<CardContent>
										{#if zone.description}
											<p class="mb-2 line-clamp-2 text-sm text-muted-foreground">
												{zone.description}
											</p>
										{/if}
										<div class="flex items-center justify-between text-sm">
											<span class="text-muted-foreground">Type: {zone.zoneType}</span>
											<span class="font-medium"
												>{getLocationCountForZone(zone.zoneId)} locations</span
											>
										</div>
									</CardContent>
								</Card>
							{/each}
						</div>
					{/if}
				</div>
			</Tabs.Content>

			<!-- Locations Tab -->
			<Tabs.Content value="locations">
				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<h3 class="text-lg font-medium">Locations</h3>
						<Button onclick={() => handleAddLocation()}>+ Add Location</Button>
					</div>

					{#if locations.length === 0}
						<Card>
							<CardContent class="py-12 text-center">
								<p class="text-muted-foreground">No locations found</p>
								<p class="mt-1 text-sm text-muted-foreground">
									Create locations to store inventory
								</p>
								<Button class="mt-4" onclick={() => handleAddLocation()}>Add Location</Button>
							</CardContent>
						</Card>
					{:else}
						<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
							{#each locations as location (location.locationId)}
								<Card>
									<CardHeader class="pb-2">
										<div class="flex items-start justify-between">
											<div>
												<CardTitle class="text-base">{location.locationCode}</CardTitle>
												{#if location.locationName}
													<p class="text-sm text-muted-foreground">{location.locationName}</p>
												{/if}
											</div>
											<div class="flex items-center gap-2">
												<Badge variant={location.isActive ? 'default' : 'secondary'}>
													{location.isActive ? 'Active' : 'Inactive'}
												</Badge>
												<DropdownMenu.Root>
													<DropdownMenu.Trigger
														class="inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-accent focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none"
													>
														<span class="sr-only">Open menu</span>
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
															<circle cx="12" cy="12" r="1" />
															<circle cx="12" cy="5" r="1" />
															<circle cx="12" cy="19" r="1" />
														</svg>
													</DropdownMenu.Trigger>
													<DropdownMenu.Content align="end">
														<DropdownMenu.Item onclick={() => handleViewStock(location)}>
															View Stock
														</DropdownMenu.Item>
														<DropdownMenu.Item onclick={() => handleEditLocation(location)}>
															Edit
														</DropdownMenu.Item>
														<DropdownMenu.Separator />
														<DropdownMenu.Item
															class="text-destructive"
															onclick={() => handleDeleteLocationClick(location)}
														>
															Delete
														</DropdownMenu.Item>
													</DropdownMenu.Content>
												</DropdownMenu.Root>
											</div>
										</div>
									</CardHeader>
									<CardContent>
										{#if location.description}
											<p class="mb-2 line-clamp-2 text-sm text-muted-foreground">
												{location.description}
											</p>
										{/if}
										<div class="flex items-center justify-between text-sm">
											<span class="text-muted-foreground">Type: {location.locationType}</span>
											<Badge variant="outline">{getZoneName(location.zoneId)}</Badge>
										</div>
									</CardContent>
								</Card>
							{/each}
						</div>
					{/if}
				</div>
			</Tabs.Content>
		</Tabs.Root>
	{/if}
</div>

<!-- Warehouse Form Modal -->
<WarehouseForm
	open={isWarehouseFormOpen}
	{warehouse}
	isSubmitting={isSubmittingWarehouse}
	onClose={() => (isWarehouseFormOpen = false)}
	onSubmit={handleWarehouseSubmit}
/>

<!-- Zone Form Dialog -->
<Dialog.Root bind:open={isZoneFormOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>{editingZone ? 'Edit Zone' : 'Create Zone'}</Dialog.Title>
			<Dialog.Description>
				{editingZone ? 'Update zone information' : 'Add a new zone to this warehouse'}
			</Dialog.Description>
		</Dialog.Header>

		<form onsubmit={handleZoneSubmit} class="space-y-4">
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="zoneCode">Zone Code *</Label>
					<Input
						id="zoneCode"
						bind:value={zoneCode}
						placeholder="ZONE-001"
						required
						disabled={!!editingZone}
					/>
				</div>
				<div class="space-y-2">
					<Label for="zoneType">Type *</Label>
					<select
						id="zoneType"
						bind:value={zoneType}
						class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none"
					>
						{#each zoneTypes as type (type.value)}
							<option value={type.value}>{type.label}</option>
						{/each}
					</select>
				</div>
			</div>

			<div class="space-y-2">
				<Label for="zoneName">Zone Name *</Label>
				<Input id="zoneName" bind:value={zoneName} placeholder="Storage Zone A" required />
			</div>

			<div class="space-y-2">
				<Label for="zoneDescription">Description</Label>
				<Input
					id="zoneDescription"
					bind:value={zoneDescription}
					placeholder="Description of the zone..."
				/>
			</div>

			<Dialog.Footer>
				<Button
					type="button"
					variant="outline"
					onclick={() => (isZoneFormOpen = false)}
					disabled={isSubmittingZone}
				>
					Cancel
				</Button>
				<Button type="submit" disabled={isSubmittingZone || !zoneCode || !zoneName}>
					{isSubmittingZone ? 'Saving...' : editingZone ? 'Update' : 'Create'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Location Form Dialog -->
<Dialog.Root bind:open={isLocationFormOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>{editingLocation ? 'Edit Location' : 'Create Location'}</Dialog.Title>
			<Dialog.Description>
				{editingLocation ? 'Update location information' : 'Add a new location to this warehouse'}
			</Dialog.Description>
		</Dialog.Header>

		<form onsubmit={handleLocationSubmit} class="space-y-4">
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="locationCode">Location Code *</Label>
					<Input
						id="locationCode"
						bind:value={locationCode}
						placeholder="LOC-001"
						required
						disabled={!!editingLocation}
					/>
				</div>
				<div class="space-y-2">
					<Label for="locationType">Type *</Label>
					<select
						id="locationType"
						bind:value={locationType}
						class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none"
					>
						{#each locationTypes as type (type.value)}
							<option value={type.value}>{type.label}</option>
						{/each}
					</select>
				</div>
			</div>

			<div class="space-y-2">
				<Label for="locationName">Location Name</Label>
				<Input id="locationName" bind:value={locationName} placeholder="Shelf A-1" />
			</div>

			<div class="space-y-2">
				<Label for="locationZone">Zone</Label>
				<select
					id="locationZone"
					bind:value={locationZoneId}
					class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none"
				>
					<option value={null}>No Zone (Unassigned)</option>
					{#each zones as zone (zone.zoneId)}
						<option value={zone.zoneId}>{zone.zoneName}</option>
					{/each}
				</select>
			</div>

			<div class="space-y-2">
				<Label for="locationDescription">Description</Label>
				<Input
					id="locationDescription"
					bind:value={locationDescription}
					placeholder="Description of the location..."
				/>
			</div>

			<Dialog.Footer>
				<Button
					type="button"
					variant="outline"
					onclick={() => (isLocationFormOpen = false)}
					disabled={isSubmittingLocation}
				>
					Cancel
				</Button>
				<Button type="submit" disabled={isSubmittingLocation || !locationCode}>
					{isSubmittingLocation ? 'Saving...' : editingLocation ? 'Update' : 'Create'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Confirmation Dialog -->
<Dialog.Root bind:open={deleteDialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Delete {deleteType === 'zone' ? 'Zone' : 'Location'}</Dialog.Title>
			<Dialog.Description>
				{#if itemToDelete}
					{#if deleteType === 'zone'}
						{@const zone = itemToDelete as WarehouseZoneResponse}
						Are you sure you want to delete "{zone.zoneName}"?
						{#if getLocationCountForZone(zone.zoneId) > 0}
							<p class="mt-2 font-medium text-destructive">
								Warning: This zone has {getLocationCountForZone(zone.zoneId)} locations. They will become
								unassigned.
							</p>
						{/if}
					{:else}
						{@const location = itemToDelete as WarehouseLocationResponse}
						Are you sure you want to delete "{location.locationCode}"?
					{/if}
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

<!-- Location Stock Summary Dialog -->
{#if stockViewLocation && warehouseId}
	<LocationStockSummary
		{warehouseId}
		location={stockViewLocation}
		bind:open={stockViewOpen}
		onClose={() => (stockViewLocation = null)}
	/>
{/if}
