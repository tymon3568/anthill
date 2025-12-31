<script lang="ts">
	import { page } from '$app/state';
	import { afterNavigate } from '$app/navigation';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import * as Collapsible from '$lib/components/ui/collapsible/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { isPathActive, hasActiveChild, type NavItem } from '$lib/config/navigation';

	interface Props {
		items: NavItem[];
	}

	let { items }: Props = $props();

	// Get sidebar context to close on mobile navigation
	const sidebar = Sidebar.useSidebar();

	// Track which collapsibles are open
	let openItems = $state<Record<string, boolean>>({});

	// Initialize open state based on current route
	$effect(() => {
		const currentPath = page.url.pathname;
		for (const item of items) {
			if (item.items && hasActiveChild(currentPath, item.items)) {
				openItems[item.title] = true;
			}
		}
	});

	// Auto-close sidebar on mobile after navigation
	afterNavigate(() => {
		if (sidebar.isMobile) {
			sidebar.setOpenMobile(false);
		}
	});

	function toggleItem(title: string) {
		openItems[title] = !openItems[title];
	}

	// Handle link click - close mobile sidebar
	function handleLinkClick() {
		if (sidebar.isMobile) {
			// Small delay to allow navigation to start
			setTimeout(() => {
				sidebar.setOpenMobile(false);
			}, 100);
		}
	}
</script>

<Sidebar.Group>
	<Sidebar.GroupLabel id="nav-main-label">Navigation</Sidebar.GroupLabel>
	<Sidebar.Menu aria-labelledby="nav-main-label" role="navigation">
		{#each items as item (item.title)}
			{@const currentPath = page.url.pathname}
			{@const isActive = isPathActive(currentPath, item.url)}
			{@const hasChildren = item.items && item.items.length > 0}
			{@const isOpen = openItems[item.title] ?? false}

			{#if hasChildren}
				<Collapsible.Root open={isOpen} onOpenChange={() => toggleItem(item.title)}>
					<Sidebar.MenuItem>
						<Collapsible.Trigger>
							{#snippet child({ props })}
								<Sidebar.MenuButton
									{...props}
									isActive={isActive || hasActiveChild(currentPath, item.items)}
									tooltipContent={item.title}
									class="min-h-[44px] md:min-h-0"
									aria-expanded={isOpen}
									aria-controls={`submenu-${item.title.toLowerCase().replace(/\s+/g, '-')}`}
								>
									{#if item.icon}
										<item.icon class="size-5 md:size-4" />
									{/if}
									<span>{item.title}</span>
									<ChevronRightIcon
										class="ml-auto size-4 shrink-0 transition-transform duration-200 {isOpen
											? 'rotate-90'
											: ''}"
									/>
								</Sidebar.MenuButton>
							{/snippet}
						</Collapsible.Trigger>
						<Collapsible.Content>
							<Sidebar.MenuSub
								id={`submenu-${item.title.toLowerCase().replace(/\s+/g, '-')}`}
								role="menu"
								aria-label={`${item.title} submenu`}
							>
								{#each item.items as subItem (subItem.url)}
									{@const subIsActive = isPathActive(currentPath, subItem.url)}
									<Sidebar.MenuSubItem role="none">
										<Sidebar.MenuSubButton
											href={subItem.url}
											isActive={subIsActive}
											onclick={handleLinkClick}
											class="min-h-[44px] md:min-h-0"
											role="menuitem"
											aria-current={subIsActive ? 'page' : undefined}
										>
											<span>{subItem.title}</span>
											{#if subItem.badge}
												<Sidebar.MenuBadge>{subItem.badge}</Sidebar.MenuBadge>
											{/if}
										</Sidebar.MenuSubButton>
									</Sidebar.MenuSubItem>
								{/each}
							</Sidebar.MenuSub>
						</Collapsible.Content>
					</Sidebar.MenuItem>
				</Collapsible.Root>
			{:else}
				<Sidebar.MenuItem>
					<Sidebar.MenuButton
						{isActive}
						tooltipContent={item.title}
						class="min-h-[44px] md:min-h-0"
					>
						{#snippet child({ props })}
							<a
								href={item.url}
								{...props}
								onclick={handleLinkClick}
								aria-current={isActive ? 'page' : undefined}
							>
								{#if item.icon}
									<item.icon class="size-5 md:size-4" />
								{/if}
								<span>{item.title}</span>
								{#if item.badge}
									<Sidebar.MenuBadge>{item.badge}</Sidebar.MenuBadge>
								{/if}
							</a>
						{/snippet}
					</Sidebar.MenuButton>
				</Sidebar.MenuItem>
			{/if}
		{/each}
	</Sidebar.Menu>
</Sidebar.Group>
