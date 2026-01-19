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
	import { Textarea } from '$lib/components/ui/textarea';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import { Separator } from '$lib/components/ui/separator';
	import { userServiceApi } from '$lib/api/user-service';
	import type {
		Invitation,
		Role,
		InvitationStatus,
		ListInvitationsParams,
		CreateInvitationRequest
	} from '$lib/api/types/user-service.types';
	import { toast } from 'svelte-sonner';
	import { onMount } from 'svelte';
	import MailIcon from '@lucide/svelte/icons/mail';
	import UserPlusIcon from '@lucide/svelte/icons/user-plus';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import XIcon from '@lucide/svelte/icons/x';
	import ClockIcon from '@lucide/svelte/icons/clock';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle';
	import XCircleIcon from '@lucide/svelte/icons/x-circle';
	import AlertCircleIcon from '@lucide/svelte/icons/alert-circle';
	import SendIcon from '@lucide/svelte/icons/send';

	// State
	let invitations = $state<Invitation[]>([]);
	let roles = $state<Role[]>([]);
	let isLoading = $state(true);
	let isLoadingRoles = $state(true);
	let totalInvitations = $state(0);
	let currentPage = $state(1);
	let perPage = $state(10);
	let totalPages = $state(1);
	let filterStatus = $state<InvitationStatus | ''>('');

	// Dialog states
	let showInviteDialog = $state(false);
	let showConfirmDialog = $state(false);
	let confirmAction = $state<{ type: 'resend' | 'revoke'; invitation: Invitation } | null>(null);
	let isSubmitting = $state(false);

	// Invite form
	let inviteForm = $state<CreateInvitationRequest>({
		email: '',
		role: 'user',
		customMessage: ''
	});
	let inviteError = $state('');

	// Load invitations on mount
	onMount(async () => {
		await Promise.all([loadInvitations(), loadRoles()]);
	});

	async function loadInvitations() {
		isLoading = true;
		try {
			const params: ListInvitationsParams = {
				page: currentPage,
				perPage: perPage
			};
			if (filterStatus) params.status = filterStatus;

			const response = await userServiceApi.listInvitations(params);
			if (response.success && response.data) {
				invitations = response.data.data;
				totalInvitations = response.data.total;
				totalPages = response.data.totalPages;
			} else {
				toast.error(response.error || 'Failed to load invitations');
			}
		} catch (error) {
			console.error('Failed to load invitations:', error);
			toast.error('Failed to load invitations');
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
			// Fallback to default roles
			roles = [
				{
					role_name: 'user',
					description: 'Regular user',
					permissions: [],
					user_count: 0
				},
				{
					role_name: 'admin',
					description: 'Administrator',
					permissions: [],
					user_count: 0
				}
			];
		} finally {
			isLoadingRoles = false;
		}
	}

	async function handlePageChange(page: number) {
		currentPage = page;
		await loadInvitations();
	}

	async function handleFilterChange(status: InvitationStatus | '') {
		filterStatus = status;
		currentPage = 1;
		await loadInvitations();
	}

	async function handleInviteUser() {
		if (!inviteForm.email) {
			inviteError = 'Email is required';
			return;
		}

		isSubmitting = true;
		inviteError = '';

		try {
			const response = await userServiceApi.createInvitation({
				email: inviteForm.email,
				role: inviteForm.role,
				customMessage: inviteForm.customMessage || undefined
			});

			if (response.success) {
				toast.success('Invitation sent successfully');
				showInviteDialog = false;
				inviteForm = { email: '', role: 'user', customMessage: '' };
				await loadInvitations();
			} else {
				// Handle specific errors
				if (
					response.error?.toLowerCase().includes('already') ||
					response.error?.toLowerCase().includes('conflict')
				) {
					inviteError = 'An invitation has already been sent to this email.';
				} else if (
					response.error?.toLowerCase().includes('rate') ||
					response.error?.toLowerCase().includes('limit')
				) {
					inviteError = 'Too many invitations sent. Please try again later.';
				} else {
					inviteError = response.error || 'Failed to send invitation';
				}
			}
		} catch (error) {
			console.error('Failed to send invitation:', error);
			inviteError = 'Failed to send invitation';
		} finally {
			isSubmitting = false;
		}
	}

	function openResendDialog(invitation: Invitation) {
		confirmAction = { type: 'resend', invitation };
		showConfirmDialog = true;
	}

	function openRevokeDialog(invitation: Invitation) {
		confirmAction = { type: 'revoke', invitation };
		showConfirmDialog = true;
	}

	async function executeConfirmAction() {
		if (!confirmAction) return;

		isSubmitting = true;
		try {
			let response;
			if (confirmAction.type === 'resend') {
				response = await userServiceApi.resendInvitation(confirmAction.invitation.id);
			} else {
				response = await userServiceApi.revokeInvitation(confirmAction.invitation.id);
			}

			if (response?.success) {
				toast.success(
					confirmAction.type === 'resend'
						? 'Invitation resent successfully'
						: 'Invitation revoked successfully'
				);
				showConfirmDialog = false;
				confirmAction = null;
				await loadInvitations();
			} else {
				if (
					response?.error?.toLowerCase().includes('rate') ||
					response?.error?.toLowerCase().includes('limit')
				) {
					toast.error('Too many resend attempts. Please try again later.');
				} else {
					toast.error(response?.error || `Failed to ${confirmAction.type} invitation`);
				}
			}
		} catch (error) {
			console.error(`Failed to ${confirmAction?.type} invitation:`, error);
			toast.error(`Failed to ${confirmAction?.type} invitation`);
		} finally {
			isSubmitting = false;
		}
	}

	function getStatusBadgeConfig(status: InvitationStatus): {
		variant: 'default' | 'secondary' | 'destructive' | 'outline';
		class: string;
		icon: typeof ClockIcon;
	} {
		switch (status) {
			case 'pending':
				return { variant: 'outline', class: 'border-yellow-500 text-yellow-600', icon: ClockIcon };
			case 'accepted':
				return {
					variant: 'outline',
					class: 'border-green-500 text-green-600',
					icon: CheckCircleIcon
				};
			case 'expired':
				return {
					variant: 'outline',
					class: 'border-gray-400 text-gray-500',
					icon: AlertCircleIcon
				};
			case 'revoked':
				return { variant: 'outline', class: 'border-red-500 text-red-600', icon: XCircleIcon };
			default:
				return { variant: 'outline', class: '', icon: ClockIcon };
		}
	}

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	function formatDateTime(dateString: string): string {
		return new Date(dateString).toLocaleString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function isExpiringSoon(expiresAt: string): boolean {
		const expiry = new Date(expiresAt);
		const now = new Date();
		const hoursRemaining = (expiry.getTime() - now.getTime()) / (1000 * 60 * 60);
		return hoursRemaining > 0 && hoursRemaining < 12;
	}
</script>

<svelte:head>
	<title>Invitation Management - Admin - Anthill</title>
</svelte:head>

<div class="space-y-6">
	<Card>
		<CardHeader>
			<div class="flex items-center justify-between">
				<div>
					<CardTitle class="flex items-center gap-2">
						<MailIcon class="h-5 w-5" />
						Invitation Management
					</CardTitle>
					<CardDescription>
						Invite new users to your organization. Manage pending invitations.
					</CardDescription>
				</div>
				<Button onclick={() => (showInviteDialog = true)}>
					<UserPlusIcon class="mr-2 h-4 w-4" />
					Invite User
				</Button>
			</div>
		</CardHeader>
		<CardContent class="space-y-4">
			<!-- Filters -->
			<div class="flex gap-4">
				<Select.Root
					type="single"
					name="statusFilter"
					value={filterStatus}
					onValueChange={(v) => handleFilterChange(v as InvitationStatus | '')}
				>
					<Select.Trigger class="w-40">
						{filterStatus
							? filterStatus.charAt(0).toUpperCase() + filterStatus.slice(1)
							: 'All Status'}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="">All Status</Select.Item>
						<Select.Item value="pending">Pending</Select.Item>
						<Select.Item value="accepted">Accepted</Select.Item>
						<Select.Item value="expired">Expired</Select.Item>
						<Select.Item value="revoked">Revoked</Select.Item>
					</Select.Content>
				</Select.Root>
				<Button variant="outline" onclick={loadInvitations}>
					<RefreshCwIcon class="mr-2 h-4 w-4" />
					Refresh
				</Button>
			</div>

			<Separator />

			<!-- Invitations Table -->
			{#if isLoading}
				<div class="space-y-3">
					{#each [0, 1, 2, 3, 4] as i (i)}
						<div class="h-12 animate-pulse rounded bg-muted"></div>
					{/each}
				</div>
			{:else if invitations.length === 0}
				<div class="py-12 text-center">
					<MailIcon class="mx-auto h-12 w-12 text-muted-foreground" />
					<h3 class="mt-4 text-lg font-medium">No invitations found</h3>
					<p class="mt-2 text-sm text-muted-foreground">
						{filterStatus
							? 'Try adjusting your filter.'
							: 'Get started by inviting your first user.'}
					</p>
					{#if !filterStatus}
						<Button class="mt-4" onclick={() => (showInviteDialog = true)}>
							<UserPlusIcon class="mr-2 h-4 w-4" />
							Invite User
						</Button>
					{/if}
				</div>
			{:else}
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>Email</Table.Head>
							<Table.Head>Role</Table.Head>
							<Table.Head>Status</Table.Head>
							<Table.Head>Invited By</Table.Head>
							<Table.Head>Expires At</Table.Head>
							<Table.Head class="text-right">Actions</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each invitations as invitation (invitation.id)}
							{@const statusConfig = getStatusBadgeConfig(invitation.status)}
							<Table.Row>
								<Table.Cell class="font-medium">{invitation.email}</Table.Cell>
								<Table.Cell>
									<Badge variant="outline" class="capitalize">{invitation.role}</Badge>
								</Table.Cell>
								<Table.Cell>
									<Badge variant={statusConfig.variant} class={statusConfig.class}>
										{#if invitation.status === 'pending'}
											<ClockIcon class="mr-1 h-3 w-3" />
										{:else if invitation.status === 'accepted'}
											<CheckCircleIcon class="mr-1 h-3 w-3" />
										{:else if invitation.status === 'expired'}
											<AlertCircleIcon class="mr-1 h-3 w-3" />
										{:else if invitation.status === 'revoked'}
											<XCircleIcon class="mr-1 h-3 w-3" />
										{/if}
										<span class="capitalize">{invitation.status}</span>
									</Badge>
								</Table.Cell>
								<Table.Cell>{invitation.invitedByName || invitation.invitedBy}</Table.Cell>
								<Table.Cell>
									{#if invitation.status === 'pending'}
										<span class={isExpiringSoon(invitation.expiresAt) ? 'text-orange-600' : ''}>
											{formatDateTime(invitation.expiresAt)}
										</span>
										{#if isExpiringSoon(invitation.expiresAt)}
											<span class="ml-1 text-xs text-orange-600">(expiring soon)</span>
										{/if}
									{:else}
										<span class="text-muted-foreground">{formatDate(invitation.expiresAt)}</span>
									{/if}
								</Table.Cell>
								<Table.Cell class="text-right">
									{#if invitation.status === 'pending'}
										<div class="flex justify-end gap-2">
											<Button
												variant="ghost"
												size="sm"
												onclick={() => openResendDialog(invitation)}
												title="Resend Invitation"
											>
												<SendIcon class="h-4 w-4" />
											</Button>
											<Button
												variant="ghost"
												size="sm"
												onclick={() => openRevokeDialog(invitation)}
												title="Revoke Invitation"
											>
												<XIcon class="h-4 w-4 text-destructive" />
											</Button>
										</div>
									{:else}
										<span class="text-sm text-muted-foreground">-</span>
									{/if}
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
								totalInvitations
							)}
							of {totalInvitations} invitations
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

<!-- Invite User Dialog -->
<Dialog.Root
	bind:open={showInviteDialog}
	onOpenChange={(open) => {
		if (!open) {
			inviteForm = { email: '', role: 'user', customMessage: '' };
			inviteError = '';
		}
	}}
>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Invite User</Dialog.Title>
			<Dialog.Description>
				Send an invitation to join your organization. The invitation expires in 48 hours.
			</Dialog.Description>
		</Dialog.Header>
		<div class="space-y-4 py-4">
			<div class="space-y-2">
				<Label for="email">Email Address *</Label>
				<Input
					id="email"
					type="email"
					bind:value={inviteForm.email}
					placeholder="colleague@example.com"
					required
				/>
			</div>
			<div class="space-y-2">
				<Label for="role">Role</Label>
				<Select.Root
					type="single"
					name="role"
					value={inviteForm.role}
					onValueChange={(v) => (inviteForm.role = v)}
				>
					<Select.Trigger class="w-full">
						{inviteForm.role.charAt(0).toUpperCase() + inviteForm.role.slice(1)}
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
			<div class="space-y-2">
				<Label for="message">Custom Message (Optional)</Label>
				<Textarea
					id="message"
					bind:value={inviteForm.customMessage}
					placeholder="Add a personal message to the invitation email..."
					rows={3}
				/>
			</div>
			{#if inviteError}
				<div class="rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-600">
					{inviteError}
				</div>
			{/if}
		</div>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showInviteDialog = false)}>Cancel</Button>
			<Button onclick={handleInviteUser} disabled={isSubmitting}>
				{isSubmitting ? 'Sending...' : 'Send Invitation'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<!-- Confirm Action Dialog -->
<Dialog.Root bind:open={showConfirmDialog}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>
				{#if confirmAction?.type === 'resend'}
					Resend Invitation
				{:else}
					Revoke Invitation
				{/if}
			</Dialog.Title>
			<Dialog.Description>
				{#if confirmAction}
					{#if confirmAction.type === 'resend'}
						Are you sure you want to resend the invitation to
						<strong>{confirmAction.invitation.email}</strong>? A new email will be sent with a fresh
						expiration time.
					{:else}
						Are you sure you want to revoke the invitation to
						<strong>{confirmAction.invitation.email}</strong>? This action cannot be undone.
					{/if}
				{/if}
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="outline" onclick={() => (showConfirmDialog = false)}>Cancel</Button>
			<Button
				variant={confirmAction?.type === 'revoke' ? 'destructive' : 'default'}
				onclick={executeConfirmAction}
				disabled={isSubmitting}
			>
				{isSubmitting ? 'Processing...' : confirmAction?.type === 'resend' ? 'Resend' : 'Revoke'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
