<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { adjustmentState, adjustmentStore } from '$lib/stores/adjustments.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import type { AdjustmentReasonCode } from '$lib/types/inventory';
	import {
		REASON_CODES,
		getReasonCodeLabel,
		CATEGORY_LABELS
	} from '$lib/constants/adjustment-reasons';

	// UI Components
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Select from '$lib/components/ui/select';

	// Icons
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SearchIcon from '@lucide/svelte/icons/search';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import ClipboardListIcon from '@lucide/svelte/icons/clipboard-list';
	import TrendingUpIcon from '@lucide/svelte/icons/trending-up';
	import TrendingDownIcon from '@lucide/svelte/icons/trending-down';
	import MinusIcon from '@lucide/svelte/icons/minus';

	// Local state
	let searchQuery = $state('');
	let warehouseFilter = $state('');
	let reasonFilter = $state<AdjustmentReasonCode | ''>('');
	let searchTimeout: ReturnType<typeof setTimeout>;

	// Reason options for filter
	const reasonOptions: { value: AdjustmentReasonCode | ''; label: string }[] = [
		{ value: '', label: 'All Reasons' },
		...REASON_CODES.map((r) => ({ value: r.code, label: r.label }))
	];

	// Derived state
	let filteredAdjustments = $derived(
		adjustmentState.items.filter((a) => {
			const matchesSearch =
				!searchQuery ||
				a.product?.sku?.toLowerCase().includes(searchQuery.toLowerCase()) ||
				a.product?.name?.toLowerCase().includes(searchQuery.toLowerCase()) ||
				a.notes?.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesWarehouse = !warehouseFilter || a.warehouseId === warehouseFilter;
			const matchesReason = !reasonFilter || a.reasonCode === reasonFilter;
			return matchesSearch && matchesWarehouse && matchesReason;
		})
	);

	// Stats
	let stats = $derived({
		total: adjustmentState.items.length,
		increases: adjustmentState.items.filter((a) => a.quantity > 0).length,
		decreases: adjustmentState.items.filter((a) => a.quantity < 0).length,
		netChange: adjustmentState.items.reduce((sum, a) => sum + a.quantity, 0)
	});

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

	function formatDate(dateStr: string | null | undefined): string {
		if (!dateStr) return '-';
		return new Date(dateStr).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}

	function formatQuantity(quantity: number): string {
		const sign = quantity > 0 ? '+' : '';
		return `${sign}${quantity}`;
	}

	function getWarehouseName(warehouseId: string | undefined | null): string {
		if (!warehouseId) return '-';
		const warehouse = warehouseState.items.find((w) => w.warehouseId === warehouseId);
		return warehouse?.warehouseName || warehouseId.slice(0, 8);
	}

	function handleSearch(e: Event) {
		const target = e.target as HTMLInputElement;
		clearTimeout(searchTimeout);
		searchTimeout = setTimeout(() => {
			searchQuery = target.value;
		}, 300);
	}

	async function handleRefresh() {
		await adjustmentStore.load();
	}

	onMount(async () => {
		await Promise.all([adjustmentStore.load(), warehouseStore.load()]);
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Breadcrumbs -->
	<div class="flex items-center gap-2 text-sm text-muted-foreground">
		<a href="/inventory" class="hover:text-foreground">Inventory</a>
		<span>/</span>
		<span class="text-foreground">Adjustments</span>
	</div>

	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Stock Adjustments</h1>
			<p class="text-muted-foreground">Record inventory changes outside normal transactions</p>
		</div>
		<Button href="/inventory/adjustments/new">
			<PlusIcon class="mr-2 h-4 w-4" />
			New Adjustment
		</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid gap-4 md:grid-cols-4">
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">Total Adjustments</p>
						<p class="text-2xl font-bold">{stats.total}</p>
					</div>
					<ClipboardListIcon class="h-8 w-8 text-muted-foreground" />
				</div>
			</Card.Content>
		</Card.Root>
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">Increases</p>
						<p class="text-2xl font-bold text-green-600">{stats.increases}</p>
					</div>
					<TrendingUpIcon class="h-8 w-8 text-green-600" />
				</div>
			</Card.Content>
		</Card.Root>
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">Decreases</p>
						<p class="text-2xl font-bold text-red-600">{stats.decreases}</p>
					</div>
					<TrendingDownIcon class="h-8 w-8 text-red-600" />
				</div>
			</Card.Content>
		</Card.Root>
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">Net Change</p>
						<p
							class="text-2xl font-bold {stats.netChange > 0
								? 'text-green-600'
								: stats.netChange < 0
									? 'text-red-600'
									: ''}"
						>
							{formatQuantity(stats.netChange)}
						</p>
					</div>
					<MinusIcon class="h-8 w-8 text-muted-foreground" />
				</div>
			</Card.Content>
		</Card.Root>
	</div>

	<!-- Filters -->
	<div class="flex flex-wrap items-center gap-4">
		<div class="relative max-w-sm min-w-[200px] flex-1">
			<SearchIcon class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<Input
				type="search"
				placeholder="Search by product, notes..."
				class="pl-10"
				oninput={handleSearch}
			/>
		</div>

		<Select.Root type="single" onValueChange={(v) => (warehouseFilter = v ?? '')}>
			<Select.Trigger class="w-[200px]">
				{warehouseState.items.find((w) => w.warehouseId === warehouseFilter)?.warehouseName ||
					'All Warehouses'}
			</Select.Trigger>
			<Select.Content>
				<Select.Item value="">All Warehouses</Select.Item>
				{#each warehouseState.items as warehouse}
					<Select.Item value={warehouse.warehouseId}>{warehouse.warehouseName}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>

		<Select.Root
			type="single"
			onValueChange={(v) => (reasonFilter = (v ?? '') as AdjustmentReasonCode | '')}
		>
			<Select.Trigger class="w-[180px]">
				{reasonOptions.find((o) => o.value === reasonFilter)?.label || 'All Reasons'}
			</Select.Trigger>
			<Select.Content>
				{#each reasonOptions as option}
					<Select.Item value={option.value}>{option.label}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>

		<Button
			variant="outline"
			size="icon"
			onclick={handleRefresh}
			disabled={adjustmentState.isLoading}
		>
			<RefreshCwIcon class="h-4 w-4 {adjustmentState.isLoading ? 'animate-spin' : ''}" />
		</Button>
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
	{:else if filteredAdjustments.length === 0}
		<!-- Empty State -->
		<div class="flex flex-col items-center justify-center py-12 text-center">
			<ClipboardListIcon class="mb-4 h-12 w-12 text-muted-foreground" />
			<h3 class="text-lg font-semibold">No adjustments found</h3>
			<p class="mb-4 text-muted-foreground">
				{searchQuery || warehouseFilter || reasonFilter
					? 'Try adjusting your filters'
					: 'Get started by creating your first adjustment'}
			</p>
			{#if !searchQuery && !warehouseFilter && !reasonFilter}
				<Button href="/inventory/adjustments/new">
					<PlusIcon class="mr-2 h-4 w-4" />
					Create Adjustment
				</Button>
			{/if}
		</div>
	{:else}
		<!-- Adjustments Table -->
		<Card.Root>
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Date</Table.Head>
						<Table.Head>Product</Table.Head>
						<Table.Head>Warehouse</Table.Head>
						<Table.Head>Quantity</Table.Head>
						<Table.Head>Reason</Table.Head>
						<Table.Head>Notes</Table.Head>
						<Table.Head class="text-right">Actions</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each filteredAdjustments as adjustment, index (adjustment.adjustmentId || `temp-${index}`)}
						<Table.Row
							class="cursor-pointer hover:bg-muted/50"
							onclick={() => goto(`/inventory/adjustments/${adjustment.adjustmentId}`)}
						>
							<Table.Cell>{formatDate(adjustment.createdAt)}</Table.Cell>
							<Table.Cell>
								<div>
									<p class="font-medium">{adjustment.product?.sku || '-'}</p>
									<p class="text-sm text-muted-foreground">{adjustment.product?.name || ''}</p>
								</div>
							</Table.Cell>
							<Table.Cell>{getWarehouseName(adjustment.warehouseId)}</Table.Cell>
							<Table.Cell>
								<Badge variant={getQuantityBadgeVariant(adjustment.quantity)}>
									{formatQuantity(adjustment.quantity)}
								</Badge>
							</Table.Cell>
							<Table.Cell>
								<Badge variant={getReasonBadgeVariant(adjustment.reasonCode)}>
									{getReasonCodeLabel(adjustment.reasonCode)}
								</Badge>
							</Table.Cell>
							<Table.Cell class="max-w-[200px] truncate">
								{adjustment.notes || '-'}
							</Table.Cell>
							<Table.Cell class="text-right">
								<Button
									variant="ghost"
									size="sm"
									href="/inventory/adjustments/{adjustment.adjustmentId}"
								>
									View
								</Button>
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</Card.Root>
	{/if}
</div>
