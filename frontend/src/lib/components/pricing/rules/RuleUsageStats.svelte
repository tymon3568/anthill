<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Progress } from '$lib/components/ui/progress';
	import type { PricingRule } from '$lib/types/pricing';
	import {
		BarChart3,
		TrendingUp,
		TrendingDown,
		Users,
		ShoppingCart,
		DollarSign,
		Calendar,
		Clock,
		Target,
		Zap
	} from 'lucide-svelte';

	interface UsageStats {
		totalApplications: number;
		totalDiscountGiven: number;
		averageDiscountPerUse: number;
		uniqueCustomers: number;
		totalOrderValue: number;
		conversionRate: number;
		usageByDay: { day: string; count: number }[];
		usageByHour: { hour: number; count: number }[];
		topProducts: { name: string; count: number }[];
		recentApplications: {
			orderId: string;
			date: string;
			discountAmount: number;
			orderTotal: number;
		}[];
	}

	interface Props {
		rule: PricingRule;
		stats?: UsageStats;
		loading?: boolean;
	}

	let { rule, stats, loading = false }: Props = $props();

	// Default mock stats for display - reactive to rule changes
	const defaultStats = $derived.by(
		(): UsageStats => ({
			totalApplications: rule.usageCount ?? 0,
			totalDiscountGiven: 2500000,
			averageDiscountPerUse: rule.usageCount ? 2500000 / rule.usageCount : 0,
			uniqueCustomers: Math.floor((rule.usageCount ?? 0) * 0.8),
			totalOrderValue: 15000000,
			conversionRate: 12.5,
			usageByDay: [
				{ day: 'Mon', count: 15 },
				{ day: 'Tue', count: 22 },
				{ day: 'Wed', count: 18 },
				{ day: 'Thu', count: 28 },
				{ day: 'Fri', count: 35 },
				{ day: 'Sat', count: 42 },
				{ day: 'Sun', count: 30 }
			],
			usageByHour: [
				{ hour: 9, count: 8 },
				{ hour: 10, count: 12 },
				{ hour: 11, count: 18 },
				{ hour: 12, count: 25 },
				{ hour: 13, count: 22 },
				{ hour: 14, count: 15 },
				{ hour: 15, count: 18 },
				{ hour: 16, count: 20 },
				{ hour: 17, count: 28 },
				{ hour: 18, count: 32 },
				{ hour: 19, count: 25 },
				{ hour: 20, count: 18 }
			],
			topProducts: [
				{ name: 'Product A', count: 45 },
				{ name: 'Product B', count: 38 },
				{ name: 'Product C', count: 28 },
				{ name: 'Product D', count: 22 },
				{ name: 'Product E', count: 15 }
			],
			recentApplications: [
				{ orderId: 'ORD-001', date: '2026-01-24', discountAmount: 50000, orderTotal: 500000 },
				{ orderId: 'ORD-002', date: '2026-01-24', discountAmount: 75000, orderTotal: 750000 },
				{ orderId: 'ORD-003', date: '2026-01-23', discountAmount: 100000, orderTotal: 1000000 }
			]
		})
	);

	const displayStats = $derived(stats ?? defaultStats);

	const usagePercent = $derived(
		rule.usageLimit ? Math.round(((rule.usageCount ?? 0) / rule.usageLimit) * 100) : 0
	);

	const maxDayCount = $derived(Math.max(...displayStats.usageByDay.map((d) => d.count)));
	const maxHourCount = $derived(Math.max(...displayStats.usageByHour.map((h) => h.count)));

	function formatCurrency(value: number): string {
		return `₫${value.toLocaleString()}`;
	}

	function getBarHeight(value: number, max: number): number {
		return max > 0 ? (value / max) * 100 : 0;
	}
</script>

<div class="space-y-6">
	<!-- Summary Cards -->
	<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Total Applications</CardTitle>
				<Target class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{displayStats.totalApplications.toLocaleString()}</div>
				{#if rule.usageLimit}
					<div class="mt-2">
						<Progress value={usagePercent} class="h-2" />
						<p class="mt-1 text-xs text-muted-foreground">
							{usagePercent}% of {rule.usageLimit.toLocaleString()} limit
						</p>
					</div>
				{:else}
					<p class="text-xs text-muted-foreground">No usage limit set</p>
				{/if}
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Total Discount Given</CardTitle>
				<DollarSign class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{formatCurrency(displayStats.totalDiscountGiven)}</div>
				<p class="text-xs text-muted-foreground">
					Avg: {formatCurrency(displayStats.averageDiscountPerUse)} per use
				</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Unique Customers</CardTitle>
				<Users class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{displayStats.uniqueCustomers.toLocaleString()}</div>
				<p class="text-xs text-muted-foreground">
					{displayStats.totalApplications > 0
						? ((displayStats.uniqueCustomers / displayStats.totalApplications) * 100).toFixed(1)
						: 0}% unique rate
				</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Order Value Generated</CardTitle>
				<ShoppingCart class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{formatCurrency(displayStats.totalOrderValue)}</div>
				<div class="flex items-center text-xs text-green-600">
					<TrendingUp class="mr-1 h-3 w-3" />
					{displayStats.conversionRate}% conversion rate
				</div>
			</CardContent>
		</Card>
	</div>

	<!-- Charts Row -->
	<div class="grid gap-4 md:grid-cols-2">
		<!-- Usage by Day -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2 text-base">
					<Calendar class="h-4 w-4" />
					Usage by Day of Week
				</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="flex h-40 items-end justify-between gap-2">
					{#each displayStats.usageByDay as day (day.day)}
						<div class="flex flex-1 flex-col items-center gap-1">
							<div
								class="w-full rounded-t bg-primary transition-all"
								style="height: {getBarHeight(day.count, maxDayCount)}%"
							></div>
							<span class="text-xs text-muted-foreground">{day.day}</span>
						</div>
					{/each}
				</div>
			</CardContent>
		</Card>

		<!-- Usage by Hour -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2 text-base">
					<Clock class="h-4 w-4" />
					Usage by Hour
				</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="flex h-40 items-end justify-between gap-1">
					{#each displayStats.usageByHour as hourData (hourData.hour)}
						<div class="flex flex-1 flex-col items-center gap-1">
							<div
								class="w-full rounded-t bg-blue-500 transition-all"
								style="height: {getBarHeight(hourData.count, maxHourCount)}%"
							></div>
							<span class="text-[10px] text-muted-foreground">{hourData.hour}</span>
						</div>
					{/each}
				</div>
			</CardContent>
		</Card>
	</div>

	<!-- Bottom Row -->
	<div class="grid gap-4 md:grid-cols-2">
		<!-- Top Products -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2 text-base">
					<Zap class="h-4 w-4" />
					Top Products
				</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="space-y-3">
					{#each displayStats.topProducts as product, index (product.name)}
						<div class="flex items-center gap-3">
							<div
								class="flex h-6 w-6 items-center justify-center rounded-full bg-muted text-xs font-medium"
							>
								{index + 1}
							</div>
							<div class="flex-1">
								<div class="flex items-center justify-between">
									<span class="text-sm font-medium">{product.name}</span>
									<Badge variant="secondary">{product.count}</Badge>
								</div>
								<Progress
									value={(product.count / displayStats.topProducts[0].count) * 100}
									class="mt-1 h-1.5"
								/>
							</div>
						</div>
					{/each}
				</div>
			</CardContent>
		</Card>

		<!-- Recent Applications -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2 text-base">
					<BarChart3 class="h-4 w-4" />
					Recent Applications
				</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="space-y-3">
					{#each displayStats.recentApplications as application (application.orderId)}
						<div class="flex items-center justify-between rounded-lg border p-3">
							<div>
								<p class="text-sm font-medium">{application.orderId}</p>
								<p class="text-xs text-muted-foreground">{application.date}</p>
							</div>
							<div class="text-right">
								<p class="text-sm font-medium text-green-600">
									-{formatCurrency(application.discountAmount)}
								</p>
								<p class="text-xs text-muted-foreground">
									Order: {formatCurrency(application.orderTotal)}
								</p>
							</div>
						</div>
					{/each}
				</div>
			</CardContent>
		</Card>
	</div>
</div>
