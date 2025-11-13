<script lang="ts">
	import { authState } from '$lib/stores/auth.svelte';
</script>

{#if authState.isLoading}
	<div class="auth-loading-overlay">
		<div class="auth-loading-spinner">
			<div class="spinner"></div>
			<p>Authenticating...</p>
		</div>
	</div>
{/if}

<style>
	.auth-loading-overlay {
		position: fixed;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		background: rgba(255, 255, 255, 0.9);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 9999;
	}

	.auth-loading-spinner {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1rem;
	}

	.spinner {
		width: 40px;
		height: 40px;
		border: 4px solid #f3f3f3;
		border-top: 4px solid #3498db;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}

	p {
		margin: 0;
		color: #666;
		font-size: 0.9rem;
	}
</style>
