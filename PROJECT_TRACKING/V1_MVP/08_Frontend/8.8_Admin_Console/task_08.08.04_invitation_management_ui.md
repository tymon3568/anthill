# Task: Admin Invitation Management UI

**Task ID:** V1_MVP/08_Frontend/8.8_Admin_Console/task_08.08.04_invitation_management_ui.md
**Version:** V1_MVP
**Phase:** 08_Frontend
**Module:** 8.8_Admin_Console
**Priority:** Medium
**Status:** Todo
**Assignee:** 
**Created Date:** 2026-01-17
**Last Updated:** 2026-01-17

## Detailed Description:
Create a comprehensive invitation management interface for Tenant Administrators. This module allows admins to invite new users to the tenant via email, manage pending invitations, resend invitation emails, and revoke invitations. The backend supports secure invitation tokens with 48-hour expiration and rate limiting.

Backend endpoints available:
- `POST /api/v1/admin/users/invite` - Create invitation
- `GET /api/v1/admin/users/invitations` - List invitations
- `DELETE /api/v1/admin/users/invitations/{id}` - Revoke invitation
- `POST /api/v1/admin/users/invitations/{id}/resend` - Resend invitation email

## Acceptance Criteria:
- [ ] Invitation list page with table showing all invitations
- [ ] "Invite User" button opens modal with email, role, and optional message fields
- [ ] Table shows: Email, Role, Status, Invited By, Expires At, Actions
- [ ] Status badges: Pending (yellow), Accepted (green), Expired (gray), Revoked (red)
- [ ] Actions column: Resend (pending only), Revoke (pending only)
- [ ] Pagination for invitation list
- [ ] Filter by status (pending, accepted, expired, revoked)
- [ ] Success/error notifications for all actions
- [ ] Rate limiting feedback for invite/resend actions
- [ ] Responsive design following Frappe UI guidelines
- [ ] Svelte 5 runes used throughout
- [ ] Code compiles without errors: `bun run check`
- [ ] Access restricted to admin users

## Specific Sub-tasks:
- [ ] 1. Create Invitation List Page (`src/routes/(app)/admin/invitations/+page.svelte`)
    - [ ] 1.1. Set up route within admin layout
    - [ ] 1.2. Create +page.server.ts for initial data load
    - [ ] 1.3. Implement data table with columns: Email, Role, Status, Invited By, Expires At, Actions
    - [ ] 1.4. Add status badge component with appropriate colors
    - [ ] 1.5. Implement pagination controls
    - [ ] 1.6. Add filter dropdown for status

- [ ] 2. Create "Invite User" Modal (`src/lib/components/admin/InviteUserModal.svelte`)
    - [ ] 2.1. Email input with validation
    - [ ] 2.2. Role selector (dropdown with available roles)
    - [ ] 2.3. Optional custom message textarea
    - [ ] 2.4. Submit button with loading state
    - [ ] 2.5. API integration: `POST /api/v1/admin/users/invite`
    - [ ] 2.6. Handle errors: duplicate email, rate limiting, validation

- [ ] 3. Implement Invitation Actions
    - [ ] 3.1. Resend invitation button (only for pending status)
    - [ ] 3.2. Confirmation dialog for resend
    - [ ] 3.3. API integration: `POST /api/v1/admin/users/invitations/{id}/resend`
    - [ ] 3.4. Revoke invitation button (only for pending status)
    - [ ] 3.5. Confirmation dialog for revoke
    - [ ] 3.6. API integration: `DELETE /api/v1/admin/users/invitations/{id}`
    - [ ] 3.7. Refresh list after successful action

- [ ] 4. Create Invitation Status Badge Component
    - [ ] 4.1. Create `InvitationStatusBadge.svelte`
    - [ ] 4.2. Define color scheme: pending=yellow, accepted=green, expired=gray, revoked=red
    - [ ] 4.3. Add icon for each status (optional)

- [ ] 5. Implement Empty States
    - [ ] 5.1. No invitations state with "Invite your first user" CTA
    - [ ] 5.2. No results for filter state

- [ ] 6. Add Navigation Link
    - [ ] 6.1. Add "Invitations" link to admin sidebar
    - [ ] 6.2. Add invitation count badge (pending count)

- [ ] 7. Error Handling & Notifications
    - [ ] 7.1. Handle 403 Forbidden (non-admin)
    - [ ] 7.2. Handle 409 Conflict (email already invited)
    - [ ] 7.3. Handle 429 Rate Limited
    - [ ] 7.4. Success toast for create/resend/revoke actions

## Dependencies:
*   Task: `task_08.07.05_user_service_api_client.md` (Status: NeedsReview)
*   Task: `task_08.08.03_admin_layout_and_nav.md` (Status: Todo)

## Files to Create/Modify:
*   `src/routes/(app)/admin/invitations/+page.svelte` - Main invitation list page
*   `src/routes/(app)/admin/invitations/+page.server.ts` - Server-side data loading
*   `src/lib/components/admin/InviteUserModal.svelte` - Invite user modal
*   `src/lib/components/admin/InvitationStatusBadge.svelte` - Status badge component
*   `src/lib/components/admin/InvitationActions.svelte` - Action buttons component
*   `src/lib/components/admin/InvitationEmptyState.svelte` - Empty state component

## Code Examples:
```svelte
<!-- src/routes/(app)/admin/invitations/+page.svelte -->
<script lang="ts">
  import { userServiceApi } from '$lib/api/user-service';
  import { Button } from '$lib/components/ui/button';
  import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui/table';
  import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '$lib/components/ui/select';
  import { toast } from 'svelte-sonner';
  import { UserPlus, RefreshCw, X } from 'lucide-svelte';
  import InviteUserModal from '$lib/components/admin/InviteUserModal.svelte';
  import InvitationStatusBadge from '$lib/components/admin/InvitationStatusBadge.svelte';
  import InvitationEmptyState from '$lib/components/admin/InvitationEmptyState.svelte';

  let { data } = $props();

  let invitations = $state(data.invitations);
  let isLoading = $state(false);
  let isModalOpen = $state(false);
  let statusFilter = $state('all');
  let currentPage = $state(1);

  async function loadInvitations() {
    isLoading = true;
    try {
      const result = await userServiceApi.listInvitations({
        page: currentPage,
        perPage: 10,
        status: statusFilter === 'all' ? undefined : statusFilter,
      });
      invitations = result;
    } catch (error) {
      toast.error('Failed to load invitations');
    } finally {
      isLoading = false;
    }
  }

  async function handleResend(invitationId: string) {
    try {
      await userServiceApi.resendInvitation(invitationId);
      toast.success('Invitation resent successfully');
      await loadInvitations();
    } catch (error) {
      if (error.code === 'RATE_LIMITED') {
        toast.error('Too many resend attempts. Please try again later.');
      } else {
        toast.error('Failed to resend invitation');
      }
    }
  }

  async function handleRevoke(invitationId: string) {
    try {
      await userServiceApi.revokeInvitation(invitationId);
      toast.success('Invitation revoked');
      await loadInvitations();
    } catch (error) {
      toast.error('Failed to revoke invitation');
    }
  }

  async function handleInviteSuccess() {
    isModalOpen = false;
    toast.success('Invitation sent successfully');
    await loadInvitations();
  }

  $effect(() => {
    if (statusFilter) {
      loadInvitations();
    }
  });
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-semibold">Invitations</h1>
      <p class="text-muted-foreground">Manage user invitations for your organization</p>
    </div>
    <Button onclick={() => isModalOpen = true}>
      <UserPlus class="w-4 h-4 mr-2" />
      Invite User
    </Button>
  </div>

  <div class="flex items-center gap-4">
    <Select bind:value={statusFilter}>
      <SelectTrigger class="w-48">
        <SelectValue placeholder="Filter by status" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="all">All Statuses</SelectItem>
        <SelectItem value="pending">Pending</SelectItem>
        <SelectItem value="accepted">Accepted</SelectItem>
        <SelectItem value="expired">Expired</SelectItem>
        <SelectItem value="revoked">Revoked</SelectItem>
      </SelectContent>
    </Select>
  </div>

  {#if invitations.data.length === 0}
    <InvitationEmptyState onInvite={() => isModalOpen = true} />
  {:else}
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Email</TableHead>
          <TableHead>Role</TableHead>
          <TableHead>Status</TableHead>
          <TableHead>Invited By</TableHead>
          <TableHead>Expires At</TableHead>
          <TableHead class="text-right">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {#each invitations.data as invitation}
          <TableRow>
            <TableCell class="font-medium">{invitation.email}</TableCell>
            <TableCell class="capitalize">{invitation.role}</TableCell>
            <TableCell>
              <InvitationStatusBadge status={invitation.status} />
            </TableCell>
            <TableCell>{invitation.invitedBy}</TableCell>
            <TableCell>
              {new Date(invitation.expiresAt).toLocaleDateString()}
            </TableCell>
            <TableCell class="text-right">
              {#if invitation.status === 'pending'}
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => handleResend(invitation.id)}
                >
                  <RefreshCw class="w-4 h-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => handleRevoke(invitation.id)}
                >
                  <X class="w-4 h-4" />
                </Button>
              {/if}
            </TableCell>
          </TableRow>
        {/each}
      </TableBody>
    </Table>

    <!-- Pagination -->
    <div class="flex items-center justify-between">
      <p class="text-sm text-muted-foreground">
        Showing {invitations.data.length} of {invitations.total} invitations
      </p>
      <div class="flex gap-2">
        <Button
          variant="outline"
          size="sm"
          disabled={currentPage === 1}
          onclick={() => currentPage--}
        >
          Previous
        </Button>
        <Button
          variant="outline"
          size="sm"
          disabled={currentPage >= invitations.totalPages}
          onclick={() => currentPage++}
        >
          Next
        </Button>
      </div>
    </div>
  {/if}
</div>

<InviteUserModal
  open={isModalOpen}
  onClose={() => isModalOpen = false}
  onSuccess={handleInviteSuccess}
/>
```

```svelte
<!-- src/lib/components/admin/InviteUserModal.svelte -->
<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '$lib/components/ui/select';
  import { userServiceApi } from '$lib/api/user-service';

  let { open, onClose, onSuccess } = $props<{
    open: boolean;
    onClose: () => void;
    onSuccess: () => void;
  }>();

  let email = $state('');
  let role = $state('user');
  let customMessage = $state('');
  let isLoading = $state(false);
  let error = $state('');
  let roles = $state<string[]>([]);

  $effect(() => {
    if (open && roles.length === 0) {
      loadRoles();
    }
  });

  async function loadRoles() {
    try {
      const roleList = await userServiceApi.listRoles();
      roles = roleList.map(r => r.name);
    } catch (e) {
      // Fallback to default roles
      roles = ['user', 'admin'];
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    isLoading = true;
    error = '';

    try {
      await userServiceApi.createInvitation({
        email,
        role,
        customMessage: customMessage || undefined,
      });
      resetForm();
      onSuccess();
    } catch (err) {
      if (err.code === 'CONFLICT') {
        error = 'An invitation has already been sent to this email.';
      } else if (err.code === 'RATE_LIMITED') {
        error = 'Too many invitations sent. Please try again later.';
      } else {
        error = err.message || 'Failed to send invitation';
      }
    } finally {
      isLoading = false;
    }
  }

  function resetForm() {
    email = '';
    role = 'user';
    customMessage = '';
    error = '';
  }
</script>

<Dialog {open} onOpenChange={(value) => !value && onClose()}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Invite User</DialogTitle>
    </DialogHeader>
    
    <form onsubmit={handleSubmit} class="space-y-4">
      <div class="space-y-2">
        <Label for="email">Email Address</Label>
        <Input
          id="email"
          type="email"
          placeholder="colleague@example.com"
          bind:value={email}
          required
        />
      </div>

      <div class="space-y-2">
        <Label for="role">Role</Label>
        <Select bind:value={role}>
          <SelectTrigger>
            <SelectValue placeholder="Select a role" />
          </SelectTrigger>
          <SelectContent>
            {#each roles as roleName}
              <SelectItem value={roleName} class="capitalize">{roleName}</SelectItem>
            {/each}
          </SelectContent>
        </Select>
      </div>

      <div class="space-y-2">
        <Label for="message">Custom Message (Optional)</Label>
        <Textarea
          id="message"
          placeholder="Add a personal message to the invitation email..."
          bind:value={customMessage}
          rows={3}
        />
      </div>

      {#if error}
        <p class="text-sm text-red-500">{error}</p>
      {/if}

      <DialogFooter>
        <Button type="button" variant="outline" onclick={onClose}>
          Cancel
        </Button>
        <Button type="submit" disabled={isLoading}>
          {isLoading ? 'Sending...' : 'Send Invitation'}
        </Button>
      </DialogFooter>
    </form>
  </DialogContent>
</Dialog>
```

```svelte
<!-- src/lib/components/admin/InvitationStatusBadge.svelte -->
<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Clock, CheckCircle, XCircle, AlertCircle } from 'lucide-svelte';

  let { status } = $props<{ status: 'pending' | 'accepted' | 'expired' | 'revoked' }>();

  const config = $derived({
    pending: { variant: 'outline', class: 'border-yellow-500 text-yellow-600', icon: Clock },
    accepted: { variant: 'outline', class: 'border-green-500 text-green-600', icon: CheckCircle },
    expired: { variant: 'outline', class: 'border-gray-400 text-gray-500', icon: AlertCircle },
    revoked: { variant: 'outline', class: 'border-red-500 text-red-600', icon: XCircle },
  }[status]);
</script>

<Badge variant={config.variant} class={config.class}>
  <svelte:component this={config.icon} class="w-3 h-3 mr-1" />
  <span class="capitalize">{status}</span>
</Badge>
```

## Testing Steps:
- [ ] Navigate to `/admin/invitations` - should show invitation list
- [ ] Click "Invite User" - modal should open
- [ ] Submit with valid email and role - should create invitation
- [ ] Submit with already-invited email - should show conflict error
- [ ] Filter by pending status - should show only pending invitations
- [ ] Click resend on pending invitation - should show success toast
- [ ] Click revoke on pending invitation - should remove from list
- [ ] Test pagination with >10 invitations
- [ ] Test empty state when no invitations exist
- [ ] Verify admin-only access (non-admin should get 403)
- [ ] Test responsive design on mobile

## Backend API Reference:
```
POST /api/v1/admin/users/invite
Body: { "email": "string", "role": "string", "custom_message"?: "string" }
Response: 201 Created | 409 Conflict | 429 Too Many Requests

GET /api/v1/admin/users/invitations
Query: ?page=1&per_page=10&status=pending
Response: { "data": [...], "total": number, "page": number, "per_page": number }

DELETE /api/v1/admin/users/invitations/{invitation_id}
Response: 204 No Content | 404 Not Found

POST /api/v1/admin/users/invitations/{invitation_id}/resend
Response: 200 OK | 404 Not Found | 429 Too Many Requests
```

## Notes / Discussion:
---
*   Invitations use SHA-256 hashed tokens stored in database
*   Invitation tokens expire after 48 hours
*   Rate limiting: 10 invitations per day per admin
*   Only pending invitations can be resent or revoked
*   Accepted invitations are read-only records
*   Custom message appears in the invitation email body
*   System roles (owner, admin, user) are available by default, custom roles if defined

## AI Agent Log:
---
*   2026-01-17 10:45: Task created by Opus
    - Created to provide dedicated UI for invitation management
    - Backend endpoints already implemented in User Service
    - Separated from task_08.08.01 for clearer scope
    - Follows admin console patterns from existing tasks
