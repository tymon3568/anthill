<script lang="ts">
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Card from '$lib/components/ui/card/index.js';

	interface Props {
		/** Number of metric cards to show */
		metricCardCount?: number;
		/** Show chart skeleton */
		showChart?: boolean;
		/** Show recent items list skeleton */
		showRecentList?: boolean;
	}

	let { metricCardCount = 4, showChart = true, showRecentList = true }: Props = $props();

	// Normalize metricCardCount to prevent RangeError with invalid values (NaN, Infinity, negative)
	const safeMetricCardCount = $derived(
		Number.isFinite(metricCardCount) ? Math.max(0, Math.floor(metricCardCount)) : 0
	);
</script>

<div class="space-y-6">
	<!-- Page Title Skeleton -->
	<div class="flex flex-col gap-2">
		<Skeleton class="h-8 w-48" />
		<Skeleton class="h-4 w-72" />
	</div>

	<!-- Metric Cards Grid Skeleton -->
	<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
		{#each Array(safeMetricCardCount) as _, i (i)}
			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Skeleton class="h-4 w-24" />
					<Skeleton class="size-4 rounded" />
				</Card.Header>
				<Card.Content>
					<Skeleton class="mb-1 h-7 w-20" />
					<Skeleton class="h-3 w-32" />
				</Card.Content>
			</Card.Root>
		{/each}
	</div>

	<!-- Main Content Area -->
	<div class="grid gap-4 lg:grid-cols-7">
		{#if showChart}
			<!-- Chart Skeleton -->
			<Card.Root class="lg:col-span-4">
				<Card.Header>
					<Skeleton class="h-5 w-32" />
					<Skeleton class="h-3 w-48" />
				</Card.Header>
				<Card.Content>
					<!-- Chart placeholder - deterministic heights to avoid SSR hydration mismatch -->
					<div class="flex h-[300px] items-end justify-between gap-2 pt-4">
						{#each [45, 72, 38, 85, 52, 28, 65, 78, 42, 90, 55, 35] as height, i (i)}
							<Skeleton class="w-full rounded-t" style="height: {height}%" />
						{/each}
					</div>
				</Card.Content>
			</Card.Root>
		{/if}

		{#if showRecentList}
			<!-- Recent Items List Skeleton -->
			<Card.Root class={showChart ? 'lg:col-span-3' : 'lg:col-span-7'}>
				<Card.Header>
					<Skeleton class="h-5 w-28" />
					<Skeleton class="h-3 w-40" />
				</Card.Header>
				<Card.Content>
					<div class="space-y-4">
						{#each Array(5) as _, i (i)}
							<div class="flex items-center gap-4">
								<Skeleton class="size-10 rounded-full" />
								<div class="flex-1 space-y-1">
									<Skeleton class="h-4 w-3/4" />
									<Skeleton class="h-3 w-1/2" />
								</div>
								<Skeleton class="h-4 w-16" />
							</div>
						{/each}
					</div>
				</Card.Content>
			</Card.Root>
		{/if}
	</div>

	<!-- Additional Content Skeleton -->
	<div class="grid gap-4 md:grid-cols-2">
		<!-- Table Skeleton -->
		<Card.Root>
			<Card.Header>
				<Skeleton class="h-5 w-36" />
			</Card.Header>
			<Card.Content>
				<div class="space-y-3">
					<!-- Table Header -->
					<div class="flex gap-4 border-b pb-2">
						<Skeleton class="h-3 w-1/4" />
						<Skeleton class="h-3 w-1/4" />
						<Skeleton class="h-3 w-1/4" />
						<Skeleton class="h-3 w-1/4" />
					</div>
					<!-- Table Rows -->
					{#each Array(4) as _, i (i)}
						<div class="flex gap-4 py-2">
							<Skeleton class="h-4 w-1/4" />
							<Skeleton class="h-4 w-1/4" />
							<Skeleton class="h-4 w-1/4" />
							<Skeleton class="h-4 w-1/4" />
						</div>
					{/each}
				</div>
			</Card.Content>
		</Card.Root>

		<!-- Alerts/Notifications Skeleton -->
		<Card.Root>
			<Card.Header>
				<Skeleton class="h-5 w-24" />
			</Card.Header>
			<Card.Content>
				<div class="space-y-3">
					{#each Array(3) as _, i (i)}
						<div class="flex items-start gap-3 rounded-lg border p-3">
							<Skeleton class="size-5 rounded" />
							<div class="flex-1 space-y-1">
								<Skeleton class="h-4 w-3/4" />
								<Skeleton class="h-3 w-full" />
							</div>
						</div>
					{/each}
				</div>
			</Card.Content>
		</Card.Root>
	</div>
</div>
