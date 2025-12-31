<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import NavMain from './nav-main.svelte';
	import NavUser from './nav-user.svelte';
	import { mainNavigation, settingsNavigation } from '$lib/config/navigation';
	import BoxesIcon from '@lucide/svelte/icons/boxes';

	interface Props {
		variant?: 'sidebar' | 'floating' | 'inset';
		collapsible?: 'offcanvas' | 'icon' | 'none';
	}

	let { variant = 'sidebar', collapsible = 'icon' }: Props = $props();

	// Get sidebar context for mobile handling
	const sidebar = Sidebar.useSidebar();

	// Auto-close mobile sidebar on navigation
	afterNavigate(() => {
		if (sidebar.isMobile) {
			sidebar.setOpenMobile(false);
		}
	});

	// Handle link click on mobile - close sidebar
	function handleMobileLinkClick() {
		if (sidebar.isMobile) {
			setTimeout(() => {
				sidebar.setOpenMobile(false);
			}, 100);
		}
	}
</script>

<Sidebar.Root {variant} {collapsible} aria-label="Main navigation sidebar">
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton
					size="lg"
					class="min-h-[52px] md:min-h-0"
					aria-label="Anthill - Go to Dashboard"
				>
					{#snippet child({ props })}
						<a href="/dashboard" {...props} onclick={handleMobileLinkClick}>
							<div
								class="flex aspect-square size-9 items-center justify-center rounded-lg bg-primary text-primary-foreground md:size-8"
							>
								<BoxesIcon class="size-5 md:size-4" />
							</div>
							<div class="grid flex-1 text-left text-sm leading-tight">
								<span class="truncate font-semibold">Anthill</span>
								<span class="truncate text-xs text-muted-foreground">Inventory Management</span>
							</div>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>

	<Sidebar.Content>
		<!-- Main Navigation -->
		<NavMain items={mainNavigation} />

		<!-- Settings Navigation -->
		<Sidebar.Group class="mt-auto">
			<Sidebar.GroupLabel id="settings-nav-label">Settings</Sidebar.GroupLabel>
			<Sidebar.Menu aria-labelledby="settings-nav-label" role="navigation">
				{#each settingsNavigation as item (item.title)}
					<Sidebar.MenuItem>
						<Sidebar.MenuButton
							tooltipContent={item.title}
							class="min-h-[44px] md:min-h-0"
							aria-label={item.title}
						>
							{#snippet child({ props })}
								<a href={item.url} {...props} onclick={handleMobileLinkClick}>
									{#if item.icon}
										<item.icon class="size-5 md:size-4" />
									{/if}
									<span>{item.title}</span>
								</a>
							{/snippet}
						</Sidebar.MenuButton>
					</Sidebar.MenuItem>
				{/each}
			</Sidebar.Menu>
		</Sidebar.Group>
	</Sidebar.Content>

	<Sidebar.Rail />

	<!-- User Profile -->
	<NavUser />
</Sidebar.Root>
