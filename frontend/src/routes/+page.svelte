<script lang="ts">
	import { goto } from '$app/navigation';
	import { authState } from '$lib/stores/auth.svelte';

	// Show loading while checking auth
	$effect(() => {
		if (!authState.isLoading) {
			if (authState.isAuthenticated) {
				goto('/dashboard');
			} else {
				goto('/login');
			}
		}
	});
</script>

<svelte:head>
	<title>Anthill Inventory</title>
	<meta name="description" content="Multi-tenant inventory management SaaS" />
</svelte:head>

{#if authState.isLoading}
	<div class="flex min-h-screen items-center justify-center bg-gray-50">
		<div class="text-center">
			<div
				class="mx-auto mb-4 h-16 w-16 animate-spin rounded-full border-4 border-primary border-t-transparent"
			></div>
			<p class="text-gray-600">Loading...</p>
		</div>
	</div>
{:else}
	<!-- This will be replaced by redirect -->
	<div class="flex min-h-screen items-center justify-center bg-gray-50">
		<div class="text-center">
			<h1 class="mb-4 text-2xl font-bold text-gray-900">Anthill Inventory</h1>
			<p class="text-gray-600">Redirecting...</p>
		</div>
	</div>
{/if}
