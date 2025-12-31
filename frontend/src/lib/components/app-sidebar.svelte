<script lang="ts">
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
</script>

<Sidebar.Root {variant} {collapsible}>
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton size="lg">
					{#snippet child({ props })}
						<a href="/dashboard" {...props}>
							<div
								class="flex aspect-square size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground"
							>
								<BoxesIcon class="size-4" />
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
			<Sidebar.GroupLabel>Settings</Sidebar.GroupLabel>
			<Sidebar.Menu>
				{#each settingsNavigation as item (item.title)}
					<Sidebar.MenuItem>
						<Sidebar.MenuButton tooltipContent={item.title}>
							{#snippet child({ props })}
								<a href={item.url} {...props}>
									{#if item.icon}
										<item.icon class="size-4" />
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
