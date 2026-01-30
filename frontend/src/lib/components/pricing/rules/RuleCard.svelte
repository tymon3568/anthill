<script lang="ts">
	import type { PricingRule, RuleType } from '$lib/types/pricing';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Progress } from '$lib/components/ui/progress';
	import {
		Percent,
		DollarSign,
		Tag,
		Gift,
		ShoppingCart,
		Package,
		Calendar,
		Users,
		TrendingUp,
		AlertCircle
	} from 'lucide-svelte';

	interface Props {
		rule: PricingRule;
		onClick?: () => void;
	}

	let { rule, onClick }: Props = $props();

	function getRuleTypeIcon(type: RuleType) {
		switch (type) {
			case 'discount_percentage':
				return Percent;
			case 'discount_amount':
				return DollarSign;
			case 'fixed_price':
				return Tag;
			case 'free_item':
				return Gift;
			case 'buy_x_get_y':
				return ShoppingCart;
			case 'bundle_price':
				return Package;
			default:
				return Tag;
		}
	}

	function getRuleStatus(r: PricingRule): 'active' | 'scheduled' | 'expired' | 'inactive' {
		if (!r.isActive) return 'inactive';

		const now = new Date();
		if (r.validFrom && r.validFrom > now) return 'scheduled';
		if (r.validTo && r.validTo < now) return 'expired';
		return 'active';
	}

	function getStatusBadgeVariant(
		status: string
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (status) {
			case 'active':
				return 'default';
			case 'scheduled':
				return 'secondary';
			case 'expired':
				return 'destructive';
			case 'inactive':
				return 'outline';
			default:
				return 'secondary';
		}
	}

	function formatDiscountValue(r: PricingRule): string {
		switch (r.ruleType) {
			case 'discount_percentage':
				return `${r.discountPercentage}% OFF`;
			case 'discount_amount':
				return `${formatCurrency(r.discountAmount ?? 0)} OFF`;
			case 'fixed_price':
				return formatCurrency(r.fixedPrice ?? 0);
			case 'buy_x_get_y':
				return `Buy ${r.buyQuantity} Get ${r.getQuantity}`;
			case 'free_item':
				return `${r.freeQuantity} Free`;
			case 'bundle_price':
				return 'Bundle Price';
			default:
				return '-';
		}
	}

	function formatCurrency(value: number): string {
		return new Intl.NumberFormat('vi-VN', {
			style: 'currency',
			currency: 'VND',
			maximumFractionDigits: 0
		}).format(value);
	}

	function formatDate(date: Date | undefined): string {
		if (!date) return '-';
		return new Intl.DateTimeFormat('vi-VN', {
			day: '2-digit',
			month: '2-digit',
			year: 'numeric'
		}).format(date);
	}

	function getDaysRemaining(endDate: Date | undefined): number | null {
		if (!endDate) return null;
		const now = new Date();
		const diff = endDate.getTime() - now.getTime();
		return Math.ceil(diff / (1000 * 60 * 60 * 24));
	}

	const Icon = $derived(getRuleTypeIcon(rule.ruleType));
	const status = $derived(getRuleStatus(rule));
	const usagePercent = $derived(
		rule.usageLimit ? Math.round(((rule.usageCount ?? 0) / rule.usageLimit) * 100) : 0
	);
	const daysRemaining = $derived(getDaysRemaining(rule.validTo));
</script>

<Card.Root class="cursor-pointer transition-shadow hover:shadow-md" onclick={onClick}>
	<Card.Header class="pb-3">
		<div class="flex items-start justify-between">
			<div class="flex items-center gap-3">
				<div class="rounded-lg bg-primary/10 p-2">
					<Icon class="h-5 w-5 text-primary" />
				</div>
				<div>
					<Card.Title class="text-base">{rule.name}</Card.Title>
					{#if rule.code}
						<p class="font-mono text-xs text-muted-foreground">{rule.code}</p>
					{/if}
				</div>
			</div>
			<Badge variant={getStatusBadgeVariant(status)}>
				{status.charAt(0).toUpperCase() + status.slice(1)}
			</Badge>
		</div>
	</Card.Header>
	<Card.Content class="space-y-4">
		<!-- Discount Value -->
		<div class="text-2xl font-bold text-primary">
			{formatDiscountValue(rule)}
		</div>

		<!-- Validity Period -->
		{#if rule.validFrom || rule.validTo}
			<div class="flex items-center gap-2 text-sm">
				<Calendar class="h-4 w-4 text-muted-foreground" />
				<span class="text-muted-foreground">
					{formatDate(rule.validFrom)} - {formatDate(rule.validTo)}
				</span>
			</div>
		{/if}

		<!-- Days Remaining Warning -->
		{#if daysRemaining !== null && daysRemaining > 0 && daysRemaining <= 7}
			<div class="flex items-center gap-2 text-sm text-amber-600">
				<AlertCircle class="h-4 w-4" />
				<span>{daysRemaining} days remaining</span>
			</div>
		{/if}

		<!-- Usage Stats -->
		{#if rule.usageLimit}
			<div class="space-y-2">
				<div class="flex items-center justify-between text-sm">
					<span class="text-muted-foreground">Usage</span>
					<span class="font-medium">{rule.usageCount ?? 0} / {rule.usageLimit}</span>
				</div>
				<Progress value={usagePercent} class="h-2" />
			</div>
		{:else if rule.usageCount}
			<div class="flex items-center gap-2 text-sm">
				<TrendingUp class="h-4 w-4 text-muted-foreground" />
				<span class="text-muted-foreground">Used {rule.usageCount} times</span>
			</div>
		{/if}

		<!-- Per Customer Limit -->
		{#if rule.perCustomerLimit}
			<div class="flex items-center gap-2 text-sm">
				<Users class="h-4 w-4 text-muted-foreground" />
				<span class="text-muted-foreground">
					Max {rule.perCustomerLimit} per customer
				</span>
			</div>
		{/if}
	</Card.Content>
</Card.Root>
