<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import UsersIcon from '@lucide/svelte/icons/users';
</script>

<svelte:head>
	<title>User Management - Admin - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<UsersIcon class="h-5 w-5" />
				User Management
			</CardTitle>
			<CardDescription>
				Manage users in your organization. Create, suspend, or delete users and assign roles.
			</CardDescription>
		</CardHeader>
		<CardContent>
			<p class="text-muted-foreground">
				User management functionality will be implemented in task 08.08.01.
			</p>
		</CardContent>
	</Card>
</div>
