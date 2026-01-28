<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { transferState, transferStore, type TransferResponse } from '$lib/stores/stock-movements.svelte';
	import { warehouseState, warehouseStore } from '$lib/stores/inventory.svelte';
	import type { TransferStatus } from '$lib/types/inventory';

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
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import PackageIcon from '@lucide/svelte/icons/package';

	// Local state
	let searchQuery = $state('');
	let statusFilter = $state<TransferStatus | ''>('');
	let sourceWarehouseFilter = $state('');
	let searchTimeout: ReturnType<typeof setTimeout>;

	// Status options for filter
	const statusOptions: { value: TransferStatus | ''; label: string }[] = [
		{ value: '', label: 'All Statuses' },
		{ value: 'draft', label: 'Draft' },
		{ value: 'confirmed', label: 'Confirmed' },
		{ value: 'picked', label: 'Picked' },
		{ value: 'shipped', label: 'Shipped' },
		{ value: 'received', label: 'Received' },
		{ value: 'cancelled', label: 'Cancelled' }
	];

	// Derived state
	let filteredTransfers = $derived(
		transferState.items.filter((t) => {
			const matchesSearch =
				!searchQuery ||
				t.transferNumber.toLowerCase().includes(searchQuery.toLowerCase()) ||
				t.referenceNumber?.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesStatus = !statusFilter || t.status === statusFilter;
			const matchesWarehouse =
				!sourceWarehouseFilter || t.sourceWarehouseId === sourceWarehouseFilter;
			return matchesSearch && matchesStatus && matchesWarehouse;
		})
	);

	// Stats
	let stats = $derived({
		total: transferState.items.length,
		draft: transferState.items.filter((t) => t.status === 'draft').length,
		inTransit: transferState.items.filter((t) => ['confirmed', 'picked', 'shipped'].includes(t.status)).length,
		completed: transferState.items.filter((t) => t.status === 'received').length
	});

	function getStatusBadgeVariant(status: TransferStatus): 'default' | 'secondary' | 'destructive' | 'outline' {
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

	function getPriorityBadgeVariant(priority: string): 'default' | 'secondary' | 'destructive' | 'outline' {
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
			year: 'numeric'
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
		await transferStore.load();
	}

	onMount(async () => {
		await Promise.all([transferStore.load(), warehouseStore.load()]);
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Stock Transfers</h1>
			<p class="text-muted-foreground">Manage inventory transfers between warehouses</p>
		</div>
		<Button href="/inventory/transfers/new">
			<PlusIcon class="mr-2 h-4 w-4" />
			New Transfer
		</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid gap-4 md:grid-cols-4">
		<Card.Root>
			<Card.Content class="pt-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm text-muted-foreground">Total Transfers</p>
						<p class="text-2xl font-bold">{stats.total}</p>
					</div>
					<PackageIcon class="h-8 w-8 text-muted-foreground" />
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
						<p class="text-sm text-muted-foreground">In Transit</p>
						<p class="text-2xl font-bold">{stats.inTransit}</p>
					</div>
					<Badge variant="default">{stats.inTransit}</Badge>
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
					<Badge variant="outline">{stats.completed}</Badge>
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
				placeholder="Search transfers..."
				class="pl-10"
				oninput={handleSearch}
			/>
		</div>

		<Select.Root
			type="single"
			onValueChange={(v) => (statusFilter = (v ?? '') as TransferStatus | '')}
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
			onValueChange={(v) => (sourceWarehouseFilter = v ?? '')}
		>
			<Select.Trigger class="w-[200px]">
				{warehouseState.items.find((w) => w.warehouseId === sourceWarehouseFilter)?.warehouseName ||
					'All Warehouses'}
			</Select.Trigger>
			<Select.Content>
				<Select.Item value="">All Warehouses</Select.Item>
				{#each warehouseState.items as warehouse}
					<Select.Item value={warehouse.warehouseId}>{warehouse.warehouseName}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>

		<Button variant="outline" size="icon" onclick={handleRefresh} disabled={transferState.isLoading}>
			<RefreshCwIcon class="h-4 w-4 {transferState.isLoading ? 'animate-spin' : ''}" />
		</Button>
	</div>

	<!-- Error State -->
	{#if transferState.error}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<p class="text-destructive">{transferState.error}</p>
			<Button variant="outline" onclick={handleRefresh} class="mt-2">Retry</Button>
		</div>
	{/if}

	<!-- Loading State -->
	{#if transferState.isLoading}
		<div class="flex items-center justify-center py-12">
			<RefreshCwIcon class="h-8 w-8 animate-spin text-muted-foreground" />
		</div>
	{:else if filteredTransfers.length === 0}
		<!-- Empty State -->
		<div class="flex flex-col items-center justify-center py-12 text-center">
			<PackageIcon class="h-12 w-12 text-muted-foreground mb-4" />
			<h3 class="text-lg font-semibold">No transfers found</h3>
			<p class="text-muted-foreground mb-4">
				{searchQuery || statusFilter || sourceWarehouseFilter
					? 'Try adjusting your filters'
					: 'Get started by creating your first transfer'}
			</p>
			{#if !searchQuery && !statusFilter && !sourceWarehouseFilter}
				<Button href="/inventory/transfers/new">
					<PlusIcon class="mr-2 h-4 w-4" />
					Create Transfer
				</Button>
			{/if}
		</div>
	{:else}
		<!-- Transfers Table -->
		<Card.Root>
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Transfer #</Table.Head>
						<Table.Head>Route</Table.Head>
						<Table.Head>Status</Table.Head>
						<Table.Head>Priority</Table.Head>
						<Table.Head>Created</Table.Head>
						<Table.Head>Expected Ship</Table.Head>
						<Table.Head class="text-right">Actions</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each filteredTransfers as transfer (transfer.transferId)}
						<Table.Row
							class="cursor-pointer hover:bg-muted/50"
							onclick={() => goto(`/inventory/transfers/${transfer.transferId}`)}
						>
							<Table.Cell class="font-medium">{transfer.transferNumber}</Table.Cell>
							<Table.Cell>
								<div class="flex items-center gap-2">
									<span class="text-sm">{getWarehouseName(transfer.sourceWarehouseId)}</span>
									<ArrowRightIcon class="h-4 w-4 text-muted-foreground" />
									<span class="text-sm">{getWarehouseName(transfer.destinationWarehouseId)}</span>
								</div>
							</Table.Cell>
							<Table.Cell>
								<Badge variant={getStatusBadgeVariant(transfer.status)}>
									{transfer.status}
								</Badge>
							</Table.Cell>
							<Table.Cell>
								<Badge variant={getPriorityBadgeVariant(transfer.priority)}>
									{transfer.priority}
								</Badge>
							</Table.Cell>
							<Table.Cell>{formatDate(transfer.createdAt)}</Table.Cell>
							<Table.Cell>{formatDate(transfer.expectedShipDate)}</Table.Cell>
							<Table.Cell class="text-right">
								<Button
									variant="ghost"
									size="sm"
									href="/inventory/transfers/{transfer.transferId}"
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
