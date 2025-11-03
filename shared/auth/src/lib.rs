pub mod enforcer;
pub mod extractors;
pub mod layer;
pub mod middleware;

// Re-export commonly used types
pub use enforcer::{
    add_policy, add_role_for_user, create_enforcer, enforce, get_roles_for_user, remove_policy,
    remove_role_for_user, SharedEnforcer,
};

// Re-export middleware
pub use middleware::{casbin_middleware, AuthError, AuthzState};

// Re-export layer
pub use layer::CasbinAuthLayer;

// Re-export extractors
pub use extractors::{
    AuthProvider, AuthUser, JwtSecretProvider, KanidmClientProvider, RequireAdmin,
    RequirePermission,
};

// Re-export Casbin types for convenience
pub use casbin;
pub use sqlx_adapter;
