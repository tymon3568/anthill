<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { ChartCard } from '$lib/components/dashboard';
	import {
		InventoryKPICard,
		LowStockAlerts,
		InventoryActivity,
		type ActivityItem
	} from '$lib/components/inventory';
	import {
		dashboardState,
		dashboardStore,
		categoryStore,
		productStore,
		warehouseStore
	} from '$lib/stores/inventory.svelte';

	// Dashboard state
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let autoRefreshInterval: ReturnType<typeof setInterval> | null = null;

	// Mock data for demonstration - will be replaced with real API data
	let lowStockItems = $state([
		{
			productId: '1',
			sku: 'SKU-001',
			name: 'Widget Pro Max',
			currentStock: 5,
			minStock: 50,
			warehouseName: 'Main Warehouse',
			severity: 'critical' as const
		},
		{
			productId: '2',
			sku: 'SKU-002',
			name: 'Gadget Standard',
			currentStock: 15,
			minStock: 30,
			warehouseName: 'Main Warehouse',
			severity: 'warning' as const
		},
		{
			productId: '3',
			sku: 'SKU-003',
			name: 'Component A-123',
			currentStock: 45,
			minStock: 60,
			warehouseName: 'Secondary Warehouse',
			severity: 'low' as const
		}
	]);

	let recentActivities = $state([
		{
			id: '1',
			type: 'receipt' as const,
			description: 'Goods received from Supplier ABC',
			reference: 'GRN-2026-0001',
			timestamp: new Date(Date.now() - 15 * 60000).toISOString(),
			user: 'John Doe'
		},
		{
			id: '2',
			type: 'shipment' as const,
			description: 'Order #12345 shipped',
			reference: 'DO-2026-0042',
			timestamp: new Date(Date.now() - 45 * 60000).toISOString(),
			user: 'Jane Smith'
		},
		{
			id: '3',
			type: 'transfer' as const,
			description: 'Stock transferred to Branch B',
			reference: 'TRF-2026-0015',
			timestamp: new Date(Date.now() - 2 * 3600000).toISOString(),
			user: 'Mike Johnson'
		},
		{
			id: '4',
			type: 'adjustment' as const,
			description: 'Inventory adjustment - damaged goods',
			reference: 'ADJ-2026-0008',
			timestamp: new Date(Date.now() - 5 * 3600000).toISOString(),
			user: 'Sarah Wilson'
		},
		{
			id: '5',
			type: 'count' as const,
			description: 'Cycle count completed - Zone A',
			reference: 'CNT-2026-0003',
			timestamp: new Date(Date.now() - 24 * 3600000).toISOString(),
			user: 'Tom Brown'
		}
	]);

	// Sample chart data
	const stockMovementData = [
		{ label: 'Mon', value: 150 },
		{ label: 'Tue', value: 230 },
		{ label: 'Wed', value: 180 },
		{ label: 'Thu', value: 290 },
		{ label: 'Fri', value: 200 },
		{ label: 'Sat', value: 120 },
		{ label: 'Sun', value: 80 }
	];

	const categoryBreakdownData = [
		{ label: 'Electronics', value: 320 },
		{ label: 'Apparel', value: 280 },
		{ label: 'Home & Garden', value: 190 },
		{ label: 'Sports', value: 150 },
		{ label: 'Other', value: 100 }
	];

	// Computed values
	const inventoryValue = $derived(
		new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD',
			minimumFractionDigits: 0,
			maximumFractionDigits: 0
		}).format(245680)
	);

	async function loadDashboardData() {
		isLoading = true;
		error = null;

		try {
			await dashboardStore.loadSummary();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load dashboard data';
		} finally {
			isLoading = false;
		}
	}

	function startAutoRefresh() {
		// Auto-refresh every 30 seconds
		autoRefreshInterval = setInterval(() => {
			loadDashboardData();
		}, 30000);
	}

	function stopAutoRefresh() {
		if (autoRefreshInterval) {
			clearInterval(autoRefreshInterval);
			autoRefreshInterval = null;
		}
	}

	function handleLowStockClick(item: (typeof lowStockItems)[0]) {
		goto(`/inventory/products/${item.productId}`);
	}

	function handleActivityClick(activity: ActivityItem) {
		// Navigate based on activity type
		switch (activity.type) {
			case 'receipt':
				goto(`/inventory/receipts/${activity.reference}`);
				break;
			case 'shipment':
				goto(`/inventory/shipments/${activity.reference}`);
				break;
			case 'transfer':
				goto(`/inventory/transfers/${activity.reference}`);
				break;
			default:
				break;
		}
	}

	onMount(() => {
		loadDashboardData();
		startAutoRefresh();

		return () => {
			stopAutoRefresh();
		};
	});
</script>

<svelte:head>
	<title>Inventory Dashboard - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<!-- Page Header -->
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-2xl font-bold tracking-tight">Inventory Dashboard</h1>
			<p class="text-muted-foreground">Overview of your inventory health and operations</p>
		</div>
		<div class="flex gap-2">
			<Button variant="outline" onclick={loadDashboardData} disabled={isLoading}>
				{isLoading ? 'Refreshing...' : 'Refresh'}
			</Button>
			<Button href="/inventory/products/new">Add Product</Button>
		</div>
	</div>

	<!-- Error State -->
	{#if error}
		<Card class="border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-900/20">
			<CardContent class="flex items-center justify-between py-4">
				<div class="flex items-center gap-2">
					<span class="text-red-600">⚠️</span>
					<span class="text-red-600">{error}</span>
				</div>
				<Button variant="outline" size="sm" onclick={loadDashboardData}>Retry</Button>
			</CardContent>
		</Card>
	{/if}

	<!-- KPI Cards Row -->
	<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
		<InventoryKPICard
			title="Total Products"
			value={dashboardState.totalProducts.toLocaleString()}
			subtitle="Active SKUs in inventory"
			icon="📦"
			variant="info"
			href="/inventory/products"
			{isLoading}
		/>
		<InventoryKPICard
			title="Low Stock Alerts"
			value={lowStockItems.length}
			subtitle="Products below minimum"
			icon="⚠️"
			variant="warning"
			href="/inventory/alerts"
			{isLoading}
		/>
		<InventoryKPICard
			title="Categories"
			value={dashboardState.totalCategories}
			subtitle="Product categories"
			icon="📁"
			href="/inventory/categories"
			{isLoading}
		/>
		<InventoryKPICard
			title="Inventory Value"
			value={inventoryValue}
			subtitle="Total stock valuation"
			icon="💰"
			variant="success"
			trend={{ value: 5.2, isPositive: true, label: 'vs last month' }}
			href="/inventory/valuation"
			{isLoading}
		/>
	</div>

	<!-- Secondary KPI Row -->
	<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
		<InventoryKPICard
			title="Warehouses"
			value={dashboardState.totalWarehouses}
			subtitle="Active locations"
			icon="🏭"
			href="/inventory/warehouses"
			{isLoading}
		/>
		<InventoryKPICard
			title="Pending Receipts"
			value="12"
			subtitle="Awaiting processing"
			icon="📥"
			variant="info"
			href="/inventory/receipts?status=pending"
			{isLoading}
		/>
		<InventoryKPICard
			title="Pending Transfers"
			value="5"
			subtitle="In transit"
			icon="🔄"
			href="/inventory/transfers?status=in_transit"
			{isLoading}
		/>
		<InventoryKPICard
			title="Pending Counts"
			value="3"
			subtitle="Cycle counts scheduled"
			icon="📋"
			href="/inventory/counts?status=scheduled"
			{isLoading}
		/>
	</div>

	<!-- Charts Row -->
	<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
		<ChartCard title="Stock Movement (7 Days)" type="line" data={stockMovementData} />
		<ChartCard title="Inventory by Category" type="bar" data={categoryBreakdownData} />
	</div>

	<!-- Tables Row -->
	<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
		<LowStockAlerts
			items={lowStockItems}
			{isLoading}
			maxItems={5}
			onViewAll={() => goto('/inventory/alerts')}
			onItemClick={handleLowStockClick}
		/>
		<InventoryActivity
			activities={recentActivities}
			{isLoading}
			maxItems={5}
			onViewAll={() => goto('/inventory/activity')}
			onItemClick={handleActivityClick}
		/>
	</div>

	<!-- Quick Actions -->
	<Card>
		<CardHeader>
			<CardTitle class="text-sm font-medium">Quick Actions</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-6">
				<a
					href="/inventory/receipts/new"
					class="flex flex-col items-center gap-2 rounded-lg p-4 transition-colors hover:bg-muted/50"
				>
					<span class="text-2xl">📥</span>
					<span class="text-center text-sm">New Receipt</span>
				</a>
				<a
					href="/inventory/shipments/new"
					class="flex flex-col items-center gap-2 rounded-lg p-4 transition-colors hover:bg-muted/50"
				>
					<span class="text-2xl">📤</span>
					<span class="text-center text-sm">New Shipment</span>
				</a>
				<a
					href="/inventory/transfers/new"
					class="flex flex-col items-center gap-2 rounded-lg p-4 transition-colors hover:bg-muted/50"
				>
					<span class="text-2xl">🔄</span>
					<span class="text-center text-sm">New Transfer</span>
				</a>
				<a
					href="/inventory/counts/new"
					class="flex flex-col items-center gap-2 rounded-lg p-4 transition-colors hover:bg-muted/50"
				>
					<span class="text-2xl">📋</span>
					<span class="text-center text-sm">Start Count</span>
				</a>
				<a
					href="/inventory/adjustments/new"
					class="flex flex-col items-center gap-2 rounded-lg p-4 transition-colors hover:bg-muted/50"
				>
					<span class="text-2xl">📝</span>
					<span class="text-center text-sm">Adjustment</span>
				</a>
				<a
					href="/inventory/reports"
					class="flex flex-col items-center gap-2 rounded-lg p-4 transition-colors hover:bg-muted/50"
				>
					<span class="text-2xl">📊</span>
					<span class="text-center text-sm">Reports</span>
				</a>
			</div>
		</CardContent>
	</Card>
</div>
