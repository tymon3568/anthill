<!-- ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
     - Use $state, $derived, $effect, $props (NOT legacy syntax)
     - Always consult MCP documentation before changes
     - See .svelte-instructions.md for guidelines -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import LoadingSpinner from '$lib/components/LoadingSpinner.svelte';
	import { authState } from '$lib/stores/auth.svelte';
	import { authApi } from '$lib/api/auth';
	import { tokenManager } from '$lib/auth/token-manager';
	import { authStore } from '$lib/stores/auth.svelte';

	let { children } = $props();

	// Access auth state directly (don't call useAuth() again - already initialized in root layout)
	const user = $derived(authState.user);
	const isAuthenticated = $derived(authState.isAuthenticated);
	const isLoading = $derived(authState.isLoading);

	// Logout function
	const logout = async () => {
		// Call backend to revoke refresh token
		const refreshToken = await tokenManager.getRefreshToken();
		if (refreshToken) {
			try {
				await authApi.logoutLegacy();
			} catch (error) {
				console.error('Logout API call failed:', error);
			}
		}

		// Clear all tokens and user data
		tokenManager.clearAll();
		authStore.logout();

		// Redirect to login
		goto('/login');
	};

	// Redirect to login if not authenticated
	$effect(() => {
		if (!isLoading && !isAuthenticated) {
			goto('/login');
		}
	});
</script>

{#if isLoading}
	<div class="flex min-h-screen items-center justify-center">
		<div class="h-8 w-8 animate-spin rounded-full border-b-2 border-primary"></div>
	</div>
{:else if isAuthenticated}
	<div class="min-h-screen bg-background">
		<!-- Header -->
		<header class="border-b bg-card">
			<div class="container mx-auto flex items-center justify-between px-4 py-4">
				<h1 class="text-2xl font-bold">Anthill Inventory</h1>
				<div class="flex items-center gap-4">
					<span class="text-sm text-muted-foreground">Welcome, {user?.name}</span>
					<button
						onclick={logout}
						class="text-destructive-foreground rounded-md bg-destructive px-4 py-2 text-sm hover:bg-destructive/90"
					>
						Logout
					</button>
				</div>
			</div>
		</header>

		<!-- Main Content -->
		<main class="container mx-auto px-4 py-8">
			{@render children()}
		</main>
	</div>
{/if}
