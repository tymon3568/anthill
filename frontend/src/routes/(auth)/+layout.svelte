<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { AuthSession } from '$lib/auth/session';

	// Redirect authenticated users away from auth pages
	// But first validate with server to handle stale localStorage data
	onMount(async () => {
		// First check if session_invalidated cookie is set (server signaled stale session)
		// This is handled inside isAuthenticated() now
		if (!AuthSession.isAuthenticated()) {
			// No local session data or session was invalidated
			return;
		}

		// localStorage has user data, but we need to verify with server
		// that the session is still valid (user might have been deleted)
		try {
			const response = await fetch('/api/v1/auth/validate', {
				method: 'GET',
				credentials: 'include'
			});

			if (response.ok) {
				// Session is valid, redirect to dashboard
				goto('/dashboard');
			} else {
				// Session is invalid (401/403/404), clear local state
				AuthSession.clearSession();
				localStorage.removeItem('anthill_tenant_slug');
				console.log('[auth-layout] Session validation failed, cleared local state');
			}
		} catch {
			// Network error - don't clear session, let user try again
			console.error('[auth-layout] Session validation request failed');
		}
	});
</script>

<slot />
