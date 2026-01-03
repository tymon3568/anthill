# Task: Implement Password Reset Flow

**Task ID:** V1_MVP/03_User_Service/3.6_Self_Auth_Enhancements/task_03.06.02_password_reset_flow.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.6_Self_Auth_Enhancements
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2026-01-04
**Last Updated:** 2026-01-04

## Detailed Description:
Implement a secure password reset flow for users who have forgotten their passwords. This is a critical security feature that must follow industry best practices to prevent account takeover attacks.

**Security requirements:**
- Reset tokens must be cryptographically secure (random 32+ bytes)
- Tokens must have short expiration (1 hour recommended)
- Tokens are single-use (invalidate after successful reset)
- Rate limit reset requests (max 3 per hour per email)
- Do not reveal whether email exists in system (timing-safe response)
- Invalidate all existing sessions after password reset
- Log password reset events for audit trail

**Flow:**
1. User requests password reset with email
2. If email exists, send reset link (always return success to prevent enumeration)
3. User clicks link, enters new password
4. System validates token, updates password, invalidates sessions
5. User must log in again with new password

## Specific Sub-tasks:
- [ ] 1. Database Schema Changes
    - [ ] 1.1. Create `password_reset_tokens` table migration
    - [ ] 1.2. Add indexes for token lookup and expiration cleanup
    - [ ] 1.3. Create `password_reset_audit` table for logging
- [ ] 2. Core Layer (DTOs and Traits)
    - [ ] 2.1. Create `PasswordResetToken` domain model
    - [ ] 2.2. Create DTOs: `RequestPasswordResetReq`, `ResetPasswordReq`, `ResetPasswordResp`
    - [ ] 2.3. Define `PasswordResetService` trait
    - [ ] 2.4. Add `reset_password` method to `AuthService` trait
- [ ] 3. Infrastructure Layer
    - [ ] 3.1. Implement `PgPasswordResetRepository`
    - [ ] 3.2. Implement `PasswordResetServiceImpl`
    - [ ] 3.3. Create email template for password reset (HTML + plain text)
    - [ ] 3.4. Implement session invalidation on password reset
- [ ] 4. API Layer
    - [ ] 4.1. Create `POST /api/v1/auth/forgot-password` endpoint (request reset)
    - [ ] 4.2. Create `POST /api/v1/auth/reset-password` endpoint (submit new password)
    - [ ] 4.3. Create `GET /api/v1/auth/reset-password/{token}/validate` endpoint (optional: validate token before showing form)
    - [ ] 4.4. Add OpenAPI annotations for new endpoints
- [ ] 5. Security Measures
    - [ ] 5.1. Implement timing-safe response for forgot-password
    - [ ] 5.2. Implement rate limiting for forgot-password endpoint
    - [ ] 5.3. Validate new password strength using zxcvbn
    - [ ] 5.4. Invalidate all user sessions after successful reset
    - [ ] 5.5. Log all reset attempts and completions
- [ ] 6. Testing
    - [ ] 6.1. Unit tests for token generation and validation
    - [ ] 6.2. Integration tests for complete reset flow
    - [ ] 6.3. Test rate limiting behavior
    - [ ] 6.4. Test expired token handling
    - [ ] 6.5. Test session invalidation
    - [ ] 6.6. Security tests (enumeration prevention, timing attacks)
- [ ] 7. Documentation
    - [ ] 7.1. Update API documentation
    - [ ] 7.2. Document security considerations
    - [ ] 7.3. Add admin guide for handling reset issues

## Acceptance Criteria:
- [ ] Users can request password reset via email
- [ ] Reset emails contain secure, time-limited tokens
- [ ] Reset tokens expire after 1 hour
- [ ] Used tokens cannot be reused
- [ ] Rate limiting prevents abuse (max 3 requests/hour)
- [ ] Email enumeration not possible (timing-safe responses)
- [ ] All sessions invalidated after password reset
- [ ] Password strength validated before accepting
- [ ] Audit log records all reset events
- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] API documentation updated with new endpoints

## Dependencies:
*   Task: `task_03.06.01_email_verification_flow.md` (Status: Todo) - Shares EmailSender infrastructure
*   Task: `task_03.01.10_remove_kanidm_integration.md` (Status: InProgress_By_Claude)
*   SMTP server available for email sending

## Related Documents:
*   `services/user_service/core/src/domains/auth/` - Auth domain
*   `services/user_service/api/src/handlers.rs` - Existing auth handlers
*   `services/user_service/infra/src/auth/session_repository.rs` - Session management
*   `shared/config/src/lib.rs` - Configuration module
*   `ARCHITECTURE.md` - System architecture
*   OWASP Password Reset Guidelines: https://cheatsheetseries.owasp.org/cheatsheets/Forgot_Password_Cheat_Sheet.html

## Files to Create/Modify:
**New Files:**
- `migrations/YYYYMMDDHHMMSS_create_password_reset_tokens.sql`
- `services/user_service/core/src/domains/auth/dto/password_reset_dto.rs`
- `services/user_service/core/src/domains/auth/domain/password_reset.rs`
- `services/user_service/infra/src/auth/password_reset_repository.rs`
- `services/user_service/infra/src/auth/password_reset_service.rs`
- `services/user_service/api/src/password_reset_handlers.rs`
- `services/user_service/templates/password_reset.html`
- `services/user_service/templates/password_reset.txt`

**Modified Files:**
- `services/user_service/core/src/domains/auth/mod.rs` (add new modules)
- `services/user_service/core/src/domains/auth/domain/service.rs` (add reset_password method)
- `services/user_service/infra/src/auth/mod.rs` (add new modules)
- `services/user_service/infra/src/auth/service.rs` (implement session invalidation)
- `services/user_service/api/src/main.rs` (add routes)
- `services/user_service/api/src/lib.rs` (export new handlers)

## Database Schema:
```sql
-- Password reset tokens table
CREATE TABLE password_reset_tokens (
    token_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id UUID NOT NULL REFERENCES users(user_id),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    token_hash VARCHAR(128) NOT NULL,  -- SHA256 hash of actual token
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45),  -- IP that requested the reset
    user_agent TEXT,
    
    CONSTRAINT unique_active_reset_token UNIQUE (user_id, token_hash)
);

CREATE INDEX idx_password_reset_token_hash ON password_reset_tokens(token_hash);
CREATE INDEX idx_password_reset_expires ON password_reset_tokens(expires_at) WHERE used_at IS NULL;
CREATE INDEX idx_password_reset_user ON password_reset_tokens(user_id, created_at DESC);

-- Audit log for password resets
CREATE TABLE password_reset_audit (
    audit_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id UUID REFERENCES users(user_id),
    tenant_id UUID REFERENCES tenants(tenant_id),
    email VARCHAR(255) NOT NULL,  -- Store email even if user doesn't exist
    event_type VARCHAR(50) NOT NULL,  -- 'requested', 'completed', 'failed', 'expired'
    ip_address VARCHAR(45),
    user_agent TEXT,
    failure_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_password_reset_audit_email ON password_reset_audit(email, created_at DESC);
CREATE INDEX idx_password_reset_audit_user ON password_reset_audit(user_id, created_at DESC);
```

## Rate Limiting Strategy:
```rust
// Rate limit key format: password_reset:{email_hash}
// Using email hash to prevent leaking email existence
// Limit: 3 requests per hour per email
// Implementation: Redis INCR with TTL or DB-based counter
```

## Email Template Example:
```html
Subject: Reset Your Password - Anthill

Hello,

We received a request to reset your password for your Anthill account.

Click the link below to reset your password:
{reset_url}

This link will expire in 1 hour.

If you did not request a password reset, please ignore this email.
Your password will remain unchanged.

For security, this request was received from:
- IP Address: {ip_address}
- Time: {timestamp}

Best regards,
The Anthill Team
```

## Notes / Discussion:
---
*   Use same EmailSender infrastructure as email verification (task 03.06.01)
*   Token format: URL-safe base64 encoded 32 random bytes (same as verification)
*   Store only hashed tokens in database (SHA256)
*   Consider adding CAPTCHA for forgot-password in future (anti-bot)
*   Frontend will need password reset pages (request form + reset form)
*   Consider implementing "password was recently changed" notification email
*   For MVP, use simple time-based rate limiting; consider sliding window for production

## Security Checklist:
- [ ] Tokens are cryptographically random (32+ bytes)
- [ ] Tokens are hashed before storage
- [ ] Tokens have short expiration (1 hour)
- [ ] Single-use tokens (marked used after reset)
- [ ] Rate limiting prevents abuse
- [ ] No email enumeration possible
- [ ] Sessions invalidated after reset
- [ ] Password strength validated
- [ ] All events logged for audit
- [ ] Secure email transmission (TLS)

## AI Agent Log:
---
*   2026-01-04 00:50: Task created as part of self-auth enhancement plan
    - Follows OWASP password reset guidelines
    - Critical security feature for production readiness
    - Depends on email infrastructure from task 03.06.01
