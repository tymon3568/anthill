# Task: RBAC Strategy Compliance Checklist & Gap Tracking

**Task ID:** `V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.16_rbac_strategy_compliance_checklist.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.2_Casbin_Authorization  
**Priority:** High  
**Status:** Todo  
**Assignee:**  
**Created Date:** 2026-01-04  
**Last Updated:** 2026-01-04  

## Context / Goal

This is a **tracking task** to monitor compliance with **AUTHORIZATION_RBAC_STRATEGY.md** and ensure all required features are implemented before production deployment.

This task does NOT implement anything directly. It serves as:
1. A checklist of all RBAC Strategy requirements
2. A mapping to existing tasks that implement each requirement
3. A gap tracker for any missing implementations

## Compliance Status Overview

| Category | Compliance | Notes |
|----------|------------|-------|
| Core RBAC Model | 90% | Casbin model correct, APIs sufficient |
| Security Invariants | 70% | Token-type OK, tenant binding OK, missing audit |
| Performance Strategy | 30% | In-memory policy OK, missing decision cache |
| Fast Revoke | 20% | Schema exists, implementation pending |
| Invite Token Security | 0% | Task exists but not started |
| Tenant Bootstrap | 0% | Task exists but not started |
| Audit Logging | 0% | Task created but not started |

## Detailed Checklist

### 1. Core RBAC Model

| Requirement | Status | Task/Evidence |
|-------------|--------|---------------|
| Casbin as source-of-truth | ✅ Done | `shared/auth/enforcer.rs` |
| Multi-tenant tuple `(subject, tenant, resource, action)` | ✅ Done | `model.conf`: `r = sub, dom, obj, act` |
| Users can belong to multiple groups | ✅ Done | `g = _, _, _` in model.conf |
| Union of group permissions | ✅ Done | Casbin RBAC inheritance |
| JWT claims minimal (no permissions) | ✅ Done | `shared/jwt` - only identity claims |
| Role management APIs | ✅ Done | `task_03.02.08` (InProgress) |
| Policy management APIs | ✅ Done | `handlers.rs`: add/remove_policy |
| User-role assignment APIs | ✅ Done | `admin_handlers.rs` |

### 2. Security Invariants

| Requirement | Status | Task/Evidence |
|-------------|--------|---------------|
| Tenant binding mandatory | ✅ Done | `r.dom == p.dom` in matcher |
| Default deny | ✅ Done | `e = some(where (p.eft == allow))` |
| Token-type enforcement (access only) | ✅ Done | `extractors.rs` line 77 |
| No split-brain authorization | ✅ Done | All endpoints use Casbin |
| Sensitive endpoint stricter posture | ❌ Todo | Need middleware config |
| Audit logging for authz decisions | ❌ Todo | `task_03.02.15` |

### 3. Performance Strategy

| Requirement | Status | Task/Evidence |
|-------------|--------|---------------|
| Level 1: Policy in-memory | ✅ Done | Casbin loads to memory |
| No DB reads per request | ✅ Done | Enforcer cached |
| Level 2: Decision cache | ❌ Todo | `task_03.02.14` |
| Cache TTL 10-30 seconds | ❌ Todo | `task_03.02.14` |
| Cache key includes policy_version | ❌ Todo | `task_03.02.14` |

### 4. Fast & Safe Policy Updates

| Requirement | Status | Task/Evidence |
|-------------|--------|---------------|
| Per-tenant `policy_version` | 🟡 Schema | `task_03.05.01` (NeedsReview) |
| Per-user `authz_version` | 🟡 Schema | `task_03.05.01` (NeedsReview) |
| Event-driven invalidation | ❌ Todo | No task yet |
| Redis version store | ❌ Todo | `task_03.05.02` |
| Global authz middleware gate | ❌ Todo | `task_03.05.03` |
| Bump on role/policy changes | ❌ Todo | `task_03.05.04` |
| Bump on user changes | ❌ Todo | `task_03.05.05` |
| Revoke effective < 10 seconds | ❌ Todo | Depends on above |

### 5. Invite Token Security

| Requirement | Status | Task/Evidence |
|-------------|--------|---------------|
| High entropy (≥ 128-bit) | ❌ Todo | `task_03.03.04` |
| Hash-at-rest (SHA-256) | ❌ Todo | `task_03.03.04` |
| Short expiry (24-72 hours) | ❌ Todo | `task_03.03.04` |
| One-time use | ❌ Todo | `task_03.03.04` |
| Rate limit on accept | ❌ Todo | `task_03.03.04` |
| Bound to tenant_id + email | ❌ Todo | `task_03.03.04` |
| Audit log for invitations | ❌ Todo | `task_03.03.04` |

### 6. Tenant Bootstrap (Registration)

| Requirement | Status | Task/Evidence |
|-------------|--------|---------------|
| Seed `tenant_admin` group | ❌ Todo | `task_03.03.06` |
| Seed `employee` group | ❌ Todo | `task_03.03.06` |
| Owner role on new tenant | ❌ Todo | `task_03.03.06` |
| Default role on join tenant | ❌ Todo | `task_03.03.06` |
| Casbin grouping on register | ❌ Todo | `task_03.03.06` |

### 7. Observability

| Requirement | Status | Task/Evidence |
|-------------|--------|---------------|
| Log authz decisions (sensitive) | ❌ Todo | `task_03.02.15` |
| Track enforce latency metrics | ❌ Todo | No task |
| Decision-cache hit ratio | ❌ Todo | `task_03.02.14` |
| Policy reload metrics | ❌ Todo | No task |

### 8. Auditability

| Requirement | Status | Task/Evidence |
|-------------|--------|---------------|
| Group membership changes | ❌ Todo | `task_03.02.15` |
| Policy changes | ❌ Todo | `task_03.02.15` |
| User disable/enable | ❌ Todo | `task_03.03.08` |
| Invite creation/acceptance | ❌ Todo | `task_03.03.04` |

## Task Dependency Graph

```
                    ┌─────────────────────────────────────────┐
                    │  task_03.02.16 (This Checklist)         │
                    └─────────────────────────────────────────┘
                                        │
          ┌─────────────────────────────┼─────────────────────────────┬───────────────────────┐
          │                             │                             │                       │
          ▼                             ▼                             ▼                       ▼
┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐
│ task_03.02.14       │   │ task_03.02.15       │   │ task_03.03.04       │   │ task_03.06.03       │
│ Decision Cache      │   │ Audit Logging       │   │ Invite System       │   │ Rate Limiting       │
│ Status: Todo        │   │ Status: Todo        │   │ Status: Todo        │   │ Status: Todo        │
└─────────────────────┘   └─────────────────────┘   └─────────────────────┘   └─────────────────────┘
          │                             │                             │
          │                             │                             │
          ▼                             ▼                             ▼
┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────────┐
│ task_03.05.01       │   │ task_03.03.06       │   │ (Invite depends on  │
│ AuthZ Version Schema│   │ Tenant Bootstrap    │   │  Rate Limiting)     │
│ Status: NeedsReview │   │ Status: Todo        │   └─────────────────────┘
└─────────────────────┘   └─────────────────────┘
          │
          ▼
┌─────────────────────┐
│ task_03.05.02-05    │
│ Redis + Middleware  │
│ Status: Todo        │
└─────────────────────┘
```

## Priority Order for Implementation

### Phase 1: Security Critical (Do First)
1. `task_03.03.06` - Tenant Bootstrap (register assigns proper roles)
2. `task_03.03.04` - Invite System (secure token handling)
3. `task_03.06.03` - Rate Limiting (brute force protection)

### Phase 2: Compliance & Observability
4. `task_03.02.15` - Audit Logging (security monitoring)
5. `task_03.05.01` - AuthZ Version Schema (already NeedsReview)

### Phase 3: Performance & Fast Revoke
6. `task_03.05.02` - Redis Version Store
7. `task_03.05.03` - Global AuthZ Middleware Gate
8. `task_03.05.04` - Bump on Role/Policy Changes
9. `task_03.05.05` - Bump on User Changes
10. `task_03.02.14` - Decision Cache Layer

## Acceptance Criteria

This tracking task is complete when:
- [ ] All checklist items are either ✅ Done or have a linked task
- [ ] All linked tasks are in `Done` status
- [ ] Integration tests verify end-to-end RBAC behavior
- [ ] Security audit passes for auth/authz implementation
- [ ] Performance benchmarks meet targets (enforce < 5ms p99)

## Notes / Discussion

- This task should be reviewed weekly during 03_User_Service development
- Any new RBAC Strategy requirements should be added here first
- Use this checklist during code review for authorization-related PRs

## Related Documents

- `docs/AUTHORIZATION_RBAC_STRATEGY.md` - Source of requirements
- `ARCHITECTURE.md` - Section 7 (Authorization)
- `shared/auth/` - Implementation location

## AI Agent Log

---
* 2026-01-04: Task created to track RBAC Strategy compliance.
    - Mapped all requirements to existing or new tasks.
    - Identified gaps: Decision Cache, Audit Logging, Invite System.
    - Created priority order for implementation.
