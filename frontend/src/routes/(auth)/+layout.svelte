<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { AuthSession } from '$lib/auth/session';

	// Redirect authenticated users away from auth pages
	onMount(() => {
		if (AuthSession.isAuthenticated()) {
			goto('/dashboard');
		}
	});
</script>

<slot />
