<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { authState } from '$lib/stores/auth.svelte';

	// Server-side and client-side route protection
	$effect(() => {
		// Only redirect if we're not loading and not authenticated
		if (!authState.isLoading && !authState.isAuthenticated) {
			// Store the intended destination for post-login redirect
			const currentPath = page.url.pathname;
			if (currentPath !== '/login') {
				goto(`/login?redirect=${encodeURIComponent(currentPath)}`);
			} else {
				goto('/login');
			}
		}
	});

	let { children } = $props();
</script>

{@render children()}
