<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { authState } from '$lib/stores/auth.svelte';

	// Handle authentication check and redirect
	onMount(() => {
		if (!authState.isLoading) {
			if (authState.isAuthenticated) {
				goto('/dashboard');
			} else {
				goto('/login');
			}
		}
	});

	// Show loading while checking auth
	$: if (!authState.isLoading) {
		if (authState.isAuthenticated) {
			goto('/dashboard');
		} else {
			goto('/login');
		}
	}
</script>

<svelte:head>
	<title>Anthill Inventory</title>
	<meta name="description" content="Multi-tenant inventory management SaaS" />
</svelte:head>

{#if authState.isLoading}
	<div class="min-h-screen flex items-center justify-center bg-gray-50">
		<div class="text-center">
			<div class="w-16 h-16 border-4 border-primary border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
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
