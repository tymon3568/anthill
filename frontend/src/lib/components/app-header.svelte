<script lang="ts">
	import { page } from '$app/state';
	import SearchIcon from '@lucide/svelte/icons/search';
	import BellIcon from '@lucide/svelte/icons/bell';
	import MenuIcon from '@lucide/svelte/icons/menu';
	import * as Breadcrumb from '$lib/components/ui/breadcrumb/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import ThemeToggle from './theme-toggle.svelte';

	interface Props {
		onSearchClick?: () => void;
	}

	let { onSearchClick }: Props = $props();

	// Generate breadcrumbs from current path
	function getBreadcrumbs(pathname: string): { label: string; href: string }[] {
		const segments = pathname.split('/').filter(Boolean);
		const crumbs: { label: string; href: string }[] = [];

		let currentPath = '';
		for (const segment of segments) {
			currentPath += `/${segment}`;
			// Capitalize first letter and replace hyphens with spaces
			const label = segment
				.split('-')
				.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
				.join(' ');
			crumbs.push({ label, href: currentPath });
		}

		return crumbs;
	}

	let breadcrumbs = $derived(getBreadcrumbs(page.url.pathname));

	// Get current page title for mobile header
	let currentPageTitle = $derived(
		breadcrumbs.length > 0 ? breadcrumbs[breadcrumbs.length - 1].label : 'Dashboard'
	);

	// Get sidebar context for mobile toggle
	const sidebar = Sidebar.useSidebar();

	function handleMobileMenuClick() {
		sidebar.toggle();
	}

	function handleSearchClick() {
		onSearchClick?.();
	}
</script>

<header
	class="flex h-14 shrink-0 items-center gap-2 border-b bg-background px-3 transition-[width,height] ease-linear group-has-[[data-collapsible=icon]]/sidebar-wrapper:h-12 md:px-4"
>
	<div class="flex flex-1 items-center gap-2">
		<!-- Mobile: Hamburger menu trigger -->
		<Button
			variant="ghost"
			size="icon"
			class="-ml-1 size-9 md:hidden"
			aria-label="Open navigation menu"
			onclick={handleMobileMenuClick}
		>
			<MenuIcon class="size-5" />
			<span class="sr-only">Toggle navigation menu</span>
		</Button>

		<!-- Desktop: Default sidebar trigger -->
		<Sidebar.Trigger class="-ml-1 hidden md:flex" aria-label="Toggle sidebar (Ctrl+B)" />

		<Separator orientation="vertical" class="mr-2 hidden h-4 md:block" />

		<!-- Desktop: Breadcrumbs -->
		<Breadcrumb.Root class="hidden md:flex" aria-label="Breadcrumb navigation">
			<Breadcrumb.List>
				{#each breadcrumbs as crumb, index (crumb.href)}
					<Breadcrumb.Item>
						{#if index === breadcrumbs.length - 1}
							<Breadcrumb.Page>{crumb.label}</Breadcrumb.Page>
						{:else}
							<Breadcrumb.Link href={crumb.href}>{crumb.label}</Breadcrumb.Link>
						{/if}
					</Breadcrumb.Item>
					{#if index < breadcrumbs.length - 1}
						<Breadcrumb.Separator />
					{/if}
				{/each}
			</Breadcrumb.List>
		</Breadcrumb.Root>

		<!-- Mobile: Current page title -->
		<h1 class="truncate text-base font-semibold md:hidden">
			{currentPageTitle}
		</h1>
	</div>

	<div class="flex items-center gap-1 md:gap-2">
		<!-- Search Button (triggers command palette) -->
		<Button
			variant="ghost"
			size="icon"
			class="size-9 md:size-8"
			aria-label="Search (Ctrl+K)"
			onclick={handleSearchClick}
		>
			<SearchIcon class="size-5 md:size-4" />
		</Button>

		<!-- Notifications -->
		<Button variant="ghost" size="icon" class="size-9 md:size-8" aria-label="Notifications">
			<BellIcon class="size-5 md:size-4" />
		</Button>

		<!-- Theme Toggle -->
		<ThemeToggle />
	</div>
</header>
