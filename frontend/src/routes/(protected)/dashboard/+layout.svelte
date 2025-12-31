<!-- ⚠️ CRITICAL: This project uses Svelte 5 runes exclusively.
     - Use $state, $derived, $effect, $props (NOT legacy syntax)
     - Always consult MCP documentation before changes
     - See .svelte-instructions.md for guidelines -->
<script lang="ts">
	let { children } = $props();
</script>

<!-- Dashboard content is rendered within the parent protected layout's sidebar structure -->
{@render children()}
