<script lang="ts">
	import MoonIcon from '@lucide/svelte/icons/moon';
	import SunIcon from '@lucide/svelte/icons/sun';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { themeStore } from '$lib/stores/theme.svelte';
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="ghost" size="icon" class="size-8">
				{#if themeStore.isDark}
					<MoonIcon class="size-4" />
				{:else}
					<SunIcon class="size-4" />
				{/if}
				<span class="sr-only">Toggle theme</span>
			</Button>
		{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end">
		<DropdownMenu.Item onclick={() => themeStore.setLight()}>
			<SunIcon class="mr-2 size-4" />
			<span>Light</span>
			{#if themeStore.theme === 'light'}
				<span class="ml-auto text-xs text-muted-foreground">✓</span>
			{/if}
		</DropdownMenu.Item>
		<DropdownMenu.Item onclick={() => themeStore.setDark()}>
			<MoonIcon class="mr-2 size-4" />
			<span>Dark</span>
			{#if themeStore.theme === 'dark'}
				<span class="ml-auto text-xs text-muted-foreground">✓</span>
			{/if}
		</DropdownMenu.Item>
		<DropdownMenu.Item onclick={() => themeStore.setSystem()}>
			<MonitorIcon class="mr-2 size-4" />
			<span>System</span>
			{#if themeStore.theme === 'system'}
				<span class="ml-auto text-xs text-muted-foreground">✓</span>
			{/if}
		</DropdownMenu.Item>
	</DropdownMenu.Content>
</DropdownMenu.Root>
