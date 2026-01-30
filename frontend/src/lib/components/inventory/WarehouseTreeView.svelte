<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import * as Collapsible from '$lib/components/ui/collapsible';
	import type { WarehouseZoneResponse, WarehouseLocationResponse } from '$lib/types/inventory';

	interface Props {
		zones: WarehouseZoneResponse[];
		locations: WarehouseLocationResponse[];
		onSelectZone?: (zone: WarehouseZoneResponse) => void;
		onSelectLocation?: (location: WarehouseLocationResponse) => void;
		onAddZone?: () => void;
		onAddLocation?: (zoneId?: string) => void;
		onEditZone?: (zone: WarehouseZoneResponse) => void;
		onEditLocation?: (location: WarehouseLocationResponse) => void;
		onDeleteZone?: (zone: WarehouseZoneResponse) => void;
		onDeleteLocation?: (location: WarehouseLocationResponse) => void;
		selectedZoneId?: string | null;
		selectedLocationId?: string | null;
		showActions?: boolean;
	}

	let {
		zones,
		locations,
		onSelectZone,
		onSelectLocation,
		onAddZone,
		onAddLocation,
		onEditZone,
		onEditLocation,
		onDeleteZone,
		onDeleteLocation,
		selectedZoneId = null,
		selectedLocationId = null,
		showActions = true
	}: Props = $props();

	// Track expanded zones
	let expandedZones = $state<Set<string>>(new Set());

	const zoneTypeIcons: Record<string, string> = {
		receiving: '📥',
		storage: '📦',
		shipping: '📤',
		quarantine: '🔒',
		returns: '↩️',
		default: '📍'
	};

	const locationTypeIcons: Record<string, string> = {
		shelf: '📚',
		bin: '📦',
		rack: '🗃️',
		floor: '⬜',
		pallet: '🎛️',
		default: '📍'
	};

	// Get locations for a specific zone
	function getLocationsForZone(zoneId: string): WarehouseLocationResponse[] {
		return locations.filter((l) => l.zoneId === zoneId);
	}

	// Get unassigned locations (no zone)
	const unassignedLocations = $derived(locations.filter((l) => !l.zoneId));

	function toggleZone(zoneId: string) {
		const newSet = new Set(expandedZones);
		if (newSet.has(zoneId)) {
			newSet.delete(zoneId);
		} else {
			newSet.add(zoneId);
		}
		expandedZones = newSet;
	}

	function handleZoneClick(zone: WarehouseZoneResponse) {
		onSelectZone?.(zone);
	}

	function handleLocationClick(location: WarehouseLocationResponse) {
		onSelectLocation?.(location);
	}
</script>

<div class="space-y-2">
	<!-- Header with Add Zone button -->
	{#if showActions && onAddZone}
		<div class="flex items-center justify-between border-b pb-2">
			<span class="text-sm font-medium text-muted-foreground">Warehouse Structure</span>
			<Button variant="ghost" size="sm" onclick={onAddZone}>+ Zone</Button>
		</div>
	{/if}

	<!-- Zones -->
	{#each zones as zone (zone.zoneId)}
		{@const zoneLocations = getLocationsForZone(zone.zoneId)}
		{@const isExpanded = expandedZones.has(zone.zoneId)}
		{@const isSelected = selectedZoneId === zone.zoneId}

		<div class="rounded-md border {isSelected ? 'border-primary bg-primary/5' : ''}">
			<div
				class="flex cursor-pointer items-center gap-2 p-2 hover:bg-muted/50"
				onclick={() => handleZoneClick(zone)}
				onkeydown={(e) => e.key === 'Enter' && handleZoneClick(zone)}
				role="button"
				tabindex="0"
			>
				<!-- Expand/Collapse Toggle -->
				<button
					class="flex h-6 w-6 items-center justify-center rounded hover:bg-muted"
					aria-label={isExpanded ? 'Collapse zone' : 'Expand zone'}
					onclick={(e) => {
						e.stopPropagation();
						toggleZone(zone.zoneId);
					}}
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
						class="transition-transform {isExpanded ? 'rotate-90' : ''}"
					>
						<polyline points="9 18 15 12 9 6" />
					</svg>
				</button>

				<!-- Zone Icon -->
				<span class="text-lg">{zoneTypeIcons[zone.zoneType] || zoneTypeIcons.default}</span>

				<!-- Zone Info -->
				<div class="flex-1">
					<div class="flex items-center gap-2">
						<span class="font-medium">{zone.zoneName}</span>
						<span class="text-xs text-muted-foreground">({zone.zoneCode})</span>
					</div>
				</div>

				<!-- Location Count -->
				<Badge variant="outline" class="text-xs">
					{zoneLocations.length} loc
				</Badge>

				<!-- Status Badge -->
				{#if !zone.isActive}
					<Badge variant="secondary">Inactive</Badge>
				{/if}

				<!-- Actions -->
				{#if showActions}
					<div class="flex gap-1">
						{#if onAddLocation}
							<button
								class="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
								onclick={(e) => {
									e.stopPropagation();
									onAddLocation(zone.zoneId);
								}}
								title="Add location"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									width="14"
									height="14"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
									stroke-linejoin="round"
								>
									<line x1="12" y1="5" x2="12" y2="19" />
									<line x1="5" y1="12" x2="19" y2="12" />
								</svg>
							</button>
						{/if}
						{#if onEditZone}
							<button
								class="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
								onclick={(e) => {
									e.stopPropagation();
									onEditZone(zone);
								}}
								title="Edit zone"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									width="14"
									height="14"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
									stroke-linejoin="round"
								>
									<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
									<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
								</svg>
							</button>
						{/if}
						{#if onDeleteZone}
							<button
								class="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
								onclick={(e) => {
									e.stopPropagation();
									onDeleteZone(zone);
								}}
								title="Delete zone"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									width="14"
									height="14"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
									stroke-linejoin="round"
								>
									<polyline points="3 6 5 6 21 6" />
									<path
										d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
									/>
								</svg>
							</button>
						{/if}
					</div>
				{/if}
			</div>

			<!-- Zone Locations (Collapsible) -->
			{#if isExpanded && zoneLocations.length > 0}
				<div class="border-t bg-muted/30 py-1 pr-2 pl-10">
					{#each zoneLocations as location (location.locationId)}
						{@const isLocationSelected = selectedLocationId === location.locationId}
						<div
							class="flex cursor-pointer items-center gap-2 rounded p-1.5 hover:bg-muted {isLocationSelected
								? 'bg-primary/10'
								: ''}"
							onclick={() => handleLocationClick(location)}
							onkeydown={(e) => e.key === 'Enter' && handleLocationClick(location)}
							role="button"
							tabindex="0"
						>
							<span class="text-sm"
								>{locationTypeIcons[location.locationType] || locationTypeIcons.default}</span
							>
							<span class="flex-1 text-sm">{location.locationCode}</span>
							{#if location.locationName}
								<span class="text-xs text-muted-foreground">{location.locationName}</span>
							{/if}
							{#if !location.isActive}
								<Badge variant="secondary" class="text-xs">Inactive</Badge>
							{/if}

							{#if showActions}
								<div
									class="flex gap-1 opacity-0 transition-opacity group-hover:opacity-100 hover:opacity-100"
								>
									{#if onEditLocation}
										<button
											class="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
											onclick={(e) => {
												e.stopPropagation();
												onEditLocation(location);
											}}
											title="Edit location"
										>
											<svg
												xmlns="http://www.w3.org/2000/svg"
												width="12"
												height="12"
												viewBox="0 0 24 24"
												fill="none"
												stroke="currentColor"
												stroke-width="2"
												stroke-linecap="round"
												stroke-linejoin="round"
											>
												<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
												<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
											</svg>
										</button>
									{/if}
									{#if onDeleteLocation}
										<button
											class="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
											onclick={(e) => {
												e.stopPropagation();
												onDeleteLocation(location);
											}}
											title="Delete location"
										>
											<svg
												xmlns="http://www.w3.org/2000/svg"
												width="12"
												height="12"
												viewBox="0 0 24 24"
												fill="none"
												stroke="currentColor"
												stroke-width="2"
												stroke-linecap="round"
												stroke-linejoin="round"
											>
												<polyline points="3 6 5 6 21 6" />
												<path
													d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
												/>
											</svg>
										</button>
									{/if}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/each}

	<!-- Unassigned Locations -->
	{#if unassignedLocations.length > 0}
		<div class="mt-4 rounded-md border border-dashed">
			<div class="flex items-center gap-2 border-b border-dashed p-2">
				<span class="text-muted-foreground">📭</span>
				<span class="flex-1 text-sm font-medium text-muted-foreground">Unassigned Locations</span>
				<Badge variant="outline" class="text-xs">{unassignedLocations.length}</Badge>
			</div>
			<div class="py-1 pr-2 pl-8">
				{#each unassignedLocations as location (location.locationId)}
					{@const isLocationSelected = selectedLocationId === location.locationId}
					<div
						class="flex cursor-pointer items-center gap-2 rounded p-1.5 hover:bg-muted {isLocationSelected
							? 'bg-primary/10'
							: ''}"
						onclick={() => handleLocationClick(location)}
						onkeydown={(e) => e.key === 'Enter' && handleLocationClick(location)}
						role="button"
						tabindex="0"
					>
						<span class="text-sm"
							>{locationTypeIcons[location.locationType] || locationTypeIcons.default}</span
						>
						<span class="flex-1 text-sm">{location.locationCode}</span>
						{#if location.locationName}
							<span class="text-xs text-muted-foreground">{location.locationName}</span>
						{/if}
						{#if !location.isActive}
							<Badge variant="secondary" class="text-xs">Inactive</Badge>
						{/if}

						{#if showActions && (onEditLocation || onDeleteLocation)}
							<div class="flex gap-1">
								{#if onEditLocation}
									<button
										class="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
										onclick={(e) => {
											e.stopPropagation();
											onEditLocation(location);
										}}
										title="Edit location"
									>
										<svg
											xmlns="http://www.w3.org/2000/svg"
											width="12"
											height="12"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										>
											<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
											<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
										</svg>
									</button>
								{/if}
								{#if onDeleteLocation}
									<button
										class="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
										onclick={(e) => {
											e.stopPropagation();
											onDeleteLocation(location);
										}}
										title="Delete location"
									>
										<svg
											xmlns="http://www.w3.org/2000/svg"
											width="12"
											height="12"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										>
											<polyline points="3 6 5 6 21 6" />
											<path
												d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
											/>
										</svg>
									</button>
								{/if}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Empty State -->
	{#if zones.length === 0 && locations.length === 0}
		<div class="py-8 text-center text-muted-foreground">
			<p>No zones or locations</p>
			{#if showActions && onAddZone}
				<Button variant="outline" size="sm" class="mt-2" onclick={onAddZone}>Add Zone</Button>
			{/if}
		</div>
	{/if}
</div>
