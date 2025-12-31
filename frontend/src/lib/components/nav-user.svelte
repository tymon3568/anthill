<script lang="ts">
	import { goto } from '$app/navigation';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import LogOutIcon from '@lucide/svelte/icons/log-out';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import UserIcon from '@lucide/svelte/icons/user';
	import * as Avatar from '$lib/components/ui/avatar/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { authState, authStore } from '$lib/stores/auth.svelte';

	// Get user initials for avatar fallback
	function getInitials(name: string | undefined): string {
		if (!name) return 'U';
		const parts = name.split(' ');
		if (parts.length >= 2) {
			return `${parts[0][0]}${parts[1][0]}`.toUpperCase();
		}
		return name.slice(0, 2).toUpperCase();
	}

	async function handleLogout() {
		await authStore.emailLogout();
		goto('/login');
	}

	function handleSettings() {
		goto('/settings/profile');
	}
</script>

<Sidebar.Footer>
	<Sidebar.Menu>
		<Sidebar.MenuItem>
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Sidebar.MenuButton
							{...props}
							size="lg"
							class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
						>
							<Avatar.Root class="size-8 rounded-lg">
								<Avatar.Fallback class="rounded-lg">
									{getInitials(authState.user?.name)}
								</Avatar.Fallback>
							</Avatar.Root>
							<div class="grid flex-1 text-left text-sm leading-tight">
								<span class="truncate font-semibold">
									{authState.user?.name ?? 'User'}
								</span>
								<span class="truncate text-xs text-muted-foreground">
									{authState.user?.email ?? ''}
								</span>
							</div>
							<ChevronsUpDownIcon class="ml-auto size-4" />
						</Sidebar.MenuButton>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content
					class="w-[--bits-dropdown-menu-anchor-width] min-w-56 rounded-lg"
					side="bottom"
					align="end"
					sideOffset={4}
				>
					<DropdownMenu.Label class="p-0 font-normal">
						<div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
							<Avatar.Root class="size-8 rounded-lg">
								<Avatar.Fallback class="rounded-lg">
									{getInitials(authState.user?.name)}
								</Avatar.Fallback>
							</Avatar.Root>
							<div class="grid flex-1 text-left text-sm leading-tight">
								<span class="truncate font-semibold">
									{authState.user?.name ?? 'User'}
								</span>
								<span class="truncate text-xs text-muted-foreground">
									{authState.user?.email ?? ''}
								</span>
							</div>
						</div>
					</DropdownMenu.Label>
					<DropdownMenu.Separator />
					{#if authState.tenant}
						<DropdownMenu.Group>
							<DropdownMenu.Item disabled>
								<span class="text-xs text-muted-foreground">
									Organization: {authState.tenant.name}
								</span>
							</DropdownMenu.Item>
						</DropdownMenu.Group>
						<DropdownMenu.Separator />
					{/if}
					<DropdownMenu.Group>
						<DropdownMenu.Item onclick={handleSettings}>
							<UserIcon class="mr-2 size-4" />
							Profile
						</DropdownMenu.Item>
						<DropdownMenu.Item onclick={() => goto('/settings')}>
							<SettingsIcon class="mr-2 size-4" />
							Settings
						</DropdownMenu.Item>
					</DropdownMenu.Group>
					<DropdownMenu.Separator />
					<DropdownMenu.Item onclick={handleLogout}>
						<LogOutIcon class="mr-2 size-4" />
						Log out
					</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		</Sidebar.MenuItem>
	</Sidebar.Menu>
</Sidebar.Footer>
