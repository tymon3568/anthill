# Security Quality Checklist for Production Enterprise

**Purpose:** This document serves as a security quality gate checklist that MUST be reviewed before completing any service or phase that handles authentication, authorization, or sensitive data.

**Last Updated:** 2026-01-18  
**Version:** 1.0

---

## How to Use This Checklist

1. Before marking a service/phase as "Done", review all applicable sections
2. Check each item that has been implemented and verified
3. Document any exceptions with justification
4. All "Critical" items MUST pass before production deployment
5. "Recommended" items should be addressed before v1.0 release

---

## 1. Authentication Security

### 1.1 Token Storage (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 1.1.1 | Access tokens stored in httpOnly cookies (not localStorage/sessionStorage) | ☐ | Prevents XSS token theft |
| 1.1.2 | Refresh tokens stored in httpOnly cookies | ☐ | Same protection as access tokens |
| 1.1.3 | Cookies have `secure` flag in production | ☐ | Only sent over HTTPS |
| 1.1.4 | Cookies have `sameSite=Strict` or `sameSite=Lax` | ☐ | CSRF protection |
| 1.1.5 | No tokens exposed in URL query parameters | ☐ | Prevents logging/referrer leaks |
| 1.1.6 | No tokens logged in server logs | ☐ | Audit log security |

### 1.2 Token Security (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 1.2.1 | JWT signed with strong secret (≥256 bits) | ☐ | HS256 minimum |
| 1.2.2 | JWT secret stored securely (env vars, not in code) | ☐ | Never commit secrets |
| 1.2.3 | Access token short-lived (15-60 minutes max) | ☐ | Limits exposure window |
| 1.2.4 | Refresh token has reasonable expiry (7-30 days) | ☐ | Balance security/UX |
| 1.2.5 | Token includes required claims (sub, exp, iat, tenant_id) | ☐ | Proper JWT structure |
| 1.2.6 | Algorithm explicitly validated (prevent "none" attack) | ☐ | Only accept HS256/RS256 |
| 1.2.7 | Token signature verified on every request | ☐ | Server-side validation |

### 1.3 Session Management (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 1.3.1 | Sessions stored in database/Redis | ☐ | Enables server-side revocation |
| 1.3.2 | Session includes IP address tracking | ☐ | Anomaly detection |
| 1.3.3 | Session includes user agent tracking | ☐ | Device identification |
| 1.3.4 | Logout invalidates refresh token server-side | ☐ | Immediate session termination |
| 1.3.5 | Session expiration enforced server-side | ☐ | Not just client-side |
| 1.3.6 | Concurrent session limit configurable | ☐ | Enterprise requirement |

### 1.4 Password Security (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 1.4.1 | Passwords hashed with bcrypt/argon2 (cost ≥ 10) | ☐ | Industry standard |
| 1.4.2 | Password minimum length ≥ 8 characters | ☐ | NIST recommendation |
| 1.4.3 | Password complexity requirements enforced | ☐ | Configurable policy |
| 1.4.4 | No password hints stored | ☐ | Security anti-pattern |
| 1.4.5 | Password reset tokens single-use | ☐ | Prevent replay |
| 1.4.6 | Password reset tokens expire (≤ 1 hour) | ☐ | Limit exposure |
| 1.4.7 | Account lockout after failed attempts | ☐ | Brute force protection |

---

## 2. Authorization Security

### 2.1 Multi-Tenant Isolation (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 2.1.1 | Tenant ID included in all queries | ☐ | Application-level filter |
| 2.1.2 | No cross-tenant data access possible | ☐ | Verified by security tests |
| 2.1.3 | Tenant context validated on every request | ☐ | Middleware enforcement |
| 2.1.4 | Admin privileges do not cross tenants | ☐ | Per-tenant admin scope |
| 2.1.5 | Deleted tenants cannot access data | ☐ | Soft-delete handling |

### 2.2 Role-Based Access Control (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 2.2.1 | RBAC enforced via middleware (Casbin/similar) | ☐ | Centralized policy |
| 2.2.2 | Default deny policy | ☐ | Explicit allow required |
| 2.2.3 | Role assignments audited | ☐ | Audit trail |
| 2.2.4 | Privilege escalation prevention | ☐ | Users cannot self-promote |
| 2.2.5 | Permission changes take immediate effect | ☐ | AuthZ versioning |

---

## 3. API Security

### 3.1 Input Validation (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 3.1.1 | All inputs validated (length, format, type) | ☐ | Server-side validation |
| 3.1.2 | SQL injection prevented (parameterized queries) | ☐ | Never string concat |
| 3.1.3 | XSS prevented (output encoding) | ☐ | JSON escaping |
| 3.1.4 | Path traversal prevented | ☐ | Validate file paths |
| 3.1.5 | Command injection prevented | ☐ | Never shell exec user input |
| 3.1.6 | Mass assignment prevented | ☐ | Explicit field allowlists |

### 3.2 Rate Limiting (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 3.2.1 | Login endpoint rate limited | ☐ | Prevent brute force |
| 3.2.2 | Registration endpoint rate limited | ☐ | Prevent spam |
| 3.2.3 | Password reset rate limited | ☐ | Prevent abuse |
| 3.2.4 | API-wide rate limiting per tenant | ☐ | DoS protection |
| 3.2.5 | Rate limit headers returned (X-RateLimit-*) | ☐ | Client visibility |

### 3.3 CORS and Headers (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 3.3.1 | CORS origin whitelist (not "*") | ☐ | Credential mode requirement |
| 3.3.2 | Access-Control-Allow-Credentials: true | ☐ | For cookie auth |
| 3.3.3 | Content-Security-Policy header set | ☐ | XSS mitigation |
| 3.3.4 | X-Content-Type-Options: nosniff | ☐ | MIME sniffing prevention |
| 3.3.5 | X-Frame-Options: DENY or SAMEORIGIN | ☐ | Clickjacking prevention |
| 3.3.6 | Strict-Transport-Security header (HSTS) | ☐ | Force HTTPS |

---

## 4. Data Protection

### 4.1 Sensitive Data Handling (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 4.1.1 | PII encrypted at rest | ☐ | Database encryption |
| 4.1.2 | Sensitive data not logged | ☐ | Passwords, tokens, PII |
| 4.1.3 | Sensitive data masked in error messages | ☐ | No stack traces to user |
| 4.1.4 | Database connections use TLS | ☐ | Encryption in transit |
| 4.1.5 | API responses don't expose internal IDs unnecessarily | ☐ | Information disclosure |

### 4.2 Audit Logging (Recommended)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 4.2.1 | Authentication events logged | ☐ | Login, logout, failures |
| 4.2.2 | Authorization failures logged | ☐ | 403 responses |
| 4.2.3 | Admin actions logged | ☐ | User/role management |
| 4.2.4 | Logs include request ID for tracing | ☐ | Correlation |
| 4.2.5 | Logs protected from tampering | ☐ | Read-only storage |

---

## 5. Infrastructure Security

### 5.1 Configuration (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 5.1.1 | No hardcoded secrets in code | ☐ | Use env vars |
| 5.1.2 | Different secrets per environment | ☐ | Dev ≠ Prod |
| 5.1.3 | Debug mode disabled in production | ☐ | No verbose errors |
| 5.1.4 | Default admin credentials disabled | ☐ | Force change on setup |
| 5.1.5 | Unnecessary services disabled | ☐ | Minimize attack surface |

### 5.2 Dependencies (Recommended)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 5.2.1 | Dependencies scanned for vulnerabilities | ☐ | cargo audit, npm audit |
| 5.2.2 | No known critical CVEs in dependencies | ☐ | Regular updates |
| 5.2.3 | Lockfiles committed to repository | ☐ | Reproducible builds |

---

## 6. Frontend Security

### 6.1 Client-Side Security (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 6.1.1 | No sensitive data in localStorage | ☐ | Only non-sensitive user info |
| 6.1.2 | API calls use credentials: 'include' | ☐ | For httpOnly cookies |
| 6.1.3 | No inline event handlers (onclick=) | ☐ | CSP compliance |
| 6.1.4 | External scripts from trusted sources only | ☐ | CDN integrity |
| 6.1.5 | Form inputs sanitized before display | ☐ | Prevent stored XSS |

### 6.2 SvelteKit Specific (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 6.2.1 | Server-side auth validation in hooks.server.ts | ☐ | Not just client-side |
| 6.2.2 | Protected routes check auth in +page.server.ts | ☐ | SSR protection |
| 6.2.3 | API routes validate auth | ☐ | +server.ts endpoints |
| 6.2.4 | CSP headers configured | ☐ | In hooks.server.ts |
| 6.2.5 | Form actions use CSRF protection | ☐ | SvelteKit built-in |

---

## 7. Testing Requirements

### 7.1 Security Testing (Critical)

| # | Requirement | Status | Notes |
|---|-------------|--------|-------|
| 7.1.1 | Tenant isolation tests pass | ☐ | See security_test_report.md |
| 7.1.2 | RBAC security tests pass | ☐ | See security_test_report.md |
| 7.1.3 | JWT security tests pass | ☐ | See security_test_report.md |
| 7.1.4 | SQL injection tests pass | ☐ | See security_test_report.md |
| 7.1.5 | Authentication flow tested end-to-end | ☐ | Manual or E2E tests |

---

## Checklist Summary

### Before Phase Completion:

1. **All Critical items must be checked** ☐
2. **Security tests must pass** ☐
3. **No known security vulnerabilities** ☐
4. **Code reviewed for security issues** ☐

### Before Production Deployment:

1. **All Critical and Recommended items checked** ☐
2. **Penetration testing completed** ☐
3. **Security audit documentation signed off** ☐
4. **Incident response plan in place** ☐

---

## Exception Documentation

If any Critical item cannot be implemented, document here:

| Item # | Reason | Risk Mitigation | Approved By | Date |
|--------|--------|-----------------|-------------|------|
| | | | | |

---

## Change History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-18 | AI Agent | Initial checklist creation |

---

## References

- [OWASP Top 10 2021](https://owasp.org/Top10/)
- [OWASP API Security Top 10](https://owasp.org/www-project-api-security/)
- [NIST Digital Identity Guidelines](https://pages.nist.gov/800-63-3/)
- [CWE/SANS Top 25](https://cwe.mitre.org/top25/)
- Internal: `docs/security_test_report.md`
- Internal: `docs/AUTHORIZATION_RBAC_STRATEGY.md`
