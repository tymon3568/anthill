<script lang="ts">
	import { onMount } from 'svelte';
	import { SvelteMap } from 'svelte/reactivity';
	import { page } from '$app/stores';
	import { toast } from 'svelte-sonner';
	import {
		stockTakeState,
		stockTakeStore,
		getLinesWithVariance,
		getCountProgress
	} from '$lib/stores/stock-take.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import type { StockTakeStatus, CountItem } from '$lib/types/stock-take';

	// UI Components
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import { Progress } from '$lib/components/ui/progress';
	import * as Dialog from '$lib/components/ui/dialog';

	// Icons
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import SaveIcon from '@lucide/svelte/icons/save';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle';
	import PlayIcon from '@lucide/svelte/icons/play';
	import XCircleIcon from '@lucide/svelte/icons/x-circle';
	import LoaderIcon from '@lucide/svelte/icons/loader';
	import TrendingUpIcon from '@lucide/svelte/icons/trending-up';
	import TrendingDownIcon from '@lucide/svelte/icons/trending-down';
	import MinusIcon from '@lucide/svelte/icons/minus';

	// Get stock take ID from URL
	let stockTakeId = $derived($page.params.id ?? '');

	// Local editing state - track pending count changes
	let pendingCounts = new SvelteMap<string, { actualQuantity: number; notes?: string }>();
	let showFinalizeDialog = $state(false);

	// Derived data
	let stockTake = $derived(stockTakeState.selected);
	let linesWithVariance = $derived(getLinesWithVariance());
	let progress = $derived(getCountProgress());

	// Check if there are unsaved changes
	let hasUnsavedChanges = $derived(pendingCounts.size > 0);

	// Check if can finalize (all lines counted)
	let canFinalize = $derived(() => {
		if (!stockTake) return false;
		if (stockTake.status !== 'in_progress' && stockTake.status !== 'draft') return false;
		// All lines must be counted (either already saved or in pending)
		return linesWithVariance.every((line) => {
			return line.isCounted || pendingCounts.has(line.lineId);
		});
	});

	function getStatusBadgeVariant(
		status: StockTakeStatus
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (status) {
			case 'draft':
				return 'secondary';
			case 'scheduled':
				return 'outline';
			case 'in_progress':
				return 'default';
			case 'completed':
				return 'default';
			case 'cancelled':
				return 'destructive';
			default:
				return 'secondary';
		}
	}

	function getStatusLabel(status: StockTakeStatus): string {
		switch (status) {
			case 'draft':
				return 'Draft';
			case 'scheduled':
				return 'Scheduled';
			case 'in_progress':
				return 'In Progress';
			case 'completed':
				return 'Completed';
			case 'cancelled':
				return 'Cancelled';
			default:
				return status;
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

	function getWarehouseName(warehouseId: string | undefined | null): string {
		if (!warehouseId) return '-';
		const warehouse = warehouseState.items.find((w) => w.warehouseId === warehouseId);
		return warehouse?.warehouseName || warehouseId.slice(0, 8);
	}

	// Get the display value for a line (pending or saved)
	function getDisplayQuantity(lineId: string, savedQuantity: number | null): string {
		const pending = pendingCounts.get(lineId);
		if (pending !== undefined) {
			return String(pending.actualQuantity);
		}
		return savedQuantity !== null ? String(savedQuantity) : '';
	}

	// Handle count input change
	function handleCountChange(lineId: string, value: string) {
		const numValue = parseInt(value);
		if (isNaN(numValue) || numValue < 0) {
			pendingCounts.delete(lineId);
			return;
		}

		pendingCounts.set(lineId, { actualQuantity: numValue });
	}

	// Save pending counts to server
	async function savePendingCounts() {
		if (pendingCounts.size === 0) return;

		const items: CountItem[] = Array.from(pendingCounts.entries()).map(([lineId, data]) => ({
			lineId,
			actualQuantity: data.actualQuantity,
			notes: data.notes
		}));

		const success = await stockTakeStore.submitCounts(stockTakeId, { items });

		if (success) {
			pendingCounts.clear();
			toast.success(`Saved ${items.length} count(s)`);
		} else {
			toast.error('Failed to save counts');
		}
	}

	// Finalize stock take
	async function handleFinalize() {
		// First save any pending counts
		if (pendingCounts.size > 0) {
			await savePendingCounts();
		}

		const success = await stockTakeStore.finalize(stockTakeId);

		if (success) {
			showFinalizeDialog = false;
			toast.success('Stock take completed successfully');
		} else {
			toast.error('Failed to finalize stock take');
		}
	}

	// Refresh data
	async function handleRefresh() {
		await stockTakeStore.get(stockTakeId);
	}

	// Check if line is editable
	function isLineEditable(): boolean {
		if (!stockTake) return false;
		return stockTake.status === 'draft' || stockTake.status === 'in_progress';
	}

	onMount(async () => {
		await Promise.all([stockTakeStore.get(stockTakeId), warehouseStore.load()]);
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Breadcrumbs -->
	<div class="flex items-center gap-2 text-sm text-muted-foreground">
		<a href="/inventory" class="hover:text-foreground">Inventory</a>
		<span>/</span>
		<a href="/inventory/stock-takes" class="hover:text-foreground">Stock Takes</a>
		<span>/</span>
		<span class="text-foreground">{stockTake?.stockTakeNumber || 'Loading...'}</span>
	</div>

	<!-- Loading State -->
	{#if stockTakeState.isLoading && !stockTake}
		<div class="flex items-center justify-center py-12">
			<RefreshCwIcon class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if !stockTake}
		<!-- Not Found -->
		<div class="flex flex-col items-center justify-center py-12 text-center">
			<XCircleIcon class="mb-4 h-12 w-12 text-muted-foreground" />
			<h3 class="text-lg font-semibold">Stock take not found</h3>
			<p class="mb-4 text-muted-foreground">The stock take you're looking for doesn't exist.</p>
			<Button href="/inventory/stock-takes">Back to Stock Takes</Button>
		</div>
	{:else}
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-4">
				<Button variant="ghost" size="icon" href="/inventory/stock-takes">
					<ArrowLeftIcon class="h-4 w-4" />
				</Button>
				<div>
					<div class="flex items-center gap-3">
						<h1 class="text-2xl font-bold">{stockTake.stockTakeNumber}</h1>
						<Badge variant={getStatusBadgeVariant(stockTake.status)}>
							{getStatusLabel(stockTake.status)}
						</Badge>
					</div>
					<p class="text-muted-foreground">
						{getWarehouseName(stockTake.warehouseId)}
						{#if stockTake.notes}
							- {stockTake.notes}
						{/if}
					</p>
				</div>
			</div>
			<div class="flex items-center gap-2">
				{#if hasUnsavedChanges}
					<Button onclick={savePendingCounts} disabled={stockTakeState.isSubmitting}>
						{#if stockTakeState.isSubmitting}
							<LoaderIcon class="mr-2 h-4 w-4 animate-spin" />
						{:else}
							<SaveIcon class="mr-2 h-4 w-4" />
						{/if}
						Save Counts ({pendingCounts.size})
					</Button>
				{/if}
				{#if stockTake.status === 'draft' || stockTake.status === 'in_progress'}
					<Button
						variant="default"
						onclick={() => (showFinalizeDialog = true)}
						disabled={!canFinalize || stockTakeState.isSubmitting}
					>
						<CheckCircleIcon class="mr-2 h-4 w-4" />
						Complete Stock Take
					</Button>
				{/if}
				<Button
					variant="outline"
					size="icon"
					onclick={handleRefresh}
					disabled={stockTakeState.isLoading}
				>
					<RefreshCwIcon class="h-4 w-4 {stockTakeState.isLoading ? 'animate-spin' : ''}" />
				</Button>
			</div>
		</div>

		<!-- Progress Card -->
		{#if stockTake.status === 'in_progress' || stockTake.status === 'draft'}
			<Card.Root>
				<Card.Content class="pt-6">
					<div class="mb-2 flex items-center justify-between">
						<span class="text-sm font-medium">Counting Progress</span>
						<span class="text-sm text-muted-foreground">
							{progress.counted} of {progress.total} items counted ({progress.percentage}%)
						</span>
					</div>
					<Progress value={progress.percentage} max={100} class="h-3" />
					{#if progress.percentage < 100}
						<p class="mt-2 text-sm text-muted-foreground">
							{progress.total - progress.counted} items remaining to count
						</p>
					{:else}
						<p class="mt-2 text-sm text-green-600">All items have been counted</p>
					{/if}
				</Card.Content>
			</Card.Root>
		{/if}

		<!-- Info Cards for Completed -->
		{#if stockTake.status === 'completed'}
			<div class="grid gap-4 md:grid-cols-3">
				<Card.Root>
					<Card.Content class="pt-6">
						<div class="flex items-center justify-between">
							<div>
								<p class="text-sm text-muted-foreground">Items Counted</p>
								<p class="text-2xl font-bold">{linesWithVariance.length}</p>
							</div>
							<CheckCircleIcon class="h-8 w-8 text-green-600" />
						</div>
					</Card.Content>
				</Card.Root>
				<Card.Root>
					<Card.Content class="pt-6">
						<div class="flex items-center justify-between">
							<div>
								<p class="text-sm text-muted-foreground">Started</p>
								<p class="text-lg font-medium">{formatDate(stockTake.startedAt)}</p>
							</div>
							<PlayIcon class="h-8 w-8 text-muted-foreground" />
						</div>
					</Card.Content>
				</Card.Root>
				<Card.Root>
					<Card.Content class="pt-6">
						<div class="flex items-center justify-between">
							<div>
								<p class="text-sm text-muted-foreground">Completed</p>
								<p class="text-lg font-medium">{formatDate(stockTake.completedAt)}</p>
							</div>
							<CheckCircleIcon class="h-8 w-8 text-green-600" />
						</div>
					</Card.Content>
				</Card.Root>
			</div>
		{/if}

		<!-- Error State -->
		{#if stockTakeState.error}
			<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
				<p class="text-destructive">{stockTakeState.error}</p>
				<Button variant="outline" onclick={handleRefresh} class="mt-2">Retry</Button>
			</div>
		{/if}

		<!-- Count Table -->
		<Card.Root>
			<Card.Header>
				<Card.Title>Stock Take Lines</Card.Title>
				<Card.Description>
					{#if isLineEditable()}
						Enter the actual quantities counted for each product
					{:else}
						Final count results
					{/if}
				</Card.Description>
			</Card.Header>
			<Card.Content>
				{#if linesWithVariance.length === 0}
					<div class="flex flex-col items-center justify-center py-8 text-center">
						<p class="text-muted-foreground">No items in this stock take</p>
					</div>
				{:else}
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head class="w-[200px]">Product</Table.Head>
								<Table.Head class="w-[120px] text-right">Expected</Table.Head>
								<Table.Head class="w-[150px] text-right">Actual</Table.Head>
								<Table.Head class="w-[120px] text-right">Variance</Table.Head>
								<Table.Head class="w-[100px]">Status</Table.Head>
								{#if stockTake.status === 'completed'}
									<Table.Head>Counted By</Table.Head>
								{/if}
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each linesWithVariance as line (line.lineId)}
								<Table.Row class={line.isCounted ? '' : 'bg-muted/30'}>
									<Table.Cell>
										<div>
											<p class="font-medium">{line.product?.sku || '-'}</p>
											<p class="text-sm text-muted-foreground">{line.product?.name || ''}</p>
										</div>
									</Table.Cell>
									<Table.Cell class="text-right font-mono">
										{line.expectedQuantity}
									</Table.Cell>
									<Table.Cell class="text-right">
										{#if isLineEditable()}
											<Input
												type="number"
												min="0"
												class="ml-auto w-24 text-right font-mono"
												value={getDisplayQuantity(line.lineId, line.actualQuantity)}
												oninput={(e) =>
													handleCountChange(line.lineId, (e.target as HTMLInputElement).value)}
												placeholder="0"
											/>
										{:else}
											<span class="font-mono">{line.actualQuantity ?? '-'}</span>
										{/if}
									</Table.Cell>
									<Table.Cell class="text-right">
										{#if line.isCounted || pendingCounts.has(line.lineId)}
											{@const pending = pendingCounts.get(line.lineId)}
											{@const actualQty = pending?.actualQuantity ?? line.actualQuantity ?? 0}
											{@const variance = actualQty - line.expectedQuantity}
											<div class="flex items-center justify-end gap-1">
												{#if variance > 0}
													<TrendingUpIcon class="h-4 w-4 text-green-600" />
													<span class="font-mono text-green-600">+{variance}</span>
												{:else if variance < 0}
													<TrendingDownIcon class="h-4 w-4 text-red-600" />
													<span class="font-mono text-red-600">{variance}</span>
												{:else}
													<MinusIcon class="h-4 w-4 text-muted-foreground" />
													<span class="font-mono text-muted-foreground">0</span>
												{/if}
											</div>
										{:else}
											<span class="text-muted-foreground">-</span>
										{/if}
									</Table.Cell>
									<Table.Cell>
										{#if pendingCounts.has(line.lineId)}
											<Badge variant="outline">Unsaved</Badge>
										{:else if line.isCounted}
											<Badge variant="default">Counted</Badge>
										{:else}
											<Badge variant="secondary">Pending</Badge>
										{/if}
									</Table.Cell>
									{#if stockTake.status === 'completed'}
										<Table.Cell class="text-sm text-muted-foreground">
											{line.countedByUser?.fullName || '-'}
										</Table.Cell>
									{/if}
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				{/if}
			</Card.Content>
		</Card.Root>
	{/if}
</div>

<!-- Finalize Confirmation Dialog -->
<Dialog.Root bind:open={showFinalizeDialog}>
	<Dialog.Content>
		<Dialog.Header>
			<Dialog.Title>Complete Stock Take?</Dialog.Title>
			<Dialog.Description>
				This will finalize the stock take and automatically create inventory adjustments for any
				variances found. This action cannot be undone.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showFinalizeDialog = false)}>Cancel</Button>
			<Button onclick={handleFinalize} disabled={stockTakeState.isSubmitting}>
				{#if stockTakeState.isSubmitting}
					<LoaderIcon class="mr-2 h-4 w-4 animate-spin" />
					Processing...
				{:else}
					Complete Stock Take
				{/if}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
