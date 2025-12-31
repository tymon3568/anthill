<script lang="ts">
	import { goto } from '$app/navigation';
	import * as Command from '$lib/components/ui/command/index.js';
	import { mainNavigation, settingsNavigation } from '$lib/config/navigation';
	import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
	import SearchIcon from '@lucide/svelte/icons/search';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import { onMount } from 'svelte';

	interface Props {
		open?: boolean;
	}

	let { open = $bindable(false) }: Props = $props();

	let searchValue = $state('');

	// Handle keyboard shortcuts
	function handleKeydown(e: KeyboardEvent) {
		// Skip shortcuts when typing in input/textarea
		const target = e.target as HTMLElement;
		if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
			// Allow Ctrl+K even in inputs to open command palette
			if (!((e.metaKey || e.ctrlKey) && e.key === 'k')) {
				return;
			}
		}

		// Ctrl/Cmd+K - Toggle command palette
		if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
			e.preventDefault();
			open = !open;
			return;
		}

		// Shortcuts only when command palette is closed (to avoid conflicts)
		if (!open && (e.metaKey || e.ctrlKey)) {
			// Ctrl/Cmd+D - Go to Dashboard
			if (e.key === 'd') {
				e.preventDefault();
				navigateTo('/dashboard');
				return;
			}
			// Ctrl/Cmd+P - Search Products
			if (e.key === 'p') {
				e.preventDefault();
				navigateTo('/inventory/products');
				return;
			}
			// Ctrl/Cmd+, - Open Settings
			if (e.key === ',') {
				e.preventDefault();
				navigateTo('/settings');
				return;
			}
		}
		// Note: Escape key handling is done by Bits UI Dialog component
	}

	onMount(() => {
		// Add global keydown listener
		document.addEventListener('keydown', handleKeydown);
		return () => {
			document.removeEventListener('keydown', handleKeydown);
		};
	});

	// Navigate to a page and close the palette
	function navigateTo(url: string) {
		open = false;
		searchValue = '';
		goto(url);
	}

	// Filter items based on search
	let filteredMainNav = $derived(
		mainNavigation.filter(
			(item) =>
				item.title.toLowerCase().includes(searchValue.toLowerCase()) ||
				item.items?.some((sub) => sub.title.toLowerCase().includes(searchValue.toLowerCase()))
		)
	);

	let filteredSettingsNav = $derived(
		settingsNavigation.filter(
			(item) =>
				item.title.toLowerCase().includes(searchValue.toLowerCase()) ||
				item.items?.some((sub) => sub.title.toLowerCase().includes(searchValue.toLowerCase()))
		)
	);
</script>

<Command.Dialog
	bind:open
	bind:value={searchValue}
	title="Command Palette"
	description="Search and navigate to pages"
>
	<Command.Input placeholder="Type a command or search..." />
	<Command.List>
		<Command.Empty>No results found.</Command.Empty>

		<!-- Quick Actions -->
		<Command.Group heading="Quick Actions">
			<Command.Item onSelect={() => navigateTo('/dashboard')}>
				<LayoutDashboardIcon class="mr-2 size-4" />
				<span>Go to Dashboard</span>
				<Command.Shortcut>⌘D</Command.Shortcut>
			</Command.Item>
			<Command.Item onSelect={() => navigateTo('/inventory/products')}>
				<SearchIcon class="mr-2 size-4" />
				<span>Search Products</span>
				<Command.Shortcut>⌘P</Command.Shortcut>
			</Command.Item>
			<Command.Item onSelect={() => navigateTo('/settings')}>
				<SettingsIcon class="mr-2 size-4" />
				<span>Open Settings</span>
				<Command.Shortcut>⌘,</Command.Shortcut>
			</Command.Item>
		</Command.Group>

		<Command.Separator />

		<!-- Main Navigation -->
		{#if filteredMainNav.length > 0}
			<Command.Group heading="Navigation">
				{#each filteredMainNav as item (item.title)}
					<Command.Item onSelect={() => navigateTo(item.url)}>
						{#if item.icon}
							<item.icon class="mr-2 size-4" />
						{/if}
						<span>{item.title}</span>
					</Command.Item>
					{#if item.items}
						{#each item.items as subItem (subItem.url)}
							<Command.Item onSelect={() => navigateTo(subItem.url)} class="pl-8">
								<span class="text-muted-foreground">→</span>
								<span class="ml-2">{subItem.title}</span>
							</Command.Item>
						{/each}
					{/if}
				{/each}
			</Command.Group>
		{/if}

		<!-- Settings Navigation -->
		{#if filteredSettingsNav.length > 0}
			<Command.Separator />
			<Command.Group heading="Settings">
				{#each filteredSettingsNav as item (item.title)}
					<Command.Item onSelect={() => navigateTo(item.url)}>
						{#if item.icon}
							<item.icon class="mr-2 size-4" />
						{/if}
						<span>{item.title}</span>
					</Command.Item>
					{#if item.items}
						{#each item.items as subItem (subItem.url)}
							<Command.Item onSelect={() => navigateTo(subItem.url)} class="pl-8">
								<span class="text-muted-foreground">→</span>
								<span class="ml-2">{subItem.title}</span>
							</Command.Item>
						{/each}
					{/if}
				{/each}
			</Command.Group>
		{/if}
	</Command.List>
</Command.Dialog>
