//! Redis-based cache implementation for inventory service
//!
//! This module provides Redis-backed caching for frequently accessed data
//! to improve performance and reduce database load.

use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::redis::AsyncCommands;
use bb8_redis::RedisConnectionManager;
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use inventory_service_core::domains::inventory::product::Product;
use inventory_service_core::models::InventoryLevel;
use inventory_service_core::services::{CacheService, InventoryCache, ProductCache};
use shared_error::AppError;

/// Redis-based cache implementation
pub struct RedisCache {
    pool: Pool<RedisConnectionManager>,
}

impl RedisCache {
    /// Create a new Redis cache instance backed by a Redis connection pool
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        let manager = RedisConnectionManager::new(redis_url).map_err(|e| {
            AppError::InternalError(format!("Redis connection manager error: {}", e))
        })?;

        let pool = Pool::builder()
            .build(manager)
            .await
            .map_err(|e| AppError::InternalError(format!("Redis pool creation error: {}", e)))?;

        Ok(Self { pool })
    }

    /// Get async pooled connection
    async fn get_connection(
        &self,
    ) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>, AppError> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::InternalError(format!("Redis pool error: {}", e)))
    }

    /// Generate cache key for product
    fn product_key(tenant_id: &Uuid, product_id: &Uuid) -> String {
        format!("product:{}:{}", tenant_id, product_id)
    }

    /// Generate cache key for inventory level
    fn inventory_key(tenant_id: &Uuid, product_id: &Uuid) -> String {
        format!("inventory:{}:{}", tenant_id, product_id)
    }
}

#[async_trait]
impl CacheService for RedisCache {
    async fn get<K, V>(&self, key: K) -> Result<Option<V>, AppError>
    where
        K: AsRef<str> + Send + Sync,
        V: serde::de::DeserializeOwned + Send,
    {
        let mut conn = self.get_connection().await?;
        let data: Option<String> = conn
            .get::<_, Option<String>>(key.as_ref())
            .await
            .map_err(|e| AppError::InternalError(format!("Redis get error: {}", e)))?;

        match data {
            Some(json) => serde_json::from_str(json.as_str())
                .map(Some)
                .map_err(|e| AppError::InternalError(format!("JSON deserialize error: {}", e))),
            None => Ok(None),
        }
    }

    async fn set<K, V>(&self, key: K, value: V, ttl: Option<Duration>) -> Result<(), AppError>
    where
        K: AsRef<str> + Send + Sync,
        V: serde::Serialize + Send,
    {
        let mut conn = self.get_connection().await?;
        let json = serde_json::to_string(&value)
            .map_err(|e| AppError::InternalError(format!("JSON serialize error: {}", e)))?;

        match ttl {
            Some(duration) => {
                let secs = duration.as_secs().max(1);
                conn.set_ex(key.as_ref(), json, secs).await
            },
            None => conn.set(key.as_ref(), json).await,
        }
        .map_err(|e| AppError::InternalError(format!("Redis set error: {}", e)))
    }

    async fn delete<K>(&self, key: K) -> Result<(), AppError>
    where
        K: AsRef<str> + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        conn.del::<_, ()>(key.as_ref())
            .await
            .map_err(|e| AppError::InternalError(format!("Redis delete error: {}", e)))?;
        Ok(())
    }

    async fn exists<K>(&self, key: K) -> Result<bool, AppError>
    where
        K: AsRef<str> + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        let exists: bool = conn
            .exists(key.as_ref())
            .await
            .map_err(|e| AppError::InternalError(format!("Redis exists error: {}", e)))?;
        Ok(exists)
    }
}

#[async_trait]
impl ProductCache for RedisCache {
    async fn get_product(
        &self,
        tenant_id: &Uuid,
        product_id: &Uuid,
    ) -> Result<Option<Product>, AppError> {
        let key = Self::product_key(tenant_id, product_id);
        self.get(&key).await
    }

    async fn set_product(
        &self,
        tenant_id: &Uuid,
        product: &Product,
        ttl: Option<Duration>,
    ) -> Result<(), AppError> {
        let key = Self::product_key(tenant_id, &product.product_id);
        self.set(&key, product, ttl).await
    }

    async fn invalidate_product(
        &self,
        tenant_id: &Uuid,
        product_id: &Uuid,
    ) -> Result<(), AppError> {
        let key = Self::product_key(tenant_id, product_id);
        self.delete(&key).await
    }
}

#[async_trait]
impl InventoryCache for RedisCache {
    async fn get_inventory_level(
        &self,
        tenant_id: &Uuid,
        product_id: &Uuid,
    ) -> Result<Option<InventoryLevel>, AppError> {
        let key = Self::inventory_key(tenant_id, product_id);
        self.get(&key).await
    }

    async fn set_inventory_level(
        &self,
        tenant_id: &Uuid,
        level: &InventoryLevel,
        ttl: Option<Duration>,
    ) -> Result<(), AppError> {
        let key = Self::inventory_key(tenant_id, &level.product_id);
        self.set(&key, level, ttl).await
    }

    async fn invalidate_inventory_level(
        &self,
        tenant_id: &Uuid,
        product_id: &Uuid,
    ) -> Result<(), AppError> {
        let key = Self::inventory_key(tenant_id, product_id);
        self.delete(&key).await
    }
}

/// Type alias for Arc-wrapped cache service
pub type SharedCache = Arc<dyn CacheService + Send + Sync>;
pub type SharedProductCache = Arc<dyn ProductCache + Send + Sync>;
pub type SharedInventoryCache = Arc<dyn InventoryCache + Send + Sync>;
