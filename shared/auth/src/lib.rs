pub mod enforcer;
pub mod extractors;
pub mod middleware;

// Re-export commonly used types
pub use enforcer::{
    add_policy, add_role_for_user, create_enforcer, enforce, get_roles_for_user, remove_policy,
    remove_role_for_user, SharedEnforcer,
};

// Re-export middleware
pub use middleware::{casbin_middleware, AuthError};

// Re-export extractors
pub use extractors::{AuthUser, RequireAdmin, RequirePermission};

// Re-export Casbin types for convenience
pub use casbin;
pub use sqlx_adapter;
