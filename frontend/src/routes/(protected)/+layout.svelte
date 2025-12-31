<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { authState, authStore } from '$lib/stores/auth.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import AppHeader from '$lib/components/app-header.svelte';
	import { Toaster } from '$lib/components/ui/sonner/index.js';

	// Initialize auth store on mount
	$effect(() => {
		authStore.initialize();
	});

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

<Sidebar.Provider>
	<AppSidebar />
	<Sidebar.Inset>
		<AppHeader />
		<main class="flex-1 overflow-auto p-4 md:p-6">
			{@render children()}
		</main>
	</Sidebar.Inset>
</Sidebar.Provider>

<!-- Toast notifications -->
<Toaster />
