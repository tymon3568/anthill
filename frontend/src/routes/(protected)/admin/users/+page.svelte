<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import { Separator } from '$lib/components/ui/separator';
	import { userServiceApi } from '$lib/api/user-service';
	import type {
		User,
		Role,
		CreateUserRequest,
		ListUsersParams,
		UserStatus
	} from '$lib/api/types/user-service.types';
	import { toast } from 'svelte-sonner';
	import { onMount } from 'svelte';
	import UsersIcon from '@lucide/svelte/icons/users';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SearchIcon from '@lucide/svelte/icons/search';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import ShieldIcon from '@lucide/svelte/icons/shield';
	import BanIcon from '@lucide/svelte/icons/ban';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import XIcon from '@lucide/svelte/icons/x';
	import LoaderIcon from '@lucide/svelte/icons/loader';

	// State
	let users = $state<User[]>([]);
	let roles = $state<Role[]>([]);
	let isLoading = $state(true);
	let isLoadingRoles = $state(true);
	let totalUsers = $state(0);
	let currentPage = $state(1);
	let perPage = $state(10);
	let totalPages = $state(1);
	let searchQuery = $state('');
	let filterStatus = $state<UserStatus | ''>('');
	let filterRole = $state('');

	// Dialog states
	let showAddUserDialog = $state(false);
	let showAssignRoleDialog = $state(false);
	let showConfirmDialog = $state(false);
	let confirmAction = $state<{ type: 'suspend' | 'unsuspend' | 'delete'; user: User } | null>(null);
	let selectedUser = $state<User | null>(null);
	let userRoles = $state<string[]>([]);
	let isLoadingUserRoles = $state(false);
	let isSubmitting = $state(false);

	// Add user form
	let newUserForm = $state<CreateUserRequest>({
		email: '',
		password: '',
		fullName: '',
		role: 'user'
	});

	// Load users on mount
	onMount(async () => {
		await Promise.all([loadUsers(), loadRoles()]);
	});

	async function loadUsers() {
		isLoading = true;
		try {
			const params: ListUsersParams = {
				page: currentPage,
				perPage: perPage
			};
			if (searchQuery) params.search = searchQuery;
			if (filterStatus) params.status = filterStatus;
			if (filterRole) params.role = filterRole;

			const response = await userServiceApi.listUsers(params);
			if (response.success && response.data) {
				users = response.data.data;
				totalUsers = response.data.total;
				totalPages = response.data.totalPages;
			} else {
				toast.error(response.error || 'Failed to load users');
			}
		} catch (error) {
			console.error('Failed to load users:', error);
			toast.error('Failed to load users');
		} finally {
			isLoading = false;
		}
	}

	async function loadRoles() {
		isLoadingRoles = true;
		try {
			const response = await userServiceApi.listRoles();
			if (response.success && response.data) {
				roles = response.data;
			}
		} catch (error) {
			console.error('Failed to load roles:', error);
			toast.error('Failed to load roles');
		} finally {
			isLoadingRoles = false;
		}
	}

	async function handleSearch() {
		currentPage = 1;
		await loadUsers();
	}

	async function handlePageChange(page: number) {
		currentPage = page;
		await loadUsers();
	}

	async function handleCreateUser() {
		if (!newUserForm.email || !newUserForm.password || !newUserForm.fullName) {
			toast.error('Please fill in all required fields');
			return;
		}

		isSubmitting = true;
		try {
			const response = await userServiceApi.createUser(newUserForm);
			if (response.success) {
				toast.success('User created successfully');
				showAddUserDialog = false;
				newUserForm = { email: '', password: '', fullName: '', role: 'user' };
				await loadUsers();
			} else {
				// Show specific backend validation errors
				toast.error(response.error || 'Failed to create user');
			}
		} catch (error) {
			console.error('Failed to create user:', error);
			toast.error('Failed to create user');
		} finally {
			isSubmitting = false;
		}
	}

	async function handleSuspendUser(user: User) {
		confirmAction = { type: 'suspend', user };
		showConfirmDialog = true;
	}

	async function handleUnsuspendUser(user: User) {
		confirmAction = { type: 'unsuspend', user };
		showConfirmDialog = true;
	}

	async function handleDeleteUser(user: User) {
		confirmAction = { type: 'delete', user };
		showConfirmDialog = true;
	}

	async function executeConfirmAction() {
		if (!confirmAction) return;

		isSubmitting = true;
		try {
			let response;
			switch (confirmAction.type) {
				case 'suspend':
					response = await userServiceApi.suspendUser(confirmAction.user.id);
					break;
				case 'unsuspend':
					response = await userServiceApi.unsuspendUser(confirmAction.user.id);
					break;
				case 'delete':
					response = await userServiceApi.deleteUser(confirmAction.user.id);
					break;
			}

			if (response?.success) {
				toast.success(
					`User ${confirmAction.type === 'delete' ? 'deleted' : confirmAction.type === 'suspend' ? 'suspended' : 'unsuspended'} successfully`
				);
				showConfirmDialog = false;
				confirmAction = null;
				await loadUsers();
			} else {
				toast.error(response?.error || `Failed to ${confirmAction.type} user`);
			}
		} catch (error) {
			console.error(`Failed to ${confirmAction?.type} user:`, error);
			toast.error(`Failed to ${confirmAction?.type} user`);
		} finally {
			isSubmitting = false;
		}
	}

	function openAssignRoleDialog(user: User) {
		selectedUser = user;
		showAssignRoleDialog = true;
		isLoadingUserRoles = true;
		userRoles = [];
		// Fetch user roles after dialog opens
		userServiceApi
			.getUserRoles(user.id)
			.then((response) => {
				if (response.success && response.data) {
					userRoles = response.data.roles;
				}
			})
			.catch((error) => {
				console.error('Failed to load user roles:', error);
				toast.error('Failed to load user roles');
			})
			.finally(() => {
				isLoadingUserRoles = false;
			});
	}

	async function handleAssignRole(roleName: string) {
		if (!selectedUser) return;

		isSubmitting = true;
		try {
			const response = await userServiceApi.assignRole(selectedUser.id, roleName);
			if (response.success) {
				toast.success(`Role "${roleName}" assigned successfully`);
				// Update local userRoles state to reflect the change
				userRoles = [...userRoles, roleName];
				await loadUsers();
			} else {
				// Handle specific error cases
				if (response.error && response.error.includes('already has role')) {
					toast.error(`User already has the "${roleName}" role`);
				} else {
					toast.error(response.error || 'Failed to assign role');
				}
			}
		} catch (error) {
			console.error('Failed to assign role:', error);
			toast.error('Failed to assign role');
		} finally {
			isSubmitting = false;
		}
	}

	async function handleRemoveRole(roleName: string) {
		if (!selectedUser) return;

		// Prevent removing the last role
		if (userRoles.length <= 1) {
			toast.error("Cannot remove user's only role");
			return;
		}

		// Prevent removing protected 'owner' role via UI
		if (roleName === 'owner') {
			toast.error('Cannot remove the owner role');
			return;
		}

		isSubmitting = true;
		try {
			const response = await userServiceApi.removeRole(selectedUser.id, roleName);
			if (response.success) {
				toast.success(`Role "${roleName}" removed successfully`);
				// Update local userRoles state to reflect the change
				userRoles = userRoles.filter((r) => r !== roleName);
				await loadUsers();
			} else {
				toast.error(response.error || 'Failed to remove role');
			}
		} catch (error) {
			console.error('Failed to remove role:', error);
			toast.error('Failed to remove role');
		} finally {
			isSubmitting = false;
		}
	}

	function getStatusBadgeVariant(
		status: UserStatus
	): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (status) {
			case 'active':
				return 'default';
			case 'suspended':
				return 'destructive';
			case 'inactive':
				return 'secondary';
			default:
				return 'outline';
		}
	}

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}
</script>

<svelte:head>
	<title>User Management - Admin - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<Card>
		<CardHeader>
			<div class="flex items-center justify-between">
				<div>
					<CardTitle class="flex items-center gap-2">
						<UsersIcon class="h-5 w-5" />
						User Management
					</CardTitle>
					<CardDescription>
						Manage users in your organization. Create, suspend, or delete users and assign roles.
					</CardDescription>
				</div>
				<Button onclick={() => (showAddUserDialog = true)}>
					<PlusIcon class="mr-2 h-4 w-4" />
					Add User
				</Button>
			</div>
		</CardHeader>
		<CardContent class="space-y-4">
			<!-- Search and Filters -->
			<div class="flex gap-4">
				<div class="relative flex-1">
					<SearchIcon
						class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground"
					/>
					<Input
						id="search"
						name="search"
						type="search"
						placeholder="Search by name or email..."
						class="pl-10"
						bind:value={searchQuery}
						onkeydown={(e) => e.key === 'Enter' && handleSearch()}
					/>
				</div>
				<Select.Root
					type="single"
					name="statusFilter"
					value={filterStatus}
					onValueChange={(v) => {
						filterStatus = v as UserStatus | '';
						handleSearch();
					}}
				>
					<Select.Trigger class="w-40">
						{filterStatus
							? filterStatus.charAt(0).toUpperCase() + filterStatus.slice(1)
							: 'All Status'}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="">All Status</Select.Item>
						<Select.Item value="active">Active</Select.Item>
						<Select.Item value="suspended">Suspended</Select.Item>
					</Select.Content>
				</Select.Root>
				<Button variant="outline" onclick={handleSearch}>
					<RefreshCwIcon class="mr-2 h-4 w-4" />
					Refresh
				</Button>
			</div>

			<Separator />

			<!-- Users Table -->
			{#if isLoading}
				<div class="space-y-3">
					{#each [0, 1, 2, 3, 4] as i (i)}
						<div class="h-12 animate-pulse rounded bg-muted"></div>
					{/each}
				</div>
			{:else if users.length === 0}
				<div class="py-12 text-center">
					<UsersIcon class="mx-auto h-12 w-12 text-muted-foreground" />
					<h3 class="mt-4 text-lg font-medium">No users found</h3>
					<p class="mt-2 text-sm text-muted-foreground">
						{searchQuery ? 'Try adjusting your search criteria.' : 'Get started by adding a user.'}
					</p>
				</div>
			{:else}
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>Name</Table.Head>
							<Table.Head>Email</Table.Head>
							<Table.Head>Role</Table.Head>
							<Table.Head>Status</Table.Head>
							<Table.Head>Created</Table.Head>
							<Table.Head class="text-right">Actions</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each users as user (user.id)}
							<Table.Row>
								<Table.Cell class="font-medium">{user.fullName}</Table.Cell>
								<Table.Cell>{user.email}</Table.Cell>
								<Table.Cell>
									<div class="flex flex-wrap gap-1">
										{#each user.roles as roleName (roleName)}
											<Badge variant="outline">{roleName}</Badge>
										{:else}
											<Badge variant="outline">{user.role}</Badge>
										{/each}
									</div>
								</Table.Cell>
								<Table.Cell>
									<Badge variant={getStatusBadgeVariant(user.status)}>{user.status}</Badge>
								</Table.Cell>
								<Table.Cell>{formatDate(user.createdAt)}</Table.Cell>
								<Table.Cell class="text-right">
									<div class="flex justify-end gap-2">
										<Button
											variant="ghost"
											size="sm"
											onclick={() => openAssignRoleDialog(user)}
											title="Assign Role"
										>
											<ShieldIcon class="h-4 w-4" />
										</Button>
										{#if user.status === 'active'}
											<Button
												variant="ghost"
												size="sm"
												onclick={() => handleSuspendUser(user)}
												title="Suspend User"
											>
												<BanIcon class="h-4 w-4" />
											</Button>
										{:else if user.status === 'suspended'}
											<Button
												variant="ghost"
												size="sm"
												onclick={() => handleUnsuspendUser(user)}
												title="Unsuspend User"
											>
												<RefreshCwIcon class="h-4 w-4" />
											</Button>
										{/if}
										<Button
											variant="ghost"
											size="sm"
											onclick={() => handleDeleteUser(user)}
											title="Delete User"
										>
											<TrashIcon class="h-4 w-4 text-destructive" />
										</Button>
									</div>
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>

				<!-- Pagination -->
				{#if totalPages > 1}
					<div class="flex items-center justify-between">
						<p class="text-sm text-muted-foreground">
							Showing {(currentPage - 1) * perPage + 1} to {Math.min(
								currentPage * perPage,
								totalUsers
							)}
							of {totalUsers} users
						</p>
						<div class="flex items-center gap-2">
							<Button
								variant="outline"
								size="sm"
								disabled={currentPage === 1}
								onclick={() => handlePageChange(currentPage - 1)}
							>
								<ChevronLeftIcon class="h-4 w-4" />
								Previous
							</Button>
							<span class="text-sm">Page {currentPage} of {totalPages}</span>
							<Button
								variant="outline"
								size="sm"
								disabled={currentPage === totalPages}
								onclick={() => handlePageChange(currentPage + 1)}
							>
								Next
								<ChevronRightIcon class="h-4 w-4" />
							</Button>
						</div>
					</div>
				{/if}
			{/if}
		</CardContent>
	</Card>
</div>

<!-- Add User Dialog -->
<Dialog.Root
	bind:open={showAddUserDialog}
	onOpenChange={(open) => {
		if (!open) newUserForm = { email: '', password: '', fullName: '', role: 'user' };
	}}
>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Add New User</Dialog.Title>
			<Dialog.Description>Create a new user account in your organization.</Dialog.Description>
		</Dialog.Header>
		<form class="space-y-4 py-4">
			<div class="space-y-2">
				<Label for="fullName">Full Name *</Label>
				<Input
					id="fullName"
					name="fullName"
					bind:value={newUserForm.fullName}
					placeholder="John Doe"
					required
				/>
			</div>
			<div class="space-y-2">
				<Label for="email">Email *</Label>
				<Input
					id="email"
					name="email"
					type="email"
					bind:value={newUserForm.email}
					placeholder="john@example.com"
					required
				/>
			</div>
			<div class="space-y-2">
				<Label for="password">Password *</Label>
				<Input
					id="password"
					name="password"
					type="password"
					bind:value={newUserForm.password}
					placeholder="••••••••"
					autocomplete="new-password"
					required
				/>
				<p class="text-xs text-muted-foreground">
					Password must be at least 8 characters with uppercase, lowercase, and numbers.
				</p>
			</div>
			<div class="space-y-2">
				<Label for="role">Role</Label>
				<Select.Root
					type="single"
					name="role"
					value={newUserForm.role}
					onValueChange={(v) => (newUserForm.role = v)}
				>
					<Select.Trigger id="role" class="w-full">
						{newUserForm.role.charAt(0).toUpperCase() + newUserForm.role.slice(1)}
					</Select.Trigger>
					<Select.Content>
						{#if isLoadingRoles}
							<Select.Item value="user" disabled>Loading...</Select.Item>
						{:else}
							{#each roles as role (role.role_name)}
								<Select.Item value={role.role_name}>
									{role.role_name.charAt(0).toUpperCase() + role.role_name.slice(1)}
								</Select.Item>
							{/each}
						{/if}
					</Select.Content>
				</Select.Root>
			</div>
		</form>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showAddUserDialog = false)}>Cancel</Button>
			<Button onclick={handleCreateUser} disabled={isSubmitting}>
				{isSubmitting ? 'Creating...' : 'Create User'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Assign Role Dialog -->
<Dialog.Root
	bind:open={showAssignRoleDialog}
	onOpenChange={(open) => {
		if (!open) {
			selectedUser = null;
			userRoles = [];
		}
	}}
>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Manage Roles</Dialog.Title>
			<Dialog.Description>
				{#if selectedUser}
					Assign or remove roles for {selectedUser.fullName}
				{/if}
			</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-4 py-4">
			{#if isLoadingRoles || isLoadingUserRoles}
				<div class="flex items-center justify-center py-4">
					<LoaderIcon class="h-6 w-6 animate-spin text-muted-foreground" />
				</div>
			{:else}
				<!-- Current Assigned Roles -->
				{#if userRoles.length > 0}
					<div class="space-y-2">
						<Label class="text-sm font-medium">Current Roles</Label>
						<div class="flex flex-wrap gap-2">
							{#each userRoles as roleName (roleName)}
								{@const isProtectedFromRemoval = roleName === 'owner' || userRoles.length <= 1}
								<Badge variant="default" class="flex items-center gap-1 py-1">
									{roleName.charAt(0).toUpperCase() + roleName.slice(1)}
									<button
										type="button"
										class="ml-1 rounded-full p-0.5 hover:bg-primary-foreground/20 disabled:cursor-not-allowed disabled:opacity-50"
										onclick={() => handleRemoveRole(roleName)}
										disabled={isSubmitting || isProtectedFromRemoval}
										title={isProtectedFromRemoval
											? roleName === 'owner'
												? 'Cannot remove owner role'
												: "Cannot remove user's only role"
											: `Remove ${roleName} role`}
									>
										<XIcon class="h-3 w-3" />
									</button>
								</Badge>
							{/each}
						</div>
					</div>
					<Separator />
				{/if}

				<!-- Available Roles to Assign -->
				<div class="space-y-2">
					<Label class="text-sm font-medium">Assign New Role</Label>
					<div class="space-y-2">
						{#each roles as role (role.role_name)}
							{@const isAlreadyAssigned = userRoles.includes(role.role_name)}
							{#if !isAlreadyAssigned}
								<Button
									variant="outline"
									class="w-full justify-start"
									onclick={() => handleAssignRole(role.role_name)}
									disabled={isSubmitting}
								>
									<ShieldIcon class="mr-2 h-4 w-4" />
									{role.role_name.charAt(0).toUpperCase() + role.role_name.slice(1)}
									{#if role.description}
										<span class="ml-2 text-xs text-muted-foreground">- {role.description}</span>
									{/if}
								</Button>
							{/if}
						{/each}
						{#if roles.every((role) => userRoles.includes(role.role_name))}
							<p class="py-2 text-center text-sm text-muted-foreground">
								User has all available roles assigned
							</p>
						{/if}
					</div>
				</div>
			{/if}
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showAssignRoleDialog = false)}>Close</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Confirm Action Dialog -->
<Dialog.Root bind:open={showConfirmDialog}>
	<Dialog.Content class="sm:max-w-md">
		{#if confirmAction}
			<Dialog.Header>
				<Dialog.Title>
					{#if confirmAction.type === 'delete'}
						Delete User
					{:else if confirmAction.type === 'suspend'}
						Suspend User
					{:else}
						Unsuspend User
					{/if}
				</Dialog.Title>
				<Dialog.Description>
					Are you sure you want to {confirmAction.type}
					<strong>{confirmAction.user.fullName}</strong>?
					{#if confirmAction.type === 'delete'}
						This action cannot be undone.
					{/if}
				</Dialog.Description>
			</Dialog.Header>
			<Dialog.Footer>
				<Button variant="outline" onclick={() => (showConfirmDialog = false)}>Cancel</Button>
				<Button
					variant={confirmAction.type === 'delete' ? 'destructive' : 'default'}
					onclick={executeConfirmAction}
					disabled={isSubmitting}
				>
					{isSubmitting
						? 'Processing...'
						: confirmAction.type === 'delete'
							? 'Delete'
							: confirmAction.type === 'suspend'
								? 'Suspend'
								: 'Unsuspend'}
				</Button>
			</Dialog.Footer>
		{/if}
	</Dialog.Content>
</Dialog.Root>
