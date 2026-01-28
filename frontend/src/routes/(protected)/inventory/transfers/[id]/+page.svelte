<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { transferState, transferStore } from '$lib/stores/stock-movements.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import type { TransferStatus } from '$lib/types/inventory';
	import { toast } from 'svelte-sonner';

	// UI Components
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import * as Card from '$lib/components/ui/card';
	import * as Dialog from '$lib/components/ui/dialog';

	// Icons
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import CheckIcon from '@lucide/svelte/icons/check';
	import TruckIcon from '@lucide/svelte/icons/truck';
	import PackageCheckIcon from '@lucide/svelte/icons/package-check';
	import XIcon from '@lucide/svelte/icons/x';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';

	let transferId = $derived(page.params.id ?? '');
	let transfer = $derived(transferState.selected);
	let isActionLoading = $state(false);
	let showCancelDialog = $state(false);
	let cancelReason = $state('');

	function getStatusBadgeVariant(
		status: TransferStatus
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (status) {
			case 'draft':
				return 'secondary';
			case 'confirmed':
			case 'picked':
				return 'default';
			case 'shipped':
				return 'default';
			case 'received':
				return 'outline';
			case 'cancelled':
				return 'destructive';
			default:
				return 'secondary';
		}
	}

	function getPriorityBadgeVariant(
		priority: string
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (priority) {
			case 'urgent':
				return 'destructive';
			case 'high':
				return 'default';
			case 'normal':
				return 'secondary';
			case 'low':
				return 'outline';
			default:
				return 'secondary';
		}
	}

	function formatDate(dateStr: string | null | undefined): string {
		if (!dateStr) return '-';
		return new Date(dateStr).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function getWarehouseName(warehouseId: string): string {
		const warehouse = warehouseState.items.find((w) => w.warehouseId === warehouseId);
		return warehouse?.warehouseName || warehouseId.slice(0, 8);
	}

	// Status flow: draft -> confirmed -> picked -> shipped -> received
	let canConfirm = $derived(transfer?.status === 'draft');
	let canShip = $derived(transfer?.status === 'confirmed' || transfer?.status === 'picked');
	let canReceive = $derived(transfer?.status === 'shipped');
	let canCancel = $derived(transfer?.status === 'draft' || transfer?.status === 'confirmed');

	async function handleConfirm() {
		if (!transfer) return;
		isActionLoading = true;
		const success = await transferStore.confirm(transfer.transferId);
		if (success) {
			toast.success('Transfer confirmed successfully');
			await transferStore.get(transfer.transferId);
		}
		isActionLoading = false;
	}

	async function handleShip() {
		if (!transfer) return;
		isActionLoading = true;
		const success = await transferStore.ship(transfer.transferId);
		if (success) {
			toast.success('Transfer shipped successfully');
			await transferStore.get(transfer.transferId);
		}
		isActionLoading = false;
	}

	async function handleReceive() {
		if (!transfer) return;
		isActionLoading = true;
		const success = await transferStore.receive(transfer.transferId);
		if (success) {
			toast.success('Transfer received successfully');
			await transferStore.get(transfer.transferId);
		}
		isActionLoading = false;
	}

	async function handleCancel() {
		if (!transfer) return;
		isActionLoading = true;
		const success = await transferStore.cancel(transfer.transferId, cancelReason);
		if (success) {
			toast.success('Transfer cancelled');
			showCancelDialog = false;
			await transferStore.get(transfer.transferId);
		}
		isActionLoading = false;
	}

	onMount(async () => {
		await Promise.all([transferStore.get(transferId), warehouseStore.load()]);
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<Button variant="ghost" size="icon" href="/inventory/transfers">
				<ArrowLeftIcon class="h-4 w-4" />
			</Button>
			<div>
				<h1 class="text-2xl font-bold">
					{transfer?.transferNumber || 'Loading...'}
				</h1>
				<p class="text-muted-foreground">Stock Transfer Details</p>
			</div>
		</div>
		{#if transfer}
			<div class="flex items-center gap-2">
				<Badge variant={getStatusBadgeVariant(transfer.status)} class="text-sm">
					{transfer.status.replace('_', ' ').toUpperCase()}
				</Badge>
				<Badge variant={getPriorityBadgeVariant(transfer.priority)} class="text-sm">
					{transfer.priority.toUpperCase()}
				</Badge>
			</div>
		{/if}
	</div>

	{#if transferState.isLoading && !transfer}
		<div class="flex items-center justify-center py-12">
			<RefreshCwIcon class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if transferState.error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<p class="text-destructive">{transferState.error}</p>
			<Button variant="outline" onclick={() => transferStore.get(transferId)} class="mt-2">
				Retry
			</Button>
		</div>
	{:else if transfer}
		<!-- Action Buttons -->
		<div class="flex flex-wrap gap-3">
			{#if canConfirm}
				<Button onclick={handleConfirm} disabled={isActionLoading}>
					<CheckIcon class="mr-2 h-4 w-4" />
					Confirm Transfer
				</Button>
			{/if}
			{#if canShip}
				<Button onclick={handleShip} disabled={isActionLoading}>
					<TruckIcon class="mr-2 h-4 w-4" />
					Ship
				</Button>
			{/if}
			{#if canReceive}
				<Button onclick={handleReceive} disabled={isActionLoading}>
					<PackageCheckIcon class="mr-2 h-4 w-4" />
					Receive
				</Button>
			{/if}
			{#if canCancel}
				<Button
					variant="destructive"
					onclick={() => (showCancelDialog = true)}
					disabled={isActionLoading}
				>
					<XIcon class="mr-2 h-4 w-4" />
					Cancel
				</Button>
			{/if}
		</div>

		<!-- Transfer Info -->
		<div class="grid gap-6 md:grid-cols-2">
			<!-- Route Card -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Transfer Route</Card.Title>
				</Card.Header>
				<Card.Content>
					<div class="flex items-center justify-center gap-4 py-4">
						<div class="text-center">
							<p class="text-lg font-semibold">{getWarehouseName(transfer.sourceWarehouseId)}</p>
							<p class="text-sm text-muted-foreground">Source</p>
						</div>
						<ArrowRightIcon class="h-8 w-8 text-muted-foreground" />
						<div class="text-center">
							<p class="text-lg font-semibold">
								{getWarehouseName(transfer.destinationWarehouseId)}
							</p>
							<p class="text-sm text-muted-foreground">Destination</p>
						</div>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Details Card -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Details</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-3">
					<div class="flex justify-between">
						<span class="text-muted-foreground">Type</span>
						<span class="font-medium">{transfer.transferType}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-muted-foreground">Reference</span>
						<span class="font-medium">{transfer.referenceNumber || '-'}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-muted-foreground">Shipping Method</span>
						<span class="font-medium">{transfer.shippingMethod || '-'}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-muted-foreground">Reason</span>
						<span class="font-medium">{transfer.reason || '-'}</span>
					</div>
				</Card.Content>
			</Card.Root>
		</div>

		<!-- Timeline -->
		<Card.Root>
			<Card.Header>
				<Card.Title>Timeline</Card.Title>
			</Card.Header>
			<Card.Content>
				<div class="grid gap-4 md:grid-cols-4">
					<div class="space-y-1">
						<p class="text-sm text-muted-foreground">Created</p>
						<p class="font-medium">{formatDate(transfer.createdAt)}</p>
					</div>
					<div class="space-y-1">
						<p class="text-sm text-muted-foreground">Expected Ship</p>
						<p class="font-medium">{formatDate(transfer.expectedShipDate)}</p>
					</div>
					<div class="space-y-1">
						<p class="text-sm text-muted-foreground">Actual Ship</p>
						<p class="font-medium">{formatDate(transfer.actualShipDate)}</p>
					</div>
					<div class="space-y-1">
						<p class="text-sm text-muted-foreground">Received</p>
						<p class="font-medium">{formatDate(transfer.actualReceiveDate)}</p>
					</div>
				</div>
			</Card.Content>
		</Card.Root>

		<!-- Notes -->
		{#if transfer.notes}
			<Card.Root>
				<Card.Header>
					<Card.Title>Notes</Card.Title>
				</Card.Header>
				<Card.Content>
					<p class="text-muted-foreground">{transfer.notes}</p>
				</Card.Content>
			</Card.Root>
		{/if}
	{/if}
</div>

<!-- Cancel Dialog -->
<Dialog.Root bind:open={showCancelDialog}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Cancel Transfer</Dialog.Title>
			<Dialog.Description>
				Are you sure you want to cancel this transfer? This action cannot be undone.
			</Dialog.Description>
		</Dialog.Header>
		<div class="py-4">
			<label for="cancelReason" class="text-sm font-medium">Reason (optional)</label>
			<textarea
				id="cancelReason"
				bind:value={cancelReason}
				class="mt-2 w-full rounded-md border p-2 text-sm"
				rows={3}
				placeholder="Enter reason for cancellation"
			></textarea>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showCancelDialog = false)}>Keep Transfer</Button>
			<Button variant="destructive" onclick={handleCancel} disabled={isActionLoading}>
				{isActionLoading ? 'Cancelling...' : 'Cancel Transfer'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
