<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { userServiceApi } from '$lib/api/user-service';
	import type { PageProps } from './$types';

	// Get user from server-side load function (SSR-safe)
	let { data }: PageProps = $props();

	let isEditing = $state(false);
	let isSaving = $state(false);
	let successMessage = $state('');
	let errorMessage = $state('');

	// Use server data with fallback - derive user info
	let userName = $derived(data.user?.name ?? data.user?.email ?? 'Not set');
	let userEmail = $derived(data.user?.email ?? 'Not set');
	let userRole = $derived(data.user?.role ?? 'user');
	let userId = $derived(data.user?.userId ?? 'Unknown');

	// Form data for editing
	let formData = $state({
		name: data.user?.name ?? '',
		email: data.user?.email ?? ''
	});

	function toggleEdit() {
		isEditing = !isEditing;
		if (!isEditing) {
			// Reset form data when canceling
			formData.name = data.user?.name ?? '';
			formData.email = data.user?.email ?? '';
		}
		successMessage = '';
		errorMessage = '';
	}

	async function handleSave() {
		isSaving = true;
		errorMessage = '';
		successMessage = '';

		try {
			// Call profile update API
			const result = await userServiceApi.updateProfile({
				fullName: formData.name
			});

			if (result.success) {
				successMessage = 'Profile updated successfully';
				isEditing = false;
				// Reload page data to reflect changes
				await invalidateAll();
			} else {
				errorMessage = result.error ?? 'Failed to update profile';
			}
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : 'Failed to update profile';
		} finally {
			isSaving = false;
		}
	}
</script>

<svelte:head>
	<title>Profile - Anthill</title>
</svelte:head>

<div class="container mx-auto max-w-2xl py-8">
	<h1 class="mb-6 text-2xl font-bold">My Profile</h1>

	<Card>
		<CardHeader>
			<CardTitle class="flex items-center justify-between">
				<span>Profile Information</span>
				<Button variant="outline" size="sm" onclick={toggleEdit}>
					{isEditing ? 'Cancel' : 'Edit'}
				</Button>
			</CardTitle>
		</CardHeader>
		<CardContent>
			{#if successMessage}
				<div class="mb-4 rounded-md border border-green-200 bg-green-50 p-3 text-sm text-green-700">
					{successMessage}
				</div>
			{/if}

			{#if errorMessage}
				<div class="mb-4 rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-600">
					{errorMessage}
				</div>
			{/if}

			<div class="space-y-4">
				<div class="space-y-2">
					<Label for="name">Name</Label>
					{#if isEditing}
						<Input id="name" bind:value={formData.name} placeholder="Enter your name" />
					{:else}
						<p class="rounded-md bg-gray-50 px-3 py-2 text-gray-900">
							{userName}
						</p>
					{/if}
				</div>

				<div class="space-y-2">
					<Label for="email">Email</Label>
					<p class="rounded-md bg-gray-50 px-3 py-2 text-gray-900">
						{userEmail}
					</p>
					<p class="text-xs text-gray-500">Email cannot be changed</p>
				</div>

				<div class="space-y-2">
					<Label>Role</Label>
					<p class="rounded-md bg-gray-50 px-3 py-2 text-gray-900 capitalize">
						{userRole}
					</p>
				</div>

				<div class="space-y-2">
					<Label>User ID</Label>
					<p class="rounded-md bg-gray-50 px-3 py-2 font-mono text-sm text-gray-600">
						{userId}
					</p>
				</div>

				{#if isEditing}
					<div class="flex justify-end pt-4">
						<Button onclick={handleSave} disabled={isSaving}>
							{#if isSaving}
								<span
									class="mr-2 inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"
								></span>
								Saving...
							{:else}
								Save Changes
							{/if}
						</Button>
					</div>
				{/if}
			</div>
		</CardContent>
	</Card>

	<Card class="mt-6">
		<CardHeader>
			<CardTitle>Security</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="space-y-4">
				<div class="flex items-center justify-between">
					<div>
						<p class="font-medium">Password</p>
						<p class="text-sm text-gray-500">Change your password</p>
					</div>
					<Button variant="outline" onclick={() => goto('/forgot-password')}>
						Change Password
					</Button>
				</div>
			</div>
		</CardContent>
	</Card>
</div>
