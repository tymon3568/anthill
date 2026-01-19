use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi, RbacApi};
use sqlx_adapter::SqlxAdapter;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Casbin enforcer type wrapped in Arc<RwLock<>> for thread-safe sharing
pub type SharedEnforcer = Arc<RwLock<Enforcer>>;

/// Resolve the Casbin model file path using multiple fallback strategies.
///
/// Search order:
/// 1. Workspace root from `CARGO_MANIFEST_DIR` (for workspace builds)
/// 2. Current directory (for CI/test environments)
/// 3. Relative to executable path
///
/// Returns the first existing path, or an error listing all searched locations.
pub(crate) fn resolve_model_path(custom_path: Option<&str>) -> Result<std::path::PathBuf, String> {
    if let Some(custom) = custom_path {
        return Ok(std::path::PathBuf::from(custom));
    }

    let default_relative = "shared/auth/model.conf";

    // Build candidate paths
    let workspace_root_path = std::env::var("CARGO_MANIFEST_DIR").ok().and_then(|p| {
        let path = std::path::PathBuf::from(p);
        path.ancestors()
            .find(|p| p.join("Cargo.toml").exists() && p.join("shared").exists())
            .map(|p| p.join(default_relative))
    });

    let current_dir_path = std::path::PathBuf::from(default_relative);

    let exe_relative_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join(default_relative)));

    // Collect all unique candidates for searching
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(ref p) = workspace_root_path {
        candidates.push(p.clone());
    }
    candidates.push(current_dir_path.clone());
    if let Some(ref p) = exe_relative_path {
        if !candidates.contains(p) {
            candidates.push(p.clone());
        }
    }

    // Find first existing path
    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    // Build descriptive error with all searched paths
    let searched: Vec<String> = candidates
        .iter()
        .map(|p| format!("  - {}", p.display()))
        .collect();
    Err(format!(
        "Could not find Casbin model file '{}'. Searched locations:\n{}",
        default_relative,
        searched.join("\n")
    ))
}

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
    // Resolve model path using helper function
    let resolved_model_path = resolve_model_path(model_path)?;

    let model_path_str = resolved_model_path.to_string_lossy();
    info!("Initializing Casbin enforcer with model: {}", model_path_str);

    // Load Casbin model
    let model = DefaultModel::from_file(
        resolved_model_path
            .to_str()
            .ok_or_else(|| format!("Invalid UTF-8 in model path: {:?}", resolved_model_path))?,
    )
    .await?;
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

/// Copy all policies from a source tenant (default_tenant) to a new tenant
///
/// When a new tenant is created, this function copies all role policies (p rules)
/// from the source tenant so that the new tenant has the same permission structure.
/// The user's role assignment (g rule) is handled separately by `add_role_for_user`.
///
/// # Arguments
/// * `enforcer` - Shared Casbin enforcer
/// * `source_domain` - Source tenant ID to copy policies from (typically "default_tenant")
/// * `target_domain` - Target tenant ID to copy policies to
///
/// # Returns
/// * `usize` - Number of policies copied
///
/// # Example
/// ```no_run
/// // Copy all policies from default_tenant to new tenant
/// let count = copy_policies_for_tenant(&enforcer, "default_tenant", "new-tenant-123").await?;
/// println!("Copied {} policies", count);
/// ```
pub async fn copy_policies_for_tenant(
    enforcer: &SharedEnforcer,
    source_domain: &str,
    target_domain: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut e = enforcer.write().await;

    // Get all policies from the source domain
    // Policy format: [subject, domain, resource, action]
    let all_policies = e.get_policy();

    let mut copied_count = 0;

    for policy in all_policies {
        // Policy is Vec<String> with format: [subject, domain, resource, action]
        if policy.len() >= 4 && policy[1] == source_domain {
            // Create new policy with target domain
            let new_policy = vec![
                policy[0].clone(),         // subject (role name like "owner", "admin", "user")
                target_domain.to_string(), // domain (new tenant ID)
                policy[2].clone(),         // resource (e.g., "/api/v1/admin/*")
                policy[3].clone(),         // action (e.g., "GET", "POST")
            ];

            // Add the new policy (ignore if already exists)
            match e.add_policy(new_policy).await {
                Ok(added) => {
                    if added {
                        copied_count += 1;
                    }
                },
                Err(err) => {
                    // Log but don't fail - some policies might fail due to constraints
                    warn!(
                        "Failed to copy policy {:?} to tenant {}: {}",
                        policy, target_domain, err
                    );
                },
            }
        }
    }

    info!("Copied {} policies from {} to {}", copied_count, source_domain, target_domain);

    Ok(copied_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use casbin::{Enforcer, MgmtApi, RbacApi};
    use sqlx::postgres::PgPoolOptions;
    use sqlx_adapter::SqlxAdapter;

    #[test]
    fn test_resolve_model_path_in_test_env() {
        // This validates that our new path resolution logic works in the test environment
        // exactly as it does in the production code (since we're calling the same function)
        let path = resolve_model_path(None).expect("Should resolve path in test env");
        assert!(path.exists(), "Model file should exist at {:?}", path);
        assert!(path.ends_with("shared/auth/model.conf"), "Path should end with model.conf");
    }

    async fn setup_test_enforcer() -> Enforcer {
        // Resolve model path using the same logic as production
        let resolved_model_path =
            resolve_model_path(None).expect("Failed to resolve model path for tests");

        let model = DefaultModel::from_file(
            resolved_model_path
                .to_str()
                .expect("Invalid UTF-8 in model path"),
        )
        .await
        .expect("Failed to load Casbin model");

        // Use PostgreSQL for testing (standard port 5432)
        // Credentials aligned with integration_utils.rs and setup scripts
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://anthill:anthill@localhost:5432/anthill_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database for enforcer tests");

        let adapter = SqlxAdapter::new_with_pool(pool).await.unwrap();
        Enforcer::new(model, adapter).await.unwrap()
    }

    #[tokio::test]
    #[ignore] // TODO: Fix PostgreSQL permissions for test database
    async fn test_role_assignments() {
        let mut e = setup_test_enforcer().await;
        e.add_grouping_policy(vec![
            "user1".to_string(),
            "admin".to_string(),
            "tenant1".to_string(),
        ])
        .await
        .unwrap();

        assert_eq!(e.get_roles_for_user("user1", Some("tenant1")), vec!["admin".to_string()]);

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
    #[ignore] // TODO: Fix PostgreSQL permissions for test database
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
    #[ignore] // TODO: Fix PostgreSQL permissions for test database
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
    #[ignore] // TODO: Fix PostgreSQL permissions for test database
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
