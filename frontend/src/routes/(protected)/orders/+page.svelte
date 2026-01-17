<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { mockOrders, type Order, type OrderStatus } from '$lib/api/orders';

	// State
	let searchQuery = $state('');
	let selectedStatus = $state('');

	// Derived filtered orders
	const filteredOrders = $derived.by(() => {
		let result = [...mockOrders];

		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			result = result.filter(
				(o) =>
					o.orderNumber.toLowerCase().includes(query) ||
					o.customerName.toLowerCase().includes(query) ||
					o.customerEmail.toLowerCase().includes(query)
			);
		}

		if (selectedStatus) {
			result = result.filter((o) => o.status === selectedStatus);
		}

		return result;
	});

	function getStatusBadgeVariant(
		status: OrderStatus
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<OrderStatus, 'default' | 'secondary' | 'destructive' | 'outline'> = {
			pending: 'secondary',
			confirmed: 'outline',
			processing: 'default',
			shipped: 'default',
			delivered: 'default',
			cancelled: 'destructive',
			refunded: 'destructive'
		};
		return variants[status] || 'outline';
	}

	function formatCurrency(value: number): string {
		return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(value);
	}

	function formatDate(date: string): string {
		return new Date(date).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<svelte:head>
	<title>Orders - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Orders</h1>
			<p class="text-muted-foreground">Manage customer orders</p>
		</div>
	</div>

	<Card>
		<CardContent class="pt-6">
			<div class="flex flex-wrap gap-4">
				<div class="flex-1">
					<Input
						type="search"
						placeholder="Search orders..."
						bind:value={searchQuery}
						class="max-w-sm"
					/>
				</div>
				<select
					bind:value={selectedStatus}
					class="rounded-md border border-input bg-background px-3 py-2 text-sm"
				>
					<option value="">All Status</option>
					<option value="pending">Pending</option>
					<option value="confirmed">Confirmed</option>
					<option value="processing">Processing</option>
					<option value="shipped">Shipped</option>
					<option value="delivered">Delivered</option>
					<option value="cancelled">Cancelled</option>
				</select>
			</div>
		</CardContent>
	</Card>

	<Card>
		<CardHeader>
			<CardTitle>Order List ({filteredOrders.length})</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead>
						<tr class="border-b text-left text-sm text-muted-foreground">
							<th class="p-3">Order #</th>
							<th class="p-3">Customer</th>
							<th class="p-3">Status</th>
							<th class="p-3">Payment</th>
							<th class="p-3">Total</th>
							<th class="p-3">Date</th>
							<th class="p-3">Actions</th>
						</tr>
					</thead>
					<tbody>
						{#each filteredOrders as order}
							<tr class="border-b hover:bg-muted/50">
								<td class="p-3 font-mono text-sm">{order.orderNumber}</td>
								<td class="p-3">
									<div>
										<p class="font-medium">{order.customerName}</p>
										<p class="text-sm text-muted-foreground">{order.customerEmail}</p>
									</div>
								</td>
								<td class="p-3">
									<Badge variant={getStatusBadgeVariant(order.status)}>{order.status}</Badge>
								</td>
								<td class="p-3">
									<Badge variant={order.paymentStatus === 'paid' ? 'default' : 'secondary'}>
										{order.paymentStatus}
									</Badge>
								</td>
								<td class="p-3 font-medium">{formatCurrency(order.total)}</td>
								<td class="p-3 text-sm">{formatDate(order.createdAt)}</td>
								<td class="p-3">
									<Button variant="ghost" size="sm" href="/orders/{order.id}">View</Button>
								</td>
							</tr>
						{:else}
							<tr>
								<td colspan="7" class="p-8 text-center text-muted-foreground">No orders found</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</CardContent>
	</Card>
</div>
