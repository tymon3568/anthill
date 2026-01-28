<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { mockPricingRules, type PricingRule, type RuleType } from '$lib/types/pricing';
	import {
		ArrowLeft,
		Pencil,
		Trash2,
		ToggleLeft,
		ToggleRight,
		Copy,
		Percent,
		DollarSign,
		Tag,
		Gift,
		ShoppingCart,
		Package,
		Calendar,
		Users,
		TrendingUp
	} from 'lucide-svelte';

	// Get rule ID from URL
	const ruleId = $derived(page.params.id);

	// Find the rule
	const rule = $derived(mockPricingRules.find((r) => r.ruleId === ruleId));

	function getRuleStatus(r: PricingRule): {
		label: string;
		variant: 'default' | 'secondary' | 'destructive' | 'outline';
	} {
		if (!r.isActive) return { label: 'Inactive', variant: 'secondary' };

		const now = new Date();
		const from = r.validFrom ? new Date(r.validFrom) : null;
		const to = r.validTo ? new Date(r.validTo) : null;

		if (from && from > now) return { label: 'Scheduled', variant: 'outline' };
		if (to && to < now) return { label: 'Expired', variant: 'destructive' };
		return { label: 'Active', variant: 'default' };
	}

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

	function getRuleTypeLabel(type: RuleType): string {
		switch (type) {
			case 'discount_percentage':
				return 'Percentage Discount';
			case 'discount_amount':
				return 'Amount Discount';
			case 'fixed_price':
				return 'Fixed Price';
			case 'free_item':
				return 'Free Item';
			case 'buy_x_get_y':
				return 'Buy X Get Y';
			case 'bundle_price':
				return 'Bundle Price';
			default:
				return type;
		}
	}

	function formatDate(date?: Date): string {
		if (!date) return '-';
		return new Date(date).toLocaleDateString('vi-VN', {
			day: '2-digit',
			month: 'short',
			year: 'numeric'
		});
	}

	function formatCurrency(value: number): string {
		return new Intl.NumberFormat('vi-VN', {
			style: 'currency',
			currency: 'VND',
			maximumFractionDigits: 0
		}).format(value);
	}

	function getDiscountDisplay(r: PricingRule): string {
		if (r.discountPercentage) {
			let display = `${r.discountPercentage}%`;
			if (r.maxDiscountAmount) {
				display += ` (max ${formatCurrency(r.maxDiscountAmount)})`;
			}
			return display;
		}
		if (r.discountAmount) {
			return formatCurrency(r.discountAmount);
		}
		if (r.fixedPrice !== undefined) {
			return formatCurrency(r.fixedPrice);
		}
		if (r.ruleType === 'buy_x_get_y') {
			return `Buy ${r.buyQuantity}, Get ${r.getQuantity} Free`;
		}
		if (r.ruleType === 'free_item') {
			return `${r.freeQuantity} free item(s)`;
		}
		return '-';
	}

	function getConditionsList(r: PricingRule): { label: string; value: string }[] {
		const conditions: { label: string; value: string }[] = [];
		const c = r.conditions;

		if (c.minQuantity)
			conditions.push({ label: 'Minimum Quantity', value: `${c.minQuantity} items` });
		if (c.minOrderAmount)
			conditions.push({ label: 'Minimum Order', value: formatCurrency(c.minOrderAmount) });
		if (c.maxOrderAmount)
			conditions.push({ label: 'Maximum Order', value: formatCurrency(c.maxOrderAmount) });
		if (c.categories?.length)
			conditions.push({ label: 'Categories', value: `${c.categories.length} selected` });
		if (c.products?.length)
			conditions.push({ label: 'Products', value: `${c.products.length} selected` });
		if (c.customerGroups?.length)
			conditions.push({ label: 'Customer Groups', value: c.customerGroups.join(', ') });
		if (c.firstOrderOnly) conditions.push({ label: 'First Order Only', value: 'Yes' });
		if (c.weekdays?.length) conditions.push({ label: 'Days', value: `${c.weekdays.length} days` });

		return conditions.length > 0 ? conditions : [{ label: 'Applies to', value: 'All orders' }];
	}

	function handleDelete() {
		if (confirm(`Delete rule "${rule?.name}"?`)) {
			goto('/inventory/pricing/rules');
		}
	}
</script>

{#if rule}
	{@const status = getRuleStatus(rule)}
	{@const Icon = getRuleTypeIcon(rule.ruleType)}

	<div class="container mx-auto max-w-4xl space-y-6 p-6">
		<!-- Header -->
		<div class="flex items-start justify-between">
			<div class="flex items-start gap-4">
				<Button variant="ghost" size="icon" onclick={() => goto('/inventory/pricing/rules')}>
					<ArrowLeft class="h-4 w-4" />
				</Button>
				<div>
					<div class="flex items-center gap-3">
						<div class="rounded-lg bg-primary/10 p-2">
							<Icon class="h-6 w-6 text-primary" />
						</div>
						<div>
							<h1 class="text-2xl font-bold">{rule.name}</h1>
							<div class="mt-1 flex items-center gap-2">
								{#if rule.code}
									<code class="rounded bg-muted px-2 py-0.5 text-sm">{rule.code}</code>
								{/if}
								<Badge variant={status.variant}>{status.label}</Badge>
							</div>
						</div>
					</div>
				</div>
			</div>
			<div class="flex items-center gap-2">
				<Button variant="outline" onclick={() => goto(`/inventory/pricing/rules/${ruleId}/edit`)}>
					<Pencil class="mr-2 h-4 w-4" />
					Edit
				</Button>
				<Button variant="destructive" onclick={handleDelete}>
					<Trash2 class="mr-2 h-4 w-4" />
					Delete
				</Button>
			</div>
		</div>

		{#if rule.description}
			<p class="text-muted-foreground">{rule.description}</p>
		{/if}

		<!-- Stats -->
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
			<Card.Root>
				<Card.Content class="flex items-center gap-4 pt-6">
					<div class="rounded-lg bg-green-100 p-3 dark:bg-green-900">
						<TrendingUp class="h-5 w-5 text-green-600" />
					</div>
					<div>
						<p class="text-2xl font-bold">{rule.usageCount}</p>
						<p class="text-sm text-muted-foreground">Times Used</p>
					</div>
				</Card.Content>
			</Card.Root>
			<Card.Root>
				<Card.Content class="flex items-center gap-4 pt-6">
					<div class="rounded-lg bg-blue-100 p-3 dark:bg-blue-900">
						<Calendar class="h-5 w-5 text-blue-600" />
					</div>
					<div>
						<p class="text-lg font-semibold">
							{#if rule.validFrom || rule.validTo}
								{formatDate(rule.validFrom)} - {formatDate(rule.validTo)}
							{:else}
								Always
							{/if}
						</p>
						<p class="text-sm text-muted-foreground">Validity</p>
					</div>
				</Card.Content>
			</Card.Root>
			<Card.Root>
				<Card.Content class="flex items-center gap-4 pt-6">
					<div class="rounded-lg bg-purple-100 p-3 dark:bg-purple-900">
						<Users class="h-5 w-5 text-purple-600" />
					</div>
					<div>
						<p class="text-lg font-semibold">
							{#if rule.usageLimit}
								{rule.usageLimit - (rule.usageCount ?? 0)} / {rule.usageLimit}
							{:else}
								Unlimited
							{/if}
						</p>
						<p class="text-sm text-muted-foreground">Remaining Uses</p>
					</div>
				</Card.Content>
			</Card.Root>
		</div>

		<!-- Rule Details -->
		<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
			<!-- Discount Info -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Discount Details</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-4">
					<div class="flex justify-between">
						<span class="text-muted-foreground">Type</span>
						<span class="font-medium">{getRuleTypeLabel(rule.ruleType)}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-muted-foreground">Value</span>
						<span class="font-medium text-green-600">{getDiscountDisplay(rule)}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-muted-foreground">Priority</span>
						<span>{rule.priority}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-muted-foreground">Combination</span>
						<span>{rule.isCombinable ? 'Combinable' : 'Exclusive'}</span>
					</div>
					{#if rule.exclusiveGroup}
						<div class="flex justify-between">
							<span class="text-muted-foreground">Exclusive Group</span>
							<code class="rounded bg-muted px-2 py-0.5 text-sm">{rule.exclusiveGroup}</code>
						</div>
					{/if}
					<div class="flex justify-between">
						<span class="text-muted-foreground">Apply On</span>
						<span>{rule.applyOn === 'line' ? 'Per line item' : 'Whole order'}</span>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Conditions -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Conditions</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-3">
					{#each getConditionsList(rule) as condition (condition.label)}
						<div class="flex justify-between">
							<span class="text-muted-foreground">{condition.label}</span>
							<span class="font-medium">{condition.value}</span>
						</div>
					{/each}
				</Card.Content>
			</Card.Root>
		</div>

		<!-- Limits -->
		<Card.Root>
			<Card.Header>
				<Card.Title>Usage Limits</Card.Title>
			</Card.Header>
			<Card.Content>
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
					<div>
						<p class="text-sm text-muted-foreground">Total Limit</p>
						<p class="text-lg font-semibold">{rule.usageLimit ?? 'Unlimited'}</p>
					</div>
					<div>
						<p class="text-sm text-muted-foreground">Per Customer</p>
						<p class="text-lg font-semibold">{rule.perCustomerLimit ?? 'Unlimited'}</p>
					</div>
					<div>
						<p class="text-sm text-muted-foreground">Used So Far</p>
						<p class="text-lg font-semibold">{rule.usageCount}</p>
					</div>
				</div>
			</Card.Content>
		</Card.Root>
	</div>
{:else}
	<div class="container mx-auto p-6 text-center">
		<h1 class="text-2xl font-bold">Rule Not Found</h1>
		<p class="mt-2 text-muted-foreground">The pricing rule you're looking for doesn't exist.</p>
		<Button class="mt-4" onclick={() => goto('/inventory/pricing/rules')}>Back to Rules</Button>
	</div>
{/if}
