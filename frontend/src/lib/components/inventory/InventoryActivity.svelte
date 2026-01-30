<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import type { ActivityItem, ActivityType } from './types';

	interface Props {
		activities: ActivityItem[];
		isLoading?: boolean;
		maxItems?: number;
		onViewAll?: () => void;
		onItemClick?: (item: ActivityItem) => void;
	}

	let {
		activities = [],
		isLoading = false,
		maxItems = 10,
		onViewAll,
		onItemClick
	}: Props = $props();

	const displayActivities = $derived(activities.slice(0, maxItems));

	function getActivityIcon(type: ActivityType): string {
		switch (type) {
			case 'receipt':
				return '📥';
			case 'shipment':
				return '📤';
			case 'transfer':
				return '🔄';
			case 'adjustment':
				return '📝';
			case 'count':
				return '📋';
			default:
				return '📦';
		}
	}

	function getActivityBadge(type: ActivityType) {
		switch (type) {
			case 'receipt':
				return { variant: 'default' as const, label: 'Receipt' };
			case 'shipment':
				return { variant: 'secondary' as const, label: 'Shipment' };
			case 'transfer':
				return { variant: 'outline' as const, label: 'Transfer' };
			case 'adjustment':
				return { variant: 'secondary' as const, label: 'Adjustment' };
			case 'count':
				return { variant: 'outline' as const, label: 'Count' };
		}
	}

	function formatTimestamp(timestamp: string): string {
		const date = new Date(timestamp);
		const now = new Date();
		const diff = now.getTime() - date.getTime();
		const minutes = Math.floor(diff / 60000);
		const hours = Math.floor(diff / 3600000);
		const days = Math.floor(diff / 86400000);

		if (minutes < 1) return 'Just now';
		if (minutes < 60) return `${minutes}m ago`;
		if (hours < 24) return `${hours}h ago`;
		if (days < 7) return `${days}d ago`;
		return date.toLocaleDateString();
	}
</script>

<Card>
	<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
		<CardTitle class="text-sm font-medium">Recent Activity</CardTitle>
		{#if activities.length > maxItems && onViewAll}
			<Button variant="ghost" size="sm" onclick={onViewAll}>View All</Button>
		{/if}
	</CardHeader>
	<CardContent>
		{#if isLoading}
			<div class="space-y-3">
				{#each Array(5) as _}
					<div class="flex animate-pulse items-start gap-3">
						<div class="h-8 w-8 rounded-full bg-muted"></div>
						<div class="flex-1 space-y-2">
							<div class="h-4 w-3/4 rounded bg-muted"></div>
							<div class="h-3 w-1/2 rounded bg-muted"></div>
						</div>
					</div>
				{/each}
			</div>
		{:else if displayActivities.length === 0}
			<div class="py-6 text-center text-muted-foreground">
				<p>No recent activity</p>
				<p class="text-sm">Activity will appear here as inventory changes</p>
			</div>
		{:else}
			<div class="space-y-3">
				{#each displayActivities as activity (activity.id)}
					{@const badge = getActivityBadge(activity.type)}
					<button
						type="button"
						class="flex w-full cursor-pointer items-start gap-3 rounded-lg p-2 text-left transition-colors hover:bg-muted/50"
						onclick={() => onItemClick?.(activity)}
					>
						<div
							class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted text-lg"
						>
							{getActivityIcon(activity.type)}
						</div>
						<div class="min-w-0 flex-1">
							<div class="flex items-center gap-2">
								<p class="truncate font-medium">{activity.description}</p>
								<Badge variant={badge.variant} class="shrink-0">{badge.label}</Badge>
							</div>
							<p class="text-sm text-muted-foreground">
								{activity.reference}
								{#if activity.user}
									• by {activity.user}
								{/if}
							</p>
						</div>
						<span class="shrink-0 text-xs text-muted-foreground">
							{formatTimestamp(activity.timestamp)}
						</span>
					</button>
				{/each}
			</div>
		{/if}
	</CardContent>
</Card>
