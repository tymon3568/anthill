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
	import { Separator } from '$lib/components/ui/separator';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { userServiceApi } from '$lib/api/user-service';
	import type {
		Role,
		Permission,
		CreateRoleRequest,
		UpdateRoleRequest
	} from '$lib/api/types/user-service.types';
	import { toast } from 'svelte-sonner';
	import { onMount } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import ShieldIcon from '@lucide/svelte/icons/shield';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import LockIcon from '@lucide/svelte/icons/lock';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';

	// State
	let roles = $state<Role[]>([]);
	let permissions = $state<Permission[]>([]);
	let isLoading = $state(true);
	let isLoadingPermissions = $state(true);

	// Dialog states
	let showCreateDialog = $state(false);
	let showEditDialog = $state(false);
	let showDeleteDialog = $state(false);
	let selectedRole = $state<Role | null>(null);
	let isSubmitting = $state(false);

	// Form state for create/edit
	let formName = $state('');
	let formDescription = $state('');
	// eslint-disable-next-line svelte/no-unnecessary-state-wrap -- need $state for reassignment reactivity
	let formPermissions = $state<SvelteSet<string>>(new SvelteSet());

	// Grouped permissions for display
	let groupedPermissions = $derived.by(() => {
		const groups: Record<string, Permission[]> = {};
		for (const perm of permissions) {
			if (!groups[perm.resource]) {
				groups[perm.resource] = [];
			}
			groups[perm.resource].push(perm);
		}
		return groups;
	});

	// Load data on mount
	onMount(async () => {
		await Promise.all([loadRoles(), loadPermissions()]);
	});

	async function loadRoles() {
		isLoading = true;
		try {
			const response = await userServiceApi.listRoles();
			if (response.success && response.data) {
				roles = response.data;
			} else {
				toast.error(response.error || 'Failed to load roles');
			}
		} catch (error) {
			console.error('Failed to load roles:', error);
			toast.error('Failed to load roles');
		} finally {
			isLoading = false;
		}
	}

	async function loadPermissions() {
		isLoadingPermissions = true;
		try {
			const response = await userServiceApi.listPermissions();
			if (response.success && response.data) {
				permissions = response.data;
			} else {
				toast.error(response.error || 'Failed to load permissions');
			}
		} catch (error) {
			console.error('Failed to load permissions:', error);
			toast.error('Failed to load permissions');
		} finally {
			isLoadingPermissions = false;
		}
	}

	function openCreateDialog() {
		formName = '';
		formDescription = '';
		formPermissions = new SvelteSet();
		showCreateDialog = true;
	}

	function openEditDialog(role: Role) {
		selectedRole = role;
		formName = role.name;
		formDescription = role.description || '';
		formPermissions = new SvelteSet(role.permissions.map((p) => `${p.resource}:${p.action}`));
		showEditDialog = true;
	}

	function openDeleteDialog(role: Role) {
		selectedRole = role;
		showDeleteDialog = true;
	}

	function togglePermission(resource: string, action: string) {
		const key = `${resource}:${action}`;
		if (formPermissions.has(key)) {
			formPermissions.delete(key);
		} else {
			formPermissions.add(key);
		}
	}

	function toggleAllResourcePermissions(resource: string) {
		const resourcePerms = groupedPermissions[resource] || [];
		const allSelected = resourcePerms.every((p) =>
			formPermissions.has(`${p.resource}:${p.action}`)
		);

		for (const perm of resourcePerms) {
			const key = `${perm.resource}:${perm.action}`;
			if (allSelected) {
				formPermissions.delete(key);
			} else {
				formPermissions.add(key);
			}
		}
	}

	function getSelectedPermissions(): Permission[] {
		return Array.from(formPermissions).map((key) => {
			const [resource, action] = key.split(':');
			return { resource, action };
		});
	}

	async function handleCreateRole() {
		if (!formName.trim()) {
			toast.error('Role name is required');
			return;
		}

		isSubmitting = true;
		try {
			const data: CreateRoleRequest = {
				name: formName.trim().toLowerCase().replace(/\s+/g, '_'),
				description: formDescription.trim() || undefined,
				permissions: getSelectedPermissions()
			};

			const response = await userServiceApi.createRole(data);
			if (response.success) {
				toast.success(`Role "${data.name}" created successfully`);
				showCreateDialog = false;
				await loadRoles();
			} else {
				toast.error(response.error || 'Failed to create role');
			}
		} catch (error) {
			console.error('Failed to create role:', error);
			toast.error('Failed to create role');
		} finally {
			isSubmitting = false;
		}
	}

	async function handleUpdateRole() {
		if (!selectedRole) return;

		isSubmitting = true;
		try {
			const data: UpdateRoleRequest = {
				description: formDescription.trim() || undefined,
				permissions: getSelectedPermissions()
			};

			const response = await userServiceApi.updateRole(selectedRole.name, data);
			if (response.success) {
				toast.success(`Role "${selectedRole.name}" updated successfully`);
				showEditDialog = false;
				selectedRole = null;
				await loadRoles();
			} else {
				toast.error(response.error || 'Failed to update role');
			}
		} catch (error) {
			console.error('Failed to update role:', error);
			toast.error('Failed to update role');
		} finally {
			isSubmitting = false;
		}
	}

	async function handleDeleteRole() {
		if (!selectedRole) return;

		isSubmitting = true;
		try {
			const response = await userServiceApi.deleteRole(selectedRole.name);
			if (response.success) {
				toast.success(`Role "${selectedRole.name}" deleted successfully`);
				showDeleteDialog = false;
				selectedRole = null;
				await loadRoles();
			} else {
				toast.error(response.error || 'Failed to delete role');
			}
		} catch (error) {
			console.error('Failed to delete role:', error);
			toast.error('Failed to delete role');
		} finally {
			isSubmitting = false;
		}
	}

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	function capitalizeFirst(str: string): string {
		return str.charAt(0).toUpperCase() + str.slice(1);
	}
</script>

<svelte:head>
	<title>Role Management - Admin - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<Card>
		<CardHeader>
			<div class="flex items-center justify-between">
				<div>
					<CardTitle class="flex items-center gap-2">
						<ShieldIcon class="h-5 w-5" />
						Role Management
					</CardTitle>
					<CardDescription>
						Manage roles and permissions. Create custom roles and assign granular permissions to
						control access.
					</CardDescription>
				</div>
				<div class="flex gap-2">
					<Button variant="outline" onclick={loadRoles}>
						<RefreshCwIcon class="mr-2 h-4 w-4" />
						Refresh
					</Button>
					<Button onclick={openCreateDialog}>
						<PlusIcon class="mr-2 h-4 w-4" />
						Create Role
					</Button>
				</div>
			</div>
		</CardHeader>
		<CardContent>
			{#if isLoading}
				<div class="space-y-3">
					{#each [0, 1, 2, 3] as i (i)}
						<div class="h-16 animate-pulse rounded bg-muted"></div>
					{/each}
				</div>
			{:else if roles.length === 0}
				<div class="py-12 text-center">
					<ShieldIcon class="mx-auto h-12 w-12 text-muted-foreground" />
					<h3 class="mt-4 text-lg font-medium">No roles found</h3>
					<p class="mt-2 text-sm text-muted-foreground">Get started by creating a custom role.</p>
				</div>
			{:else}
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>Role</Table.Head>
							<Table.Head>Description</Table.Head>
							<Table.Head>Permissions</Table.Head>
							<Table.Head>Type</Table.Head>
							<Table.Head>Created</Table.Head>
							<Table.Head class="text-right">Actions</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each roles as role (role.name)}
							<Table.Row>
								<Table.Cell class="font-medium">
									<div class="flex items-center gap-2">
										{#if role.isSystem}
											<LockIcon class="h-4 w-4 text-muted-foreground" />
										{/if}
										{capitalizeFirst(role.name)}
									</div>
								</Table.Cell>
								<Table.Cell class="max-w-xs truncate text-muted-foreground">
									{role.description || '-'}
								</Table.Cell>
								<Table.Cell>
									<Badge variant="secondary">
										{role.permissions.length} permission{role.permissions.length !== 1 ? 's' : ''}
									</Badge>
								</Table.Cell>
								<Table.Cell>
									<Badge variant={role.isSystem ? 'default' : 'outline'}>
										{role.isSystem ? 'System' : 'Custom'}
									</Badge>
								</Table.Cell>
								<Table.Cell>{formatDate(role.createdAt)}</Table.Cell>
								<Table.Cell class="text-right">
									<div class="flex justify-end gap-2">
										<Button
											variant="ghost"
											size="sm"
											onclick={() => openEditDialog(role)}
											disabled={role.isSystem}
											title={role.isSystem ? 'System roles cannot be edited' : 'Edit Role'}
										>
											<PencilIcon class="h-4 w-4" />
										</Button>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => openDeleteDialog(role)}
											disabled={role.isSystem}
											title={role.isSystem ? 'System roles cannot be deleted' : 'Delete Role'}
										>
											<TrashIcon class="h-4 w-4 text-destructive" />
										</Button>
									</div>
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			{/if}
		</CardContent>
	</Card>
</div>

<!-- Create Role Dialog -->
<Dialog.Root
	bind:open={showCreateDialog}
	onOpenChange={(open) => {
		if (!open) {
			formName = '';
			formDescription = '';
			formPermissions = new SvelteSet();
		}
	}}
>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title>Create New Role</Dialog.Title>
			<Dialog.Description>
				Define a custom role with specific permissions for your organization.
			</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-6 py-4">
			<div class="grid gap-4 sm:grid-cols-2">
				<div class="space-y-2">
					<Label for="roleName">Role Name *</Label>
					<Input
						id="roleName"
						bind:value={formName}
						placeholder="e.g., warehouse_manager"
						required
					/>
					<p class="text-xs text-muted-foreground">
						Lowercase with underscores. Cannot be changed after creation.
					</p>
				</div>
				<div class="space-y-2">
					<Label for="roleDescription">Description</Label>
					<Input
						id="roleDescription"
						bind:value={formDescription}
						placeholder="e.g., Manages warehouse operations"
					/>
				</div>
			</div>

			<Separator />

			<div class="space-y-4">
				<h4 class="font-medium">Permissions</h4>
				{#if isLoadingPermissions}
					<div class="space-y-2">
						{#each [0, 1, 2] as i (i)}
							<div class="h-20 animate-pulse rounded bg-muted"></div>
						{/each}
					</div>
				{:else if Object.keys(groupedPermissions).length === 0}
					<p class="text-sm text-muted-foreground">No permissions available.</p>
				{:else}
					<div class="space-y-4">
						{#each Object.entries(groupedPermissions) as [resource, perms] (resource)}
							{@const allSelected = perms.every((p) =>
								formPermissions.has(`${p.resource}:${p.action}`)
							)}
							<div class="rounded-lg border p-4">
								<div class="mb-3 flex items-center justify-between">
									<div class="flex items-center gap-2">
										<Checkbox
											id="resource-{resource}"
											checked={allSelected}
											onCheckedChange={() => toggleAllResourcePermissions(resource)}
										/>
										<Label for="resource-{resource}" class="font-medium">
											{capitalizeFirst(resource)}
										</Label>
									</div>
									<Badge variant="outline">
										{perms.filter((p) => formPermissions.has(`${p.resource}:${p.action}`))
											.length}/{perms.length}
									</Badge>
								</div>
								<div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
									{#each perms as perm (`${perm.resource}:${perm.action}`)}
										{@const isChecked = formPermissions.has(`${perm.resource}:${perm.action}`)}
										<div class="flex items-center gap-2">
											<Checkbox
												id="perm-{perm.resource}-{perm.action}"
												checked={isChecked}
												onCheckedChange={() => togglePermission(perm.resource, perm.action)}
											/>
											<Label for="perm-{perm.resource}-{perm.action}" class="text-sm font-normal">
												{capitalizeFirst(perm.action)}
											</Label>
										</div>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showCreateDialog = false)}>Cancel</Button>
			<Button onclick={handleCreateRole} disabled={isSubmitting || !formName.trim()}>
				{isSubmitting ? 'Creating...' : 'Create Role'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Edit Role Dialog -->
<Dialog.Root
	bind:open={showEditDialog}
	onOpenChange={(open) => {
		if (!open) {
			selectedRole = null;
			formName = '';
			formDescription = '';
			formPermissions = new SvelteSet();
		}
	}}
>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title>Edit Role: {selectedRole?.name}</Dialog.Title>
			<Dialog.Description>
				Update the permissions for this role. Changes will affect all users with this role.
			</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-6 py-4">
			<div class="space-y-2">
				<Label for="editRoleDescription">Description</Label>
				<Input
					id="editRoleDescription"
					bind:value={formDescription}
					placeholder="e.g., Manages warehouse operations"
				/>
			</div>

			<Separator />

			<div class="space-y-4">
				<h4 class="font-medium">Permissions</h4>
				{#if isLoadingPermissions}
					<div class="space-y-2">
						{#each [0, 1, 2] as i (i)}
							<div class="h-20 animate-pulse rounded bg-muted"></div>
						{/each}
					</div>
				{:else if Object.keys(groupedPermissions).length === 0}
					<p class="text-sm text-muted-foreground">No permissions available.</p>
				{:else}
					<div class="space-y-4">
						{#each Object.entries(groupedPermissions) as [resource, perms] (resource)}
							{@const allSelected = perms.every((p) =>
								formPermissions.has(`${p.resource}:${p.action}`)
							)}
							<div class="rounded-lg border p-4">
								<div class="mb-3 flex items-center justify-between">
									<div class="flex items-center gap-2">
										<Checkbox
											id="edit-resource-{resource}"
											checked={allSelected}
											onCheckedChange={() => toggleAllResourcePermissions(resource)}
										/>
										<Label for="edit-resource-{resource}" class="font-medium">
											{capitalizeFirst(resource)}
										</Label>
									</div>
									<Badge variant="outline">
										{perms.filter((p) => formPermissions.has(`${p.resource}:${p.action}`))
											.length}/{perms.length}
									</Badge>
								</div>
								<div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
									{#each perms as perm (`${perm.resource}:${perm.action}`)}
										{@const isChecked = formPermissions.has(`${perm.resource}:${perm.action}`)}
										<div class="flex items-center gap-2">
											<Checkbox
												id="edit-perm-{perm.resource}-{perm.action}"
												checked={isChecked}
												onCheckedChange={() => togglePermission(perm.resource, perm.action)}
											/>
											<Label
												for="edit-perm-{perm.resource}-{perm.action}"
												class="text-sm font-normal"
											>
												{capitalizeFirst(perm.action)}
											</Label>
										</div>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showEditDialog = false)}>Cancel</Button>
			<Button onclick={handleUpdateRole} disabled={isSubmitting}>
				{isSubmitting ? 'Saving...' : 'Save Changes'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Role Dialog -->
<Dialog.Root bind:open={showDeleteDialog}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Delete Role</Dialog.Title>
			<Dialog.Description>
				{#if selectedRole}
					Are you sure you want to delete the role <strong>{selectedRole.name}</strong>? This action
					cannot be undone. Users with this role will lose their associated permissions.
				{/if}
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showDeleteDialog = false)}>Cancel</Button>
			<Button variant="destructive" onclick={handleDeleteRole} disabled={isSubmitting}>
				{isSubmitting ? 'Deleting...' : 'Delete Role'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
