<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import AppHeader from '$lib/components/app-header.svelte';
	import CommandPalette from '$lib/components/command-palette.svelte';
	import { Toaster } from '$lib/components/ui/sonner/index.js';
	import { authStore } from '$lib/stores/auth.svelte';
	import type { LayoutProps } from './$types';

	// Get server-validated user data
	let { data, children }: LayoutProps = $props();

	// Command palette state
	let commandPaletteOpen = $state(false);

	// Sync authState from server-validated user data
	// This ensures client-side state matches server-side auth
	$effect(() => {
		if (data.user) {
			// Map server UserInfo to client User format
			const user = {
				id: data.user.userId,
				email: data.user.email,
				name: data.user.name ?? data.user.email,
				role: (data.user.role as 'owner' | 'admin' | 'manager' | 'user') ?? 'user',
				tenantId: data.user.tenantId ?? '',
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
				groups: data.user.groups
			};
			authStore.setUser(user);
		}
	});

	// Open command palette handler (passed to header)
	function openCommandPalette() {
		commandPaletteOpen = true;
	}
</script>

<!-- Authenticated Content - Server has already validated auth -->
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

<!-- Toast notifications -->
<Toaster />
