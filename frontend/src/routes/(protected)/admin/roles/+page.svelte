<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import ShieldIcon from '@lucide/svelte/icons/shield';
</script>

<svelte:head>
	<title>Role Management - Admin - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<ShieldIcon class="h-5 w-5" />
				Role Management
			</CardTitle>
			<CardDescription>
				Manage roles and permissions. Create custom roles and assign permissions.
			</CardDescription>
		</CardHeader>
		<CardContent>
			<p class="text-muted-foreground">
				Role management functionality will be implemented in task 08.08.02.
			</p>
		</CardContent>
	</Card>
</div>
