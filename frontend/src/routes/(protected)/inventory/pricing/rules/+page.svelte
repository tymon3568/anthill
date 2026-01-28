<script lang="ts">
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Select from '$lib/components/ui/select';
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { mockPricingRules, type PricingRule, type RuleType } from '$lib/types/pricing';
	import {
		Plus,
		Search,
		MoreHorizontal,
		Pencil,
		Trash2,
		Copy,
		ToggleLeft,
		ToggleRight,
		Clock,
		CheckCircle,
		XCircle,
		Calendar,
		Tag,
		Percent,
		DollarSign,
		Gift,
		ShoppingCart,
		Package
	} from 'lucide-svelte';

	// State
	let searchQuery = $state('');
	let statusFilter = $state<string>('all');
	let typeFilter = $state<string>('all');

	// Get rules from mock data
	let rules = $state<PricingRule[]>([...mockPricingRules]);

	// Filter rules
	const filteredRules = $derived.by(() => {
		let result = rules;

		// Search filter
		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			result = result.filter(
				(rule) =>
					rule.name.toLowerCase().includes(query) ||
					rule.code?.toLowerCase().includes(query) ||
					rule.description?.toLowerCase().includes(query)
			);
		}

		// Type filter
		if (typeFilter !== 'all') {
			result = result.filter((rule) => rule.ruleType === typeFilter);
		}

		// Status filter
		if (statusFilter !== 'all') {
			const now = new Date();
			result = result.filter((rule) => {
				const status = getRuleStatus(rule);
				return status === statusFilter;
			});
		}

		return result;
	});

	// Group rules by status
	const groupedRules = $derived.by(() => {
		const active: PricingRule[] = [];
		const scheduled: PricingRule[] = [];
		const expired: PricingRule[] = [];
		const inactive: PricingRule[] = [];

		for (const rule of filteredRules) {
			const status = getRuleStatus(rule);
			if (status === 'active') active.push(rule);
			else if (status === 'scheduled') scheduled.push(rule);
			else if (status === 'expired') expired.push(rule);
			else inactive.push(rule);
		}

		return { active, scheduled, expired, inactive };
	});

	function getRuleStatus(rule: PricingRule): 'active' | 'scheduled' | 'expired' | 'inactive' {
		if (!rule.isActive) return 'inactive';

		const now = new Date();
		const from = rule.validFrom ? new Date(rule.validFrom) : null;
		const to = rule.validTo ? new Date(rule.validTo) : null;

		if (from && from > now) return 'scheduled';
		if (to && to < now) return 'expired';
		return 'active';
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
				return 'Percentage Off';
			case 'discount_amount':
				return 'Amount Off';
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
		if (!date) return '';
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

	function getDiscountDisplay(rule: PricingRule): string {
		if (rule.discountPercentage) {
			let display = `${rule.discountPercentage}% off`;
			if (rule.maxDiscountAmount) {
				display += ` (max ${formatCurrency(rule.maxDiscountAmount)})`;
			}
			return display;
		}
		if (rule.discountAmount) {
			return `${formatCurrency(rule.discountAmount)} off`;
		}
		if (rule.fixedPrice !== undefined) {
			return `Fixed: ${formatCurrency(rule.fixedPrice)}`;
		}
		if (rule.ruleType === 'buy_x_get_y') {
			return `Buy ${rule.buyQuantity}, Get ${rule.getQuantity} Free`;
		}
		if (rule.ruleType === 'free_item') {
			return `Free item (x${rule.freeQuantity})`;
		}
		return '-';
	}

	function getConditionsSummary(rule: PricingRule): string[] {
		const conditions: string[] = [];
		const c = rule.conditions;

		if (c.minQuantity) conditions.push(`Min qty: ${c.minQuantity}`);
		if (c.minOrderAmount) conditions.push(`Min order: ${formatCurrency(c.minOrderAmount)}`);
		if (c.categories?.length) conditions.push(`Categories: ${c.categories.length}`);
		if (c.products?.length) conditions.push(`Products: ${c.products.length}`);
		if (c.customerGroups?.length) conditions.push(`Groups: ${c.customerGroups.join(', ')}`);
		if (c.firstOrderOnly) conditions.push('First order only');

		return conditions.length > 0 ? conditions : ['All orders'];
	}

	function handleToggleActive(rule: PricingRule) {
		rules = rules.map((r) => (r.ruleId === rule.ruleId ? { ...r, isActive: !r.isActive } : r));
	}

	function handleDelete(rule: PricingRule) {
		if (confirm(`Delete rule "${rule.name}"?`)) {
			rules = rules.filter((r) => r.ruleId !== rule.ruleId);
		}
	}

	function handleDuplicate(rule: PricingRule) {
		const newRule: PricingRule = {
			...rule,
			ruleId: `rule-${Date.now()}`,
			name: `${rule.name} (Copy)`,
			code: rule.code ? `${rule.code}_COPY` : undefined,
			usageCount: 0,
			isActive: false,
			createdAt: new Date(),
			updatedAt: new Date()
		};
		rules = [...rules, newRule];
	}

	function handleStatusChange(value: string | undefined) {
		statusFilter = value ?? 'all';
	}

	function handleTypeChange(value: string | undefined) {
		typeFilter = value ?? 'all';
	}
</script>

<div class="container mx-auto space-y-6 p-6">
	<!-- Header -->
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-2xl font-bold">Pricing Rules</h1>
			<p class="text-muted-foreground">Manage discounts, promotions, and special pricing</p>
		</div>
		<Button onclick={() => goto('/inventory/pricing/rules/new')}>
			<Plus class="mr-2 h-4 w-4" />
			New Rule
		</Button>
	</div>

	<!-- Filters -->
	<div class="flex flex-col gap-4 sm:flex-row">
		<div class="relative flex-1">
			<Search class="absolute top-2.5 left-2.5 h-4 w-4 text-muted-foreground" />
			<Input bind:value={searchQuery} placeholder="Search rules..." class="pl-8" />
		</div>
		<Select.Root type="single" value={statusFilter} onValueChange={handleStatusChange}>
			<Select.Trigger class="w-full sm:w-40">
				{statusFilter === 'all'
					? 'All Status'
					: statusFilter.charAt(0).toUpperCase() + statusFilter.slice(1)}
			</Select.Trigger>
			<Select.Content>
				<Select.Item value="all">All Status</Select.Item>
				<Select.Item value="active">Active</Select.Item>
				<Select.Item value="scheduled">Scheduled</Select.Item>
				<Select.Item value="expired">Expired</Select.Item>
				<Select.Item value="inactive">Inactive</Select.Item>
			</Select.Content>
		</Select.Root>
		<Select.Root type="single" value={typeFilter} onValueChange={handleTypeChange}>
			<Select.Trigger class="w-full sm:w-48">
				{typeFilter === 'all' ? 'All Types' : getRuleTypeLabel(typeFilter as RuleType)}
			</Select.Trigger>
			<Select.Content>
				<Select.Item value="all">All Types</Select.Item>
				<Select.Item value="discount_percentage">Percentage Off</Select.Item>
				<Select.Item value="discount_amount">Amount Off</Select.Item>
				<Select.Item value="fixed_price">Fixed Price</Select.Item>
				<Select.Item value="free_item">Free Item</Select.Item>
				<Select.Item value="buy_x_get_y">Buy X Get Y</Select.Item>
				<Select.Item value="bundle_price">Bundle Price</Select.Item>
			</Select.Content>
		</Select.Root>
	</div>

	<!-- Rules List -->
	<div class="space-y-6">
		<!-- Active Rules -->
		{#if groupedRules.active.length > 0}
			<Card.Root>
				<Card.Header class="pb-3">
					<Card.Title class="flex items-center gap-2 text-base">
						<CheckCircle class="h-4 w-4 text-green-600" />
						Active Promotions ({groupedRules.active.length})
					</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-3">
					{#each groupedRules.active as rule (rule.ruleId)}
						{@const Icon = getRuleTypeIcon(rule.ruleType)}
						<div class="rounded-lg border p-4">
							<div class="flex items-start justify-between">
								<div class="flex items-start gap-3">
									<div class="rounded-lg bg-primary/10 p-2">
										<Icon class="h-5 w-5 text-primary" />
									</div>
									<div>
										<div class="flex items-center gap-2">
											<h3 class="font-semibold">{rule.name}</h3>
											{#if rule.code}
												<code class="rounded bg-muted px-1.5 py-0.5 text-xs">{rule.code}</code>
											{/if}
										</div>
										<p class="text-sm text-muted-foreground">
											{getRuleTypeLabel(rule.ruleType)} • Priority: {rule.priority}
										</p>
									</div>
								</div>
								<div class="flex items-center gap-2">
									<Button
										variant="outline"
										size="sm"
										onclick={() => goto(`/inventory/pricing/rules/${rule.ruleId}/edit`)}
									>
										<Pencil class="h-3 w-3" />
									</Button>
									<DropdownMenu.Root>
										<DropdownMenu.Trigger>
											{#snippet child({ props })}
												<Button {...props} variant="ghost" size="icon" class="h-8 w-8">
													<MoreHorizontal class="h-4 w-4" />
												</Button>
											{/snippet}
										</DropdownMenu.Trigger>
										<DropdownMenu.Content align="end">
											<DropdownMenu.Item onclick={() => handleDuplicate(rule)}>
												<Copy class="mr-2 h-4 w-4" />
												Duplicate
											</DropdownMenu.Item>
											<DropdownMenu.Item onclick={() => handleToggleActive(rule)}>
												<ToggleLeft class="mr-2 h-4 w-4" />
												Deactivate
											</DropdownMenu.Item>
											<DropdownMenu.Separator />
											<DropdownMenu.Item
												class="text-destructive"
												onclick={() => handleDelete(rule)}
											>
												<Trash2 class="mr-2 h-4 w-4" />
												Delete
											</DropdownMenu.Item>
										</DropdownMenu.Content>
									</DropdownMenu.Root>
								</div>
							</div>

							<div class="mt-3 border-t pt-3">
								<div class="grid grid-cols-1 gap-2 text-sm sm:grid-cols-2">
									<div>
										<span class="text-muted-foreground">Conditions:</span>
										<span class="ml-1">{getConditionsSummary(rule).join(', ')}</span>
									</div>
									<div>
										<span class="text-muted-foreground">Discount:</span>
										<span class="ml-1 font-medium text-green-600">{getDiscountDisplay(rule)}</span>
									</div>
								</div>
								<div class="mt-2 flex items-center gap-4 text-xs text-muted-foreground">
									{#if rule.isCombinable}
										<span class="text-green-600">Combinable</span>
									{:else}
										<span class="text-orange-600">Exclusive</span>
									{/if}
									<span>•</span>
									<span>
										{#if rule.validFrom || rule.validTo}
											{formatDate(rule.validFrom)} - {formatDate(rule.validTo) || 'No end'}
										{:else}
											Always valid
										{/if}
									</span>
									<span>•</span>
									<span>Used: {rule.usageCount} times</span>
								</div>
							</div>
						</div>
					{/each}
				</Card.Content>
			</Card.Root>
		{/if}

		<!-- Scheduled Rules -->
		{#if groupedRules.scheduled.length > 0}
			<Card.Root>
				<Card.Header class="pb-3">
					<Card.Title class="flex items-center gap-2 text-base">
						<Calendar class="h-4 w-4 text-blue-600" />
						Scheduled ({groupedRules.scheduled.length})
					</Card.Title>
				</Card.Header>
				<Card.Content>
					<div class="divide-y">
						{#each groupedRules.scheduled as rule (rule.ruleId)}
							<div class="flex items-center justify-between py-3">
								<div>
									<span class="font-medium">{rule.name}</span>
									<span class="ml-2 text-sm text-muted-foreground">
										Starts: {formatDate(rule.validFrom)}
									</span>
								</div>
								<Button
									variant="outline"
									size="sm"
									onclick={() => goto(`/inventory/pricing/rules/${rule.ruleId}/edit`)}
								>
									Edit
								</Button>
							</div>
						{/each}
					</div>
				</Card.Content>
			</Card.Root>
		{/if}

		<!-- Expired Rules -->
		{#if groupedRules.expired.length > 0}
			<Card.Root>
				<Card.Header class="pb-3">
					<Card.Title class="flex items-center gap-2 text-base">
						<Clock class="h-4 w-4 text-orange-600" />
						Expired ({groupedRules.expired.length})
					</Card.Title>
				</Card.Header>
				<Card.Content>
					<div class="divide-y">
						{#each groupedRules.expired as rule (rule.ruleId)}
							<div class="flex items-center justify-between py-3">
								<div>
									<span class="font-medium">{rule.name}</span>
									<span class="ml-2 text-sm text-muted-foreground">
										Ended: {formatDate(rule.validTo)}
									</span>
								</div>
								<Button
									variant="outline"
									size="sm"
									onclick={() => goto(`/inventory/pricing/rules/${rule.ruleId}/edit`)}
								>
									Edit
								</Button>
							</div>
						{/each}
					</div>
				</Card.Content>
			</Card.Root>
		{/if}

		<!-- Inactive Rules -->
		{#if groupedRules.inactive.length > 0}
			<Card.Root>
				<Card.Header class="pb-3">
					<Card.Title class="flex items-center gap-2 text-base">
						<XCircle class="h-4 w-4 text-gray-400" />
						Inactive ({groupedRules.inactive.length})
					</Card.Title>
				</Card.Header>
				<Card.Content>
					<div class="divide-y">
						{#each groupedRules.inactive as rule (rule.ruleId)}
							<div class="flex items-center justify-between py-3">
								<div>
									<span class="font-medium">{rule.name}</span>
									{#if rule.code}
										<code class="ml-2 rounded bg-muted px-1.5 py-0.5 text-xs">{rule.code}</code>
									{/if}
								</div>
								<div class="flex items-center gap-2">
									<Button variant="outline" size="sm" onclick={() => handleToggleActive(rule)}>
										<ToggleRight class="mr-1 h-3 w-3" />
										Activate
									</Button>
									<Button
										variant="outline"
										size="sm"
										onclick={() => goto(`/inventory/pricing/rules/${rule.ruleId}/edit`)}
									>
										Edit
									</Button>
								</div>
							</div>
						{/each}
					</div>
				</Card.Content>
			</Card.Root>
		{/if}

		<!-- Empty State -->
		{#if filteredRules.length === 0}
			<Card.Root>
				<Card.Content class="py-12 text-center">
					<Tag class="mx-auto h-12 w-12 text-muted-foreground/50" />
					<h3 class="mt-4 text-lg font-semibold">No pricing rules found</h3>
					<p class="mt-2 text-muted-foreground">
						{#if searchQuery || statusFilter !== 'all' || typeFilter !== 'all'}
							Try adjusting your filters
						{:else}
							Create your first pricing rule to start offering discounts
						{/if}
					</p>
					{#if !searchQuery && statusFilter === 'all' && typeFilter === 'all'}
						<Button class="mt-4" onclick={() => goto('/inventory/pricing/rules/new')}>
							<Plus class="mr-2 h-4 w-4" />
							Create Rule
						</Button>
					{/if}
				</Card.Content>
			</Card.Root>
		{/if}
	</div>
</div>
