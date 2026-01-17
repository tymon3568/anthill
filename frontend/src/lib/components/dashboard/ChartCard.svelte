<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

	interface Props {
		title: string;
		type: 'line' | 'bar' | 'pie' | 'area';
		data?: Array<{ label: string; value: number }>;
	}

	let { title, type, data = [] }: Props = $props();

	// Calculate max value for scaling
	const maxValue = $derived(Math.max(...data.map((d) => d.value), 1));

	// Calculate safe denominator for x-coordinate (avoid division by zero)
	const xDenom = $derived(Math.max(data.length - 1, 1));
</script>

<Card class="h-full">
	<CardHeader>
		<CardTitle class="text-sm font-medium">{title}</CardTitle>
	</CardHeader>
	<CardContent>
		{#if data.length === 0}
			<div class="flex h-48 items-center justify-center text-muted-foreground">
				<p>No data available</p>
			</div>
		{:else if type === 'bar'}
			<div class="flex h-48 items-end justify-between gap-2">
				{#each data as item}
					<div class="flex flex-1 flex-col items-center gap-1">
						<div
							class="w-full rounded-t bg-primary transition-all"
							style="height: {(item.value / maxValue) * 160}px"
						></div>
						<span class="text-xs text-muted-foreground">{item.label}</span>
					</div>
				{/each}
			</div>
		{:else if type === 'line'}
			<div class="relative h-48">
				<svg class="h-full w-full" viewBox="0 0 100 50" preserveAspectRatio="none">
					{#if data.length === 1}
						<!-- Single data point: render a circle instead of a line -->
						<circle
							cx="50"
							cy={50 - (data[0].value / maxValue) * 45}
							r="2"
							fill="hsl(var(--primary))"
						/>
					{:else}
						<polyline
							fill="none"
							stroke="hsl(var(--primary))"
							stroke-width="1"
							points={data
								.map((d, i) => `${(i / xDenom) * 100},${50 - (d.value / maxValue) * 45}`)
								.join(' ')}
						/>
					{/if}
				</svg>
				<div class="mt-2 flex justify-between text-xs text-muted-foreground">
					{#each data as item, i}
						{#if i === 0 || i === data.length - 1 || i === Math.floor(data.length / 2)}
							<span>{item.label}</span>
						{/if}
					{/each}
				</div>
			</div>
		{:else}
			<div class="flex h-48 items-center justify-center text-muted-foreground">
				<p>Chart type: {type}</p>
			</div>
		{/if}
	</CardContent>
</Card>
