<script lang="ts">
	import { goto } from '$app/navigation';
	import { authState } from '$lib/stores/auth.svelte';
	import LoadingSpinner from '$lib/components/LoadingSpinner.svelte';

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
	<div class="min-h-screen flex items-center justify-center bg-gray-50">
		<div class="text-center">
			<LoadingSpinner size="lg" class="mx-auto mb-4" />
			<p class="text-gray-600">Loading...</p>
		</div>
	</div>
{:else}
	<!-- This will be replaced by redirect -->
	<div class="min-h-screen flex items-center justify-center bg-gray-50">
		<div class="text-center">
			<h1 class="text-2xl font-bold text-gray-900 mb-4">Anthill Inventory</h1>
			<p class="text-gray-600">Redirecting...</p>
		</div>
	</div>
{/if}
