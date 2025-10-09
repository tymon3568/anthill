pub mod enforcer;
pub mod middleware;
pub mod extractors;

// Re-export commonly used types
pub use enforcer::{
    create_enforcer, enforce, add_policy, remove_policy,
    add_role_for_user, remove_role_for_user, get_roles_for_user,
    SharedEnforcer,
};

// Re-export middleware
pub use middleware::{casbin_middleware, AuthError};

// Re-export extractors
pub use extractors::{AuthUser, RequireAdmin, RequirePermission};

// Re-export Casbin types for convenience
pub use casbin;
pub use sqlx_adapter;
