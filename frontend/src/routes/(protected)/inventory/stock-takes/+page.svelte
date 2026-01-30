<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		stockTakeState,
		stockTakeStore,
		getItemsWithProgress
	} from '$lib/stores/stock-take.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import type { StockTakeStatus } from '$lib/types/stock-take';

	// UI Components
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Select from '$lib/components/ui/select';
	import { Progress } from '$lib/components/ui/progress';

	// Icons
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SearchIcon from '@lucide/svelte/icons/search';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import ClipboardCheckIcon from '@lucide/svelte/icons/clipboard-check';
	import ClockIcon from '@lucide/svelte/icons/clock';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle';
	import PlayIcon from '@lucide/svelte/icons/play';

	// Local state
	let searchQuery = $state('');
	let warehouseFilter = $state('');
	let statusFilter = $state<StockTakeStatus | ''>('');
	let searchTimeout: ReturnType<typeof setTimeout>;

	// Status options for filter
	const statusOptions: { value: StockTakeStatus | ''; label: string }[] = [
		{ value: '', label: 'All Statuses' },
		{ value: 'draft', label: 'Draft' },
		{ value: 'scheduled', label: 'Scheduled' },
		{ value: 'in_progress', label: 'In Progress' },
		{ value: 'completed', label: 'Completed' },
		{ value: 'cancelled', label: 'Cancelled' }
	];

	// Derived state with progress
	let stockTakesWithProgress = $derived(getItemsWithProgress());

	let filteredStockTakes = $derived(
		stockTakesWithProgress.filter((st) => {
			const matchesSearch =
				!searchQuery ||
				st.stockTakeNumber.toLowerCase().includes(searchQuery.toLowerCase()) ||
				st.notes?.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesWarehouse = !warehouseFilter || st.warehouseId === warehouseFilter;
			const matchesStatus = !statusFilter || st.status === statusFilter;
			return matchesSearch && matchesWarehouse && matchesStatus;
		})
	);

	// Stats
	let stats = $derived({
		total: stockTakeState.items.length,
		draft: stockTakeState.items.filter((s) => s.status === 'draft').length,
		inProgress: stockTakeState.items.filter((s) => s.status === 'in_progress').length,
		completed: stockTakeState.items.filter((s) => s.status === 'completed').length
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
			year: 'numeric'
		});
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
		await stockTakeStore.load();
	}

	onMount(async () => {
		await Promise.all([stockTakeStore.load(), warehouseStore.load()]);
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Breadcrumbs -->
	<div class="flex items-center gap-2 text-sm text-muted-foreground">
		<a href="/inventory" class="hover:text-foreground">Inventory</a>
		<span>/</span>
		<span class="text-foreground">Stock Takes</span>
	</div>

	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Stock Takes</h1>
			<p class="text-muted-foreground">Physical inventory counts and reconciliation</p>
		</div>
		<Button href="/inventory/stock-takes/new">
			<PlusIcon class="mr-2 h-4 w-4" />
			New Stock Take
		</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid gap-4 md:grid-cols-4">
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">Total Stock Takes</p>
						<p class="text-2xl font-bold">{stats.total}</p>
					</div>
					<ClipboardCheckIcon class="h-8 w-8 text-muted-foreground" />
				</div>
			</Card.Content>
		</Card.Root>
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">Draft</p>
						<p class="text-2xl font-bold">{stats.draft}</p>
					</div>
					<ClockIcon class="h-8 w-8 text-muted-foreground" />
				</div>
			</Card.Content>
		</Card.Root>
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">In Progress</p>
						<p class="text-2xl font-bold text-blue-600">{stats.inProgress}</p>
					</div>
					<PlayIcon class="h-8 w-8 text-blue-600" />
				</div>
			</Card.Content>
		</Card.Root>
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">Completed</p>
						<p class="text-2xl font-bold text-green-600">{stats.completed}</p>
					</div>
					<CheckCircleIcon class="h-8 w-8 text-green-600" />
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
				placeholder="Search by stock take number..."
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
				{#each warehouseState.items as warehouse (warehouse.warehouseId)}
					<Select.Item value={warehouse.warehouseId}>{warehouse.warehouseName}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>

		<Select.Root
			type="single"
			onValueChange={(v) => (statusFilter = (v ?? '') as StockTakeStatus | '')}
		>
			<Select.Trigger class="w-[180px]">
				{statusOptions.find((o) => o.value === statusFilter)?.label || 'All Statuses'}
			</Select.Trigger>
			<Select.Content>
				{#each statusOptions as option (option.value)}
					<Select.Item value={option.value}>{option.label}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>

		<Button
			variant="outline"
			size="icon"
			onclick={handleRefresh}
			disabled={stockTakeState.isLoading}
		>
			<RefreshCwIcon class="h-4 w-4 {stockTakeState.isLoading ? 'animate-spin' : ''}" />
		</Button>
	</div>

	<!-- Error State -->
	{#if stockTakeState.error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<p class="text-destructive">{stockTakeState.error}</p>
			<Button variant="outline" onclick={handleRefresh} class="mt-2">Retry</Button>
		</div>
	{/if}

	<!-- Loading State -->
	{#if stockTakeState.isLoading}
		<div class="flex items-center justify-center py-12">
			<RefreshCwIcon class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if filteredStockTakes.length === 0}
		<!-- Empty State -->
		<div class="flex flex-col items-center justify-center py-12 text-center">
			<ClipboardCheckIcon class="mb-4 h-12 w-12 text-muted-foreground" />
			<h3 class="text-lg font-semibold">No stock takes found</h3>
			<p class="mb-4 text-muted-foreground">
				{searchQuery || warehouseFilter || statusFilter
					? 'Try adjusting your filters'
					: 'Get started by creating your first stock take'}
			</p>
			{#if !searchQuery && !warehouseFilter && !statusFilter}
				<Button href="/inventory/stock-takes/new">
					<PlusIcon class="mr-2 h-4 w-4" />
					Create Stock Take
				</Button>
			{/if}
		</div>
	{:else}
		<!-- Stock Takes Table -->
		<Card.Root>
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Number</Table.Head>
						<Table.Head>Warehouse</Table.Head>
						<Table.Head>Status</Table.Head>
						<Table.Head>Progress</Table.Head>
						<Table.Head>Created</Table.Head>
						<Table.Head>Completed</Table.Head>
						<Table.Head class="text-right">Actions</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each filteredStockTakes as stockTake (stockTake.stockTakeId)}
						<Table.Row
							class="cursor-pointer hover:bg-muted/50"
							onclick={() => goto(`/inventory/stock-takes/${stockTake.stockTakeId}`)}
						>
							<Table.Cell>
								<div>
									<p class="font-medium">{stockTake.stockTakeNumber}</p>
									{#if stockTake.notes}
										<p class="max-w-[200px] truncate text-sm text-muted-foreground">
											{stockTake.notes}
										</p>
									{/if}
								</div>
							</Table.Cell>
							<Table.Cell>{getWarehouseName(stockTake.warehouseId)}</Table.Cell>
							<Table.Cell>
								<Badge variant={getStatusBadgeVariant(stockTake.status)}>
									{getStatusLabel(stockTake.status)}
								</Badge>
							</Table.Cell>
							<Table.Cell>
								{#if stockTake.status === 'in_progress' || stockTake.status === 'completed'}
									<div class="flex items-center gap-2">
										<Progress value={stockTake.progress} max={100} class="h-2 w-20" />
										<span class="text-sm text-muted-foreground">
											{stockTake.countedCount}/{stockTake.totalCount}
										</span>
									</div>
								{:else}
									<span class="text-sm text-muted-foreground">-</span>
								{/if}
							</Table.Cell>
							<Table.Cell>{formatDate(stockTake.createdAt)}</Table.Cell>
							<Table.Cell>{formatDate(stockTake.completedAt)}</Table.Cell>
							<Table.Cell class="text-right">
								<Button
									variant="ghost"
									size="sm"
									href="/inventory/stock-takes/{stockTake.stockTakeId}"
								>
									{stockTake.status === 'in_progress' ? 'Continue' : 'View'}
								</Button>
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</Card.Root>
	{/if}
</div>
