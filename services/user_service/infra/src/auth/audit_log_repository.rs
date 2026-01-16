//! PostgreSQL-backed Audit Log Repository Implementation
//!
//! Provides persistent storage for authorization audit events.
//! Uses async channels for non-blocking writes to avoid impacting request latency.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared_error::AppError;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use user_service_core::domains::auth::domain::audit_log_repository::{
    AuditEvent, AuditLogEntry, AuditLogPage, AuditLogQuery, AuditLogRepository,
};
use uuid::Uuid;

/// Channel buffer size for async audit log writes
const AUDIT_CHANNEL_BUFFER: usize = 1000;

/// Batch size for bulk inserts
const BATCH_INSERT_SIZE: usize = 100;

/// PostgreSQL-backed Audit Log Repository
///
/// Uses an async channel to buffer audit events and write them in batches,
/// ensuring that audit logging does not block request processing.
pub struct PgAuditLogRepository {
    pool: PgPool,
    sender: mpsc::Sender<AuditEvent>,
}

impl PgAuditLogRepository {
    /// Create a new PgAuditLogRepository with background writer
    ///
    /// Spawns a background task that consumes audit events from the channel
    /// and writes them to the database in batches.
    pub fn new(pool: PgPool) -> Arc<Self> {
        let (sender, receiver) = mpsc::channel(AUDIT_CHANNEL_BUFFER);

        let repo = Arc::new(Self {
            pool: pool.clone(),
            sender,
        });

        // Spawn background writer task
        let writer_pool = pool;
        tokio::spawn(async move {
            Self::background_writer(writer_pool, receiver).await;
        });

        info!("PgAuditLogRepository initialized with background writer");
        repo
    }

    /// Create a synchronous repository (for testing)
    ///
    /// This version writes directly without background batching.
    pub fn new_sync(pool: PgPool) -> Arc<Self> {
        let (sender, _receiver) = mpsc::channel(1);
        Arc::new(Self { pool, sender })
    }

    /// Background writer task that consumes events and writes in batches
    async fn background_writer(pool: PgPool, mut receiver: mpsc::Receiver<AuditEvent>) {
        let mut batch: Vec<AuditEvent> = Vec::with_capacity(BATCH_INSERT_SIZE);
        let mut flush_interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            tokio::select! {
                // Receive new event
                event = receiver.recv() => {
                    match event {
                        Some(e) => {
                            batch.push(e);
                            if batch.len() >= BATCH_INSERT_SIZE {
                                Self::flush_batch(&pool, &mut batch).await;
                            }
                        }
                        None => {
                            // Channel closed, flush remaining and exit
                            if !batch.is_empty() {
                                Self::flush_batch(&pool, &mut batch).await;
                            }
                            info!("Audit log background writer shutting down");
                            break;
                        }
                    }
                }
                // Periodic flush
                _ = flush_interval.tick() => {
                    if !batch.is_empty() {
                        Self::flush_batch(&pool, &mut batch).await;
                    }
                }
            }
        }
    }

    /// Flush a batch of events to the database
    async fn flush_batch(pool: &PgPool, batch: &mut Vec<AuditEvent>) {
        if batch.is_empty() {
            return;
        }

        let count = batch.len();
        debug!("Flushing {} audit events to database", count);

        match Self::insert_batch(pool, batch).await {
            Ok(_) => {
                debug!("Successfully wrote {} audit events", count);
            },
            Err(e) => {
                error!("Failed to write audit events: {}", e);
                // Events are lost, but we don't want to block or retry indefinitely
                // In production, consider writing to a fallback (file, dead letter queue)
            },
        }

        batch.clear();
    }

    /// Insert a batch of events using a single query
    async fn insert_batch(pool: &PgPool, events: &[AuditEvent]) -> Result<(), AppError> {
        if events.is_empty() {
            return Ok(());
        }

        // Build bulk insert query
        let mut query = String::from(
            r#"
            INSERT INTO authz_audit_logs (
                tenant_id, user_id, session_id, event_type, event_action,
                resource, action, decision, policy_version,
                target_entity_type, target_entity_id, old_value, new_value,
                ip_address, user_agent, request_id, metadata
            ) VALUES
            "#,
        );

        let mut params: Vec<String> = Vec::new();
        let mut param_idx = 1;

        for (i, _event) in events.iter().enumerate() {
            if i > 0 {
                query.push_str(", ");
            }
            query.push_str(&format!(
                "(${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${})",
                param_idx, param_idx + 1, param_idx + 2, param_idx + 3, param_idx + 4,
                param_idx + 5, param_idx + 6, param_idx + 7, param_idx + 8,
                param_idx + 9, param_idx + 10, param_idx + 11, param_idx + 12,
                param_idx + 13, param_idx + 14, param_idx + 15, param_idx + 16
            ));
            params.push(format!("event_{}", i));
            param_idx += 17;
        }

        // Use individual inserts for simplicity (batch insert with dynamic params is complex in sqlx)
        // For high-volume production, consider using COPY or prepared statements
        for event in events {
            sqlx::query(
                r#"
                INSERT INTO authz_audit_logs (
                    tenant_id, user_id, session_id, event_type, event_action,
                    resource, action, decision, policy_version,
                    target_entity_type, target_entity_id, old_value, new_value,
                    ip_address, user_agent, request_id, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
                "#,
            )
            .bind(event.tenant_id)
            .bind(event.user_id)
            .bind(event.session_id)
            .bind(event.event_type.to_string())
            .bind(&event.event_action)
            .bind(&event.resource)
            .bind(&event.action)
            .bind(&event.decision)
            .bind(event.policy_version)
            .bind(&event.target_entity_type)
            .bind(&event.target_entity_id)
            .bind(&event.old_value)
            .bind(&event.new_value)
            .bind(&event.ip_address)
            .bind(&event.user_agent)
            .bind(event.request_id)
            .bind(event.metadata.clone().unwrap_or(serde_json::json!({})))
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// Write a single event synchronously (for direct writes)
    async fn write_single(&self, event: &AuditEvent) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO authz_audit_logs (
                tenant_id, user_id, session_id, event_type, event_action,
                resource, action, decision, policy_version,
                target_entity_type, target_entity_id, old_value, new_value,
                ip_address, user_agent, request_id, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            "#,
        )
        .bind(event.tenant_id)
        .bind(event.user_id)
        .bind(event.session_id)
        .bind(event.event_type.to_string())
        .bind(&event.event_action)
        .bind(&event.resource)
        .bind(&event.action)
        .bind(&event.decision)
        .bind(event.policy_version)
        .bind(&event.target_entity_type)
        .bind(&event.target_entity_id)
        .bind(&event.old_value)
        .bind(&event.new_value)
        .bind(&event.ip_address)
        .bind(&event.user_agent)
        .bind(event.request_id)
        .bind(event.metadata.clone().unwrap_or(serde_json::json!({})))
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl AuditLogRepository for PgAuditLogRepository {
    async fn log(&self, event: AuditEvent) -> Result<(), AppError> {
        // Try to send to background writer
        match self.sender.try_send(event.clone()) {
            Ok(_) => {
                debug!(
                    "Audit event queued: type={}, action={}",
                    event.event_type, event.event_action
                );
                Ok(())
            },
            Err(mpsc::error::TrySendError::Full(e)) => {
                // Channel full - write synchronously as fallback
                warn!("Audit channel full, writing synchronously");
                self.write_single(&e).await
            },
            Err(mpsc::error::TrySendError::Closed(e)) => {
                // Channel closed - write synchronously
                warn!("Audit channel closed, writing synchronously");
                self.write_single(&e).await
            },
        }
    }

    async fn log_batch(&self, events: Vec<AuditEvent>) -> Result<(), AppError> {
        for event in events {
            self.log(event).await?;
        }
        Ok(())
    }

    async fn query(&self, query: AuditLogQuery) -> Result<AuditLogPage, AppError> {
        let offset = ((query.page - 1) * query.page_size) as i64;
        let limit = query.page_size as i64;

        // Build dynamic WHERE clause
        let mut conditions = vec!["tenant_id = $1".to_string()];
        let mut param_idx = 2;

        if query.user_id.is_some() {
            conditions.push(format!("user_id = ${}", param_idx));
            param_idx += 1;
        }
        if query.event_type.is_some() {
            conditions.push(format!("event_type = ${}", param_idx));
            param_idx += 1;
        }
        if query.event_action.is_some() {
            conditions.push(format!("event_action = ${}", param_idx));
            param_idx += 1;
        }
        if query.decision.is_some() {
            conditions.push(format!("decision = ${}", param_idx));
            param_idx += 1;
        }
        if query.start_time.is_some() {
            conditions.push(format!("created_at >= ${}", param_idx));
            param_idx += 1;
        }
        if query.end_time.is_some() {
            conditions.push(format!("created_at <= ${}", param_idx));
            param_idx += 1;
        }

        let where_clause = conditions.join(" AND ");

        // Count total
        let count_sql = format!("SELECT COUNT(*) FROM authz_audit_logs WHERE {}", where_clause);

        // Build count query with dynamic bindings
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql).bind(query.tenant_id);

        if let Some(ref user_id) = query.user_id {
            count_query = count_query.bind(user_id);
        }
        if let Some(ref event_type) = query.event_type {
            count_query = count_query.bind(event_type);
        }
        if let Some(ref event_action) = query.event_action {
            count_query = count_query.bind(event_action);
        }
        if let Some(ref decision) = query.decision {
            count_query = count_query.bind(decision);
        }
        if let Some(ref start_time) = query.start_time {
            count_query = count_query.bind(start_time);
        }
        if let Some(ref end_time) = query.end_time {
            count_query = count_query.bind(end_time);
        }

        let total = count_query.fetch_one(&self.pool).await?;

        // Fetch page
        let select_sql = format!(
            r#"
            SELECT
                audit_id, tenant_id, user_id, session_id,
                event_type, event_action, resource, action, decision, policy_version,
                target_entity_type, target_entity_id, old_value, new_value,
                ip_address, user_agent, request_id, metadata, created_at
            FROM authz_audit_logs
            WHERE {}
            ORDER BY created_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            param_idx,
            param_idx + 1
        );

        let mut select_query = sqlx::query_as::<_, AuditLogRow>(&select_sql).bind(query.tenant_id);

        if let Some(ref user_id) = query.user_id {
            select_query = select_query.bind(user_id);
        }
        if let Some(ref event_type) = query.event_type {
            select_query = select_query.bind(event_type);
        }
        if let Some(ref event_action) = query.event_action {
            select_query = select_query.bind(event_action);
        }
        if let Some(ref decision) = query.decision {
            select_query = select_query.bind(decision);
        }
        if let Some(ref start_time) = query.start_time {
            select_query = select_query.bind(start_time);
        }
        if let Some(ref end_time) = query.end_time {
            select_query = select_query.bind(end_time);
        }

        let rows = select_query
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let logs: Vec<AuditLogEntry> = rows.into_iter().map(|row| row.into()).collect();

        Ok(AuditLogPage {
            logs,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }

    async fn get_by_id(
        &self,
        tenant_id: Uuid,
        audit_id: Uuid,
    ) -> Result<Option<AuditLogEntry>, AppError> {
        let row = sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT
                audit_id, tenant_id, user_id, session_id,
                event_type, event_action, resource, action, decision, policy_version,
                target_entity_type, target_entity_id, old_value, new_value,
                ip_address, user_agent, request_id, metadata, created_at
            FROM authz_audit_logs
            WHERE tenant_id = $1 AND audit_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(audit_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn cleanup_before(&self, before: DateTime<Utc>) -> Result<u64, AppError> {
        let result = sqlx::query("DELETE FROM authz_audit_logs WHERE created_at < $1")
            .bind(before)
            .execute(&self.pool)
            .await?;

        let deleted = result.rows_affected();
        info!("Cleaned up {} audit log entries older than {}", deleted, before);

        Ok(deleted)
    }
}

/// Internal row type for database queries
#[derive(sqlx::FromRow)]
struct AuditLogRow {
    audit_id: Uuid,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    session_id: Option<Uuid>,
    event_type: String,
    event_action: String,
    resource: Option<String>,
    action: Option<String>,
    decision: Option<String>,
    policy_version: Option<i64>,
    target_entity_type: Option<String>,
    target_entity_id: Option<String>,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    request_id: Option<Uuid>,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<AuditLogRow> for AuditLogEntry {
    fn from(row: AuditLogRow) -> Self {
        Self {
            audit_id: row.audit_id,
            tenant_id: row.tenant_id,
            user_id: row.user_id,
            session_id: row.session_id,
            event_type: row.event_type,
            event_action: row.event_action,
            resource: row.resource,
            action: row.action,
            decision: row.decision,
            policy_version: row.policy_version,
            target_entity_type: row.target_entity_type,
            target_entity_id: row.target_entity_id,
            old_value: row.old_value,
            new_value: row.new_value,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            request_id: row.request_id,
            metadata: row.metadata,
            created_at: row.created_at,
        }
    }
}

impl std::fmt::Debug for PgAuditLogRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgAuditLogRepository")
            .field("channel_capacity", &AUDIT_CHANNEL_BUFFER)
            .finish()
    }
}
