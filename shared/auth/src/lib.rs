pub mod enforcer;
// pub mod middleware;  // TODO: Implement in next step
// pub mod extractors;  // TODO: Implement in next step

// Re-export commonly used types
pub use enforcer::{
    create_enforcer, enforce, add_policy, remove_policy,
    add_role_for_user, remove_role_for_user, get_roles_for_user,
    SharedEnforcer,
};

// Re-export Casbin types for convenience
pub use casbin;
pub use sqlx_adapter;
