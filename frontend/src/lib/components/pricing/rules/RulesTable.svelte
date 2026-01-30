<script lang="ts">
	import type { PricingRule, RuleType } from '$lib/types/pricing';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import * as Table from '$lib/components/ui/table';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import {
		MoreHorizontal,
		Pencil,
		Trash2,
		Eye,
		Power,
		PowerOff,
		Copy,
		Percent,
		DollarSign,
		Tag,
		Gift,
		ShoppingCart,
		Package
	} from 'lucide-svelte';

	interface Props {
		rules: PricingRule[];
		selectedIds?: string[];
		onSelectionChange?: (ids: string[]) => void;
		onView?: (rule: PricingRule) => void;
		onEdit?: (rule: PricingRule) => void;
		onDelete?: (rule: PricingRule) => void;
		onToggleActive?: (rule: PricingRule) => void;
		onDuplicate?: (rule: PricingRule) => void;
	}

	let {
		rules,
		selectedIds = [],
		onSelectionChange,
		onView,
		onEdit,
		onDelete,
		onToggleActive,
		onDuplicate
	}: Props = $props();

	const allSelected = $derived(rules.length > 0 && selectedIds.length === rules.length);
	const someSelected = $derived(selectedIds.length > 0 && selectedIds.length < rules.length);

	function toggleAll() {
		if (allSelected) {
			onSelectionChange?.([]);
		} else {
			onSelectionChange?.(rules.map((r) => r.ruleId));
		}
	}

	function toggleOne(ruleId: string) {
		if (selectedIds.includes(ruleId)) {
			onSelectionChange?.(selectedIds.filter((id) => id !== ruleId));
		} else {
			onSelectionChange?.([...selectedIds, ruleId]);
		}
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
				return '% Off';
			case 'discount_amount':
				return '$ Off';
			case 'fixed_price':
				return 'Fixed';
			case 'free_item':
				return 'Free Item';
			case 'buy_x_get_y':
				return 'Buy X Get Y';
			case 'bundle_price':
				return 'Bundle';
			default:
				return type;
		}
	}

	function getRuleStatus(rule: PricingRule): 'active' | 'scheduled' | 'expired' | 'inactive' {
		if (!rule.isActive) return 'inactive';

		const now = new Date();
		if (rule.validFrom && rule.validFrom > now) return 'scheduled';
		if (rule.validTo && rule.validTo < now) return 'expired';
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

	function formatDiscountValue(rule: PricingRule): string {
		switch (rule.ruleType) {
			case 'discount_percentage':
				return `${rule.discountPercentage}%`;
			case 'discount_amount':
				return formatCurrency(rule.discountAmount ?? 0);
			case 'fixed_price':
				return formatCurrency(rule.fixedPrice ?? 0);
			case 'buy_x_get_y':
				return `Buy ${rule.buyQuantity} Get ${rule.getQuantity}`;
			case 'free_item':
				return `${rule.freeQuantity} Free`;
			case 'bundle_price':
				return 'Bundle';
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
</script>

<Table.Root>
	<Table.Header>
		<Table.Row>
			<Table.Head class="w-12">
				<Checkbox checked={allSelected} indeterminate={someSelected} onCheckedChange={toggleAll} />
			</Table.Head>
			<Table.Head>Rule</Table.Head>
			<Table.Head>Type</Table.Head>
			<Table.Head>Value</Table.Head>
			<Table.Head>Validity</Table.Head>
			<Table.Head>Usage</Table.Head>
			<Table.Head>Status</Table.Head>
			<Table.Head class="w-12"></Table.Head>
		</Table.Row>
	</Table.Header>
	<Table.Body>
		{#each rules as rule (rule.ruleId)}
			{@const Icon = getRuleTypeIcon(rule.ruleType)}
			{@const status = getRuleStatus(rule)}
			<Table.Row class={selectedIds.includes(rule.ruleId) ? 'bg-muted/50' : ''}>
				<Table.Cell>
					<Checkbox
						checked={selectedIds.includes(rule.ruleId)}
						onCheckedChange={() => toggleOne(rule.ruleId)}
					/>
				</Table.Cell>
				<Table.Cell>
					<div class="flex flex-col">
						<button class="text-left font-medium hover:underline" onclick={() => onView?.(rule)}>
							{rule.name}
						</button>
						{#if rule.code}
							<span class="font-mono text-xs text-muted-foreground">{rule.code}</span>
						{/if}
					</div>
				</Table.Cell>
				<Table.Cell>
					<div class="flex items-center gap-2">
						<Icon class="h-4 w-4 text-muted-foreground" />
						<span class="text-sm">{getRuleTypeLabel(rule.ruleType)}</span>
					</div>
				</Table.Cell>
				<Table.Cell>
					<span class="font-medium">{formatDiscountValue(rule)}</span>
				</Table.Cell>
				<Table.Cell>
					<div class="text-xs text-muted-foreground">
						{#if rule.validFrom || rule.validTo}
							{formatDate(rule.validFrom)} - {formatDate(rule.validTo)}
						{:else}
							Always
						{/if}
					</div>
				</Table.Cell>
				<Table.Cell>
					{#if rule.usageLimit}
						<span class="text-sm">
							{rule.usageCount ?? 0} / {rule.usageLimit}
						</span>
					{:else}
						<span class="text-sm text-muted-foreground">{rule.usageCount ?? 0}</span>
					{/if}
				</Table.Cell>
				<Table.Cell>
					<Badge variant={getStatusBadgeVariant(status)}>
						{status.charAt(0).toUpperCase() + status.slice(1)}
					</Badge>
				</Table.Cell>
				<Table.Cell>
					<DropdownMenu.Root>
						<DropdownMenu.Trigger>
							{#snippet child({ props })}
								<Button {...props} variant="ghost" size="icon">
									<MoreHorizontal class="h-4 w-4" />
								</Button>
							{/snippet}
						</DropdownMenu.Trigger>
						<DropdownMenu.Content align="end">
							<DropdownMenu.Item onclick={() => onView?.(rule)}>
								<Eye class="mr-2 h-4 w-4" />
								View Details
							</DropdownMenu.Item>
							<DropdownMenu.Item onclick={() => onEdit?.(rule)}>
								<Pencil class="mr-2 h-4 w-4" />
								Edit
							</DropdownMenu.Item>
							<DropdownMenu.Item onclick={() => onDuplicate?.(rule)}>
								<Copy class="mr-2 h-4 w-4" />
								Duplicate
							</DropdownMenu.Item>
							<DropdownMenu.Separator />
							<DropdownMenu.Item onclick={() => onToggleActive?.(rule)}>
								{#if rule.isActive}
									<PowerOff class="mr-2 h-4 w-4" />
									Deactivate
								{:else}
									<Power class="mr-2 h-4 w-4" />
									Activate
								{/if}
							</DropdownMenu.Item>
							<DropdownMenu.Separator />
							<DropdownMenu.Item class="text-destructive" onclick={() => onDelete?.(rule)}>
								<Trash2 class="mr-2 h-4 w-4" />
								Delete
							</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				</Table.Cell>
			</Table.Row>
		{/each}

		{#if rules.length === 0}
			<Table.Row>
				<Table.Cell colspan={8} class="py-8 text-center">
					<p class="text-muted-foreground">No pricing rules found</p>
				</Table.Cell>
			</Table.Row>
		{/if}
	</Table.Body>
</Table.Root>
