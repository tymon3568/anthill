//! Distributed locking service trait
//!
//! Defines the interface for distributed locking operations using Redis.
//! This trait provides locking mechanisms to prevent race conditions
//! during concurrent stock mutations.

use async_trait::async_trait;
use uuid::Uuid;

use crate::Result;

/// Service trait for distributed locking operations
///
/// This trait defines operations for acquiring and releasing distributed locks
/// to ensure atomicity of stock mutations across multiple service instances.
/// Locks are typically acquired on product-warehouse combinations.
#[async_trait]
pub trait DistributedLockService: Send + Sync {
    /// Acquire a lock for a specific resource
    ///
    /// # Business Rules
    /// - Lock key format: "lock:{tenant_id}:{resource_type}:{resource_id}"
    /// - Lock TTL prevents permanent locks in case of service crashes
    /// - Returns lock token for release verification
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `resource_type` - Type of resource (e.g., "product_warehouse")
    /// * `resource_id` - Resource identifier (e.g., "product_id:warehouse_id")
    /// * `ttl_seconds` - Time-to-live for the lock in seconds
    ///
    /// # Returns
    /// Lock token if acquired, None if already locked
    ///
    /// # Errors
    /// - `InternalError` if Redis operation fails
    async fn acquire_lock(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_id: &str,
        ttl_seconds: u32,
    ) -> Result<Option<String>>;

    /// Release a previously acquired lock
    ///
    /// # Business Rules
    /// - Only releases lock if token matches (prevents accidental release)
    /// - Uses Lua script for atomic check-and-delete operation
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `resource_type` - Type of resource
    /// * `resource_id` - Resource identifier
    /// * `lock_token` - Token returned from acquire_lock
    ///
    /// # Returns
    /// true if lock was released, false if token didn't match or lock expired
    ///
    /// # Errors
    /// - `InternalError` if Redis operation fails
    async fn release_lock(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_id: &str,
        lock_token: &str,
    ) -> Result<bool>;

    /// Check if a resource is currently locked
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `resource_type` - Type of resource
    /// * `resource_id` - Resource identifier
    ///
    /// # Returns
    /// true if locked, false otherwise
    ///
    /// # Errors
    /// - `InternalError` if Redis operation fails
    async fn is_locked(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<bool>;

    /// Extend the TTL of an existing lock
    ///
    /// # Business Rules
    /// - Only extends if token matches
    /// - Useful for long-running operations
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `resource_type` - Type of resource
    /// * `resource_id` - Resource identifier
    /// * `lock_token` - Token returned from acquire_lock
    /// * `ttl_seconds` - New TTL in seconds
    ///
    /// # Returns
    /// true if extended, false if token didn't match or lock expired
    ///
    /// # Errors
    /// - `InternalError` if Redis operation fails
    async fn extend_lock(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_id: &str,
        lock_token: &str,
        ttl_seconds: u32,
    ) -> Result<bool>;

    /// Force release a lock (admin operation)
    ///
    /// # Business Rules
    /// - Should only be used in emergency situations
    /// - Bypasses token verification
    /// - Should be logged for audit purposes
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `resource_type` - Type of resource
    /// * `resource_id` - Resource identifier
    ///
    /// # Returns
    /// true if lock was released, false if not locked
    ///
    /// # Errors
    /// - `InternalError` if Redis operation fails
    async fn force_release_lock(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<bool>;
}
