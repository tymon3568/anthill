<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { stockTakeState, stockTakeStore } from '$lib/stores/stock-movements.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import type { StockTakeStatus } from '$lib/api/inventory/stock-take';

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
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle';
	import ClockIcon from '@lucide/svelte/icons/clock';

	// Local state
	let searchQuery = $state('');
	let statusFilter = $state<StockTakeStatus | ''>('');
	let warehouseFilter = $state('');
	let searchTimeout: ReturnType<typeof setTimeout>;

	// Status options for filter
	const statusOptions: { value: StockTakeStatus | ''; label: string }[] = [
		{ value: '', label: 'All Statuses' },
		{ value: 'draft', label: 'Draft' },
		{ value: 'in_progress', label: 'In Progress' },
		{ value: 'completed', label: 'Completed' },
		{ value: 'cancelled', label: 'Cancelled' }
	];

	// Derived state
	let filteredStockTakes = $derived(
		stockTakeState.items.filter((st) => {
			const matchesSearch =
				!searchQuery ||
				st.stockTakeNumber.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesStatus = !statusFilter || st.status === statusFilter;
			const matchesWarehouse = !warehouseFilter || st.warehouseId === warehouseFilter;
			return matchesSearch && matchesStatus && matchesWarehouse;
		})
	);

	// Stats
	let stats = $derived({
		total: stockTakeState.items.length,
		draft: stockTakeState.items.filter((st) => st.status === 'draft').length,
		inProgress: stockTakeState.items.filter((st) => st.status === 'in_progress').length,
		completed: stockTakeState.items.filter((st) => st.status === 'completed').length
	});

	function getStatusBadgeVariant(status: StockTakeStatus): 'default' | 'secondary' | 'destructive' | 'outline' {
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
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Stock Take</h1>
			<p class="text-muted-foreground">Physical inventory counting and reconciliation</p>
		</div>
		<Button href="/inventory/stock-take/new">
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
					<ClipboardListIcon class="h-8 w-8 text-muted-foreground" />
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
					<Badge variant="secondary">{stats.draft}</Badge>
				</div>
			</Card.Content>
		</Card.Root>
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">In Progress</p>
						<p class="text-2xl font-bold">{stats.inProgress}</p>
					</div>
					<ClockIcon class="h-8 w-8 text-blue-500" />
				</div>
			</Card.Content>
		</Card.Root>
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">Completed</p>
						<p class="text-2xl font-bold">{stats.completed}</p>
					</div>
					<CheckCircleIcon class="h-8 w-8 text-green-500" />
				</div>
			</Card.Content>
		</Card.Root>
	</div>

	<!-- Filters -->
	<div class="flex flex-wrap items-center gap-4">
		<div class="relative flex-1 min-w-[200px] max-w-sm">
			<SearchIcon class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<Input
				type="search"
				placeholder="Search stock takes..."
				class="pl-10"
				oninput={handleSearch}
			/>
		</div>

		<Select.Root
			type="single"
			onValueChange={(v) => (statusFilter = (v ?? '') as StockTakeStatus | '')}
		>
			<Select.Trigger class="w-[180px]">
				{statusOptions.find((o) => o.value === statusFilter)?.label || 'All Statuses'}
			</Select.Trigger>
			<Select.Content>
				{#each statusOptions as option}
					<Select.Item value={option.value}>{option.label}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>

		<Select.Root
			type="single"
			onValueChange={(v) => (warehouseFilter = v ?? '')}
		>
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

		<Button variant="outline" size="icon" onclick={handleRefresh} disabled={stockTakeState.isLoading}>
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
			<ClipboardListIcon class="h-12 w-12 text-muted-foreground mb-4" />
			<h3 class="text-lg font-semibold">No stock takes found</h3>
			<p class="text-muted-foreground mb-4">
				{searchQuery || statusFilter || warehouseFilter
					? 'Try adjusting your filters'
					: 'Start a physical inventory count to verify stock levels'}
			</p>
			{#if !searchQuery && !statusFilter && !warehouseFilter}
				<Button href="/inventory/stock-take/new">
					<PlusIcon class="mr-2 h-4 w-4" />
					Start Stock Take
				</Button>
			{/if}
		</div>
	{:else}
		<!-- Stock Takes Table -->
		<Card.Root>
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Stock Take #</Table.Head>
						<Table.Head>Warehouse</Table.Head>
						<Table.Head>Status</Table.Head>
						<Table.Head>Started</Table.Head>
						<Table.Head>Completed</Table.Head>
						<Table.Head>Notes</Table.Head>
						<Table.Head class="text-right">Actions</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each filteredStockTakes as stockTake (stockTake.stockTakeId)}
						<Table.Row
							class="cursor-pointer hover:bg-muted/50"
							onclick={() => goto(`/inventory/stock-take/${stockTake.stockTakeId}`)}
						>
							<Table.Cell class="font-medium">{stockTake.stockTakeNumber}</Table.Cell>
							<Table.Cell>{getWarehouseName(stockTake.warehouseId)}</Table.Cell>
							<Table.Cell>
								<Badge variant={getStatusBadgeVariant(stockTake.status)}>
									{stockTake.status.replace('_', ' ')}
								</Badge>
							</Table.Cell>
							<Table.Cell>{formatDate(stockTake.startedAt)}</Table.Cell>
							<Table.Cell>{formatDate(stockTake.completedAt)}</Table.Cell>
							<Table.Cell class="max-w-[200px] truncate">
								{stockTake.notes || '-'}
							</Table.Cell>
							<Table.Cell class="text-right">
								<Button
									variant="ghost"
									size="sm"
									href="/inventory/stock-take/{stockTake.stockTakeId}"
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
