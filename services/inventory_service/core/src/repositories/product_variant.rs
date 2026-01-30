//! Product Variant repository trait
//!
//! Defines the data access interface for product variant operations.
//! This trait abstracts database operations for product variants.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domains::inventory::product_variant::ProductVariant;
use crate::dto::product_variant::{VariantListQuery, VariantResponse};
use crate::Result;

/// Repository trait for product variant data access
///
/// This trait defines all database operations for product variants.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait ProductVariantRepository: Send + Sync {
    // ========================================================================
    // CRUD Operations
    // ========================================================================

    /// Get variant by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `variant_id` - Variant identifier
    ///
    /// # Returns
    /// Variant if found
    async fn find_by_id(&self, tenant_id: Uuid, variant_id: Uuid)
        -> Result<Option<ProductVariant>>;

    /// Get variant by ID with parent product info
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `variant_id` - Variant identifier
    ///
    /// # Returns
    /// VariantResponse with parent product name and SKU if found
    async fn find_by_id_with_parent(
        &self,
        tenant_id: Uuid,
        variant_id: Uuid,
    ) -> Result<Option<VariantResponse>>;

    /// Get variant by SKU
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `sku` - Variant SKU
    ///
    /// # Returns
    /// VariantResponse if found
    async fn find_by_sku(&self, tenant_id: Uuid, sku: &str) -> Result<Option<VariantResponse>>;

    /// Get variant by barcode
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `barcode` - Variant barcode
    ///
    /// # Returns
    /// VariantResponse if found
    async fn find_by_barcode(
        &self,
        tenant_id: Uuid,
        barcode: &str,
    ) -> Result<Option<VariantResponse>>;

    /// List variants with filtering and pagination
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `query` - Query parameters
    ///
    /// # Returns
    /// List of variants with parent info and total count
    async fn list(
        &self,
        tenant_id: Uuid,
        query: &VariantListQuery,
    ) -> Result<(Vec<VariantResponse>, i64)>;

    /// Create new variant
    ///
    /// # Arguments
    /// * `variant` - Variant to create
    ///
    /// # Returns
    /// Created variant
    async fn create(&self, variant: &ProductVariant) -> Result<ProductVariant>;

    /// Update existing variant
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `variant_id` - Variant to update
    /// * `variant` - Updated variant data
    ///
    /// # Returns
    /// Updated variant
    async fn update(
        &self,
        tenant_id: Uuid,
        variant_id: Uuid,
        variant: &ProductVariant,
    ) -> Result<ProductVariant>;

    /// Delete variant (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `variant_id` - Variant to delete
    ///
    /// # Returns
    /// Success status
    async fn delete(&self, tenant_id: Uuid, variant_id: Uuid) -> Result<bool>;

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    /// Bulk activate variants
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `variant_ids` - List of variant IDs to activate
    ///
    /// # Returns
    /// Number of affected rows
    async fn bulk_activate(&self, tenant_id: Uuid, variant_ids: &[Uuid]) -> Result<i64>;

    /// Bulk deactivate variants
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `variant_ids` - List of variant IDs to deactivate
    ///
    /// # Returns
    /// Number of affected rows
    async fn bulk_deactivate(&self, tenant_id: Uuid, variant_ids: &[Uuid]) -> Result<i64>;

    /// Bulk delete variants (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `variant_ids` - List of variant IDs to delete
    ///
    /// # Returns
    /// Number of affected rows
    async fn bulk_delete(&self, tenant_id: Uuid, variant_ids: &[Uuid]) -> Result<i64>;

    // ========================================================================
    // Validation
    // ========================================================================

    /// Check if SKU exists for tenant (excluding a specific variant)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `sku` - SKU to check
    /// * `exclude_variant_id` - Optional variant ID to exclude from check
    ///
    /// # Returns
    /// True if SKU exists
    async fn sku_exists(
        &self,
        tenant_id: Uuid,
        sku: &str,
        exclude_variant_id: Option<Uuid>,
    ) -> Result<bool>;

    /// Check if variant attributes combination exists for product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `parent_product_id` - Parent product ID
    /// * `variant_attributes` - Attributes to check
    /// * `exclude_variant_id` - Optional variant ID to exclude from check
    ///
    /// # Returns
    /// True if attributes combination exists
    async fn attributes_exist(
        &self,
        tenant_id: Uuid,
        parent_product_id: Uuid,
        variant_attributes: &serde_json::Value,
        exclude_variant_id: Option<Uuid>,
    ) -> Result<bool>;
}
