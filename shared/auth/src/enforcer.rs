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
    let added = e.add_policy(vec![subject.to_string(), domain.to_string(), resource.to_string(), action.to_string()]).await?;
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
    let removed = e.remove_policy(vec![subject.to_string(), domain.to_string(), resource.to_string(), action.to_string()]).await?;
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

    // Note: These tests require a PostgreSQL database connection
    // You can run them with: cargo test --features test-db

    #[tokio::test]
    #[ignore] // Ignore by default (requires DB)
    async fn test_enforcer_initialization() {
        // This would need a test database
        // let pool = PgPool::connect("postgres://localhost/test_db").await.unwrap();
        // let enforcer = create_enforcer(pool, None).await.unwrap();
        // assert!(enforcer is initialized);
    }
}
