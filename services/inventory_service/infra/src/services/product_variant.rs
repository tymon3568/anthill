//! Product Variant service implementation
//!
//! PostgreSQL-based implementation of the ProductVariantService trait.

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use inventory_service_core::domains::inventory::product_variant::ProductVariant;
use inventory_service_core::dto::common::PaginationInfo;
use inventory_service_core::dto::product_variant::{
    BulkVariantOperationResponse, VariantCreateRequest, VariantListQuery, VariantListResponse,
    VariantResponse, VariantUpdateRequest,
};
use inventory_service_core::repositories::product::ProductRepository;
use inventory_service_core::repositories::product_variant::ProductVariantRepository;
use inventory_service_core::services::product_variant::ProductVariantService;
use inventory_service_core::Result;
use shared_error::AppError;

/// PostgreSQL-based implementation of ProductVariantService
pub struct ProductVariantServiceImpl {
    variant_repository: Arc<dyn ProductVariantRepository>,
    product_repository: Arc<dyn ProductRepository>,
}

impl ProductVariantServiceImpl {
    /// Create new service instance
    pub fn new(
        variant_repository: Arc<dyn ProductVariantRepository>,
        product_repository: Arc<dyn ProductRepository>,
    ) -> Self {
        Self {
            variant_repository,
            product_repository,
        }
    }
}

#[async_trait]
impl ProductVariantService for ProductVariantServiceImpl {
    // ========================================================================
    // CRUD Operations
    // ========================================================================

    async fn create_variant(
        &self,
        tenant_id: Uuid,
        request: VariantCreateRequest,
    ) -> Result<VariantResponse> {
        // Validate parent product exists
        let parent_product = self
            .product_repository
            .find_by_id(tenant_id, request.parent_product_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Parent product not found".to_string()))?;

        // Check if parent product is active
        if !parent_product.is_active {
            return Err(AppError::ValidationError(
                "Cannot create variant for inactive product".to_string(),
            ));
        }

        // Check SKU uniqueness
        if self
            .variant_repository
            .sku_exists(tenant_id, &request.sku, None)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "Variant with SKU '{}' already exists",
                request.sku
            )));
        }

        // Check attributes uniqueness for this product
        if self
            .variant_repository
            .attributes_exist(
                tenant_id,
                request.parent_product_id,
                &request.variant_attributes,
                None,
            )
            .await?
        {
            return Err(AppError::Conflict(
                "Variant with these attributes already exists for this product".to_string(),
            ));
        }

        // Create variant entity
        let mut variant = ProductVariant::new(
            tenant_id,
            request.parent_product_id,
            request.sku,
            request.variant_attributes,
        );

        variant.barcode = request.barcode;
        variant.price_difference = request.price_difference.unwrap_or(0);
        variant.is_active = request.is_active.unwrap_or(true);

        // Save to database
        let created = self.variant_repository.create(&variant).await?;

        // Return with parent info
        Ok(VariantResponse::with_parent_info(
            created,
            Some(parent_product.name),
            Some(parent_product.sku),
        ))
    }

    async fn get_variant(&self, tenant_id: Uuid, variant_id: Uuid) -> Result<VariantResponse> {
        self.variant_repository
            .find_by_id_with_parent(tenant_id, variant_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Variant not found".to_string()))
    }

    async fn get_variant_by_sku(&self, tenant_id: Uuid, sku: &str) -> Result<VariantResponse> {
        self.variant_repository
            .find_by_sku(tenant_id, sku)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Variant with SKU '{}' not found", sku)))
    }

    async fn get_variant_by_barcode(
        &self,
        tenant_id: Uuid,
        barcode: &str,
    ) -> Result<VariantResponse> {
        self.variant_repository
            .find_by_barcode(tenant_id, barcode)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Variant with barcode '{}' not found", barcode))
            })
    }

    async fn list_variants(
        &self,
        tenant_id: Uuid,
        query: VariantListQuery,
    ) -> Result<VariantListResponse> {
        let (variants, total_count) = self.variant_repository.list(tenant_id, &query).await?;

        let total_pages = (total_count as u32).div_ceil(query.page_size as u32).max(1);

        let pagination = PaginationInfo {
            page: query.page as u32,
            page_size: query.page_size as u32,
            total_items: total_count as u64,
            total_pages,
            has_next: query.page < total_pages as i64,
            has_prev: query.page > 1,
        };

        Ok(VariantListResponse {
            variants,
            pagination,
        })
    }

    async fn update_variant(
        &self,
        tenant_id: Uuid,
        variant_id: Uuid,
        request: VariantUpdateRequest,
    ) -> Result<VariantResponse> {
        // Get existing variant
        let mut variant = self
            .variant_repository
            .find_by_id(tenant_id, variant_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Variant not found".to_string()))?;

        // Check SKU uniqueness if changing
        if let Some(ref new_sku) = request.sku {
            if new_sku != &variant.sku
                && self
                    .variant_repository
                    .sku_exists(tenant_id, new_sku, Some(variant_id))
                    .await?
            {
                return Err(AppError::Conflict(format!(
                    "Variant with SKU '{}' already exists",
                    new_sku
                )));
            }
            variant.sku = new_sku.clone();
        }

        // Check attributes uniqueness if changing
        if let Some(ref new_attrs) = request.variant_attributes {
            if new_attrs != &variant.variant_attributes
                && self
                    .variant_repository
                    .attributes_exist(
                        tenant_id,
                        variant.parent_product_id,
                        new_attrs,
                        Some(variant_id),
                    )
                    .await?
            {
                return Err(AppError::Conflict(
                    "Variant with these attributes already exists for this product".to_string(),
                ));
            }
            variant.variant_attributes = new_attrs.clone();
        }

        // Apply other updates
        if let Some(barcode) = request.barcode {
            variant.barcode = Some(barcode);
        }
        if let Some(price_difference) = request.price_difference {
            variant.price_difference = price_difference;
        }
        if let Some(is_active) = request.is_active {
            variant.is_active = is_active;
        }

        variant.touch();

        // Save to database
        let updated = self
            .variant_repository
            .update(tenant_id, variant_id, &variant)
            .await?;

        // Get parent product info
        let parent = self
            .product_repository
            .find_by_id(tenant_id, updated.parent_product_id)
            .await?;

        Ok(VariantResponse::with_parent_info(
            updated,
            parent.as_ref().map(|p| p.name.clone()),
            parent.as_ref().map(|p| p.sku.clone()),
        ))
    }

    async fn delete_variant(&self, tenant_id: Uuid, variant_id: Uuid) -> Result<()> {
        let deleted = self
            .variant_repository
            .delete(tenant_id, variant_id)
            .await?;

        if !deleted {
            return Err(AppError::NotFound("Variant not found".to_string()));
        }

        Ok(())
    }

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    async fn bulk_activate(
        &self,
        tenant_id: Uuid,
        variant_ids: Vec<Uuid>,
    ) -> Result<BulkVariantOperationResponse> {
        if variant_ids.is_empty() {
            return Ok(BulkVariantOperationResponse::failure(
                "No variant IDs provided".to_string(),
            ));
        }

        let affected = self
            .variant_repository
            .bulk_activate(tenant_id, &variant_ids)
            .await?;

        Ok(BulkVariantOperationResponse::success(affected, "activated"))
    }

    async fn bulk_deactivate(
        &self,
        tenant_id: Uuid,
        variant_ids: Vec<Uuid>,
    ) -> Result<BulkVariantOperationResponse> {
        if variant_ids.is_empty() {
            return Ok(BulkVariantOperationResponse::failure(
                "No variant IDs provided".to_string(),
            ));
        }

        let affected = self
            .variant_repository
            .bulk_deactivate(tenant_id, &variant_ids)
            .await?;

        Ok(BulkVariantOperationResponse::success(affected, "deactivated"))
    }

    async fn bulk_delete(
        &self,
        tenant_id: Uuid,
        variant_ids: Vec<Uuid>,
    ) -> Result<BulkVariantOperationResponse> {
        if variant_ids.is_empty() {
            return Ok(BulkVariantOperationResponse::failure(
                "No variant IDs provided".to_string(),
            ));
        }

        let affected = self
            .variant_repository
            .bulk_delete(tenant_id, &variant_ids)
            .await?;

        Ok(BulkVariantOperationResponse::success(affected, "deleted"))
    }
}
