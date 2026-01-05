# Task: Implement Authorization Audit Logging System

**Task ID:** `V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.15_implement_authz_audit_logging.md`  
**Version:** V1_MVP  
**Phase:** 03_User_Service  
**Module:** 3.2_Casbin_Authorization  
**Priority:** Medium  
**Status:** Todo  
**Assignee:**  
**Created Date:** 2026-01-04  
**Last Updated:** 2026-01-04  

## Context / Goal

Implement comprehensive **authorization audit logging** to support security monitoring, compliance, and incident investigation.

Per **AUTHORIZATION_RBAC_STRATEGY.md** (Observability section):
> Log authorization decisions for sensitive endpoints with:
> - tenant_id, user_id, resource, action, decision, policy_version

This task creates a structured audit trail for:
1. **Authorization decisions** (allow/deny on protected endpoints)
2. **Policy changes** (role/permission CRUD operations)
3. **Role assignment changes** (user added/removed from roles)
4. **Security events** (account lockout, suspicious patterns)

## Requirements

### Functional

- Log all authorization decisions for **sensitive endpoints** (configurable list)
- Log all **policy management operations** (create/update/delete role, add/remove policy)
- Log all **role assignment changes** (assign/revoke user role)
- Store logs in database for queryability and retention
- Support log export for SIEM integration (future)
- Provide admin API to query audit logs

### Non-Functional

- **Performance**: Logging must not add > 5ms latency to requests (async write)
- **Reliability**: Audit writes should be best-effort, not blocking request completion
- **Retention**: Configurable retention period (default: 90 days)
- **Compliance**: Logs must be immutable (append-only, no updates/deletes by API)
- **Privacy**: No sensitive data in logs (no passwords, tokens, PII beyond user_id)

### Architecture (3-crate pattern)

- `core/`: Audit log trait and DTOs
- `infra/`: PostgreSQL implementation
- `api/`: Middleware integration and admin query endpoints

## Database Schema

### Table: `authz_audit_logs`

```sql
CREATE TABLE authz_audit_logs (
    audit_id UUID PRIMARY KEY DEFAULT uuid_generate_v7(),
    
    -- Context
    tenant_id UUID NOT NULL,
    user_id UUID,                          -- NULL for system/anonymous actions
    session_id UUID,                       -- Link to session if available
    
    -- Event details
    event_type VARCHAR(50) NOT NULL,       -- 'authorization', 'policy_change', 'role_assignment', 'security'
    event_action VARCHAR(100) NOT NULL,    -- 'enforce', 'create_role', 'assign_role', 'account_locked', etc.
    
    -- Authorization context (for event_type = 'authorization')
    resource VARCHAR(255),                 -- e.g., '/api/v1/users'
    action VARCHAR(50),                    -- e.g., 'POST', 'read', 'write'
    decision VARCHAR(10),                  -- 'allow', 'deny'
    policy_version BIGINT,                 -- tenant's authz_version at decision time
    
    -- Change context (for policy/role changes)
    target_entity_type VARCHAR(50),        -- 'role', 'policy', 'user_role'
    target_entity_id VARCHAR(255),         -- role name, policy key, or user_id
    old_value JSONB,                       -- Previous state (for updates)
    new_value JSONB,                       -- New state
    
    -- Request context
    ip_address TEXT,
    user_agent TEXT,
    request_id UUID,                       -- Correlation ID for distributed tracing
    
    -- Metadata
    metadata JSONB DEFAULT '{}',           -- Additional context
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT authz_audit_event_type_check CHECK (
        event_type IN ('authorization', 'policy_change', 'role_assignment', 'security')
    )
);

-- Indexes for common queries
CREATE INDEX idx_authz_audit_tenant_time ON authz_audit_logs(tenant_id, created_at DESC);
CREATE INDEX idx_authz_audit_user_time ON authz_audit_logs(user_id, created_at DESC) WHERE user_id IS NOT NULL;
CREATE INDEX idx_authz_audit_event_type ON authz_audit_logs(tenant_id, event_type, created_at DESC);
CREATE INDEX idx_authz_audit_decision_deny ON authz_audit_logs(tenant_id, created_at DESC) WHERE decision = 'deny';
CREATE INDEX idx_authz_audit_security ON authz_audit_logs(tenant_id, created_at DESC) WHERE event_type = 'security';

-- Partitioning by month for large-scale deployments (optional)
-- Consider: CREATE TABLE authz_audit_logs (...) PARTITION BY RANGE (created_at);

COMMENT ON TABLE authz_audit_logs IS 'Immutable audit trail for authorization events';
```

## Core Trait

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_error::AppError;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    Authorization,
    PolicyChange,
    RoleAssignment,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub event_type: AuditEventType,
    pub event_action: String,
    
    // Authorization context
    pub resource: Option<String>,
    pub action: Option<String>,
    pub decision: Option<String>,
    pub policy_version: Option<i64>,
    
    // Change context
    pub target_entity_type: Option<String>,
    pub target_entity_id: Option<String>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    
    // Request context
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub audit_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub event_action: String,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub decision: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct AuditLogQuery {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub decision: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub page: i32,
    pub page_size: i32,
}

#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// Write an audit event (async, non-blocking)
    async fn log(&self, event: AuditEvent) -> Result<(), AppError>;
    
    /// Query audit logs with filters
    async fn query(&self, query: AuditLogQuery) -> Result<(Vec<AuditLogEntry>, i64), AppError>;
    
    /// Get audit log by ID
    async fn get_by_id(&self, tenant_id: Uuid, audit_id: Uuid) -> Result<Option<AuditLogEntry>, AppError>;
    
    /// Cleanup old logs (for retention policy)
    async fn cleanup_before(&self, before: DateTime<Utc>) -> Result<u64, AppError>;
}
```

## Sensitive Endpoints (Default List)

Authorization decisions should be logged for these endpoints:

```rust
pub const SENSITIVE_ENDPOINTS: &[&str] = &[
    // User management
    "/api/v1/admin/users/*",
    "/api/v1/admin/users/invite",
    
    // Role/Policy management
    "/api/v1/admin/roles",
    "/api/v1/admin/roles/*",
    "/api/v1/admin/policies",
    
    // Security settings
    "/api/v1/admin/security/*",
    "/api/v1/auth/logout",
    
    // Data export
    "/api/v1/*/export",
    
    // Tenant settings
    "/api/v1/admin/tenant/*",
];
```

## Specific Sub-tasks

- [ ] 1. Database schema
  - [ ] 1.1. Create migration for `authz_audit_logs` table
  - [ ] 1.2. Add indexes for common query patterns
  - [ ] 1.3. Document partitioning strategy for scale (optional)
- [ ] 2. Core layer
  - [ ] 2.1. Define `AuditEvent` and `AuditLogEntry` DTOs
  - [ ] 2.2. Define `AuditLogRepository` trait
  - [ ] 2.3. Define `AuditLogQuery` filters
- [ ] 3. Infra layer
  - [ ] 3.1. Implement `PgAuditLogRepository`
  - [ ] 3.2. Use async channel for non-blocking writes
  - [ ] 3.3. Implement batch insert for performance
  - [ ] 3.4. Implement query with pagination
- [ ] 4. Authorization event logging
  - [ ] 4.1. Create `AuditingEnforcerWrapper` or middleware hook
  - [ ] 4.2. Log deny decisions always
  - [ ] 4.3. Log allow decisions for sensitive endpoints only
  - [ ] 4.4. Include policy_version in logged events
- [ ] 5. Policy change logging
  - [ ] 5.1. Add audit logging to `create_role` handler
  - [ ] 5.2. Add audit logging to `update_role` handler
  - [ ] 5.3. Add audit logging to `delete_role` handler
  - [ ] 5.4. Add audit logging to `add_policy`/`remove_policy` handlers
- [ ] 6. Role assignment logging
  - [ ] 6.1. Add audit logging to `assign_role_to_user` handler
  - [ ] 6.2. Add audit logging to `remove_role_from_user` handler
  - [ ] 6.3. Log old and new role states
- [ ] 7. Admin query API
  - [ ] 7.1. Create `GET /api/v1/admin/audit-logs` endpoint
  - [ ] 7.2. Implement filtering (user, event_type, time range)
  - [ ] 7.3. Implement pagination
  - [ ] 7.4. Add OpenAPI documentation
- [ ] 8. Retention and cleanup
  - [ ] 8.1. Create scheduled cleanup job/task
  - [ ] 8.2. Make retention period configurable (default: 90 days)
  - [ ] 8.3. Document manual cleanup procedure
- [ ] 9. Testing
  - [ ] 9.1. Unit tests for audit event creation
  - [ ] 9.2. Integration tests for audit log storage
  - [ ] 9.3. Test query filters and pagination
  - [ ] 9.4. Test that sensitive endpoints are logged
- [ ] 10. Configuration
  - [ ] 10.1. Add `AUDIT_LOG_ENABLED` env var
  - [ ] 10.2. Add `AUDIT_LOG_RETENTION_DAYS` env var
  - [ ] 10.3. Add `AUDIT_LOG_SENSITIVE_ENDPOINTS` configurable list
  - [ ] 10.4. Document configuration options

## Acceptance Criteria

- [ ] All authorization denials are logged with full context
- [ ] Sensitive endpoint authorizations (allow) are logged
- [ ] Policy CRUD operations are logged with before/after state
- [ ] Role assignment changes are logged
- [ ] Audit logs are queryable via admin API
- [ ] Logs include: tenant_id, user_id, resource, action, decision, policy_version
- [ ] Logging is async and does not block requests
- [ ] Retention cleanup works and is configurable
- [ ] No sensitive data (passwords, tokens) in logs
- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes

## Dependencies

- `V1_MVP/03_User_Service/3.5_Authz_Versioning/task_03.05.01_add_authz_versioning_schema.md` (Status: NeedsReview) - Provides `policy_version`
- `V1_MVP/03_User_Service/3.2_Casbin_Authorization/task_03.02.08_create_role_management_apis.md` (Status: InProgress) - Handlers to instrument

## Configuration

```toml
# Audit logging configuration
audit_log_enabled = true
audit_log_retention_days = 90
audit_log_batch_size = 100
audit_log_flush_interval_ms = 1000
```

## Admin API

### List Audit Logs

```
GET /api/v1/admin/audit-logs
```

**Query Parameters:**
- `user_id` (optional): Filter by user
- `event_type` (optional): authorization, policy_change, role_assignment, security
- `decision` (optional): allow, deny
- `start_time` (optional): ISO 8601 timestamp
- `end_time` (optional): ISO 8601 timestamp
- `page` (default: 1)
- `page_size` (default: 50, max: 100)

**Response:**
```json
{
  "logs": [
    {
      "audit_id": "uuid",
      "tenant_id": "uuid",
      "user_id": "uuid",
      "event_type": "authorization",
      "event_action": "enforce",
      "resource": "/api/v1/admin/users",
      "action": "POST",
      "decision": "deny",
      "policy_version": 5,
      "created_at": "2026-01-04T12:00:00Z"
    }
  ],
  "total": 1234,
  "page": 1,
  "page_size": 50
}
```

### Get Audit Log Detail

```
GET /api/v1/admin/audit-logs/{audit_id}
```

Returns full audit log entry with all metadata.

## Notes / Discussion

- **Async writes**: Use `tokio::spawn` or channel-based writer to avoid blocking request path.
- **Batch inserts**: For high volume, batch multiple events into single INSERT.
- **SIEM integration**: Future task can add log streaming to external systems (Elasticsearch, Splunk).
- **Immutability**: No update/delete APIs for audit logs; only admin can query.
- **Cross-tenant security**: Audit log queries must be tenant-scoped.
- **Log sampling**: For extremely high traffic, consider sampling allow decisions (log 10%).

## Related Documents

- `docs/AUTHORIZATION_RBAC_STRATEGY.md` - Observability section
- `services/user_service/api/src/admin_handlers.rs` - Handlers to instrument
- `services/user_service/api/src/handlers.rs` - Policy handlers to instrument

## AI Agent Log

---
* 2026-01-04: Task created to implement authorization audit logging per AUTHORIZATION_RBAC_STRATEGY.md requirements.
  - Covers authorization decisions, policy changes, and role assignments.
  - Includes admin query API for security monitoring.
