<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { authState, authStore } from '$lib/stores/auth.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import AppHeader from '$lib/components/app-header.svelte';
	import CommandPalette from '$lib/components/command-palette.svelte';
	import SidebarSkeleton from '$lib/components/sidebar-skeleton.svelte';
	import DashboardSkeleton from '$lib/components/dashboard-skeleton.svelte';
	import { Toaster } from '$lib/components/ui/sonner/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';

	// Command palette state
	let commandPaletteOpen = $state(false);

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

	// Open command palette handler (passed to header)
	function openCommandPalette() {
		commandPaletteOpen = true;
	}

	let { children } = $props();

	// Determine if we should show loading state
	let isLoading = $derived(authState.isLoading);
</script>

{#if isLoading}
	<!-- Loading Skeleton State -->
	<Sidebar.Provider>
		<Sidebar.Root variant="sidebar" collapsible="icon" aria-label="Loading navigation">
			<SidebarSkeleton />
		</Sidebar.Root>
		<Sidebar.Inset>
			<!-- Header Skeleton -->
			<header
				class="flex h-14 shrink-0 items-center gap-2 border-b bg-background px-3 md:px-4"
				aria-busy="true"
			>
				<div class="flex flex-1 items-center gap-2">
					<Skeleton class="size-7 rounded" />
					<Skeleton class="hidden h-4 w-32 md:block" />
				</div>
				<div class="flex items-center gap-2">
					<Skeleton class="size-8 rounded" />
					<Skeleton class="size-8 rounded" />
					<Skeleton class="size-8 rounded" />
				</div>
			</header>
			<main class="flex-1 overflow-auto p-4 md:p-6" aria-busy="true" aria-label="Loading content">
				<DashboardSkeleton />
			</main>
		</Sidebar.Inset>
	</Sidebar.Provider>
{:else}
	<!-- Authenticated Content -->
	<Sidebar.Provider>
		<AppSidebar />
		<Sidebar.Inset>
			<AppHeader onSearchClick={openCommandPalette} />
			<main class="flex-1 overflow-auto p-4 md:p-6">
				{@render children()}
			</main>
		</Sidebar.Inset>
	</Sidebar.Provider>

	<!-- Command Palette (Ctrl+K) -->
	<CommandPalette bind:open={commandPaletteOpen} />
{/if}

<!-- Toast notifications -->
<Toaster />
