<!-- ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
     - Use $state, $derived, $effect, $props (NOT legacy syntax)
     - Always consult MCP documentation before changes
     - See .svelte-instructions.md for guidelines -->
<script lang="ts">
	interface Props {
		size?: 'sm' | 'md' | 'lg';
		class?: string;
	}

	let { size = 'md', class: className = '' }: Props = $props();

	const sizeClasses = {
		sm: 'h-4 w-4 border-2',
		md: 'h-8 w-8 border-2',
		lg: 'h-12 w-12 border-4'
	};
</script>

<div
	class="animate-spin rounded-full border-b-primary border-t-transparent {sizeClasses[
		size
	]} {className}"
	role="status"
	aria-label="Loading"
>
	<span class="sr-only">Loading...</span>
</div>
