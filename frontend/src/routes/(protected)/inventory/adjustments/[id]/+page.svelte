<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { adjustmentState, adjustmentStore } from '$lib/stores/adjustments.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import { getReasonCodeLabel } from '$lib/constants/adjustment-reasons';
	import { REASON_CODES } from '$lib/constants/adjustment-reasons';

	// UI Components
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import * as Card from '$lib/components/ui/card';
	import { Separator } from '$lib/components/ui/separator';

	// Icons
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import PrinterIcon from '@lucide/svelte/icons/printer';
	import ClipboardListIcon from '@lucide/svelte/icons/clipboard-list';

	let adjustmentId = $derived($page.params.id);

	function formatDate(dateStr: string | null | undefined): string {
		if (!dateStr) return '-';
		return new Date(dateStr).toLocaleDateString('en-US', {
			weekday: 'short',
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function formatQuantity(quantity: number): string {
		const sign = quantity > 0 ? '+' : '';
		return `${sign}${quantity}`;
	}

	function getQuantityBadgeVariant(
		quantity: number
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		if (quantity > 0) return 'default';
		if (quantity < 0) return 'destructive';
		return 'secondary';
	}

	function getReasonBadgeVariant(
		reasonCode: string
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		const reason = REASON_CODES.find((r) => r.code === reasonCode);
		if (!reason) return 'secondary';

		switch (reason.category) {
			case 'quality':
				return 'destructive';
			case 'loss':
				return 'destructive';
			case 'inventory_count':
				return 'default';
			default:
				return 'secondary';
		}
	}

	function getWarehouseName(warehouseId: string): string {
		const warehouse = warehouseState.items.find((w) => w.warehouseId === warehouseId);
		return warehouse?.warehouseName || warehouseId.slice(0, 8);
	}

	function handlePrint() {
		window.print();
	}

	async function handleRefresh() {
		if (adjustmentId) {
			await adjustmentStore.get(adjustmentId);
		}
	}

	onMount(async () => {
		if (adjustmentId) {
			await Promise.all([adjustmentStore.get(adjustmentId), warehouseStore.load()]);
		}
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Breadcrumbs -->
	<div class="flex items-center gap-2 text-sm text-muted-foreground">
		<a href="/inventory" class="hover:text-foreground">Inventory</a>
		<span>/</span>
		<a href="/inventory/adjustments" class="hover:text-foreground">Adjustments</a>
		<span>/</span>
		<span class="text-foreground"
			>{adjustmentState.selected?.adjustmentId?.slice(0, 8) || 'Detail'}</span
		>
	</div>

	<!-- Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<Button variant="ghost" size="icon" href="/inventory/adjustments">
				<ArrowLeftIcon class="h-4 w-4" />
			</Button>
			<div>
				<h1 class="text-2xl font-bold">Adjustment Detail</h1>
				<p class="text-muted-foreground">
					{adjustmentState.selected?.adjustmentId?.slice(0, 8) || 'Loading...'}
				</p>
			</div>
		</div>
		<div class="flex gap-2">
			<Button
				variant="outline"
				size="icon"
				onclick={handleRefresh}
				disabled={adjustmentState.isLoading}
			>
				<RefreshCwIcon class="h-4 w-4 {adjustmentState.isLoading ? 'animate-spin' : ''}" />
			</Button>
			<Button variant="outline" onclick={handlePrint}>
				<PrinterIcon class="mr-2 h-4 w-4" />
				Print
			</Button>
		</div>
	</div>

	<!-- Error State -->
	{#if adjustmentState.error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<p class="text-destructive">{adjustmentState.error}</p>
			<Button variant="outline" onclick={handleRefresh} class="mt-2">Retry</Button>
		</div>
	{/if}

	<!-- Loading State -->
	{#if adjustmentState.isLoading}
		<div class="flex items-center justify-center py-12">
			<RefreshCwIcon class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if !adjustmentState.selected}
		<!-- Not Found State -->
		<div class="flex flex-col items-center justify-center py-12 text-center">
			<ClipboardListIcon class="mb-4 h-12 w-12 text-muted-foreground" />
			<h3 class="text-lg font-semibold">Adjustment not found</h3>
			<p class="mb-4 text-muted-foreground">
				The adjustment you're looking for doesn't exist or has been deleted.
			</p>
			<Button href="/inventory/adjustments">Back to Adjustments</Button>
		</div>
	{:else}
		{@const adjustment = adjustmentState.selected}
		<div class="grid gap-6 md:grid-cols-2">
			<!-- Details Card -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Adjustment Information</Card.Title>
				</Card.Header>
				<Card.Content class="grid gap-4">
					<div class="grid grid-cols-2 gap-4">
						<div>
							<p class="text-sm text-muted-foreground">ID</p>
							<p class="font-mono text-sm">{adjustment.adjustmentId}</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Status</p>
							<Badge variant="outline">Posted</Badge>
						</div>
					</div>

					<Separator />

					<div class="grid grid-cols-2 gap-4">
						<div>
							<p class="text-sm text-muted-foreground">Warehouse</p>
							<p class="font-medium">{getWarehouseName(adjustment.warehouseId)}</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Created At</p>
							<p class="font-medium">{formatDate(adjustment.createdAt)}</p>
						</div>
					</div>

					<div class="grid grid-cols-2 gap-4">
						<div>
							<p class="text-sm text-muted-foreground">Approved By</p>
							<p class="font-medium">
								{adjustment.approvedByUser?.fullName || adjustment.approvedBy?.slice(0, 8) || '-'}
							</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Last Updated</p>
							<p class="font-medium">{formatDate(adjustment.updatedAt)}</p>
						</div>
					</div>

					{#if adjustment.notes}
						<Separator />
						<div>
							<p class="text-sm text-muted-foreground">Notes</p>
							<p class="mt-1">{adjustment.notes}</p>
						</div>
					{/if}
				</Card.Content>
			</Card.Root>

			<!-- Product Card -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Product Details</Card.Title>
				</Card.Header>
				<Card.Content class="grid gap-4">
					<div>
						<p class="text-sm text-muted-foreground">Product</p>
						<p class="font-medium">
							{adjustment.product?.sku || '-'}
							{#if adjustment.product?.name}
								<span class="text-muted-foreground">- {adjustment.product.name}</span>
							{/if}
						</p>
					</div>

					<Separator />

					<div class="grid grid-cols-2 gap-4">
						<div>
							<p class="text-sm text-muted-foreground">Quantity Change</p>
							<Badge variant={getQuantityBadgeVariant(adjustment.quantity)} class="text-lg">
								{formatQuantity(adjustment.quantity)}
							</Badge>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Reason</p>
							<Badge variant={getReasonBadgeVariant(adjustment.reasonCode)}>
								{getReasonCodeLabel(adjustment.reasonCode)}
							</Badge>
						</div>
					</div>

					{#if adjustment.stockMove}
						<Separator />
						<div>
							<p class="text-sm text-muted-foreground">Stock Move Reference</p>
							<p class="font-mono text-sm">{adjustment.stockMove.moveId}</p>
							<p class="mt-1 text-sm text-muted-foreground">
								Move Type: {adjustment.stockMove.moveType} | Date: {formatDate(
									adjustment.stockMove.moveDate
								)}
							</p>
						</div>
					{/if}
				</Card.Content>
			</Card.Root>
		</div>

		<!-- Actions -->
		<div class="flex justify-between">
			<Button variant="outline" href="/inventory/adjustments">
				<ArrowLeftIcon class="mr-2 h-4 w-4" />
				Back to List
			</Button>
		</div>
	{/if}
</div>
