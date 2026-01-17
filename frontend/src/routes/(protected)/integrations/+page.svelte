<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { mockIntegrations, type Integration } from '$lib/api/integrations';

	function getStatusBadgeVariant(
		status: Integration['status']
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		const variants: Record<
			Integration['status'],
			'default' | 'secondary' | 'destructive' | 'outline'
		> = {
			active: 'default',
			inactive: 'secondary',
			error: 'destructive',
			pending: 'outline'
		};
		return variants[status] || 'outline';
	}

	function getTypeIcon(type: Integration['type']): string {
		const icons = {
			marketplace: '🛒',
			shipping: '📦',
			payment: '💳',
			erp: '🏢',
			custom: '⚙️'
		};
		return icons[type] || '🔗';
	}

	function formatDate(date: string | undefined): string {
		if (!date) return 'Never';
		return new Date(date).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<svelte:head>
	<title>Integrations - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Integrations</h1>
			<p class="text-muted-foreground">Connect with marketplaces and services</p>
		</div>
		<Button>Add Integration</Button>
	</div>

	<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
		{#each mockIntegrations as integration}
			<Card>
				<CardHeader>
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-2">
							<span class="text-2xl">{getTypeIcon(integration.type)}</span>
							<CardTitle class="text-lg">{integration.name}</CardTitle>
						</div>
						<Badge variant={getStatusBadgeVariant(integration.status)}>{integration.status}</Badge>
					</div>
				</CardHeader>
				<CardContent>
					<div class="space-y-2 text-sm">
						<div class="flex justify-between">
							<span class="text-muted-foreground">Provider</span>
							<span class="font-medium capitalize">{integration.provider}</span>
						</div>
						<div class="flex justify-between">
							<span class="text-muted-foreground">Type</span>
							<span class="capitalize">{integration.type}</span>
						</div>
						<div class="flex justify-between">
							<span class="text-muted-foreground">Last Sync</span>
							<span>{formatDate(integration.lastSyncAt)}</span>
						</div>
					</div>
					<div class="mt-4 flex gap-2">
						<Button variant="outline" size="sm" class="flex-1">Configure</Button>
						<Button variant="outline" size="sm">Sync Now</Button>
					</div>
				</CardContent>
			</Card>
		{/each}
	</div>

	<Card>
		<CardHeader>
			<CardTitle>Available Integrations</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="grid gap-4 md:grid-cols-4">
				{#each ['Shopify', 'WooCommerce', 'Amazon', 'eBay', 'Stripe', 'PayPal', 'FedEx', 'UPS'] as provider}
					<div class="flex items-center justify-between rounded-lg border p-4">
						<span class="font-medium">{provider}</span>
						<Button variant="outline" size="sm">Connect</Button>
					</div>
				{/each}
			</div>
		</CardContent>
	</Card>
</div>
