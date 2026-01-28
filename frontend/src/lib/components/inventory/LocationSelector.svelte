<script lang="ts">
	import { warehouseApi } from '$lib/api/inventory/warehouses';
	import type { WarehouseZoneResponse, WarehouseLocationResponse } from '$lib/types/inventory';
	import * as Select from '$lib/components/ui/select';
	import { Label } from '$lib/components/ui/label';

	interface Props {
		warehouseId: string;
		selectedZoneId?: string | null;
		selectedLocationId?: string | null;
		label?: string;
		required?: boolean;
		disabled?: boolean;
		onZoneChange?: (zoneId: string | null) => void;
		onLocationChange?: (locationId: string | null) => void;
	}

	let {
		warehouseId,
		selectedZoneId = $bindable(null),
		selectedLocationId = $bindable(null),
		label = 'Location',
		required = false,
		disabled = false,
		onZoneChange,
		onLocationChange
	}: Props = $props();

	let zones = $state<WarehouseZoneResponse[]>([]);
	let locations = $state<WarehouseLocationResponse[]>([]);
	let isLoadingZones = $state(false);
	let isLoadingLocations = $state(false);

	// Load zones when warehouseId changes
	$effect(() => {
		if (warehouseId) {
			loadZones(warehouseId);
		} else {
			zones = [];
			locations = [];
			selectedZoneId = null;
			selectedLocationId = null;
		}
	});

	// Load locations when zoneId changes (or load all if no zone selected)
	$effect(() => {
		if (warehouseId) {
			loadLocations(warehouseId, selectedZoneId ?? undefined);
		}
	});

	async function loadZones(whId: string) {
		isLoadingZones = true;
		const response = await warehouseApi.listZones(whId);
		if (response.success && response.data) {
			zones = response.data.zones;
		} else {
			zones = [];
		}
		isLoadingZones = false;
	}

	async function loadLocations(whId: string, zoneId?: string) {
		isLoadingLocations = true;
		const response = await warehouseApi.listLocations(whId, { zoneId });
		if (response.success && response.data) {
			locations = response.data.locations;
		} else {
			locations = [];
		}
		isLoadingLocations = false;
	}

	function handleZoneChange(value: string | undefined) {
		const newZoneId = value === '__none__' ? null : (value ?? null);
		selectedZoneId = newZoneId;
		selectedLocationId = null; // Reset location when zone changes
		onZoneChange?.(newZoneId);
		onLocationChange?.(null);
	}

	function handleLocationChange(value: string | undefined) {
		const newLocationId = value === '__none__' ? null : (value ?? null);
		selectedLocationId = newLocationId;
		onLocationChange?.(newLocationId);
	}

	const selectedZone = $derived(zones.find((z) => z.zoneId === selectedZoneId));
	const selectedLocation = $derived(locations.find((l) => l.locationId === selectedLocationId));
</script>

<div class="flex flex-col gap-2">
	{#if label}
		<Label class="text-xs text-muted-foreground">{label}{required ? ' *' : ''}</Label>
	{/if}

	<div class="flex gap-2">
		<!-- Zone Selector -->
		<div class="flex-1">
			<Select.Root
				type="single"
				value={selectedZoneId ?? '__none__'}
				onValueChange={handleZoneChange}
				disabled={disabled || !warehouseId || isLoadingZones}
			>
				<Select.Trigger class="w-full text-xs">
					{#if isLoadingZones}
						Loading...
					{:else if selectedZone}
						{selectedZone.zoneName}
					{:else}
						Any Zone
					{/if}
				</Select.Trigger>
				<Select.Content>
					<Select.Item value="__none__">Any Zone</Select.Item>
					{#each zones as zone (zone.zoneId)}
						<Select.Item value={zone.zoneId}>
							{zone.zoneCode} - {zone.zoneName}
						</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>

		<!-- Location Selector -->
		<div class="flex-1">
			<Select.Root
				type="single"
				value={selectedLocationId ?? '__none__'}
				onValueChange={handleLocationChange}
				disabled={disabled || !warehouseId || isLoadingLocations}
			>
				<Select.Trigger class="w-full text-xs">
					{#if isLoadingLocations}
						Loading...
					{:else if selectedLocation}
						{selectedLocation.locationCode}
					{:else}
						Default Location
					{/if}
				</Select.Trigger>
				<Select.Content>
					<Select.Item value="__none__">Default Location</Select.Item>
					{#each locations as location (location.locationId)}
						<Select.Item value={location.locationId}>
							{location.locationCode}{location.locationName ? ` - ${location.locationName}` : ''}
						</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>
	</div>
</div>
