<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

	interface Props {
		title: string;
		value: string | number;
		subtitle?: string;
		icon?: string;
		trend?: {
			value: number;
			isPositive: boolean;
			label?: string;
		};
		variant?: 'default' | 'success' | 'warning' | 'danger' | 'info';
		href?: string;
		isLoading?: boolean;
	}

	let {
		title,
		value,
		subtitle,
		icon,
		trend,
		variant = 'default',
		href,
		isLoading = false
	}: Props = $props();

	const variantStyles = {
		default: {
			text: 'text-foreground',
			bg: 'bg-muted',
			iconBg: 'bg-muted'
		},
		success: {
			text: 'text-green-600 dark:text-green-400',
			bg: 'bg-green-50 dark:bg-green-900/20',
			iconBg: 'bg-green-100 dark:bg-green-900/30'
		},
		warning: {
			text: 'text-yellow-600 dark:text-yellow-400',
			bg: 'bg-yellow-50 dark:bg-yellow-900/20',
			iconBg: 'bg-yellow-100 dark:bg-yellow-900/30'
		},
		danger: {
			text: 'text-red-600 dark:text-red-400',
			bg: 'bg-red-50 dark:bg-red-900/20',
			iconBg: 'bg-red-100 dark:bg-red-900/30'
		},
		info: {
			text: 'text-blue-600 dark:text-blue-400',
			bg: 'bg-blue-50 dark:bg-blue-900/20',
			iconBg: 'bg-blue-100 dark:bg-blue-900/30'
		}
	};

	const styles = $derived(variantStyles[variant]);
</script>

{#if href}
	<a {href} class="block transition-transform hover:scale-[1.02]">
		<Card class="h-full {styles.bg}">
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium text-muted-foreground">{title}</CardTitle>
				{#if icon}
					<div class="flex h-8 w-8 items-center justify-center rounded-md {styles.iconBg}">
						<span class="text-lg">{icon}</span>
					</div>
				{/if}
			</CardHeader>
			<CardContent>
				{#if isLoading}
					<div class="animate-pulse">
						<div class="h-7 w-24 rounded bg-muted"></div>
						{#if subtitle}
							<div class="mt-1 h-4 w-32 rounded bg-muted"></div>
						{/if}
					</div>
				{:else}
					<div class="text-2xl font-bold {styles.text}">{value}</div>
					{#if subtitle}
						<p class="text-xs text-muted-foreground">{subtitle}</p>
					{/if}
					{#if trend}
						<div class="mt-1 flex items-center text-xs">
							<span class={trend.isPositive ? 'text-green-600' : 'text-red-600'}>
								{trend.isPositive ? '↑' : '↓'}
								{Math.abs(trend.value)}%
							</span>
							<span class="ml-1 text-muted-foreground">
								{trend.label ?? 'vs last period'}
							</span>
						</div>
					{/if}
				{/if}
			</CardContent>
		</Card>
	</a>
{:else}
	<Card class="h-full {styles.bg}">
		<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
			<CardTitle class="text-sm font-medium text-muted-foreground">{title}</CardTitle>
			{#if icon}
				<div class="flex h-8 w-8 items-center justify-center rounded-md {styles.iconBg}">
					<span class="text-lg">{icon}</span>
				</div>
			{/if}
		</CardHeader>
		<CardContent>
			{#if isLoading}
				<div class="animate-pulse">
					<div class="h-7 w-24 rounded bg-muted"></div>
					{#if subtitle}
						<div class="mt-1 h-4 w-32 rounded bg-muted"></div>
					{/if}
				</div>
			{:else}
				<div class="text-2xl font-bold {styles.text}">{value}</div>
				{#if subtitle}
					<p class="text-xs text-muted-foreground">{subtitle}</p>
				{/if}
				{#if trend}
					<div class="mt-1 flex items-center text-xs">
						<span class={trend.isPositive ? 'text-green-600' : 'text-red-600'}>
							{trend.isPositive ? '↑' : '↓'}
							{Math.abs(trend.value)}%
						</span>
						<span class="ml-1 text-muted-foreground">
							{trend.label ?? 'vs last period'}
						</span>
					</div>
				{/if}
			{/if}
		</CardContent>
	</Card>
{/if}
