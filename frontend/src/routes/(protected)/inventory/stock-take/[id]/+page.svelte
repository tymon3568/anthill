<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { stockTakeState, stockTakeStore } from '$lib/stores/stock-movements.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import type { StockTakeStatus, StockTakeLineResponse } from '$lib/api/inventory/stock-take';
	import { toast } from 'svelte-sonner';

	// UI Components
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';

	// Icons
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import SaveIcon from '@lucide/svelte/icons/save';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import AlertTriangleIcon from '@lucide/svelte/icons/alert-triangle';

	let stockTakeId = $derived(page.params.id ?? '');
	let stockTake = $derived(stockTakeState.selected);
	let lines = $derived(stockTakeState.lines);
	let adjustments = $derived(stockTakeState.adjustments);

	let isActionLoading = $state(false);
	let showFinalizeDialog = $state(false);
	let editedCounts = $state<Record<string, { quantity: number; notes: string }>>({});

	function getStatusBadgeVariant(
		status: StockTakeStatus
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (status) {
			case 'draft':
				return 'secondary';
			case 'in_progress':
				return 'default';
			case 'completed':
				return 'outline';
			case 'cancelled':
				return 'destructive';
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

	function getVarianceClass(line: StockTakeLineResponse): string {
		if (line.actualQuantity === null || line.actualQuantity === undefined) return '';
		const diff = line.differenceQuantity ?? 0;
		if (diff === 0) return 'text-green-600';
		if (diff > 0) return 'text-blue-600';
		return 'text-red-600';
	}

	function initializeEditedCounts() {
		const counts: Record<string, { quantity: number; notes: string }> = {};
		lines.forEach((line) => {
			counts[line.productId] = {
				quantity: line.actualQuantity ?? line.expectedQuantity,
				notes: line.notes || ''
			};
		});
		editedCounts = counts;
	}

	// Stats
	let stats = $derived({
		total: lines.length,
		counted: lines.filter((l) => l.actualQuantity !== null && l.actualQuantity !== undefined)
			.length,
		withVariance: lines.filter((l) => l.differenceQuantity && l.differenceQuantity !== 0).length
	});

	let canCount = $derived(stockTake?.status === 'draft' || stockTake?.status === 'in_progress');
	let canFinalize = $derived(stockTake?.status === 'in_progress' && stats.counted === stats.total);

	async function handleSaveCounts() {
		if (!stockTake) return;
		isActionLoading = true;

		const items = Object.entries(editedCounts).map(([productId, data]) => ({
			productId,
			actualQuantity: data.quantity,
			notes: data.notes || undefined
		}));

		const success = await stockTakeStore.count(stockTake.stockTakeId, { items });

		if (success) {
			toast.success('Counts saved successfully');
		}
		isActionLoading = false;
	}

	async function handleFinalize() {
		if (!stockTake) return;
		isActionLoading = true;

		const success = await stockTakeStore.finalize(stockTake.stockTakeId);

		if (success) {
			toast.success('Stock take finalized. Adjustments have been created.');
			showFinalizeDialog = false;
		}
		isActionLoading = false;
	}

	$effect(() => {
		if (lines.length > 0 && Object.keys(editedCounts).length === 0) {
			initializeEditedCounts();
		}
	});

	onMount(async () => {
		await Promise.all([stockTakeStore.get(stockTakeId), warehouseStore.load()]);
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<Button variant="ghost" size="icon" href="/inventory/stock-take">
				<ArrowLeftIcon class="h-4 w-4" />
			</Button>
			<div>
				<h1 class="text-2xl font-bold">
					{stockTake?.stockTakeNumber || 'Loading...'}
				</h1>
				<p class="text-muted-foreground">
					{stockTake ? getWarehouseName(stockTake.warehouseId) : ''}
				</p>
			</div>
		</div>
		{#if stockTake}
			<Badge variant={getStatusBadgeVariant(stockTake.status)} class="text-sm">
				{stockTake.status.replace('_', ' ').toUpperCase()}
			</Badge>
		{/if}
	</div>

	{#if stockTakeState.isLoading && !stockTake}
		<div class="flex items-center justify-center py-12">
			<RefreshCwIcon class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if stockTakeState.error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<p class="text-destructive">{stockTakeState.error}</p>
			<Button variant="outline" onclick={() => stockTakeStore.get(stockTakeId)} class="mt-2">
				Retry
			</Button>
		</div>
	{:else if stockTake}
		<!-- Stats & Actions -->
		<div class="flex flex-wrap items-center justify-between gap-4">
			<div class="flex gap-6">
				<div>
					<p class="text-sm text-muted-foreground">Total Items</p>
					<p class="text-2xl font-bold">{stats.total}</p>
				</div>
				<div>
					<p class="text-sm text-muted-foreground">Counted</p>
					<p class="text-2xl font-bold">{stats.counted} / {stats.total}</p>
				</div>
				<div>
					<p class="text-sm text-muted-foreground">With Variance</p>
					<p class="text-2xl font-bold">{stats.withVariance}</p>
				</div>
			</div>

			{#if canCount}
				<div class="flex gap-3">
					<Button variant="outline" onclick={handleSaveCounts} disabled={isActionLoading}>
						<SaveIcon class="mr-2 h-4 w-4" />
						Save Counts
					</Button>
					{#if canFinalize}
						<Button onclick={() => (showFinalizeDialog = true)} disabled={isActionLoading}>
							<CheckCircleIcon class="mr-2 h-4 w-4" />
							Finalize
						</Button>
					{/if}
				</div>
			{/if}
		</div>

		<!-- Info Card -->
		<Card.Root>
			<Card.Content class="grid gap-4 pt-6 md:grid-cols-4">
				<div>
					<p class="text-sm text-muted-foreground">Started</p>
					<p class="font-medium">{formatDate(stockTake.startedAt)}</p>
				</div>
				<div>
					<p class="text-sm text-muted-foreground">Completed</p>
					<p class="font-medium">{formatDate(stockTake.completedAt)}</p>
				</div>
				<div>
					<p class="text-sm text-muted-foreground">Created By</p>
					<p class="font-medium">{stockTake.createdBy.slice(0, 8)}...</p>
				</div>
				<div>
					<p class="text-sm text-muted-foreground">Notes</p>
					<p class="font-medium">{stockTake.notes || '-'}</p>
				</div>
			</Card.Content>
		</Card.Root>

		<!-- Count Lines Table -->
		<Card.Root>
			<Card.Header>
				<Card.Title>Count Lines</Card.Title>
				<Card.Description>
					{#if canCount}
						Enter the actual quantities you count for each product.
					{:else}
						Review the counted quantities and any variances.
					{/if}
				</Card.Description>
			</Card.Header>
			<Card.Content>
				{#if lines.length === 0}
					<div class="flex flex-col items-center justify-center py-8 text-center">
						<p class="text-muted-foreground">No items to count in this stock take.</p>
					</div>
				{:else}
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head>Product</Table.Head>
								<Table.Head class="text-right">Expected</Table.Head>
								<Table.Head class="w-[150px] text-right">Actual</Table.Head>
								<Table.Head class="text-right">Variance</Table.Head>
								<Table.Head>Notes</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each lines as line (line.lineId)}
								<Table.Row>
									<Table.Cell>
										<div>
											<p class="font-medium">{line.productName || line.productId.slice(0, 8)}</p>
											{#if line.productSku}
												<p class="text-sm text-muted-foreground">{line.productSku}</p>
											{/if}
										</div>
									</Table.Cell>
									<Table.Cell class="text-right font-medium">
										{line.expectedQuantity}
									</Table.Cell>
									<Table.Cell class="text-right">
										{#if canCount}
											<Input
												type="number"
												min="0"
												class="ml-auto w-24 text-right"
												value={editedCounts[line.productId]?.quantity ?? line.expectedQuantity}
												onchange={(e) => {
													const target = e.target as HTMLInputElement;
													editedCounts = {
														...editedCounts,
														[line.productId]: {
															...editedCounts[line.productId],
															quantity: parseInt(target.value) || 0
														}
													};
												}}
											/>
										{:else}
											<span class="font-medium">{line.actualQuantity ?? '-'}</span>
										{/if}
									</Table.Cell>
									<Table.Cell class="text-right">
										<span class={getVarianceClass(line)}>
											{#if line.differenceQuantity !== null && line.differenceQuantity !== undefined}
												{line.differenceQuantity > 0 ? '+' : ''}{line.differenceQuantity}
											{:else}
												-
											{/if}
										</span>
									</Table.Cell>
									<Table.Cell>
										{#if canCount}
											<Input
												type="text"
												placeholder="Notes"
												class="w-full"
												value={editedCounts[line.productId]?.notes ?? ''}
												onchange={(e) => {
													const target = e.target as HTMLInputElement;
													editedCounts = {
														...editedCounts,
														[line.productId]: {
															...editedCounts[line.productId],
															notes: target.value
														}
													};
												}}
											/>
										{:else}
											{line.notes || '-'}
										{/if}
									</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				{/if}
			</Card.Content>
		</Card.Root>

		<!-- Adjustments (if finalized) -->
		{#if adjustments.length > 0}
			<Card.Root>
				<Card.Header>
					<Card.Title>Generated Adjustments</Card.Title>
					<Card.Description
						>These adjustments were created when the stock take was finalized.</Card.Description
					>
				</Card.Header>
				<Card.Content>
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head>Adjustment ID</Table.Head>
								<Table.Head>Product</Table.Head>
								<Table.Head class="text-right">Quantity</Table.Head>
								<Table.Head>Reason</Table.Head>
								<Table.Head>Adjusted At</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each adjustments as adj (adj.adjustmentId)}
								<Table.Row>
									<Table.Cell class="font-mono text-sm"
										>{adj.adjustmentId.slice(0, 8)}...</Table.Cell
									>
									<Table.Cell>{adj.productId.slice(0, 8)}...</Table.Cell>
									<Table.Cell class="text-right">
										<span class={adj.quantity > 0 ? 'text-green-600' : 'text-red-600'}>
											{adj.quantity > 0 ? '+' : ''}{adj.quantity}
										</span>
									</Table.Cell>
									<Table.Cell>{adj.reason}</Table.Cell>
									<Table.Cell>{formatDate(adj.adjustedAt)}</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				</Card.Content>
			</Card.Root>
		{/if}
	{/if}
</div>

<!-- Finalize Dialog -->
<Dialog.Root bind:open={showFinalizeDialog}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title class="flex items-center gap-2">
				<AlertTriangleIcon class="h-5 w-5 text-amber-500" />
				Finalize Stock Take
			</Dialog.Title>
			<Dialog.Description>
				This will create inventory adjustments for all items with variances. This action cannot be
				undone.
			</Dialog.Description>
		</Dialog.Header>
		<div class="py-4">
			<p class="text-sm">
				<strong>{stats.withVariance}</strong> item(s) will be adjusted.
			</p>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showFinalizeDialog = false)}>Cancel</Button>
			<Button onclick={handleFinalize} disabled={isActionLoading}>
				{isActionLoading ? 'Finalizing...' : 'Finalize Stock Take'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
