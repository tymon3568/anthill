<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';

	interface LowStockItem {
		productId: string;
		sku: string;
		name: string;
		currentStock: number;
		minStock: number;
		warehouseName: string;
		severity: 'critical' | 'warning' | 'low';
	}

	interface Props {
		items: LowStockItem[];
		isLoading?: boolean;
		maxItems?: number;
		onViewAll?: () => void;
		onItemClick?: (item: LowStockItem) => void;
	}

	let { items = [], isLoading = false, maxItems = 5, onViewAll, onItemClick }: Props = $props();

	const displayItems = $derived(items.slice(0, maxItems));

	function getSeverityBadge(severity: LowStockItem['severity']) {
		switch (severity) {
			case 'critical':
				return { variant: 'destructive' as const, label: 'Critical' };
			case 'warning':
				return { variant: 'secondary' as const, label: 'Warning' };
			case 'low':
				return { variant: 'outline' as const, label: 'Low' };
		}
	}

	function getStockPercentage(current: number, min: number): number {
		if (min === 0) return 100;
		return Math.round((current / min) * 100);
	}
</script>

<Card>
	<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
		<CardTitle class="text-sm font-medium">Low Stock Alerts</CardTitle>
		{#if items.length > maxItems && onViewAll}
			<Button variant="ghost" size="sm" onclick={onViewAll}>View All ({items.length})</Button>
		{/if}
	</CardHeader>
	<CardContent>
		{#if isLoading}
			<div class="space-y-3">
				{#each Array(3) as _}
					<div class="flex animate-pulse items-center justify-between">
						<div class="space-y-2">
							<div class="h-4 w-32 rounded bg-muted"></div>
							<div class="h-3 w-24 rounded bg-muted"></div>
						</div>
						<div class="h-5 w-16 rounded bg-muted"></div>
					</div>
				{/each}
			</div>
		{:else if displayItems.length === 0}
			<div class="py-6 text-center text-muted-foreground">
				<p>No low stock alerts</p>
				<p class="text-sm">All products are well stocked</p>
			</div>
		{:else}
			<div class="space-y-3">
				{#each displayItems as item (item.productId)}
					{@const badge = getSeverityBadge(item.severity)}
					{@const percentage = getStockPercentage(item.currentStock, item.minStock)}
					<button
						type="button"
						class="flex w-full cursor-pointer items-center justify-between rounded-lg p-2 text-left transition-colors hover:bg-muted/50"
						onclick={() => onItemClick?.(item)}
					>
						<div class="min-w-0 flex-1">
							<p class="truncate font-medium">{item.name}</p>
							<p class="text-sm text-muted-foreground">
								SKU: {item.sku} | {item.warehouseName}
							</p>
							<div class="mt-1 flex items-center gap-2">
								<div class="h-1.5 w-20 rounded-full bg-muted">
									<div
										class="h-1.5 rounded-full transition-all"
										class:bg-red-500={percentage < 25}
										class:bg-yellow-500={percentage >= 25 && percentage < 50}
										class:bg-green-500={percentage >= 50}
										style="width: {Math.min(percentage, 100)}%"
									></div>
								</div>
								<span class="text-xs text-muted-foreground">
									{item.currentStock} / {item.minStock}
								</span>
							</div>
						</div>
						<Badge variant={badge.variant}>{badge.label}</Badge>
					</button>
				{/each}
			</div>
		{/if}
	</CardContent>
</Card>
