<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import type { PriceResult, QuantityBreak } from '$lib/types/pricing';
	import { TrendingDown, TrendingUp, Minus, Info } from 'lucide-svelte';

	interface Props {
		basePrice: number;
		finalPrice?: number;
		priceResult?: PriceResult | null;
		quantityBreaks?: QuantityBreak[];
		currencyCode?: string;
		showBreakdown?: boolean;
	}

	let {
		basePrice,
		finalPrice,
		priceResult = null,
		quantityBreaks = [],
		currencyCode = 'VND',
		showBreakdown = true
	}: Props = $props();

	const displayPrice = $derived(priceResult?.finalPrice ?? finalPrice ?? basePrice);
	const discount = $derived(basePrice - displayPrice);
	const discountPercentage = $derived(basePrice > 0 ? (discount / basePrice) * 100 : 0);

	function formatCurrency(value: number): string {
		return new Intl.NumberFormat('vi-VN', {
			style: 'currency',
			currency: currencyCode,
			maximumFractionDigits: 0
		}).format(value);
	}

	function formatPercentage(value: number): string {
		return `${value >= 0 ? '+' : ''}${value.toFixed(1)}%`;
	}
</script>

<Card.Root>
	<Card.Header class="pb-2">
		<Card.Title class="text-base">Price Preview</Card.Title>
	</Card.Header>
	<Card.Content class="space-y-4">
		<!-- Main Price Display -->
		<div class="text-center">
			<div class="text-3xl font-bold">
				{formatCurrency(displayPrice)}
			</div>
			{#if discount !== 0}
				<div class="mt-1 flex items-center justify-center gap-2">
					<span class="text-sm text-muted-foreground line-through">
						{formatCurrency(basePrice)}
					</span>
					{#if discount > 0}
						<Badge variant="default" class="bg-green-600">
							<TrendingDown class="mr-1 h-3 w-3" />
							Save {formatCurrency(discount)}
						</Badge>
					{:else}
						<Badge variant="destructive">
							<TrendingUp class="mr-1 h-3 w-3" />
							+{formatCurrency(Math.abs(discount))}
						</Badge>
					{/if}
				</div>
			{/if}
		</div>

		<!-- Price Breakdown -->
		{#if showBreakdown && priceResult}
			<div class="space-y-2 border-t pt-4">
				<div class="flex justify-between text-sm">
					<span class="text-muted-foreground">Base Price</span>
					<span>{formatCurrency(priceResult.basePrice)}</span>
				</div>

				{#if priceResult.priceListUsed}
					<div class="flex justify-between text-sm">
						<span class="text-muted-foreground">
							{priceResult.priceListUsed.name}
						</span>
						<span class="text-green-600">
							-{formatCurrency(priceResult.basePrice - priceResult.listPrice)}
						</span>
					</div>
				{/if}

				{#each priceResult.discounts as discountItem (discountItem.id)}
					<div class="flex justify-between text-sm">
						<span class="text-muted-foreground">
							{discountItem.name}
							{#if discountItem.percentage}
								({discountItem.percentage}%)
							{/if}
						</span>
						<span class="text-green-600">
							-{formatCurrency(discountItem.amount)}
						</span>
					</div>
				{/each}

				<div class="flex justify-between border-t pt-2 font-medium">
					<span>Final Price</span>
					<span>{formatCurrency(priceResult.finalPrice)}</span>
				</div>

				{#if priceResult.marginPercentage !== undefined}
					<div class="flex items-center justify-between text-xs text-muted-foreground">
						<span class="flex items-center gap-1">
							<Info class="h-3 w-3" />
							Margin
						</span>
						<span>{priceResult.marginPercentage.toFixed(1)}%</span>
					</div>
				{/if}
			</div>
		{/if}

		<!-- Quantity Breaks -->
		{#if quantityBreaks.length > 0}
			<div class="space-y-2 border-t pt-4">
				<div class="flex items-center gap-2 text-sm font-medium">
					<Minus class="h-4 w-4" />
					Quantity Discounts
				</div>
				<div class="space-y-1">
					{#each quantityBreaks as qb, index (index)}
						<div class="flex justify-between text-sm">
							<span class="text-muted-foreground">
								{qb.minQty}{qb.maxQty ? `-${qb.maxQty}` : '+'} units
							</span>
							<span
								class={qb.discountPercentage && qb.discountPercentage > 0 ? 'text-green-600' : ''}
							>
								{formatCurrency(qb.unitPrice)}
								{#if qb.discountPercentage}
									<span class="text-xs">(-{qb.discountPercentage}%)</span>
								{/if}
							</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Savings Summary -->
		{#if priceResult && priceResult.totalSavings > 0}
			<div class="rounded-lg bg-green-50 p-3 dark:bg-green-950">
				<div class="flex items-center justify-between">
					<span class="text-sm font-medium text-green-700 dark:text-green-300">
						Total Savings
					</span>
					<span class="font-bold text-green-700 dark:text-green-300">
						{formatCurrency(priceResult.totalSavings)}
						<span class="text-xs font-normal">
							({priceResult.savingsPercentage.toFixed(1)}% off)
						</span>
					</span>
				</div>
			</div>
		{/if}
	</Card.Content>
</Card.Root>
