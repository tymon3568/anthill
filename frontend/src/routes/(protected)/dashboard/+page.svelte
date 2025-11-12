<script lang="ts">
	import { authState, authStore } from '$lib/stores/auth.svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { goto } from '$app/navigation';

	async function handleLogout() {
		await authStore.emailLogout();
		goto('/login');
	}
</script>

<svelte:head>
	<title>Dashboard - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold">Dashboard</h1>
			<p class="text-muted-foreground">Welcome back, {authState.user?.name || 'User'}!</p>
		</div>
		<Button variant="outline" onclick={handleLogout}>Logout</Button>
	</div>

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

	<!-- Stats Cards - Placeholder -->
	<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
		<Card>
			<CardHeader>
				<CardTitle>Total Products</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-3xl font-bold">0</div>
				<p class="text-sm text-muted-foreground">Active products in inventory</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>Low Stock Alerts</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-3xl font-bold text-destructive">0</div>
				<p class="text-sm text-muted-foreground">Products below minimum stock</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>Categories</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-3xl font-bold">0</div>
				<p class="text-sm text-muted-foreground">Product categories</p>
			</CardContent>
		</Card>
	</div>

	<!-- Coming Soon Notice -->
	<Card>
		<CardContent class="pt-6">
			<div class="text-center text-muted-foreground">
				<p class="mb-2 text-lg font-semibold">Inventory Features Coming Soon</p>
				<p>
					Product management, stock tracking, and reporting features will be available once the
					backend services are ready.
				</p>
			</div>
		</CardContent>
	</Card>
</div>
