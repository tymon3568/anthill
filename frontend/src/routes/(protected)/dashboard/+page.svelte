<script lang="ts">
	import { goto } from '$app/navigation';
	import { authState } from '$lib/stores/auth.svelte';
	import { inventoryState, inventoryStore } from '$lib/stores/inventory.svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import type { Product } from '$lib/types';

	// Load initial data
	import { onMount } from 'svelte';
	import { inventoryApi } from '$lib/api/inventory';

	// Client-side auth check
	$: if (!authState.isLoading && !authState.isAuthenticated) {
		goto('/login');
	}

	onMount(async () => {
		inventoryStore.setLoading(true);
		try {
			const [productsResult, categoriesResult] = await Promise.all([
				inventoryApi.getProducts(1, 100),
				inventoryApi.getCategories()
			]);

			if (productsResult.success && productsResult.data) {
				inventoryStore.setProducts(productsResult.data.data);
			}

			if (categoriesResult.success && categoriesResult.data) {
				inventoryStore.setCategories(categoriesResult.data);
			}
		} catch {
			inventoryStore.setError('Failed to load inventory data');
		} finally {
			inventoryStore.setLoading(false);
		}
	});
</script>

<div class="space-y-6">
	<div>
		<h1 class="text-3xl font-bold">Dashboard</h1>
		<p class="text-muted-foreground">Overview of your inventory management system</p>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
		<Card>
			<CardHeader>
				<CardTitle>Total Products</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-3xl font-bold">{inventoryState.products?.length || 0}</div>
				<p class="text-sm text-muted-foreground">Active products in inventory</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>Low Stock Alerts</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-3xl font-bold text-destructive">
					{inventoryState.products?.filter((p: Product) => p.stock <= p.minStock).length || 0}
				</div>
				<p class="text-sm text-muted-foreground">Products below minimum stock</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>Total Value</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="text-3xl font-bold">
					${(inventoryState.products?.reduce(
						(sum: number, p: Product) => sum + p.price * p.stock,
						0
					) || 0) / 100}
				</div>
				<p class="text-sm text-muted-foreground">Current inventory value</p>
			</CardContent>
		</Card>
	</div>

	<!-- Recent Products -->
	<Card>
		<CardHeader>
			<CardTitle>Recent Products</CardTitle>
		</CardHeader>
		<CardContent>
			{#if inventoryState.isLoading}
				<div class="flex items-center justify-center py-8">
					<div class="h-6 w-6 animate-spin rounded-full border-b-2 border-primary"></div>
				</div>
			{:else if (inventoryState.products?.length || 0) === 0}
				<p class="py-8 text-center text-muted-foreground">No products found</p>
			{:else}
				<div class="space-y-4">
					{#each inventoryState.products?.slice(0, 5) || [] as product (product.id)}
						<div class="flex items-center justify-between rounded-lg border p-4">
							<div>
								<h3 class="font-medium">{product.name}</h3>
								<p class="text-sm text-muted-foreground">SKU: {product.sku}</p>
							</div>
							<div class="flex items-center gap-2">
								<Badge variant={product.stock <= product.minStock ? 'destructive' : 'secondary'}>
									{product.stock} in stock
								</Badge>
								<span class="font-medium">${(product.price / 100).toFixed(2)}</span>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</CardContent>
	</Card>
</div>
