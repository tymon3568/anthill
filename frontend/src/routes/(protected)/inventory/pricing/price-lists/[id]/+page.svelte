<script lang="ts">
	import { page } from '$app/state';
	import {
		Card,
		CardContent,
		CardHeader,
		CardTitle,
		CardDescription
	} from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import * as Tabs from '$lib/components/ui/tabs';
	import {
		mockPriceLists,
		type PriceList,
		type PriceListItem,
		type CustomerPriceList,
		type ApplyTo,
		type ComputeMethod
	} from '$lib/types/pricing';

	// Get price list ID from URL (using $app/state for Svelte 5)
	const priceListId = $derived(page.params.id ?? '');

	// Find the price list (in real app, this would be an API call)
	const priceList = $derived(mockPriceLists.find((pl) => pl.priceListId === priceListId));

	// Mock items data - using $derived.by to ensure reactivity with priceListId
	const mockItems = $derived.by((): PriceListItem[] => {
		const id = priceListId;
		return [
			{
				itemId: 'item-001',
				tenantId: 'tenant-001',
				priceListId: id,
				applyTo: 'product',
				productId: 'prod-001',
				minQuantity: 1,
				computeMethod: 'percentage',
				percentage: -15,
				roundingMethod: 'round_nearest',
				roundingPrecision: 0,
				createdAt: new Date(),
				updatedAt: new Date(),
				product: { name: 'Laptop Pro 15"', sku: 'LAPTOP-PRO-15', salePrice: 2500000000 }
			},
			{
				itemId: 'item-002',
				tenantId: 'tenant-001',
				priceListId: id,
				applyTo: 'product',
				productId: 'prod-001',
				minQuantity: 10,
				computeMethod: 'percentage',
				percentage: -20,
				roundingMethod: 'round_nearest',
				roundingPrecision: 0,
				createdAt: new Date(),
				updatedAt: new Date(),
				product: { name: 'Laptop Pro 15"', sku: 'LAPTOP-PRO-15', salePrice: 2500000000 }
			},
			{
				itemId: 'item-003',
				tenantId: 'tenant-001',
				priceListId: id,
				applyTo: 'product',
				productId: 'prod-002',
				minQuantity: 1,
				computeMethod: 'fixed',
				fixedPrice: 40000000,
				roundingMethod: 'none',
				roundingPrecision: 0,
				createdAt: new Date(),
				updatedAt: new Date(),
				product: { name: 'Wireless Mouse', sku: 'MOUSE-WL-01', salePrice: 50000000 }
			},
			{
				itemId: 'item-004',
				tenantId: 'tenant-001',
				priceListId: id,
				applyTo: 'category',
				categoryId: 'cat-electronics',
				minQuantity: 1,
				computeMethod: 'percentage',
				percentage: -10,
				roundingMethod: 'round_nearest',
				roundingPrecision: 0,
				createdAt: new Date(),
				updatedAt: new Date(),
				category: { name: 'Electronics' }
			}
		];
	});

	// Mock customers data - using $derived.by to ensure reactivity with priceListId
	const mockCustomers = $derived.by((): CustomerPriceList[] => {
		const id = priceListId;
		return [
			{
				id: 'cpl-001',
				tenantId: 'tenant-001',
				customerId: 'cust-001',
				priceListId: id,
				priority: 0,
				createdAt: new Date(),
				customer: { name: 'ABC Corporation', email: 'abc@example.com' }
			},
			{
				id: 'cpl-002',
				tenantId: 'tenant-001',
				customerId: 'cust-002',
				priceListId: id,
				priority: 0,
				validFrom: new Date('2026-01-01'),
				validTo: new Date('2026-12-31'),
				createdAt: new Date(),
				customer: { name: 'XYZ Trading', email: 'xyz@example.com' }
			}
		];
	});

	// State
	let activeTab = $state('items');
	let itemSearch = $state('');
	let customerSearch = $state('');

	// Filtered items
	const filteredItems = $derived.by(() => {
		if (!itemSearch) return mockItems;
		const query = itemSearch.toLowerCase();
		return mockItems.filter(
			(item) =>
				item.product?.name.toLowerCase().includes(query) ||
				item.product?.sku.toLowerCase().includes(query) ||
				item.category?.name.toLowerCase().includes(query)
		);
	});

	// Filtered customers
	const filteredCustomers = $derived.by(() => {
		if (!customerSearch) return mockCustomers;
		const query = customerSearch.toLowerCase();
		return mockCustomers.filter(
			(cpl) =>
				cpl.customer?.name.toLowerCase().includes(query) ||
				cpl.customer?.email.toLowerCase().includes(query)
		);
	});

	// Helpers
	function formatCurrency(cents: number, currency: string = 'VND'): string {
		const value = cents / 100;
		return new Intl.NumberFormat('vi-VN', {
			style: 'currency',
			currency: currency,
			minimumFractionDigits: 0
		}).format(value);
	}

	function formatValidity(pl: PriceList): string {
		if (!pl.validFrom && !pl.validTo) return 'Always valid';
		const from = pl.validFrom ? pl.validFrom.toLocaleDateString('vi-VN') : '';
		const to = pl.validTo ? pl.validTo.toLocaleDateString('vi-VN') : '';
		if (from && to) return `${from} - ${to}`;
		if (from) return `From ${from}`;
		if (to) return `Until ${to}`;
		return 'Always valid';
	}

	function getApplyToLabel(applyTo: ApplyTo): string {
		switch (applyTo) {
			case 'product':
				return 'Product';
			case 'variant':
				return 'Variant';
			case 'category':
				return 'Category';
			case 'all':
				return 'All Products';
		}
	}

	function getMethodLabel(method: ComputeMethod): string {
		switch (method) {
			case 'fixed':
				return 'Fixed';
			case 'percentage':
				return 'Percentage';
			case 'formula':
				return 'Formula';
			case 'margin':
				return 'Margin';
		}
	}

	function formatItemValue(item: PriceListItem): string {
		switch (item.computeMethod) {
			case 'fixed':
				return formatCurrency(item.fixedPrice ?? 0);
			case 'percentage':
				return `${item.percentage ?? 0}%`;
			case 'margin':
				return `${item.marginPercentage ?? 0}% margin`;
			default:
				return '-';
		}
	}

	function handleDelete() {
		if (confirm('Are you sure you want to delete this price list?')) {
			console.log('Deleting price list:', priceListId);
			// TODO: Call API and redirect
		}
	}

	function handleSetDefault() {
		console.log('Setting as default:', priceListId);
		// TODO: Call API
	}
</script>

<svelte:head>
	<title>{priceList?.name ?? 'Price List'} - Anthill</title>
</svelte:head>

{#if priceList}
	<div class="space-y-6">
		<!-- Header -->
		<div class="flex items-start justify-between">
			<div>
				<div class="mb-2 flex items-center gap-2 text-sm text-muted-foreground">
					<a href="/inventory/pricing/price-lists" class="hover:underline">Price Lists</a>
					<span>/</span>
					<span>{priceList.code}</span>
				</div>
				<h1 class="flex items-center gap-3 text-2xl font-bold">
					{priceList.name}
					{#if priceList.isDefault}
						<Badge variant="outline">Default</Badge>
					{/if}
					<Badge variant={priceList.isActive ? 'default' : 'secondary'}>
						{priceList.isActive ? 'Active' : 'Inactive'}
					</Badge>
				</h1>
				<p class="mt-1 text-muted-foreground">
					{priceList.code} • {priceList.priceListType === 'sale' ? 'Sale' : 'Purchase'} • {priceList.currencyCode}
					{#if priceList.basedOn === 'base_price' && priceList.defaultPercentage !== 0}
						• {priceList.defaultPercentage > 0 ? '+' : ''}{priceList.defaultPercentage}% from base
					{/if}
				</p>
			</div>
			<div class="flex gap-2">
				{#if !priceList.isDefault}
					<Button variant="outline" onclick={handleSetDefault}>Set as Default</Button>
				{/if}
				<Button variant="outline" href="/inventory/pricing/price-lists/{priceListId}/edit">
					Edit
				</Button>
				<Button variant="destructive" onclick={handleDelete}>Delete</Button>
			</div>
		</div>

		<!-- Summary Cards -->
		<div class="grid grid-cols-4 gap-4">
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold">{priceList.itemCount ?? 0}</div>
					<p class="text-sm text-muted-foreground">Price Items</p>
				</CardContent>
			</Card>
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold">{priceList.customerCount ?? 0}</div>
					<p class="text-sm text-muted-foreground">Assigned Customers</p>
				</CardContent>
			</Card>
			<Card>
				<CardContent class="pt-6">
					<div class="text-2xl font-bold">{priceList.priority}</div>
					<p class="text-sm text-muted-foreground">Priority</p>
				</CardContent>
			</Card>
			<Card>
				<CardContent class="pt-6">
					<div class="text-sm font-medium">{formatValidity(priceList)}</div>
					<p class="text-sm text-muted-foreground">Validity</p>
				</CardContent>
			</Card>
		</div>

		<!-- Tabs -->
		<Tabs.Root bind:value={activeTab}>
			<Tabs.List>
				<Tabs.Trigger value="items">Items ({mockItems.length})</Tabs.Trigger>
				<Tabs.Trigger value="customers">Customers ({mockCustomers.length})</Tabs.Trigger>
			</Tabs.List>

			<!-- Items Tab -->
			<Tabs.Content value="items">
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle>Price List Items</CardTitle>
							<Button>Add Item</Button>
						</div>
						<div class="mt-4">
							<Input
								type="search"
								placeholder="Search products..."
								bind:value={itemSearch}
								class="max-w-sm"
							/>
						</div>
					</CardHeader>
					<CardContent>
						<div class="overflow-x-auto">
							<table class="w-full">
								<thead>
									<tr class="border-b text-left text-sm text-muted-foreground">
										<th class="p-3">Product/Category</th>
										<th class="p-3">Apply To</th>
										<th class="p-3 text-right">Min Qty</th>
										<th class="p-3">Method</th>
										<th class="p-3 text-right">Value</th>
										<th class="w-24 p-3">Actions</th>
									</tr>
								</thead>
								<tbody>
									{#each filteredItems as item (item.itemId)}
										<tr class="border-b hover:bg-muted/50">
											<td class="p-3">
												{#if item.applyTo === 'product' || item.applyTo === 'variant'}
													<div>
														<span class="font-medium">{item.product?.name}</span>
														<span class="ml-2 font-mono text-sm text-muted-foreground">
															{item.product?.sku}
														</span>
													</div>
												{:else if item.applyTo === 'category'}
													<div class="flex items-center gap-2">
														<span class="text-muted-foreground">📁</span>
														<span class="font-medium">{item.category?.name}</span>
														<span class="text-xs text-muted-foreground">(Category)</span>
													</div>
												{:else}
													<span class="text-muted-foreground">All Products</span>
												{/if}
											</td>
											<td class="p-3">
												<Badge variant="outline" class="text-xs">
													{getApplyToLabel(item.applyTo)}
												</Badge>
											</td>
											<td class="p-3 text-right">{item.minQuantity}</td>
											<td class="p-3">{getMethodLabel(item.computeMethod)}</td>
											<td class="p-3 text-right font-medium">
												{formatItemValue(item)}
											</td>
											<td class="p-3">
												<div class="flex gap-1">
													<Button variant="ghost" size="sm">Edit</Button>
													<Button variant="ghost" size="sm" class="text-destructive">Delete</Button>
												</div>
											</td>
										</tr>
									{:else}
										<tr>
											<td colspan="6" class="p-8 text-center text-muted-foreground">
												{#if itemSearch}
													No items match your search.
												{:else}
													No items in this price list yet.
													<Button variant="link" class="ml-1">Add your first item</Button>
												{/if}
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					</CardContent>
				</Card>
			</Tabs.Content>

			<!-- Customers Tab -->
			<Tabs.Content value="customers">
				<Card>
					<CardHeader>
						<div class="flex items-center justify-between">
							<CardTitle>Assigned Customers</CardTitle>
							<Button>Assign Customer</Button>
						</div>
						<div class="mt-4">
							<Input
								type="search"
								placeholder="Search customers..."
								bind:value={customerSearch}
								class="max-w-sm"
							/>
						</div>
					</CardHeader>
					<CardContent>
						<div class="overflow-x-auto">
							<table class="w-full">
								<thead>
									<tr class="border-b text-left text-sm text-muted-foreground">
										<th class="p-3">Customer</th>
										<th class="p-3">Priority</th>
										<th class="p-3">Valid From</th>
										<th class="p-3">Valid To</th>
										<th class="w-24 p-3">Actions</th>
									</tr>
								</thead>
								<tbody>
									{#each filteredCustomers as cpl (cpl.id)}
										<tr class="border-b hover:bg-muted/50">
											<td class="p-3">
												<div>
													<span class="font-medium">{cpl.customer?.name}</span>
													<p class="text-sm text-muted-foreground">{cpl.customer?.email}</p>
												</div>
											</td>
											<td class="p-3">{cpl.priority}</td>
											<td class="p-3">
												{cpl.validFrom ? cpl.validFrom.toLocaleDateString('vi-VN') : '-'}
											</td>
											<td class="p-3">
												{cpl.validTo ? cpl.validTo.toLocaleDateString('vi-VN') : '-'}
											</td>
											<td class="p-3">
												<div class="flex gap-1">
													<Button variant="ghost" size="sm">Edit</Button>
													<Button variant="ghost" size="sm" class="text-destructive">Remove</Button>
												</div>
											</td>
										</tr>
									{:else}
										<tr>
											<td colspan="5" class="p-8 text-center text-muted-foreground">
												{#if customerSearch}
													No customers match your search.
												{:else}
													No customers assigned to this price list.
													<Button variant="link" class="ml-1">Assign a customer</Button>
												{/if}
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					</CardContent>
				</Card>
			</Tabs.Content>
		</Tabs.Root>
	</div>
{:else}
	<div class="flex flex-col items-center justify-center py-12">
		<h1 class="text-2xl font-bold">Price List Not Found</h1>
		<p class="mt-2 text-muted-foreground">The price list you're looking for doesn't exist.</p>
		<Button href="/inventory/pricing/price-lists" class="mt-4">Back to Price Lists</Button>
	</div>
{/if}
