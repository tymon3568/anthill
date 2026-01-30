//! Product Variant service trait
//!
//! Defines the business logic interface for product variant operations.
//! This trait coordinates between repositories and implements business rules.

use async_trait::async_trait;
use uuid::Uuid;

use crate::dto::product_variant::{
    BulkVariantOperationResponse, VariantCreateRequest, VariantListQuery, VariantListResponse,
    VariantResponse, VariantUpdateRequest,
};
use crate::Result;

/// Service trait for product variant business logic
///
/// This trait defines all business operations for product variants.
/// Infrastructure layer will provide the actual implementation.
#[async_trait]
pub trait ProductVariantService: Send + Sync {
    // ========================================================================
    // CRUD Operations
    // ========================================================================

    /// Create a new product variant
    ///
    /// # Business Rules
    /// - Parent product must exist and be active
    /// - SKU must be unique within tenant
    /// - Variant attributes must be unique for the product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `request` - Variant creation data
    ///
    /// # Returns
    /// Created variant with parent info
    ///
    /// # Errors
    /// - `ValidationError` if request validation fails
    /// - `NotFound` if parent product doesn't exist
    /// - `Conflict` if SKU or attributes already exist
    async fn create_variant(
        &self,
        tenant_id: Uuid,
        request: VariantCreateRequest,
    ) -> Result<VariantResponse>;

    /// Get variant by ID
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `variant_id` - Variant identifier
    ///
    /// # Returns
    /// Variant with parent info
    ///
    /// # Errors
    /// - `NotFound` if variant doesn't exist
    async fn get_variant(&self, tenant_id: Uuid, variant_id: Uuid) -> Result<VariantResponse>;

    /// Get variant by SKU
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `sku` - Variant SKU
    ///
    /// # Returns
    /// Variant with parent info
    ///
    /// # Errors
    /// - `NotFound` if variant doesn't exist
    async fn get_variant_by_sku(&self, tenant_id: Uuid, sku: &str) -> Result<VariantResponse>;

    /// Get variant by barcode
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `barcode` - Variant barcode
    ///
    /// # Returns
    /// Variant with parent info
    ///
    /// # Errors
    /// - `NotFound` if variant doesn't exist
    async fn get_variant_by_barcode(
        &self,
        tenant_id: Uuid,
        barcode: &str,
    ) -> Result<VariantResponse>;

    /// List variants with filtering and pagination
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `query` - List query parameters
    ///
    /// # Returns
    /// Paginated list of variants
    async fn list_variants(
        &self,
        tenant_id: Uuid,
        query: VariantListQuery,
    ) -> Result<VariantListResponse>;

    /// Update an existing variant
    ///
    /// # Business Rules
    /// - Variant must exist and not be deleted
    /// - If SKU is changed, must be unique within tenant
    /// - If attributes are changed, must be unique for the product
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `variant_id` - Variant identifier
    /// * `request` - Update data
    ///
    /// # Returns
    /// Updated variant with parent info
    ///
    /// # Errors
    /// - `NotFound` if variant doesn't exist
    /// - `ValidationError` if request validation fails
    /// - `Conflict` if SKU or attributes conflict
    async fn update_variant(
        &self,
        tenant_id: Uuid,
        variant_id: Uuid,
        request: VariantUpdateRequest,
    ) -> Result<VariantResponse>;

    /// Delete a variant (soft delete)
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `variant_id` - Variant identifier
    ///
    /// # Returns
    /// Success status
    ///
    /// # Errors
    /// - `NotFound` if variant doesn't exist
    async fn delete_variant(&self, tenant_id: Uuid, variant_id: Uuid) -> Result<()>;

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    /// Bulk activate variants
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `variant_ids` - List of variant IDs to activate
    ///
    /// # Returns
    /// Bulk operation result
    async fn bulk_activate(
        &self,
        tenant_id: Uuid,
        variant_ids: Vec<Uuid>,
    ) -> Result<BulkVariantOperationResponse>;

    /// Bulk deactivate variants
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `variant_ids` - List of variant IDs to deactivate
    ///
    /// # Returns
    /// Bulk operation result
    async fn bulk_deactivate(
        &self,
        tenant_id: Uuid,
        variant_ids: Vec<Uuid>,
    ) -> Result<BulkVariantOperationResponse>;

    /// Bulk delete variants
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier
    /// * `variant_ids` - List of variant IDs to delete
    ///
    /// # Returns
    /// Bulk operation result
    async fn bulk_delete(
        &self,
        tenant_id: Uuid,
        variant_ids: Vec<Uuid>,
    ) -> Result<BulkVariantOperationResponse>;
}
