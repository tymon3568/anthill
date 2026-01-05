# Task: Create User Invitation System with Secure Tokens

**Task ID:** `V1_MVP/03_User_Service/3.3_User_Management/task_03.03.04_create_user_invitation_system.md`
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.3_User_Management
**Priority:** High
**Status:** Todo
**Assignee:**
**Created Date:** 2025-01-21
**Last Updated:** 2026-01-04

## Context / Goal

Implement a **secure user invitation system** that allows tenant administrators to invite new users to join their tenant. This is the recommended provisioning flow per **AUTHORIZATION_RBAC_STRATEGY.md**:

> **Invite flow**: admin invites user; user sets password via invite token.
> Admin should not be responsible for choosing/password handling.

The invitation system must follow strict security requirements for token handling to prevent account takeover attacks.

## Security Requirements (from RBAC Strategy)

Per **AUTHORIZATION_RBAC_STRATEGY.md** (Invite Token Hygiene section):

> Invite tokens must be treated like password-reset tokens:
> - high entropy (≥ 128-bit random)
> - **store only a hash** of the token at rest
> - short expiry (typical 24–72 hours)
> - one-time use (`accepted_at`)
> - rate limit on accept attempts
> - bind invite to:
>   - `tenant_id`
>   - intended `email` (or user id created in invited state)
> - audit log:
>   - who invited
>   - when
>   - for which tenant/user/email
>   - acceptance time and IP/user-agent if available

## Database Schema

### Table: `user_invitations`

```sql
CREATE TABLE user_invitations (
    invitation_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    
    -- Token (SECURITY: only hash stored, never plaintext)
    token_hash TEXT NOT NULL,              -- SHA-256 hash of the token
    
    -- Invitation target
    email VARCHAR(255) NOT NULL,
    invited_role VARCHAR(50) NOT NULL DEFAULT 'user',
    
    -- Inviter context
    invited_by_user_id UUID NOT NULL REFERENCES users(user_id),
    
    -- Status tracking
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, accepted, expired, revoked
    
    -- Expiry and acceptance
    expires_at TIMESTAMPTZ NOT NULL,       -- Default: 48 hours from creation
    accepted_at TIMESTAMPTZ,
    accepted_user_id UUID REFERENCES users(user_id),  -- User created on acceptance
    
    -- Request context for audit
    invited_from_ip TEXT,
    invited_from_user_agent TEXT,
    accepted_from_ip TEXT,
    accepted_from_user_agent TEXT,
    
    -- Rate limiting
    accept_attempts INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    
    -- Metadata
    custom_message TEXT,                   -- Optional message from inviter
    metadata JSONB DEFAULT '{}',
    
    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Soft delete (per project pattern)
    deleted_at TIMESTAMPTZ,                -- NULL = active, set = soft-deleted
    
    -- Constraints
    CONSTRAINT user_invitations_status_check CHECK (
        status IN ('pending', 'accepted', 'expired', 'revoked')
    ),
    CONSTRAINT user_invitations_role_check CHECK (
        invited_role IN ('owner', 'admin', 'manager', 'user', 'viewer')
    )
);

-- Indexes
CREATE INDEX idx_invitations_tenant ON user_invitations(tenant_id, status, created_at DESC);
CREATE INDEX idx_invitations_email ON user_invitations(email, tenant_id) WHERE status = 'pending' AND deleted_at IS NULL;
CREATE INDEX idx_invitations_token ON user_invitations(token_hash) WHERE status = 'pending' AND deleted_at IS NULL;
CREATE INDEX idx_invitations_expires ON user_invitations(expires_at) WHERE status = 'pending' AND deleted_at IS NULL;
CREATE INDEX idx_invitations_inviter ON user_invitations(invited_by_user_id, created_at DESC);
-- Soft-delete filtered index for active records
CREATE INDEX idx_invitations_active ON user_invitations(tenant_id, status) WHERE deleted_at IS NULL;

-- Partial unique index for pending invitations (PostgreSQL requires CREATE UNIQUE INDEX for partial unique)
CREATE UNIQUE INDEX idx_invitations_unique_pending ON user_invitations(tenant_id, email) 
    WHERE status = 'pending' AND deleted_at IS NULL;

COMMENT ON TABLE user_invitations IS 'Secure user invitation tokens with hash-at-rest';
COMMENT ON COLUMN user_invitations.token_hash IS 'SHA-256 hash of invite token - never store plaintext';
COMMENT ON COLUMN user_invitations.deleted_at IS 'Soft delete timestamp - NULL means active';
```

## Token Security Implementation

### Token Generation

```rust
use rand::RngCore;
use sha2::{Sha256, Digest};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

/// Generate a cryptographically secure invite token
/// Returns: (plaintext_token, token_hash)
pub fn generate_invite_token() -> (String, String) {
    // Generate 128-bit (16 bytes) of entropy
    let mut token_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    
    // Encode as URL-safe base64 for transmission
    let plaintext_token = URL_SAFE_NO_PAD.encode(&token_bytes);
    
    // Hash for storage (never store plaintext)
    let mut hasher = Sha256::new();
    hasher.update(plaintext_token.as_bytes());
    let hash_bytes = hasher.finalize();
    let token_hash = hex::encode(hash_bytes);
    
    (plaintext_token, token_hash)
}

/// Hash an incoming token for lookup
pub fn hash_token(plaintext_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plaintext_token.as_bytes());
    hex::encode(hasher.finalize())
}
```

### Token Validation Flow

```rust
pub async fn accept_invite(
    pool: &PgPool,
    token: &str,
    password: &str,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<User, AppError> {
    // 1. Hash the incoming token
    let token_hash = hash_token(token);
    
    // 2. Find invitation by hash
    let invitation = find_pending_invitation_by_hash(&token_hash).await?
        .ok_or(AppError::NotFound("Invalid or expired invitation".into()))?;
    
    // 3. Rate limit check (max 5 attempts per token)
    // NOTE: IP-based rate limiting (sub-task 8.1) should be applied BEFORE token lookup
    // to prevent token enumeration attacks. This per-token limit is a secondary defense.
    if invitation.accept_attempts >= 5 {
        return Err(AppError::TooManyRequests("Too many attempts".into()));
    }
    
    // 4. Increment attempt counter for this valid token
    // (IP-based rate limiting in middleware handles enumeration prevention)
    increment_accept_attempts(&invitation.invitation_id).await?;
    
    // 5. Check expiry
    if invitation.expires_at < Utc::now() {
        mark_invitation_expired(&invitation.invitation_id).await?;
        return Err(AppError::Gone("Invitation has expired".into()));
    }
    
    // 6. Check not already accepted (one-time use)
    if invitation.status != "pending" {
        return Err(AppError::Conflict("Invitation already used".into()));
    }
    
    // 7. Create user with invited role
    let user = create_user_from_invitation(&invitation, password).await?;
    
    // 8. Mark invitation as accepted
    mark_invitation_accepted(
        &invitation.invitation_id,
        &user.user_id,
        ip_address,
        user_agent,
    ).await?;
    
    // 9. Add Casbin grouping for the invited role
    add_role_for_user(&enforcer, &user.user_id, &invitation.invited_role, &invitation.tenant_id).await?;
    
    Ok(user)
}
```

## API Endpoints

### 1. Create Invitation (Admin)

```
POST /api/v1/admin/users/invite
Authorization: Bearer {admin_token}

Request:
{
    "email": "newuser@example.com",
    "role": "user",                    // optional, default: "user"
    "custom_message": "Welcome!"       // optional
}

Response (201 Created):
{
    "invitation_id": "uuid",
    "email": "newuser@example.com",
    "role": "user",
    "expires_at": "2026-01-06T12:00:00Z",
    "invite_link": "https://app.example.com/invite/{token}"  // Only shown once!
}
```

### 2. Accept Invitation (Public)

```
POST /api/v1/auth/accept-invite
Content-Type: application/json

Request:
{
    "token": "base64_encoded_token",
    "password": "SecurePassword123!",
    "full_name": "John Doe"            // optional
}

Response (201 Created):
{
    "access_token": "...",
    "refresh_token": "...",
    "token_type": "Bearer",
    "expires_in": 900,
    "user": {
        "id": "uuid",
        "email": "newuser@example.com",
        "role": "user",
        "tenant_id": "uuid"
    }
}
```

### 3. List Invitations (Admin)

```
GET /api/v1/admin/users/invitations?status=pending&page=1&page_size=20
Authorization: Bearer {admin_token}

Response:
{
    "invitations": [
        {
            "invitation_id": "uuid",
            "email": "pending@example.com",
            "role": "user",
            "status": "pending",
            "invited_by": { "id": "uuid", "email": "admin@example.com" },
            "expires_at": "2026-01-06T12:00:00Z",
            "created_at": "2026-01-04T12:00:00Z"
        }
    ],
    "total": 15,
    "page": 1,
    "page_size": 20
}
```

### 4. Revoke Invitation (Admin)

```
DELETE /api/v1/admin/users/invitations/{invitation_id}
Authorization: Bearer {admin_token}

Response: 204 No Content
```

### 5. Resend Invitation (Admin)

```
POST /api/v1/admin/users/invitations/{invitation_id}/resend
Authorization: Bearer {admin_token}

Response (200 OK):
{
    "invitation_id": "uuid",
    "email": "user@example.com",
    "new_expires_at": "2026-01-06T12:00:00Z",
    "invite_link": "https://app.example.com/invite/{new_token}"
}
```

## Specific Sub-tasks

- [ ] 1. Database schema
    - [ ] 1.1. Create migration for `user_invitations` table
    - [ ] 1.2. Add indexes for token lookup and cleanup
    - [ ] 1.3. Add unique constraint for pending invitations
- [ ] 2. Core layer (traits and DTOs)
    - [ ] 2.1. Create `InvitationRepository` trait
    - [ ] 2.2. Create invitation DTOs (CreateInvitationReq, InvitationResp, AcceptInviteReq)
    - [ ] 2.3. Create `InvitationService` trait
- [ ] 3. Token security
    - [ ] 3.1. Implement `generate_invite_token()` with 128-bit entropy
    - [ ] 3.2. Implement `hash_token()` using SHA-256
    - [ ] 3.3. Add token validation utilities
- [ ] 4. Infra layer (repository implementation)
    - [ ] 4.1. Implement `PgInvitationRepository`
    - [ ] 4.2. Implement token lookup by hash
    - [ ] 4.3. Implement rate limit tracking
    - [ ] 4.4. Implement expiry cleanup job
- [ ] 5. Service layer
    - [ ] 5.1. Implement `InvitationServiceImpl`
    - [ ] 5.2. Implement invite creation with hash storage
    - [ ] 5.3. Implement invite acceptance with validation
    - [ ] 5.4. Integrate with Casbin for role assignment
- [ ] 6. API handlers
    - [ ] 6.1. Create `POST /api/v1/admin/users/invite` endpoint
    - [ ] 6.2. Create `POST /api/v1/auth/accept-invite` endpoint (public)
    - [ ] 6.3. Create `GET /api/v1/admin/users/invitations` endpoint
    - [ ] 6.4. Create `DELETE /api/v1/admin/users/invitations/{id}` endpoint
    - [ ] 6.5. Create `POST /api/v1/admin/users/invitations/{id}/resend` endpoint
- [ ] 7. Email notifications (placeholder)
    - [ ] 7.1. Create email template for invitation
    - [ ] 7.2. Document email service integration requirements
    - [ ] 7.3. Add config for invite URL base
- [ ] 8. Rate limiting
    - [ ] 8.1. Add per-IP rate limit for accept-invite endpoint
    - [ ] 8.2. Add per-token attempt tracking
    - [ ] 8.3. Add per-admin invite creation limits
- [ ] 9. Audit logging
    - [ ] 9.1. Log invitation creation (who, when, for whom)
    - [ ] 9.2. Log invitation acceptance (IP, user-agent)
    - [ ] 9.3. Log failed acceptance attempts
- [ ] 10. Cleanup job
    - [ ] 10.1. Create scheduled task to expire old invitations
    - [ ] 10.2. Mark expired invitations as 'expired'
    - [ ] 10.3. Optionally purge very old invitation records
- [ ] 11. Testing
    - [ ] 11.1. Unit tests for token generation and hashing
    - [ ] 11.2. Unit tests for expiry validation
    - [ ] 11.3. Integration tests for invite flow
    - [ ] 11.4. Security tests: token replay, enumeration, timing attacks
- [ ] 12. Documentation
    - [ ] 12.1. Add OpenAPI annotations
    - [ ] 12.2. Document security properties
    - [ ] 12.3. Add runbook for invitation issues

## Acceptance Criteria

- [ ] Tokens use ≥ 128-bit entropy (cryptographically secure)
- [ ] Only token hash stored in database (never plaintext)
- [ ] Invitations expire after configurable period (default: 48 hours)
- [ ] One-time use: accepted invitations cannot be reused
- [ ] Rate limiting: max 5 acceptance attempts per token
- [ ] Invitation bound to tenant_id and email
- [ ] Audit trail: inviter, timestamp, acceptance IP/user-agent
- [ ] Admin can list, revoke, and resend invitations
- [ ] Accepted users have correct Casbin role assignment
- [ ] Password strength validation on acceptance
- [ ] Email notification capability (integration documented)
- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes

## Dependencies

- `V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.07_seed_default_roles_and_policies.md` (Done) - Roles must exist
- `V1_MVP/03_User_Service/3.6_Self_Auth_Enhancements/task_03.06.03_rate_limiting.md` (Todo) - Rate limit infrastructure

## Configuration

```toml
# Invitation settings
invitation_expiry_hours = 48
invitation_max_accept_attempts = 5
invitation_base_url = "https://app.example.com/invite"
invitation_email_enabled = false  # Enable when email service ready
```

## Security Checklist

- [ ] Token entropy ≥ 128 bits
- [ ] Token hash-at-rest (SHA-256)
- [ ] Token never logged or exposed after creation
- [ ] Rate limiting on accept endpoint
- [ ] Timing-safe token comparison (via hash lookup)
- [ ] Cross-tenant isolation verified
- [ ] Expired tokens rejected
- [ ] Replay attacks prevented (one-time use)
- [ ] Email enumeration mitigated (generic error messages)

## Related Documents

- `docs/AUTHORIZATION_RBAC_STRATEGY.md` - Invite token hygiene section
- `services/user_service/api/src/handlers.rs` - Auth handlers
- `migrations/` - Database schema

## Notes / Discussion

- Token is shown to admin only once at creation time
- Email service integration is documented but not implemented (future task)
- Consider adding invite link expiry warning emails (future enhancement)
- Resend creates a new token (old token invalidated)
- Admin can only invite roles ≤ their own level (prevent privilege escalation)

## AI Agent Log

---
* 2025-01-21: Task created for secure user invitation system
* 2026-01-04: Updated with detailed security requirements from AUTHORIZATION_RBAC_STRATEGY.md
    - Added hash-at-rest requirement
    - Added entropy specification (≥ 128-bit)
    - Added rate limiting details
    - Added audit logging requirements
    - Added comprehensive security checklist
    - Added token implementation examples
