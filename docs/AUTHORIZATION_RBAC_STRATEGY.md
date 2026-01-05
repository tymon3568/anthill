# Authorization & RBAC Strategy (Odoo-like) — Anthill

**Status:** Proposed (recommended baseline)  
**Audience:** Backend engineers, platform engineers, reviewers  
**Scope:** Multi-tenant authorization model, how roles/groups and policies work, and how we keep it **secure** and **high-performance** at scale.

---

## Goals

- **Odoo-like RBAC**: permissions modeled via *Groups/Roles* and policies per module/resource/action.
- **Multi-tenant isolation**: authorization decisions must be bound to `tenant_id` (application-level isolation; no Postgres RLS).
- **Security-first**:
  - prevent cross-tenant access
  - prevent privilege escalation
  - support fast revoke / policy updates
- **High performance at scale**:
  - avoid DB roundtrips in the request hot path
  - stable p95/p99 behavior under high QPS and dense policy sets

---

## Non-Goals

- ABAC/attribute-level policy DSL beyond `(subject, tenant, resource, action)` for now.
- Fine-grained row-level authorization at DB-level (we do **not** use Postgres RLS).
- Full admin UI design for group/policy management (document only).

---

## Terms & Concepts

### Tenant
A logical customer boundary. Every tenant-scoped table includes `tenant_id UUID NOT NULL`, and repository queries must filter by tenant context.

### Subject
The principal being authorized. Typically `user_id` from JWT `sub`.

### Group / Role (Odoo-like)
- We use “role” and “group” interchangeably as a concept: a named collection of permissions.
- Users can belong to **multiple groups** within a tenant.
- Effective privileges are the **union** of all group permissions.

### Resource & Action
- **Resource**: a stable string identifier for a domain entity or capability, e.g. `inventory.product`, `inventory.stock_adjustment`, `sales.order`.
- **Action**: the operation, e.g. `read`, `create`, `update`, `delete`, `approve`, `export`, `import`.

> Recommendation: keep `action` small and consistent across services; avoid per-service ad-hoc verbs.

### Casbin
Casbin enforces policies. Our model is multi-tenant:

- **Authorization tuple**: `(subject, tenant, resource, action)`
- Policies stored in `casbin_rule`.

---

## Design Decision Summary (1/2/3)

### 1) Role model
✅ **Multi custom groups per tenant, user can belong to multiple groups** (Odoo-like).

This is required to support module-based permission sets without forcing one global role.

### 2) Provisioning: admin creates users
✅ **Invite flow**: admin invites user; user sets password via invite token.

Admin should not be responsible for choosing/password handling.

### 3) Source-of-truth
✅ **Casbin policy/group membership is source-of-truth**. JWT carries identity and tenant context, not permissions.

JWT claims minimal:
- `sub` (user id)
- `tenant_id`
- `exp`
- `token_type` (must be `"access"` for request auth)
- optional `email` for display (not used for authorization)
- optional `primary_role`/label for UI (not used for authorization)

---

## Security Model & Invariants (Must-Haves)

### 1) Tenant binding is mandatory
- `tenant_id` must come from authenticated context (JWT → request context).
- Authorization must always include tenant:
  - `enforce(subject, tenant_id, resource, action)`
- Never authorize using tenant_id from request body/header without verifying it matches JWT.

### 2) Default deny
If a policy is missing or evaluation fails:
- **deny** by default.

### 3) Token-type enforcement
Only **access tokens** can authorize API requests.
- Refresh tokens must not pass auth middleware/extractors.

### 4) No “split-brain” authorization
Do not mix:
- “role string in JWT decides access” in some endpoints
- Casbin permission checks in other endpoints

Rule:
- For any protected endpoint, the definitive decision must be made via Casbin permission checks (or a single standardized wrapper that maps admin group → permissions).

### 5) Sensitive endpoints require stricter posture
Even with caching (see below), the following categories must be treated as sensitive:
- user management (invite/create/disable/reset)
- role/group/policy management
- data export/reporting endpoints
- approval workflows (if applicable)
- security settings (sessions, tokens, secrets)

Mitigations:
- bypass decision-cache for these endpoints (still in-memory Casbin enforce)
- require “fresh” policy version checks (see caching section)
- ensure audit logging is enabled for these actions

### 6) Invite token hygiene (account takeover prevention)
Invite tokens must be treated like password-reset tokens:
- high entropy (≥ 128-bit random)
- **store only a hash** of the token at rest
- short expiry (typical 24–72 hours)
- one-time use (`accepted_at`)
- rate limit on accept attempts
- bind invite to:
  - `tenant_id`
  - intended `email` (or user id created in invited state)
- audit log:
  - who invited
  - when
  - for which tenant/user/email
  - acceptance time and IP/user-agent if available

---

## Performance Strategy (Why we still cache even with Rust)

Rust makes in-process execution fast, but authorization at scale is dominated by:
- DB/network roundtrips (ms)
- policy evaluation cost multiplied by QPS
- lock contention in shared enforcer structures

Therefore we must avoid DB reads on the request path and minimize repeated expensive enforcement checks.

### Two-level caching model

#### Level 1 (required): Policy in-memory
- Services should not query `casbin_rule` per request.
- Policies (and group mappings) must live in memory in each service instance.
- Policy changes are applied via reload/update, not per-request DB fetch.

#### Level 2 (recommended at scale): Decision cache (allow/deny)
Cache enforcement results for a short time:
- Key: `(tenant_id, policy_version, subject, resource, action) -> allow/deny`
- TTL: **10–30 seconds** recommended

This reduces:
- CPU cost of repeated enforcement
- enforcer contention under high QPS
- tail latencies under dense policy sets

---

## Fast & Safe Policy Updates: Versioning + Event Invalidation

### Target: revoke effective in < 10 seconds (general), < 1 second (sensitive)
We choose a hybrid approach:

1) **Per-tenant `policy_version` integer**
- Stored in DB (tenant metadata table or a dedicated table).
- Increment on any of:
  - group membership change (user added/removed to group)
  - policy change (casbin_rule changes)
  - user disabled / tenant admin “logout all”

2) **Event-driven invalidation**
- Publish an event: `TenantPolicyUpdated { tenant_id, policy_version }`
- Each service instance:
  - updates its in-memory `policy_version` for that tenant
  - clears decision-cache entries for that tenant (optional if version is in cache key)
  - reloads policies if needed (policy changes), or updates incremental mapping

### Why versioning is useful
If cache key includes `policy_version`, old decisions become unreachable immediately once the version changes. This is safer than TTL-only caching because it eliminates “silent” stale hits.

### Failure modes & fallbacks
- If events are delayed:
  - TTL provides a bounded fallback, but you must treat this as degraded security posture.
- If event bus is unavailable:
  - periodic refresh of `policy_version` can be used (slower revoke), but should be treated as a temporary fallback.

---

## Naming Conventions (Resource & Action)

### Recommended format
- `resource`: `module.entity` (lowercase, dot-separated)
  - examples: `inventory.product`, `inventory.stock_adjustment`, `user.user`, `rbac.policy`
- `action`: fixed set:
  - `read`, `create`, `update`, `delete`, `approve`, `export`, `import`

### Principles
- Keep resources stable across services.
- Avoid leaking internal route paths into resource names.
- Prefer coarse first; add special actions only if needed.

---

## Default Groups & Seeding (Tenant Bootstrap)

When a tenant is created (e.g., during registration), seed at minimum:

### Groups
- `tenant_admin`
- `employee` (or `tenant_user`)

### Policies
- `tenant_admin`: allow `*` on `*` within tenant (or a curated “all admin actions” set)
- `employee`: minimal baseline needed for daily operations (prefer read-only at first)

> Tip: Keep seeding conservative; it’s easier to grant than revoke.

---

## Operational Guidance

### Observability
- Log authorization decisions for sensitive endpoints with:
  - tenant_id, user_id, resource, action, decision, policy_version
- Track metrics:
  - enforce latency (p50/p95/p99)
  - decision-cache hit ratio
  - policy reload counts and duration
  - policy update event lag

### Auditability
Maintain an audit trail for:
- group membership changes
- policy changes
- user disable/enable
- invite creation/acceptance

### Testing Matrix (minimum)
- Cross-tenant isolation: user A cannot access tenant B data even if IDs are guessed.
- Permission deny: missing policy denies.
- Permission grant: correct policy and group grants.
- Revoke behavior:
  - remove user from admin group → access removed within target window.
- Token misuse:
  - refresh token cannot access protected endpoints.
- Invite:
  - expired token rejected
  - token replay rejected
  - wrong-tenant token rejected

---

## Migration & Compatibility Notes

- Some legacy code may still contain `role` as a JWT claim.
  - It may be kept for UI labeling or transitional compatibility,
  - but must not be used as the definitive authorization source once Casbin is in place.
- Policy versioning can be introduced without breaking APIs:
  - it is internal control-plane state.

---

## Open Questions / Future Enhancements

- Do we need sub-tenant scopes (e.g., warehouse/company) beyond tenant?  
  If yes, we may need to extend the enforcement tuple or encode scope into `resource`.
- Should sensitive actions require step-up auth (re-auth / 2FA) later?
- Should we add an admin UI for RBAC management (group/policy editor, audit views)?

---

## Quick Summary

- Use **Casbin** as the only truth for authorization decisions (Odoo-like RBAC).
- Users have **multiple groups**; permissions are union of group policies.
- Hot path must be **in-memory enforce** (no DB reads per request).
- Add **decision caching** with TTL 10–30s, keyed by `policy_version`.
- Use **policy_version + events** for fast revocation and safe cache invalidation.
- Keep invite tokens secure: random, hashed-at-rest, expiry, one-time, rate limited, audited.
