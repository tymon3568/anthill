<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { authState } from '$lib/stores/auth.svelte';

	// Settings sections
	let activeSection = $state('profile');

	// Profile form state
	let profileForm = $state({
		name: authState.user?.name || '',
		email: authState.user?.email || '',
		timezone: 'UTC',
		language: 'en'
	});

	// Notification settings
	let notifications = $state({
		emailOrders: true,
		emailInventory: true,
		emailMarketing: false,
		pushOrders: true,
		pushInventory: false
	});

	async function saveProfile() {
		console.log('Saving profile:', profileForm);
		// TODO: API call
	}

	async function saveNotifications() {
		console.log('Saving notifications:', notifications);
		// TODO: API call
	}
</script>

<svelte:head>
	<title>Settings - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<div>
		<h1 class="text-2xl font-bold">Settings</h1>
		<p class="text-muted-foreground">Manage your account and preferences</p>
	</div>

	<div class="flex gap-6">
		<!-- Sidebar -->
		<div class="w-48 space-y-1">
			<button
				class="w-full rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
				'profile'
					? 'bg-muted font-medium'
					: ''}"
				onclick={() => (activeSection = 'profile')}
			>
				Profile
			</button>
			<button
				class="w-full rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
				'notifications'
					? 'bg-muted font-medium'
					: ''}"
				onclick={() => (activeSection = 'notifications')}
			>
				Notifications
			</button>
			<button
				class="w-full rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
				'security'
					? 'bg-muted font-medium'
					: ''}"
				onclick={() => (activeSection = 'security')}
			>
				Security
			</button>
			<button
				class="w-full rounded-md px-3 py-2 text-left text-sm hover:bg-muted {activeSection ===
				'billing'
					? 'bg-muted font-medium'
					: ''}"
				onclick={() => (activeSection = 'billing')}
			>
				Billing
			</button>
		</div>

		<!-- Content -->
		<div class="flex-1">
			{#if activeSection === 'profile'}
				<Card>
					<CardHeader>
						<CardTitle>Profile Settings</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div class="space-y-2">
								<Label for="name">Display Name</Label>
								<Input id="name" bind:value={profileForm.name} />
							</div>
							<div class="space-y-2">
								<Label for="email">Email</Label>
								<Input id="email" type="email" bind:value={profileForm.email} />
							</div>
						</div>
						<div class="grid grid-cols-2 gap-4">
							<div class="space-y-2">
								<Label for="timezone">Timezone</Label>
								<select
									id="timezone"
									bind:value={profileForm.timezone}
									class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
								>
									<option value="UTC">UTC</option>
									<option value="America/New_York">Eastern Time</option>
									<option value="America/Los_Angeles">Pacific Time</option>
									<option value="Europe/London">London</option>
									<option value="Asia/Tokyo">Tokyo</option>
								</select>
							</div>
							<div class="space-y-2">
								<Label for="language">Language</Label>
								<select
									id="language"
									bind:value={profileForm.language}
									class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
								>
									<option value="en">English</option>
									<option value="es">Spanish</option>
									<option value="fr">French</option>
									<option value="de">German</option>
								</select>
							</div>
						</div>
						<div class="flex justify-end">
							<Button onclick={saveProfile}>Save Changes</Button>
						</div>
					</CardContent>
				</Card>
			{:else if activeSection === 'notifications'}
				<Card>
					<CardHeader>
						<CardTitle>Notification Preferences</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-4">
							<h4 class="font-medium">Email Notifications</h4>
							<div class="space-y-2">
								<label class="flex items-center gap-2">
									<input type="checkbox" bind:checked={notifications.emailOrders} />
									<span class="text-sm">Order updates</span>
								</label>
								<label class="flex items-center gap-2">
									<input type="checkbox" bind:checked={notifications.emailInventory} />
									<span class="text-sm">Low stock alerts</span>
								</label>
								<label class="flex items-center gap-2">
									<input type="checkbox" bind:checked={notifications.emailMarketing} />
									<span class="text-sm">Marketing emails</span>
								</label>
							</div>
						</div>
						<div class="space-y-4">
							<h4 class="font-medium">Push Notifications</h4>
							<div class="space-y-2">
								<label class="flex items-center gap-2">
									<input type="checkbox" bind:checked={notifications.pushOrders} />
									<span class="text-sm">Order updates</span>
								</label>
								<label class="flex items-center gap-2">
									<input type="checkbox" bind:checked={notifications.pushInventory} />
									<span class="text-sm">Low stock alerts</span>
								</label>
							</div>
						</div>
						<div class="flex justify-end">
							<Button onclick={saveNotifications}>Save Preferences</Button>
						</div>
					</CardContent>
				</Card>
			{:else if activeSection === 'security'}
				<Card>
					<CardHeader>
						<CardTitle>Security Settings</CardTitle>
					</CardHeader>
					<CardContent class="space-y-4">
						<div class="space-y-2">
							<Label>Change Password</Label>
							<div class="grid gap-2">
								<Input type="password" placeholder="Current password" />
								<Input type="password" placeholder="New password" />
								<Input type="password" placeholder="Confirm new password" />
							</div>
						</div>
						<div class="flex justify-end">
							<Button>Update Password</Button>
						</div>
					</CardContent>
				</Card>
			{:else if activeSection === 'billing'}
				<Card>
					<CardHeader>
						<CardTitle>Billing & Subscription</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="rounded-lg border p-4">
							<div class="flex items-center justify-between">
								<div>
									<h4 class="font-medium">Current Plan: Professional</h4>
									<p class="text-sm text-muted-foreground">$49/month, billed monthly</p>
								</div>
								<Button variant="outline">Manage Subscription</Button>
							</div>
						</div>
					</CardContent>
				</Card>
			{/if}
		</div>
	</div>
</div>
