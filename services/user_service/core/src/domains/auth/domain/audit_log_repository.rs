//! Authorization Audit Logging
//!
//! Provides audit trail for authorization events, policy changes, and role assignments.
//! Supports security monitoring, compliance, and incident investigation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_error::AppError;
use uuid::Uuid;

/// Types of audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Authorization decision (allow/deny)
    Authorization,
    /// Policy or role definition change
    PolicyChange,
    /// User role assignment change
    RoleAssignment,
    /// Security event (lockout, suspicious activity)
    Security,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::Authorization => write!(f, "authorization"),
            AuditEventType::PolicyChange => write!(f, "policy_change"),
            AuditEventType::RoleAssignment => write!(f, "role_assignment"),
            AuditEventType::Security => write!(f, "security"),
        }
    }
}

impl TryFrom<&str> for AuditEventType {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "authorization" => Ok(AuditEventType::Authorization),
            "policy_change" => Ok(AuditEventType::PolicyChange),
            "role_assignment" => Ok(AuditEventType::RoleAssignment),
            "security" => Ok(AuditEventType::Security),
            _ => Err(AppError::ValidationError(format!("Invalid audit event type: {}", value))),
        }
    }
}

/// Audit event to be logged
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

impl AuditEvent {
    /// Create an authorization decision event
    pub fn authorization(
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        resource: &str,
        action: &str,
        decision: bool,
        policy_version: i64,
    ) -> Self {
        Self {
            tenant_id,
            user_id,
            session_id: None,
            event_type: AuditEventType::Authorization,
            event_action: "enforce".to_string(),
            resource: Some(resource.to_string()),
            action: Some(action.to_string()),
            decision: Some(if decision { "allow" } else { "deny" }.to_string()),
            policy_version: Some(policy_version),
            target_entity_type: None,
            target_entity_id: None,
            old_value: None,
            new_value: None,
            ip_address: None,
            user_agent: None,
            request_id: None,
            metadata: None,
        }
    }

    /// Create a policy change event
    pub fn policy_change(
        tenant_id: Uuid,
        user_id: Uuid,
        action: &str,
        entity_type: &str,
        entity_id: &str,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
    ) -> Self {
        Self {
            tenant_id,
            user_id: Some(user_id),
            session_id: None,
            event_type: AuditEventType::PolicyChange,
            event_action: action.to_string(),
            resource: None,
            action: None,
            decision: None,
            policy_version: None,
            target_entity_type: Some(entity_type.to_string()),
            target_entity_id: Some(entity_id.to_string()),
            old_value,
            new_value,
            ip_address: None,
            user_agent: None,
            request_id: None,
            metadata: None,
        }
    }

    /// Create a role assignment event
    pub fn role_assignment(
        tenant_id: Uuid,
        actor_user_id: Uuid,
        target_user_id: Uuid,
        action: &str,
        role_name: &str,
        old_roles: Option<Vec<String>>,
        new_roles: Option<Vec<String>>,
    ) -> Self {
        Self {
            tenant_id,
            user_id: Some(actor_user_id),
            session_id: None,
            event_type: AuditEventType::RoleAssignment,
            event_action: action.to_string(),
            resource: None,
            action: None,
            decision: None,
            policy_version: None,
            target_entity_type: Some("user_role".to_string()),
            target_entity_id: Some(target_user_id.to_string()),
            old_value: old_roles
                .map(|r| serde_json::json!({ "roles": r, "changed_role": role_name })),
            new_value: new_roles
                .map(|r| serde_json::json!({ "roles": r, "changed_role": role_name })),
            ip_address: None,
            user_agent: None,
            request_id: None,
            metadata: None,
        }
    }

    /// Create a security event
    pub fn security(
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        action: &str,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            tenant_id,
            user_id,
            session_id: None,
            event_type: AuditEventType::Security,
            event_action: action.to_string(),
            resource: None,
            action: None,
            decision: None,
            policy_version: None,
            target_entity_type: None,
            target_entity_id: None,
            old_value: None,
            new_value: None,
            ip_address: None,
            user_agent: None,
            request_id: None,
            metadata,
        }
    }

    /// Add request context to the event
    pub fn with_request_context(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
        request_id: Option<Uuid>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self.request_id = request_id;
        self
    }

    /// Add session ID to the event
    pub fn with_session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Stored audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub audit_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub event_type: String,
    pub event_action: String,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub decision: Option<String>,
    pub policy_version: Option<i64>,
    pub target_entity_type: Option<String>,
    pub target_entity_id: Option<String>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Query parameters for audit log search
#[derive(Debug, Clone, Default)]
pub struct AuditLogQuery {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub event_action: Option<String>,
    pub decision: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub page: i32,
    pub page_size: i32,
}

impl AuditLogQuery {
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            tenant_id,
            page: 1,
            page_size: 50,
            ..Default::default()
        }
    }

    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_event_type(mut self, event_type: &str) -> Self {
        self.event_type = Some(event_type.to_string());
        self
    }

    pub fn with_decision(mut self, decision: &str) -> Self {
        self.decision = Some(decision.to_string());
        self
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn with_pagination(mut self, page: i32, page_size: i32) -> Self {
        self.page = page.max(1);
        self.page_size = page_size.clamp(1, 100);
        self
    }
}

/// Paginated audit log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogPage {
    pub logs: Vec<AuditLogEntry>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// Audit Log Repository Trait
///
/// Provides audit logging and query capabilities for authorization events.
/// Implementations should use async writes to avoid blocking request paths.
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// Write an audit event asynchronously
    ///
    /// This should not block the calling request. Implementations should use
    /// background workers or channels for non-blocking writes.
    async fn log(&self, event: AuditEvent) -> Result<(), AppError>;

    /// Write multiple audit events in batch
    ///
    /// For high-volume scenarios, batch inserts are more efficient.
    async fn log_batch(&self, events: Vec<AuditEvent>) -> Result<(), AppError>;

    /// Query audit logs with filters and pagination
    async fn query(&self, query: AuditLogQuery) -> Result<AuditLogPage, AppError>;

    /// Get a specific audit log entry by ID
    async fn get_by_id(
        &self,
        tenant_id: Uuid,
        audit_id: Uuid,
    ) -> Result<Option<AuditLogEntry>, AppError>;

    /// Cleanup old audit logs (for retention policy)
    ///
    /// Deletes logs older than the specified timestamp.
    /// Returns the number of deleted records.
    async fn cleanup_before(&self, before: DateTime<Utc>) -> Result<u64, AppError>;
}

/// No-op audit log repository for when auditing is disabled
pub struct NoOpAuditLogRepository;

#[async_trait]
impl AuditLogRepository for NoOpAuditLogRepository {
    async fn log(&self, _event: AuditEvent) -> Result<(), AppError> {
        Ok(())
    }

    async fn log_batch(&self, _events: Vec<AuditEvent>) -> Result<(), AppError> {
        Ok(())
    }

    async fn query(&self, query: AuditLogQuery) -> Result<AuditLogPage, AppError> {
        Ok(AuditLogPage {
            logs: vec![],
            total: 0,
            page: query.page,
            page_size: query.page_size,
        })
    }

    async fn get_by_id(
        &self,
        _tenant_id: Uuid,
        _audit_id: Uuid,
    ) -> Result<Option<AuditLogEntry>, AppError> {
        Ok(None)
    }

    async fn cleanup_before(&self, _before: DateTime<Utc>) -> Result<u64, AppError> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_type_display() {
        assert_eq!(AuditEventType::Authorization.to_string(), "authorization");
        assert_eq!(AuditEventType::PolicyChange.to_string(), "policy_change");
        assert_eq!(AuditEventType::RoleAssignment.to_string(), "role_assignment");
        assert_eq!(AuditEventType::Security.to_string(), "security");
    }

    #[test]
    fn test_audit_event_type_try_from() {
        assert_eq!(
            AuditEventType::try_from("authorization").unwrap(),
            AuditEventType::Authorization
        );
        assert_eq!(
            AuditEventType::try_from("policy_change").unwrap(),
            AuditEventType::PolicyChange
        );
        assert!(AuditEventType::try_from("invalid").is_err());
    }

    #[test]
    fn test_authorization_event_creation() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let event =
            AuditEvent::authorization(tenant_id, Some(user_id), "/api/v1/users", "POST", true, 5);

        assert_eq!(event.tenant_id, tenant_id);
        assert_eq!(event.user_id, Some(user_id));
        assert_eq!(event.event_type, AuditEventType::Authorization);
        assert_eq!(event.event_action, "enforce");
        assert_eq!(event.resource, Some("/api/v1/users".to_string()));
        assert_eq!(event.action, Some("POST".to_string()));
        assert_eq!(event.decision, Some("allow".to_string()));
        assert_eq!(event.policy_version, Some(5));
    }

    #[test]
    fn test_policy_change_event_creation() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let event = AuditEvent::policy_change(
            tenant_id,
            user_id,
            "create_role",
            "role",
            "admin",
            None,
            Some(serde_json::json!({"name": "admin", "permissions": ["read", "write"]})),
        );

        assert_eq!(event.event_type, AuditEventType::PolicyChange);
        assert_eq!(event.event_action, "create_role");
        assert_eq!(event.target_entity_type, Some("role".to_string()));
        assert_eq!(event.target_entity_id, Some("admin".to_string()));
    }

    #[test]
    fn test_role_assignment_event_creation() {
        let tenant_id = Uuid::new_v4();
        let actor_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();

        let event = AuditEvent::role_assignment(
            tenant_id,
            actor_id,
            target_id,
            "assign_role",
            "admin",
            Some(vec!["user".to_string()]),
            Some(vec!["user".to_string(), "admin".to_string()]),
        );

        assert_eq!(event.event_type, AuditEventType::RoleAssignment);
        assert_eq!(event.event_action, "assign_role");
        assert_eq!(event.target_entity_id, Some(target_id.to_string()));
    }

    #[test]
    fn test_security_event_creation() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let event = AuditEvent::security(
            tenant_id,
            Some(user_id),
            "account_locked",
            Some(serde_json::json!({"reason": "too_many_failed_attempts", "attempts": 5})),
        );

        assert_eq!(event.event_type, AuditEventType::Security);
        assert_eq!(event.event_action, "account_locked");
    }

    #[test]
    fn test_event_with_request_context() {
        let tenant_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();

        let event = AuditEvent::authorization(tenant_id, None, "/api/test", "GET", false, 1)
            .with_request_context(
                Some("192.168.1.1".to_string()),
                Some("Mozilla/5.0".to_string()),
                Some(request_id),
            );

        assert_eq!(event.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(event.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(event.request_id, Some(request_id));
    }

    #[test]
    fn test_audit_log_query_builder() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let query = AuditLogQuery::new(tenant_id)
            .with_user(user_id)
            .with_event_type("authorization")
            .with_decision("deny")
            .with_pagination(2, 25);

        assert_eq!(query.tenant_id, tenant_id);
        assert_eq!(query.user_id, Some(user_id));
        assert_eq!(query.event_type, Some("authorization".to_string()));
        assert_eq!(query.decision, Some("deny".to_string()));
        assert_eq!(query.page, 2);
        assert_eq!(query.page_size, 25);
    }

    #[test]
    fn test_pagination_clamping() {
        let tenant_id = Uuid::new_v4();

        let query = AuditLogQuery::new(tenant_id).with_pagination(-1, 500);

        assert_eq!(query.page, 1); // min 1
        assert_eq!(query.page_size, 100); // max 100
    }
}
