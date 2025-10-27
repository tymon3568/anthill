use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi, RbacApi};
use sqlx_adapter::SqlxAdapter;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Casbin enforcer type wrapped in Arc<RwLock<>> for thread-safe sharing
pub type SharedEnforcer = Arc<RwLock<Enforcer>>;

/// Initialize Casbin enforcer with PostgreSQL adapter
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string (e.g., "postgres://user:pass@localhost/db")
/// * `model_path` - Path to Casbin model.conf file (default: "shared/auth/model.conf")
///
/// # Returns
/// * `SharedEnforcer` - Thread-safe Casbin enforcer wrapped in Arc<RwLock<>>
///
/// # Example
/// ```no_run
/// use shared_auth::enforcer::create_enforcer;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let enforcer = create_enforcer("postgres://localhost/mydb", None).await?;
///     Ok(())
/// }
/// ```
pub async fn create_enforcer(
    database_url: &str,
    model_path: Option<&str>,
) -> Result<SharedEnforcer, Box<dyn std::error::Error>> {
    let model_path = model_path.unwrap_or("shared/auth/model.conf");

    info!("Initializing Casbin enforcer with model: {}", model_path);

    // Load Casbin model
    let model = DefaultModel::from_file(model_path).await?;
    info!("Casbin model loaded successfully");

    // Create SQLx adapter for PostgreSQL
    // Use 2 connections: sufficient for policy loads and occasional updates
    let adapter = SqlxAdapter::new(database_url, 2).await?;
    info!("Casbin SQLx adapter initialized with 2 connections");

    // Create enforcer with model and adapter
    let mut enforcer = Enforcer::new(model, adapter).await?;

    // Enable logging for debugging
    enforcer.enable_log(true);

    // Enable auto-save (policies will be saved to DB automatically)
    enforcer.enable_auto_save(true);

    info!("Casbin enforcer initialized successfully");

    Ok(Arc::new(RwLock::new(enforcer)))
}

/// Check if a user has permission to perform an action on a resource
///
/// # Arguments
/// * `enforcer` - Shared Casbin enforcer
/// * `user_id` - User ID or role name
/// * `tenant_id` - Tenant ID (domain)
/// * `resource` - Resource path (e.g., "/api/v1/products")
/// * `action` - Action (e.g., "GET", "POST", "read", "write")
///
/// # Returns
/// * `bool` - True if allowed, false otherwise
pub async fn enforce(
    enforcer: &SharedEnforcer,
    user_id: &str,
    tenant_id: &str,
    resource: &str,
    action: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let e = enforcer.read().await;

    let allowed = e.enforce((user_id, tenant_id, resource, action))?;

    if !allowed {
        warn!(
            "Permission denied: user={}, tenant={}, resource={}, action={}",
            user_id, tenant_id, resource, action
        );
    }

    Ok(allowed)
}

/// Add a policy rule
///
/// # Example
/// ```no_run
/// // Grant admin role full access to products
/// add_policy(&enforcer, "admin", "tenant-123", "/api/v1/products", "GET").await?;
/// add_policy(&enforcer, "admin", "tenant-123", "/api/v1/products", "POST").await?;
/// ```
pub async fn add_policy(
    enforcer: &SharedEnforcer,
    subject: &str,
    domain: &str,
    resource: &str,
    action: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut e = enforcer.write().await;
    let added = e
        .add_policy(vec![
            subject.to_string(),
            domain.to_string(),
            resource.to_string(),
            action.to_string(),
        ])
        .await?;
    Ok(added)
}

/// Remove a policy rule
pub async fn remove_policy(
    enforcer: &SharedEnforcer,
    subject: &str,
    domain: &str,
    resource: &str,
    action: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut e = enforcer.write().await;
    let removed = e
        .remove_policy(vec![
            subject.to_string(),
            domain.to_string(),
            resource.to_string(),
            action.to_string(),
        ])
        .await?;
    Ok(removed)
}

/// Assign a role to a user in a specific tenant
///
/// # Example
/// ```no_run
/// // Assign "admin" role to user in tenant
/// add_role_for_user(&enforcer, "user-456", "admin", "tenant-123").await?;
/// ```
pub async fn add_role_for_user(
    enforcer: &SharedEnforcer,
    user_id: &str,
    role: &str,
    domain: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut e = enforcer.write().await;
    let added = e.add_role_for_user(user_id, role, Some(domain)).await?;
    Ok(added)
}

/// Remove a role from a user in a specific tenant
pub async fn remove_role_for_user(
    enforcer: &SharedEnforcer,
    user_id: &str,
    role: &str,
    domain: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut e = enforcer.write().await;
    let removed = e.delete_role_for_user(user_id, role, Some(domain)).await?;
    Ok(removed)
}

/// Get all roles for a user in a specific tenant
pub async fn get_roles_for_user(
    enforcer: &SharedEnforcer,
    user_id: &str,
    domain: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let e = enforcer.read().await;
    let roles = e.get_roles_for_user(user_id, Some(domain));
    Ok(roles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use casbin::{DefaultModel, Enforcer, MgmtApi, RbacApi};
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx_adapter::SqlxAdapter;

    async fn setup_test_enforcer() -> Enforcer {
        // Note: The model file path is relative to the crate root
        let model = DefaultModel::from_file("model.conf").await.unwrap();
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let adapter = SqlxAdapter::new_with_pool(pool).await.unwrap();
        Enforcer::new(model, adapter).await.unwrap()
    }

    #[tokio::test]
    async fn test_role_assignments() {
        let mut e = setup_test_enforcer().await;
        e.add_grouping_policy(vec![
            "user1".to_string(),
            "admin".to_string(),
            "tenant1".to_string(),
        ])
        .await
        .unwrap();

        assert_eq!(
            e.get_roles_for_user("user1", Some("tenant1")),
            vec!["admin".to_string()]
        );

        e.remove_grouping_policy(vec![
            "user1".to_string(),
            "admin".to_string(),
            "tenant1".to_string(),
        ])
        .await
        .unwrap();

        assert!(e.get_roles_for_user("user1", Some("tenant1")).is_empty());
    }

    #[tokio::test]
    async fn test_permission_checks() {
        let mut e = setup_test_enforcer().await;
        e.add_policy(vec![
            "admin".to_string(),
            "tenant1".to_string(),
            "data1".to_string(),
            "read".to_string(),
        ])
        .await
        .unwrap();

        assert!(e.enforce(("admin", "tenant1", "data1", "read")).unwrap());
        assert!(!e.enforce(("admin", "tenant1", "data1", "write")).unwrap());
        assert!(!e.enforce(("user", "tenant1", "data1", "read")).unwrap());
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let mut e = setup_test_enforcer().await;
        e.add_policy(vec![
            "user1".to_string(),
            "tenant1".to_string(),
            "data1".to_string(),
            "read".to_string(),
        ])
        .await
        .unwrap();

        assert!(e.enforce(("user1", "tenant1", "data1", "read")).unwrap());
        assert!(!e.enforce(("user1", "tenant2", "data1", "read")).unwrap());
    }

    #[tokio::test]
    async fn test_admin_role_access() {
        let mut e = setup_test_enforcer().await;
        e.add_grouping_policy(vec![
            "alice".to_string(),
            "admin".to_string(),
            "tenant1".to_string(),
        ])
        .await
        .unwrap();
        e.add_policy(vec![
            "admin".to_string(),
            "tenant1".to_string(),
            "/api/v1/admin/users".to_string(),
            "POST".to_string(),
        ])
        .await
        .unwrap();

        assert!(e
            .enforce(("alice", "tenant1", "/api/v1/admin/users", "POST"))
            .unwrap());
        assert!(!e
            .enforce(("bob", "tenant1", "/api/v1/admin/users", "POST"))
            .unwrap());
    }
}
