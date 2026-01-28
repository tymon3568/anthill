<script lang="ts">
	import type { RuleType } from '$lib/types/pricing';
	import { Percent, DollarSign, Tag, Gift, ShoppingCart, Package } from 'lucide-svelte';

	interface Props {
		value: RuleType;
		onchange?: (value: RuleType) => void;
		disabled?: boolean;
	}

	let { value = $bindable(), onchange, disabled = false }: Props = $props();

	const ruleTypes: { value: RuleType; label: string; icon: typeof Percent; description: string }[] =
		[
			{
				value: 'discount_percentage',
				label: '% Off',
				icon: Percent,
				description: 'Percentage discount'
			},
			{
				value: 'discount_amount',
				label: '$ Off',
				icon: DollarSign,
				description: 'Fixed amount off'
			},
			{ value: 'fixed_price', label: 'Fixed', icon: Tag, description: 'Override to fixed price' },
			{
				value: 'free_item',
				label: 'Free Item',
				icon: Gift,
				description: 'Free item with purchase'
			},
			{
				value: 'buy_x_get_y',
				label: 'Buy X Get Y',
				icon: ShoppingCart,
				description: 'Buy X get Y free'
			},
			{
				value: 'bundle_price',
				label: 'Bundle',
				icon: Package,
				description: 'Special bundle pricing'
			}
		];

	function handleSelect(ruleType: RuleType) {
		if (disabled) return;
		value = ruleType;
		onchange?.(ruleType);
	}
</script>

<div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
	{#each ruleTypes as rt (rt.value)}
		{@const Icon = rt.icon}
		<button
			type="button"
			class="flex flex-col items-center gap-2 rounded-lg border p-4 text-center transition-colors {disabled
				? 'cursor-not-allowed opacity-50'
				: 'hover:bg-muted/50'} {value === rt.value ? 'border-primary bg-primary/5' : ''}"
			onclick={() => handleSelect(rt.value)}
			{disabled}
		>
			<Icon class="h-6 w-6 {value === rt.value ? 'text-primary' : 'text-muted-foreground'}" />
			<span class="text-sm font-medium">{rt.label}</span>
			<span class="text-xs text-muted-foreground">{rt.description}</span>
		</button>
	{/each}
</div>
