<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

	interface Props {
		title: string;
		value: string | number;
		description?: string;
		trend?: {
			value: number;
			isPositive: boolean;
		};
		icon?: string;
		variant?: 'default' | 'success' | 'warning' | 'danger';
	}

	let { title, value, description, trend, icon, variant = 'default' }: Props = $props();

	const variantClasses = {
		default: 'text-foreground',
		success: 'text-green-600',
		warning: 'text-yellow-600',
		danger: 'text-red-600'
	};
</script>

<Card>
	<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
		<CardTitle class="text-sm font-medium">{title}</CardTitle>
		{#if icon}
			<span class="text-muted-foreground">{icon}</span>
		{/if}
	</CardHeader>
	<CardContent>
		<div class="text-2xl font-bold {variantClasses[variant]}">{value}</div>
		{#if description}
			<p class="text-xs text-muted-foreground">{description}</p>
		{/if}
		{#if trend}
			<div class="mt-1 flex items-center text-xs">
				<span class={trend.isPositive ? 'text-green-600' : 'text-red-600'}>
					{trend.isPositive ? '+' : ''}{trend.value}%
				</span>
				<span class="ml-1 text-muted-foreground">from last period</span>
			</div>
		{/if}
	</CardContent>
</Card>
