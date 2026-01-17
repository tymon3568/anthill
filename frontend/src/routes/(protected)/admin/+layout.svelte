<script lang="ts">
	import { page } from '$app/state';
	import UsersIcon from '@lucide/svelte/icons/users';
	import ShieldIcon from '@lucide/svelte/icons/shield';
	import MailIcon from '@lucide/svelte/icons/mail';
	import SettingsIcon from '@lucide/svelte/icons/settings';

	let { children } = $props();

	// Admin navigation items
	const adminNavItems = [
		{
			title: 'Users',
			href: '/admin/users',
			icon: UsersIcon,
			description: 'Manage users in your organization'
		},
		{
			title: 'Roles',
			href: '/admin/roles',
			icon: ShieldIcon,
			description: 'Manage roles and permissions'
		},
		{
			title: 'Invitations',
			href: '/admin/invitations',
			icon: MailIcon,
			description: 'Manage user invitations'
		}
	] as const;

	// Check if a nav item is active
	function isActive(href: string): boolean {
		return page.url.pathname === href || page.url.pathname.startsWith(href + '/');
	}
</script>

<div class="space-y-6">
	<!-- Admin Header -->
	<div class="flex items-center justify-between">
		<div>
			<div class="flex items-center gap-2">
				<SettingsIcon class="h-6 w-6 text-muted-foreground" />
				<h1 class="text-2xl font-bold">Admin Console</h1>
			</div>
			<p class="text-muted-foreground">Manage users, roles, and organization settings</p>
		</div>
	</div>

	<!-- Admin Navigation Tabs -->
	<nav class="flex border-b" aria-label="Admin navigation">
		{#each adminNavItems as item (item.href)}
			<a
				href={item.href}
				class="flex items-center gap-2 border-b-2 px-4 py-3 text-sm font-medium transition-colors hover:text-foreground {isActive(
					item.href
				)
					? 'border-primary text-foreground'
					: 'border-transparent text-muted-foreground'}"
				aria-current={isActive(item.href) ? 'page' : undefined}
			>
				<item.icon class="h-4 w-4" />
				{item.title}
			</a>
		{/each}
	</nav>

	<!-- Admin Content -->
	<div>
		{@render children()}
	</div>
</div>
