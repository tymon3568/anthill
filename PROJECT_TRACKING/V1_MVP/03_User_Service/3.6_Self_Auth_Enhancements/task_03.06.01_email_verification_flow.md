# Task: Implement Email Verification Flow

**Task ID:** V1_MVP/03_User_Service/3.6_Self_Auth_Enhancements/task_03.06.01_email_verification_flow.md
**Version:** V1_MVP
**Phase:** 03_User_Service
**Module:** 3.6_Self_Auth_Enhancements
**Priority:** High
**Status:** Todo
**Assignee:** 
**Created Date:** 2026-01-04
**Last Updated:** 2026-01-04

## Detailed Description:
Implement a production-ready email verification flow for user registration. When users register, they should receive an email with a verification link. Unverified users should have limited access until they verify their email address.

**Security requirements:**
- Verification tokens must be cryptographically secure (random 32+ bytes)
- Tokens must have expiration (24 hours recommended)
- Tokens are single-use (invalidate after successful verification)
- Rate limit verification email resending (max 3 per hour)

**Email delivery:**
- Use SMTP for email delivery
- Support configurable email templates
- Handle bounce/failure scenarios gracefully

## Specific Sub-tasks:
- [ ] 1. Database Schema Changes
    - [ ] 1.1. Create `email_verification_tokens` table migration
    - [ ] 1.2. Add indexes for token lookup and expiration cleanup
- [ ] 2. Core Layer (DTOs and Traits)
    - [ ] 2.1. Create `EmailVerificationToken` domain model
    - [ ] 2.2. Create DTOs: `SendVerificationReq`, `VerifyEmailReq`, `VerificationResp`
    - [ ] 2.3. Define `EmailVerificationService` trait
    - [ ] 2.4. Define `EmailSender` trait for SMTP abstraction
- [ ] 3. Infrastructure Layer
    - [ ] 3.1. Implement `PgEmailVerificationRepository`
    - [ ] 3.2. Implement `SmtpEmailSender` using `lettre` crate
    - [ ] 3.3. Implement `EmailVerificationServiceImpl`
    - [ ] 3.4. Create email template for verification email (HTML + plain text)
- [ ] 4. API Layer
    - [ ] 4.1. Create `POST /api/v1/auth/verify-email` endpoint (submit token)
    - [ ] 4.2. Create `POST /api/v1/auth/resend-verification` endpoint
    - [ ] 4.3. Add OpenAPI annotations for new endpoints
    - [ ] 4.4. Update register endpoint to trigger verification email
- [ ] 5. Configuration
    - [ ] 5.1. Add SMTP config to `shared/config` (host, port, user, password, from_address)
    - [ ] 5.2. Add verification URL base config
    - [ ] 5.3. Update `.env.example` with new variables
- [ ] 6. Rate Limiting
    - [ ] 6.1. Implement rate limit for resend-verification endpoint
    - [ ] 6.2. Store rate limit data in Redis (or DB fallback)
- [ ] 7. Testing
    - [ ] 7.1. Unit tests for token generation and validation
    - [ ] 7.2. Integration tests for verification flow
    - [ ] 7.3. Test rate limiting behavior
    - [ ] 7.4. Test expired token handling
- [ ] 8. Documentation
    - [ ] 8.1. Update API documentation
    - [ ] 8.2. Document email template customization

## Acceptance Criteria:
- [ ] Users receive verification email upon registration
- [ ] Clicking verification link marks email as verified in DB
- [ ] Verification tokens expire after 24 hours
- [ ] Used tokens cannot be reused
- [ ] Rate limiting prevents email spam (max 3 resends/hour)
- [ ] Verification works with both HTML and plain text email clients
- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] API documentation updated with new endpoints

## Dependencies:
*   Task: `task_03.01.10_remove_kanidm_integration.md` (Status: InProgress_By_Claude)
*   SMTP server available for email sending (can use dev SMTP like Mailhog for testing)

## Related Documents:
*   `services/user_service/core/src/domains/auth/` - Auth domain
*   `services/user_service/api/src/handlers.rs` - Existing auth handlers
*   `shared/config/src/lib.rs` - Configuration module
*   `ARCHITECTURE.md` - System architecture

## Files to Create/Modify:
**New Files:**
- `migrations/YYYYMMDDHHMMSS_create_email_verification_tokens.sql`
- `services/user_service/core/src/domains/auth/dto/verification_dto.rs`
- `services/user_service/core/src/domains/auth/domain/email_verification.rs`
- `services/user_service/infra/src/auth/email_verification_repository.rs`
- `services/user_service/infra/src/auth/email_sender.rs`
- `services/user_service/infra/src/auth/email_verification_service.rs`
- `services/user_service/api/src/verification_handlers.rs`
- `services/user_service/templates/email_verification.html`
- `services/user_service/templates/email_verification.txt`

**Modified Files:**
- `services/user_service/core/src/domains/auth/mod.rs` (add new modules)
- `services/user_service/infra/src/auth/mod.rs` (add new modules)
- `services/user_service/api/src/main.rs` (add routes)
- `services/user_service/api/src/handlers.rs` (update register)
- `shared/config/src/lib.rs` (add SMTP config)
- `.env.example`

## Database Schema:
```sql
CREATE TABLE email_verification_tokens (
    token_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    user_id UUID NOT NULL REFERENCES users(user_id),
    tenant_id UUID NOT NULL REFERENCES tenants(tenant_id),
    token_hash VARCHAR(128) NOT NULL,  -- SHA256 hash of actual token
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_active_token UNIQUE (user_id, token_hash)
);

CREATE INDEX idx_verification_token_hash ON email_verification_tokens(token_hash);
CREATE INDEX idx_verification_token_expires ON email_verification_tokens(expires_at) WHERE used_at IS NULL;
```

## Notes / Discussion:
---
*   Consider using `lettre` crate for SMTP (mature, async support)
*   For development, use Mailhog (localhost:1025) as SMTP server
*   Token format: URL-safe base64 encoded 32 random bytes
*   Frontend will need a verification page to call the API with the token
*   Consider adding webhook for email bounce handling in future

## AI Agent Log:
---
*   2026-01-04 00:50: Task created as part of self-auth enhancement plan
    - Follows removal of Kanidm integration
    - Part of production-ready auth features
