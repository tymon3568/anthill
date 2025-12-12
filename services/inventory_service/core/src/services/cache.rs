//! Cache service trait for performance optimization
//!
//! This module defines traits for caching frequently accessed data
//! to reduce database load and improve response times.

use crate::domains::inventory::product::Product;
use crate::models::InventoryLevel;
use async_trait::async_trait;
use shared_error::AppError;
use std::time::Duration;

/// Generic cache service trait
#[async_trait]
pub trait CacheService: Send + Sync {
    /// Get a value from cache by key
    async fn get<K, V>(&self, key: K) -> Result<Option<V>, AppError>
    where
        K: AsRef<str> + Send + Sync,
        V: serde::de::DeserializeOwned + Send;

    /// Set a value in cache with optional TTL
    async fn set<K, V>(&self, key: K, value: V, ttl: Option<Duration>) -> Result<(), AppError>
    where
        K: AsRef<str> + Send + Sync,
        V: serde::Serialize + Send;

    /// Delete a value from cache
    async fn delete<K>(&self, key: K) -> Result<(), AppError>
    where
        K: AsRef<str> + Send + Sync;

    /// Check if key exists in cache
    async fn exists<K>(&self, key: K) -> Result<bool, AppError>
    where
        K: AsRef<str> + Send + Sync;
}

/// Product cache operations
#[async_trait]
pub trait ProductCache: Send + Sync {
    /// Get cached product by ID
    async fn get_product(
        &self,
        tenant_id: &uuid::Uuid,
        product_id: &uuid::Uuid,
    ) -> Result<Option<Product>, AppError>;

    /// Cache product with TTL
    async fn set_product(
        &self,
        tenant_id: &uuid::Uuid,
        product: &Product,
        ttl: Option<Duration>,
    ) -> Result<(), AppError>;

    /// Invalidate product cache
    async fn invalidate_product(
        &self,
        tenant_id: &uuid::Uuid,
        product_id: &uuid::Uuid,
    ) -> Result<(), AppError>;
}

/// Inventory level cache operations
#[async_trait]
pub trait InventoryCache: Send + Sync {
    /// Get cached inventory level by product ID
    async fn get_inventory_level(
        &self,
        tenant_id: &uuid::Uuid,
        product_id: &uuid::Uuid,
    ) -> Result<Option<InventoryLevel>, AppError>;

    /// Cache inventory level with TTL
    async fn set_inventory_level(
        &self,
        tenant_id: &uuid::Uuid,
        level: &InventoryLevel,
        ttl: Option<Duration>,
    ) -> Result<(), AppError>;

    /// Invalidate inventory level cache
    async fn invalidate_inventory_level(
        &self,
        tenant_id: &uuid::Uuid,
        product_id: &uuid::Uuid,
    ) -> Result<(), AppError>;
}
