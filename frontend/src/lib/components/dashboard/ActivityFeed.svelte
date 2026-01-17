<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';

	interface Activity {
		id: string;
		type: 'order' | 'inventory' | 'alert' | 'user';
		message: string;
		timestamp: string;
		status?: 'success' | 'warning' | 'error' | 'info';
	}

	interface Props {
		title: string;
		activities: Activity[];
		maxItems?: number;
	}

	let { title, activities, maxItems = 5 }: Props = $props();

	const displayActivities = $derived(activities.slice(0, maxItems));

	const statusColors = {
		success: 'bg-green-100 text-green-800',
		warning: 'bg-yellow-100 text-yellow-800',
		error: 'bg-red-100 text-red-800',
		info: 'bg-blue-100 text-blue-800'
	};

	const typeIcons = {
		order: '📦',
		inventory: '📊',
		alert: '⚠️',
		user: '👤'
	};
</script>

<Card class="h-full">
	<CardHeader>
		<CardTitle class="text-sm font-medium">{title}</CardTitle>
	</CardHeader>
	<CardContent>
		{#if displayActivities.length === 0}
			<div class="flex h-32 items-center justify-center text-muted-foreground">
				<p>No recent activity</p>
			</div>
		{:else}
			<div class="space-y-3">
				{#each displayActivities as activity}
					<div class="flex items-start gap-3 rounded-lg p-2 hover:bg-muted/50">
						<span class="text-lg">{typeIcons[activity.type]}</span>
						<div class="flex-1 space-y-1">
							<p class="text-sm">{activity.message}</p>
							<div class="flex items-center gap-2">
								<span class="text-xs text-muted-foreground">{activity.timestamp}</span>
								{#if activity.status}
									<Badge variant="outline" class={statusColors[activity.status]}>
										{activity.status}
									</Badge>
								{/if}
							</div>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</CardContent>
</Card>
