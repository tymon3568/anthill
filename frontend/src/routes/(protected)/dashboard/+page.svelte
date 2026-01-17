<script lang="ts">
	import { authState } from '$lib/stores/auth.svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { MetricsCard, ChartCard, ActivityFeed } from '$lib/components/dashboard';

	// Mock data for demonstration - in production, this would come from API
	const metrics = $state({
		totalProducts: 1248,
		lowStockAlerts: 23,
		categories: 45,
		totalOrders: 892,
		inventoryValue: 124500,
		pendingShipments: 67
	});

	// Sample chart data
	const salesTrendData = [
		{ label: 'Jan', value: 4200 },
		{ label: 'Feb', value: 3800 },
		{ label: 'Mar', value: 5100 },
		{ label: 'Apr', value: 4600 },
		{ label: 'May', value: 5800 },
		{ label: 'Jun', value: 6200 }
	];

	const inventoryLevelData = [
		{ label: 'Electronics', value: 320 },
		{ label: 'Clothing', value: 450 },
		{ label: 'Home', value: 280 },
		{ label: 'Sports', value: 180 },
		{ label: 'Books', value: 120 }
	];

	// Sample activity data
	const recentActivities = [
		{
			id: '1',
			type: 'order' as const,
			message: 'New order #12345 received',
			timestamp: '2 minutes ago',
			status: 'success' as const
		},
		{
			id: '2',
			type: 'inventory' as const,
			message: 'Stock updated for SKU-789',
			timestamp: '15 minutes ago',
			status: 'info' as const
		},
		{
			id: '3',
			type: 'alert' as const,
			message: 'Low stock warning: Widget Pro',
			timestamp: '1 hour ago',
			status: 'warning' as const
		},
		{
			id: '4',
			type: 'order' as const,
			message: 'Order #12340 shipped',
			timestamp: '2 hours ago',
			status: 'success' as const
		},
		{
			id: '5',
			type: 'user' as const,
			message: 'New team member added',
			timestamp: '3 hours ago',
			status: 'info' as const
		}
	];

	// Format currency
	function formatCurrency(value: number): string {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD',
			minimumFractionDigits: 0
		}).format(value);
	}
</script>

<svelte:head>
	<title>Dashboard - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<!-- Welcome Card -->
	<Card>
		<CardHeader>
			<CardTitle>Welcome to Anthill Inventory Management</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="space-y-4">
				<p class="text-muted-foreground">Your inventory management system is ready to use.</p>
				{#if authState.user}
					<div class="space-y-2">
						<p><strong>Email:</strong> {authState.user.email}</p>
						<p><strong>Role:</strong> <Badge>{authState.user.role}</Badge></p>
						{#if authState.tenant}
							<p><strong>Organization:</strong> {authState.tenant.name}</p>
						{/if}
					</div>
				{/if}
			</div>
		</CardContent>
	</Card>

	<!-- Key Metrics -->
	<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
		<MetricsCard
			title="Total Products"
			value={metrics.totalProducts.toLocaleString()}
			description="Active products in inventory"
			trend={{ value: 12, isPositive: true }}
		/>
		<MetricsCard
			title="Low Stock Alerts"
			value={metrics.lowStockAlerts}
			description="Products below minimum"
			variant="danger"
		/>
		<MetricsCard title="Categories" value={metrics.categories} description="Product categories" />
		<MetricsCard
			title="Total Orders"
			value={metrics.totalOrders.toLocaleString()}
			description="This month"
			trend={{ value: 8, isPositive: true }}
		/>
		<MetricsCard
			title="Inventory Value"
			value={formatCurrency(metrics.inventoryValue)}
			description="Total stock value"
			trend={{ value: 5, isPositive: true }}
			variant="success"
		/>
		<MetricsCard
			title="Pending Shipments"
			value={metrics.pendingShipments}
			description="Awaiting dispatch"
			variant="warning"
		/>
	</div>

	<!-- Charts Section -->
	<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
		<ChartCard title="Sales Trend (6 Months)" type="line" data={salesTrendData} />
		<ChartCard title="Inventory by Category" type="bar" data={inventoryLevelData} />
	</div>

	<!-- Activity Feed and Quick Actions -->
	<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
		<div class="lg:col-span-2">
			<ActivityFeed title="Recent Activity" activities={recentActivities} maxItems={5} />
		</div>
		<Card>
			<CardHeader>
				<CardTitle class="text-sm font-medium">Quick Actions</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="space-y-2">
					<a href="/products/new" class="flex items-center gap-2 rounded-lg p-3 hover:bg-muted/50">
						<span>➕</span>
						<span class="text-sm">Add New Product</span>
					</a>
					<a href="/orders" class="flex items-center gap-2 rounded-lg p-3 hover:bg-muted/50">
						<span>📋</span>
						<span class="text-sm">View Orders</span>
					</a>
					<a href="/inventory" class="flex items-center gap-2 rounded-lg p-3 hover:bg-muted/50">
						<span>📊</span>
						<span class="text-sm">Stock Report</span>
					</a>
					<a href="/settings" class="flex items-center gap-2 rounded-lg p-3 hover:bg-muted/50">
						<span>⚙️</span>
						<span class="text-sm">Settings</span>
					</a>
				</div>
			</CardContent>
		</Card>
	</div>
</div>
