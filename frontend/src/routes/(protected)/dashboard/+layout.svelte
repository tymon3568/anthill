<!-- ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
     - Use $state, $derived, $effect, $props (NOT legacy syntax)
     - Always consult MCP documentation before changes
     - See .svelte-instructions.md for guidelines -->
<script lang="ts">
	import { useAuth } from '$lib/hooks/useAuth';
	import { goto } from '$app/navigation';

	let { children } = $props();

	const { user, isAuthenticated, isLoading, logout } = useAuth();

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
