<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import type { WarehouseResponse } from '$lib/types/inventory';

	interface Props {
		warehouse: WarehouseResponse;
		zoneCount?: number;
		locationCount?: number;
		onView?: (warehouse: WarehouseResponse) => void;
		onEdit?: (warehouse: WarehouseResponse) => void;
		onDelete?: (warehouse: WarehouseResponse) => void;
	}

	let { warehouse, zoneCount = 0, locationCount = 0, onView, onEdit, onDelete }: Props = $props();

	const warehouseTypeIcons: Record<string, string> = {
		main: '🏭',
		satellite: '🏢',
		distribution: '📦',
		storage: '🗄️',
		default: '🏠'
	};

	const icon = $derived(warehouseTypeIcons[warehouse.warehouseType] ?? warehouseTypeIcons.default);
</script>

<Card
	class="cursor-pointer transition-shadow hover:shadow-md"
	onclick={() => onView?.(warehouse)}
	onkeydown={(e) => e.key === 'Enter' && onView?.(warehouse)}
	role="button"
	tabindex={0}
>
	<CardHeader class="pb-2">
		<div class="flex items-start justify-between">
			<div class="flex items-center gap-2">
				<span class="text-2xl">{icon}</span>
				<div>
					<CardTitle class="text-base">{warehouse.warehouseName}</CardTitle>
					<p class="text-sm text-muted-foreground">{warehouse.warehouseCode}</p>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<Badge variant={warehouse.isActive ? 'default' : 'secondary'}>
					{warehouse.isActive ? 'Active' : 'Inactive'}
				</Badge>
				<DropdownMenu.Root>
					<DropdownMenu.Trigger
						onclick={(e: MouseEvent) => e.stopPropagation()}
						class="inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-accent"
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
						<DropdownMenu.Item
							onclick={(e) => {
								e.stopPropagation();
								onView?.(warehouse);
							}}
						>
							View Details
						</DropdownMenu.Item>
						<DropdownMenu.Item
							onclick={(e) => {
								e.stopPropagation();
								onEdit?.(warehouse);
							}}
						>
							Edit
						</DropdownMenu.Item>
						<DropdownMenu.Separator />
						<DropdownMenu.Item
							class="text-destructive"
							onclick={(e) => {
								e.stopPropagation();
								onDelete?.(warehouse);
							}}
						>
							Delete
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</div>
		</div>
	</CardHeader>
	<CardContent>
		{#if warehouse.description}
			<p class="mb-3 line-clamp-2 text-sm text-muted-foreground">{warehouse.description}</p>
		{/if}

		<div class="grid grid-cols-2 gap-4 rounded-lg bg-muted/50 p-3">
			<div class="text-center">
				<p class="text-lg font-semibold">{zoneCount}</p>
				<p class="text-xs text-muted-foreground">Zones</p>
			</div>
			<div class="text-center">
				<p class="text-lg font-semibold">{locationCount}</p>
				<p class="text-xs text-muted-foreground">Locations</p>
			</div>
		</div>

		<div class="mt-3 flex items-center justify-between text-xs text-muted-foreground">
			<span>Type: {warehouse.warehouseType}</span>
			{#if warehouse.address && typeof warehouse.address === 'object' && 'city' in warehouse.address}
				<span>{warehouse.address.city}</span>
			{/if}
		</div>
	</CardContent>
</Card>
