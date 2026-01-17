<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import MailIcon from '@lucide/svelte/icons/mail';
</script>

<svelte:head>
	<title>Invitation Management - Admin - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<MailIcon class="h-5 w-5" />
				Invitation Management
			</CardTitle>
			<CardDescription>
				Manage user invitations. Send, resend, or revoke invitations.
			</CardDescription>
		</CardHeader>
		<CardContent>
			<p class="text-muted-foreground">
				Invitation management functionality will be implemented in task 08.08.04.
			</p>
		</CardContent>
	</Card>
</div>
