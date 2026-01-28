<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import * as Dialog from '$lib/components/ui/dialog';
	import { warehouseApi } from '$lib/api/inventory/warehouses';
	import type { WarehouseLocationResponse, LotSerial } from '$lib/types/inventory';

	interface Props {
		warehouseId: string;
		location: WarehouseLocationResponse;
		open?: boolean;
		onClose?: () => void;
	}

	let { warehouseId, location, open = $bindable(false), onClose }: Props = $props();

	let lotSerials = $state<LotSerial[]>([]);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// Compute stock summary from lot/serials
	const stockSummary = $derived.by(() => {
		const productMap = new Map<string, { productId: string; quantity: number; lots: number }>();

		for (const ls of lotSerials) {
			const existing = productMap.get(ls.productId);
			const qty = ls.remainingQuantity || 0;
			if (existing) {
				existing.quantity += qty;
				existing.lots += 1;
			} else {
				productMap.set(ls.productId, { productId: ls.productId, quantity: qty, lots: 1 });
			}
		}

		return {
			totalItems: lotSerials.length,
			totalQuantity: lotSerials.reduce((sum, ls) => sum + (ls.remainingQuantity || 0), 0),
			uniqueProducts: productMap.size,
			byProduct: Array.from(productMap.values())
		};
	});

	async function loadStock() {
		isLoading = true;
		error = null;

		const response = await warehouseApi.getLocationStock(warehouseId, location.locationId, {
			pageSize: 100
		});

		if (response.success && response.data) {
			lotSerials = response.data.lotSerials;
		} else {
			error = response.error || 'Failed to load stock data';
		}

		isLoading = false;
	}

	function getStatusBadgeVariant(
		status: string
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (status) {
			case 'available':
				return 'default';
			case 'reserved':
				return 'secondary';
			case 'quarantined':
				return 'destructive';
			default:
				return 'outline';
		}
	}

	function formatDate(dateStr: string | null | undefined): string {
		if (!dateStr) return '-';
		return new Date(dateStr).toLocaleDateString();
	}

	function handleClose() {
		open = false;
		onClose?.();
	}

	$effect(() => {
		if (open && location) {
			loadStock();
		}
	});
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[80vh] overflow-y-auto sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title>Stock at {location.locationCode}</Dialog.Title>
			<Dialog.Description>
				{location.locationName || location.locationType} - View all stock items at this location
			</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="space-y-4 py-4">
				<div class="h-20 animate-pulse rounded bg-muted"></div>
				<div class="h-40 animate-pulse rounded bg-muted"></div>
			</div>
		{:else if error}
			<div class="py-8 text-center">
				<p class="text-destructive">{error}</p>
				<Button variant="outline" size="sm" class="mt-2" onclick={loadStock}>Retry</Button>
			</div>
		{:else}
			<!-- Summary Cards -->
			<div class="grid grid-cols-3 gap-4 py-4">
				<Card>
					<CardContent class="pt-4">
						<div class="text-2xl font-bold">{stockSummary.totalItems}</div>
						<p class="text-sm text-muted-foreground">Lot/Serial Items</p>
					</CardContent>
				</Card>
				<Card>
					<CardContent class="pt-4">
						<div class="text-2xl font-bold">{stockSummary.totalQuantity}</div>
						<p class="text-sm text-muted-foreground">Total Quantity</p>
					</CardContent>
				</Card>
				<Card>
					<CardContent class="pt-4">
						<div class="text-2xl font-bold">{stockSummary.uniqueProducts}</div>
						<p class="text-sm text-muted-foreground">Unique Products</p>
					</CardContent>
				</Card>
			</div>

			<!-- Stock List -->
			{#if lotSerials.length === 0}
				<div class="py-12 text-center">
					<p class="text-muted-foreground">No stock at this location</p>
					<p class="mt-1 text-sm text-muted-foreground">
						Items will appear here once stock is received or transferred to this location
					</p>
				</div>
			{:else}
				<div class="space-y-2">
					<h4 class="text-sm font-medium">Stock Details</h4>
					<div class="max-h-64 overflow-y-auto rounded-md border">
						<table class="w-full text-sm">
							<thead class="sticky top-0 bg-muted">
								<tr>
									<th class="px-3 py-2 text-left font-medium">Lot/Serial</th>
									<th class="px-3 py-2 text-left font-medium">Type</th>
									<th class="px-3 py-2 text-right font-medium">Qty</th>
									<th class="px-3 py-2 text-left font-medium">Status</th>
									<th class="px-3 py-2 text-left font-medium">Expiry</th>
								</tr>
							</thead>
							<tbody>
								{#each lotSerials as ls (ls.lotSerialId)}
									<tr class="border-t">
										<td class="px-3 py-2 font-mono text-xs">
											{ls.lotNumber || ls.serialNumber || '-'}
										</td>
										<td class="px-3 py-2">
											<Badge variant="outline" class="text-xs">
												{ls.trackingType}
											</Badge>
										</td>
										<td class="px-3 py-2 text-right font-medium">
											{ls.remainingQuantity ?? '-'}
										</td>
										<td class="px-3 py-2">
											<Badge variant={getStatusBadgeVariant(ls.status)} class="text-xs">
												{ls.status}
											</Badge>
										</td>
										<td class="px-3 py-2 text-xs text-muted-foreground">
											{formatDate(ls.expiryDate)}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</div>
			{/if}
		{/if}

		<Dialog.Footer>
			<Button variant="outline" onclick={handleClose}>Close</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
