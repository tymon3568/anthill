<script lang="ts">
	import { page } from '$app/state';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import * as Collapsible from '$lib/components/ui/collapsible/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { isPathActive, hasActiveChild, type NavItem } from '$lib/config/navigation';

	interface Props {
		items: NavItem[];
	}

	let { items }: Props = $props();

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

	function toggleItem(title: string) {
		openItems[title] = !openItems[title];
	}
</script>

<Sidebar.Group>
	<Sidebar.GroupLabel>Navigation</Sidebar.GroupLabel>
	<Sidebar.Menu>
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
								>
									{#if item.icon}
										<item.icon class="size-4" />
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
							<Sidebar.MenuSub>
								{#each item.items as subItem (subItem.url)}
									{@const subIsActive = isPathActive(currentPath, subItem.url)}
									<Sidebar.MenuSubItem>
										<Sidebar.MenuSubButton href={subItem.url} isActive={subIsActive}>
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
					<Sidebar.MenuButton {isActive} tooltipContent={item.title}>
						{#snippet child({ props })}
							<a href={item.url} {...props}>
								{#if item.icon}
									<item.icon class="size-4" />
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
