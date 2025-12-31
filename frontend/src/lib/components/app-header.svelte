<script lang="ts">
	import { page } from '$app/state';
	import SearchIcon from '@lucide/svelte/icons/search';
	import BellIcon from '@lucide/svelte/icons/bell';
	import * as Breadcrumb from '$lib/components/ui/breadcrumb/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import ThemeToggle from './theme-toggle.svelte';

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
</script>

<header
	class="flex h-14 shrink-0 items-center gap-2 border-b bg-background px-4 transition-[width,height] ease-linear group-has-[[data-collapsible=icon]]/sidebar-wrapper:h-12"
>
	<div class="flex items-center gap-2">
		<Sidebar.Trigger class="-ml-1" />
		<Separator orientation="vertical" class="mr-2 h-4" />

		<!-- Breadcrumbs -->
		<Breadcrumb.Root class="hidden md:flex">
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
	</div>

	<div class="ml-auto flex items-center gap-2">
		<!-- Search Button (triggers command palette) -->
		<Button variant="ghost" size="icon" class="size-8" aria-label="Search">
			<SearchIcon class="size-4" />
		</Button>

		<!-- Notifications -->
		<Button variant="ghost" size="icon" class="size-8" aria-label="Notifications">
			<BellIcon class="size-4" />
		</Button>

		<!-- Theme Toggle -->
		<ThemeToggle />
	</div>
</header>
