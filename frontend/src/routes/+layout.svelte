<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { useAuth } from '$lib/hooks/useAuth';

	let { children } = $props();

	// Initialize auth state for entire app
	useAuth();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<AuthLoading />
{@render children()}
