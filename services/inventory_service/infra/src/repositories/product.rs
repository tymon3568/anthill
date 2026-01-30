//! Product repository implementation
//!
//! PostgreSQL implementation of the ProductRepository trait.

use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder, Row};
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::search_dto::{
    AppliedFilters, ProductSearchRequest, ProductSearchResponse, ProductSortBy,
    SearchSuggestionsRequest, SearchSuggestionsResponse, SortOrder,
};
use inventory_service_core::domains::inventory::product::{
    BarcodeType, Product, ProductTrackingMethod,
};
use inventory_service_core::repositories::product::ProductRepository;
use inventory_service_core::Result;

/// PostgreSQL implementation of ProductRepository
pub struct ProductRepositoryImpl {
    pool: PgPool,
}

impl ProductRepositoryImpl {
    /// Create new repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for ProductRepositoryImpl {
    // ========================================================================
    // Search Operations
    // ========================================================================

    async fn search_products(
        &self,
        tenant_id: Uuid,
        request: ProductSearchRequest,
    ) -> Result<ProductSearchResponse> {
        use inventory_service_core::domains::inventory::dto::search_dto::{
            ProductSearchResult, SearchFacets, SearchMeta,
        };
        use inventory_service_core::dto::common::PaginationInfo;

        let start_time = std::time::Instant::now();

        // Build query using QueryBuilder for safety
        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT
                p.product_id,
                p.sku,
                p.name,
                p.description,
                p.product_type,
                p.category_id,
                c.name as category_name,
                p.track_inventory,
                p.sale_price,
                p.cost_price,
                p.currency_code,
                p.is_active,
                p.is_sellable,
                p.created_at,
                p.updated_at
            "#,
        );

        // Add relevance_score if there's a search query
        if let Some(q) = &request.query {
            query_builder.push(
                r#",
                ts_rank(
                    to_tsvector('english', p.name || ' ' || COALESCE(p.description, '')),
                    plainto_tsquery('english', "#,
            );
            query_builder.push_bind(q.as_str());
            query_builder.push(
                r#")
                ) AS relevance_score
            "#,
            );
        } else {
            query_builder.push(
                r#",
                NULL AS relevance_score
            "#,
            );
        }

        query_builder.push(
            r#"
            FROM products p
            LEFT JOIN product_categories c ON p.category_id = c.category_id AND c.tenant_id = p.tenant_id
            WHERE p.tenant_id =
            "#,
        );
        query_builder.push_bind(tenant_id);
        query_builder.push(" AND p.deleted_at IS NULL");

        // Add full-text search if query provided
        if let Some(q) = &request.query {
            query_builder.push(" AND to_tsvector('english', p.name || ' ' || COALESCE(p.description, '')) @@ plainto_tsquery('english', ");
            query_builder.push_bind(q.as_str());
            query_builder.push(")");
        }

        // Add category filtering
        if let Some(category_ids) = &request.category_ids {
            if !category_ids.is_empty() {
                query_builder.push(" AND p.category_id = ANY(");
                query_builder.push_bind(category_ids.as_slice());
                query_builder.push(")");
            }
        }

        // Add price filtering
        if let Some(min_price) = request.price_min {
            query_builder.push(" AND p.sale_price >= ");
            query_builder.push_bind(min_price);
        }
        if let Some(max_price) = request.price_max {
            query_builder.push(" AND p.sale_price <= ");
            query_builder.push_bind(max_price);
        }

        // Add product type filtering
        if let Some(types) = &request.product_types {
            if !types.is_empty() {
                query_builder.push(" AND p.product_type = ANY(");
                query_builder.push_bind(types.as_slice());
                query_builder.push(")");
            }
        }

        // Add status filters
        // active_only: Some(true) = show only active, Some(false) = show only inactive, None = show all
        if let Some(active) = request.active_only {
            if active {
                query_builder.push(" AND p.is_active = true");
            } else {
                query_builder.push(" AND p.is_active = false");
            }
        }
        // sellable_only: Some(true) = show only sellable, Some(false) = show only non-sellable, None = show all
        if let Some(sellable) = request.sellable_only {
            if sellable {
                query_builder.push(" AND p.is_sellable = true");
            } else {
                query_builder.push(" AND p.is_sellable = false");
            }
        }

        // Add in-stock filter
        if request.in_stock_only.unwrap_or(false) {
            // Workaround until inventory_levels table is implemented:
            // Only return products with track_inventory=false (considered "always in stock").
            // Products with track_inventory=true require actual stock data and are excluded.
            query_builder.push(" AND p.track_inventory = false");
        }

        // Add sorting
        let sort_by = request
            .sort_by
            .as_ref()
            .unwrap_or(&ProductSortBy::Relevance);
        let sort_order = request.sort_order.as_ref().unwrap_or(&SortOrder::Desc);

        let order_clause = match sort_by {
            ProductSortBy::Relevance => {
                if request.query.is_some() {
                    "relevance_score"
                } else {
                    "p.created_at"
                }
            },
            ProductSortBy::Name => "p.name",
            ProductSortBy::Price => "p.sale_price",
            ProductSortBy::Popularity => "p.created_at", // TODO: implement popularity
            ProductSortBy::CreatedAt => "p.created_at",
            ProductSortBy::UpdatedAt => "p.updated_at",
        };

        query_builder.push(" ORDER BY ");
        query_builder.push(order_clause);

        let order_direction = match sort_order {
            SortOrder::Asc => " ASC",
            SortOrder::Desc => " DESC",
        };
        query_builder.push(order_direction);

        // Add pagination
        let page = request.page.unwrap_or(1).max(1);
        let limit = request.limit.unwrap_or(20).min(100);
        let offset = (page - 1) * limit;

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        // Execute query
        let rows = query_builder.build().fetch_all(&self.pool).await?;

        // Convert rows to results
        let products: Vec<ProductSearchResult> = rows
            .into_iter()
            .map(|row| {
                let highlights = if let Some(q) = &request.query {
                    vec![format!("...{}...", q)] // TODO: proper highlighting
                } else {
                    vec![]
                };

                ProductSearchResult {
                    product_id: row.get("product_id"),
                    sku: row.get("sku"),
                    name: row.get("name"),
                    description: row.get("description"),
                    sale_price: row.get("sale_price"),
                    cost_price: row.get("cost_price"),
                    currency_code: row.get("currency_code"),
                    product_type: row.get("product_type"),
                    category_id: row.get("category_id"),
                    category_name: row.get("category_name"),
                    category_path: None, // TODO: implement path
                    track_inventory: row.get("track_inventory"),
                    // Temporary: if not tracking inventory, assume always in stock
                    in_stock: Some(!row.get::<bool, _>("track_inventory")),
                    is_active: row.get("is_active"),
                    is_sellable: row.get("is_sellable"),
                    highlights,
                    relevance_score: row.try_get::<f32, _>("relevance_score").unwrap_or(0.0),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .collect();

        // Get total count
        let mut count_builder =
            QueryBuilder::new("SELECT COUNT(*) as count FROM products p WHERE p.tenant_id = ");
        count_builder.push_bind(tenant_id);
        count_builder.push(" AND p.deleted_at IS NULL");

        // Apply same filters for count
        if let Some(q) = &request.query {
            count_builder.push(" AND to_tsvector('english', p.name || ' ' || COALESCE(p.description, '')) @@ plainto_tsquery('english', ");
            count_builder.push_bind(q.as_str());
            count_builder.push(")");
        }

        if let Some(category_ids) = &request.category_ids {
            if !category_ids.is_empty() {
                count_builder.push(" AND p.category_id = ANY(");
                count_builder.push_bind(category_ids.as_slice());
                count_builder.push(")");
            }
        }

        if let Some(min_price) = request.price_min {
            count_builder.push(" AND p.sale_price >= ");
            count_builder.push_bind(min_price);
        }
        if let Some(max_price) = request.price_max {
            count_builder.push(" AND p.sale_price <= ");
            count_builder.push_bind(max_price);
        }

        if let Some(types) = &request.product_types {
            if !types.is_empty() {
                count_builder.push(" AND p.product_type = ANY(");
                count_builder.push_bind(types.as_slice());
                count_builder.push(")");
            }
        }

        // Add status filters (must match main query logic)
        if let Some(active) = request.active_only {
            if active {
                count_builder.push(" AND p.is_active = true");
            } else {
                count_builder.push(" AND p.is_active = false");
            }
        }
        if let Some(sellable) = request.sellable_only {
            if sellable {
                count_builder.push(" AND p.is_sellable = true");
            } else {
                count_builder.push(" AND p.is_sellable = false");
            }
        }

        // Add in-stock filter (must match main query logic)
        if request.in_stock_only.unwrap_or(false) {
            count_builder.push(" AND p.track_inventory = false");
        }

        let total_count: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;

        // Build pagination info
        #[allow(clippy::manual_div_ceil)]
        let total_pages = ((total_count as u32 + limit - 1) / limit).max(1);
        let pagination = PaginationInfo {
            page,
            page_size: limit,
            total_items: total_count as u64,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        };

        // TODO: Implement facets
        let facets = SearchFacets {
            categories: vec![],
            price_ranges: vec![],
            product_types: vec![],
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        let meta = SearchMeta {
            query: request.query.clone(),
            execution_time_ms: execution_time,
            total_found: total_count as u64,
            applied_filters: AppliedFilters {
                category_ids: request.category_ids,
                price_min: request.price_min,
                price_max: request.price_max,
                in_stock_only: request.in_stock_only,
                product_types: request.product_types,
                active_only: request.active_only,
                sellable_only: request.sellable_only,
            },
        };

        Ok(ProductSearchResponse {
            products,
            pagination,
            facets,
            meta,
        })
    }

    async fn get_search_suggestions(
        &self,
        tenant_id: Uuid,
        request: SearchSuggestionsRequest,
    ) -> Result<SearchSuggestionsResponse> {
        use inventory_service_core::domains::inventory::dto::search_dto::{
            SearchSuggestion, SuggestionType,
        };

        let limit = request.limit.unwrap_or(10).min(20) as i64;
        let search_pattern = format!("%{}%", request.query);

        // Search for product names
        let product_suggestions = sqlx::query!(
            r#"
            SELECT name as text, COUNT(*) as count
            FROM products
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND name ILIKE $2
            GROUP BY name
            ORDER BY count DESC, name
            LIMIT $3
            "#,
            tenant_id,
            search_pattern,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        // Search for SKUs
        let sku_suggestions = sqlx::query!(
            r#"
            SELECT sku as text, COUNT(*) as count
            FROM products
            WHERE tenant_id = $1
              AND deleted_at IS NULL
              AND sku ILIKE $2
            GROUP BY sku
            ORDER BY count DESC, sku
            LIMIT $3
            "#,
            tenant_id,
            search_pattern,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        // Combine and deduplicate suggestions
        let mut suggestions_map: std::collections::HashMap<String, SearchSuggestion> =
            std::collections::HashMap::new();

        for row in product_suggestions {
            let text = row.text;
            let count = row.count.unwrap_or(0) as u32;
            suggestions_map
                .entry(text.clone())
                .or_insert(SearchSuggestion {
                    text,
                    product_count: count,
                    suggestion_type: SuggestionType::ProductName,
                });
        }

        for row in sku_suggestions {
            let text = row.text;
            let count = row.count.unwrap_or(0) as u32;
            suggestions_map
                .entry(text.clone())
                .and_modify(|s| {
                    // If already exists, keep ProductName type but add counts
                    s.product_count = s.product_count.max(count);
                })
                .or_insert(SearchSuggestion {
                    text,
                    product_count: count,
                    suggestion_type: SuggestionType::Sku,
                });
        }

        let mut suggestions: Vec<SearchSuggestion> = suggestions_map.into_values().collect();
        suggestions.sort_by(|a, b| b.product_count.cmp(&a.product_count));
        suggestions.truncate(limit as usize);

        Ok(SearchSuggestionsResponse { suggestions })
    }

    // ========================================================================
    // CRUD Operations (Future)
    // ========================================================================

    async fn find_by_id(&self, tenant_id: Uuid, product_id: Uuid) -> Result<Option<Product>> {
        let row = sqlx::query!(
            r#"
            SELECT
                product_id, tenant_id, sku, name, description,
                product_type, barcode, barcode_type, category_id, item_group_id, track_inventory, tracking_method,
                default_uom_id, sale_price, cost_price, currency_code,
                weight_grams, dimensions, attributes,
                is_active, is_sellable, is_purchaseable,
                created_at, updated_at, deleted_at
            FROM products
            WHERE tenant_id = $1 AND product_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            product_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Product {
            product_id: row.product_id,
            tenant_id: row.tenant_id,
            sku: row.sku,
            name: row.name,
            description: row.description,
            product_type: row.product_type,
            barcode: row.barcode,
            barcode_type: row.barcode_type.and_then(|s| s.parse::<BarcodeType>().ok()),
            category_id: row.category_id,
            item_group_id: row.item_group_id,
            track_inventory: row.track_inventory,
            tracking_method: row
                .tracking_method
                .unwrap_or_else(|| "none".to_string())
                .parse::<ProductTrackingMethod>()
                .unwrap_or(ProductTrackingMethod::None),
            default_uom_id: row.default_uom_id,
            sale_price: row.sale_price,
            cost_price: row.cost_price,
            currency_code: row.currency_code.unwrap_or_else(|| "VND".to_string()),
            weight_grams: row.weight_grams,
            dimensions: row.dimensions,
            attributes: row.attributes,
            is_active: row.is_active,
            is_sellable: row.is_sellable,
            is_purchaseable: row.is_purchaseable,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        }))
    }

    async fn find_by_ids(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<Vec<Product>> {
        if product_ids.is_empty() {
            return Ok(vec![]);
        }

        let rows = sqlx::query!(
            r#"
            SELECT
                product_id, tenant_id, sku, name, description,
                product_type, barcode, barcode_type, category_id, item_group_id, track_inventory, tracking_method,
                default_uom_id, sale_price, cost_price, currency_code,
                weight_grams, dimensions, attributes,
                is_active, is_sellable, is_purchaseable,
                created_at, updated_at, deleted_at
            FROM products
            WHERE tenant_id = $1 AND product_id = ANY($2) AND deleted_at IS NULL
            "#,
            tenant_id,
            &product_ids
        )
        .fetch_all(&self.pool)
        .await?;

        let products = rows
            .into_iter()
            .map(|row| Product {
                product_id: row.product_id,
                tenant_id: row.tenant_id,
                sku: row.sku,
                name: row.name,
                description: row.description,
                product_type: row.product_type,
                barcode: row.barcode,
                barcode_type: row.barcode_type.and_then(|s| s.parse::<BarcodeType>().ok()),
                category_id: row.category_id,
                item_group_id: row.item_group_id,
                track_inventory: row.track_inventory,
                tracking_method: row
                    .tracking_method
                    .unwrap_or_else(|| "none".to_string())
                    .parse::<ProductTrackingMethod>()
                    .unwrap_or(ProductTrackingMethod::None),
                default_uom_id: row.default_uom_id,
                sale_price: row.sale_price,
                cost_price: row.cost_price,
                currency_code: row.currency_code.unwrap_or_else(|| "VND".to_string()),
                weight_grams: row.weight_grams,
                dimensions: row.dimensions,
                attributes: row.attributes,
                is_active: row.is_active,
                is_sellable: row.is_sellable,
                is_purchaseable: row.is_purchaseable,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            })
            .collect();

        Ok(products)
    }

    async fn find_by_sku(&self, tenant_id: Uuid, sku: &str) -> Result<Option<Product>> {
        let row = sqlx::query!(
            r#"
            SELECT
                product_id, tenant_id, sku, name, description,
                product_type, barcode, barcode_type, category_id, item_group_id, track_inventory, tracking_method,
                default_uom_id, sale_price, cost_price, currency_code,
                weight_grams, dimensions, attributes,
                is_active, is_sellable, is_purchaseable,
                created_at, updated_at, deleted_at
            FROM products
            WHERE tenant_id = $1 AND sku = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            sku
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Product {
            product_id: row.product_id,
            tenant_id: row.tenant_id,
            sku: row.sku,
            name: row.name,
            description: row.description,
            product_type: row.product_type,
            barcode: row.barcode,
            barcode_type: row.barcode_type.and_then(|s| s.parse::<BarcodeType>().ok()),
            category_id: row.category_id,
            item_group_id: row.item_group_id,
            track_inventory: row.track_inventory,
            tracking_method: row
                .tracking_method
                .unwrap_or_else(|| "none".to_string())
                .parse::<ProductTrackingMethod>()
                .unwrap_or(ProductTrackingMethod::None),
            default_uom_id: row.default_uom_id,
            sale_price: row.sale_price,
            cost_price: row.cost_price,
            currency_code: row.currency_code.unwrap_or_else(|| "VND".to_string()),
            weight_grams: row.weight_grams,
            dimensions: row.dimensions,
            attributes: row.attributes,
            is_active: row.is_active,
            is_sellable: row.is_sellable,
            is_purchaseable: row.is_purchaseable,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        }))
    }

    async fn find_by_barcode(&self, tenant_id: Uuid, barcode: &str) -> Result<Option<Product>> {
        // First, try to find in products.barcode column (new dedicated field)
        let row = sqlx::query!(
            r#"
            SELECT
                product_id, tenant_id, sku, name, description,
                product_type, barcode, barcode_type, category_id, item_group_id, track_inventory, tracking_method,
                default_uom_id, sale_price, cost_price, currency_code,
                weight_grams, dimensions, attributes,
                is_active, is_sellable, is_purchaseable,
                created_at, updated_at, deleted_at
            FROM products
            WHERE tenant_id = $1 AND barcode = $2 AND deleted_at IS NULL
            LIMIT 1
            "#,
            tenant_id,
            barcode
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let product = Product {
                product_id: row.product_id,
                tenant_id: row.tenant_id,
                sku: row.sku,
                name: row.name,
                description: row.description,
                product_type: row.product_type,
                barcode: row.barcode,
                barcode_type: row.barcode_type.and_then(|s| s.parse::<BarcodeType>().ok()),
                category_id: row.category_id,
                item_group_id: row.item_group_id,
                track_inventory: row.track_inventory,
                tracking_method: row
                    .tracking_method
                    .unwrap_or_else(|| "none".to_string())
                    .parse::<ProductTrackingMethod>()
                    .unwrap_or(ProductTrackingMethod::None),
                default_uom_id: row.default_uom_id,
                sale_price: row.sale_price,
                cost_price: row.cost_price,
                currency_code: row.currency_code.unwrap_or_else(|| "VND".to_string()),
                weight_grams: row.weight_grams,
                dimensions: row.dimensions,
                attributes: row.attributes,
                is_active: row.is_active,
                is_sellable: row.is_sellable,
                is_purchaseable: row.is_purchaseable,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            };
            return Ok(Some(product));
        }

        // Second, try to find in product_variants table
        let row = sqlx::query!(
            r#"
            SELECT
                p.product_id, p.tenant_id, p.sku, p.name, p.description,
                p.product_type, p.barcode, p.barcode_type, p.category_id, p.item_group_id, p.track_inventory, p.tracking_method,
                p.default_uom_id, p.sale_price, p.cost_price, p.currency_code,
                p.weight_grams, p.dimensions, p.attributes,
                p.is_active, p.is_sellable, p.is_purchaseable,
                p.created_at, p.updated_at, p.deleted_at
            FROM product_variants pv
            JOIN products p ON pv.parent_product_id = p.product_id
            WHERE pv.tenant_id = $1 AND pv.barcode = $2 AND pv.deleted_at IS NULL
              AND p.deleted_at IS NULL
            LIMIT 1
            "#,
            tenant_id,
            barcode
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let product = Product {
                product_id: row.product_id,
                tenant_id: row.tenant_id,
                sku: row.sku,
                name: row.name,
                description: row.description,
                product_type: row.product_type,
                barcode: row.barcode,
                barcode_type: row.barcode_type.and_then(|s| s.parse::<BarcodeType>().ok()),
                category_id: row.category_id,
                item_group_id: row.item_group_id,
                track_inventory: row.track_inventory,
                tracking_method: row
                    .tracking_method
                    .unwrap_or_else(|| "none".to_string())
                    .parse::<ProductTrackingMethod>()
                    .unwrap_or(ProductTrackingMethod::None),
                default_uom_id: row.default_uom_id,
                sale_price: row.sale_price,
                cost_price: row.cost_price,
                currency_code: row.currency_code.unwrap_or_else(|| "VND".to_string()),
                weight_grams: row.weight_grams,
                dimensions: row.dimensions,
                attributes: row.attributes,
                is_active: row.is_active,
                is_sellable: row.is_sellable,
                is_purchaseable: row.is_purchaseable,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            };
            return Ok(Some(product));
        }

        Ok(None)
    }

    async fn create(&self, product: &Product) -> Result<Product> {
        let tracking_method_str = product.tracking_method.to_string();
        let barcode_type_str = product.barcode_type.as_ref().map(|bt| bt.to_string());

        let row = sqlx::query!(
            r#"
            INSERT INTO products (
                product_id, tenant_id, sku, name, description,
                product_type, barcode, barcode_type, category_id, item_group_id, track_inventory, tracking_method,
                default_uom_id, sale_price, cost_price, currency_code,
                weight_grams, dimensions, attributes,
                is_active, is_sellable, is_purchaseable,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16,
                $17, $18, $19,
                $20, $21, $22,
                $23, $24
            )
            RETURNING
                product_id, tenant_id, sku, name, description,
                product_type, barcode, barcode_type, category_id, item_group_id, track_inventory, tracking_method,
                default_uom_id, sale_price, cost_price, currency_code,
                weight_grams, dimensions, attributes,
                is_active, is_sellable, is_purchaseable,
                created_at, updated_at, deleted_at
            "#,
            product.product_id,
            product.tenant_id,
            product.sku,
            product.name,
            product.description,
            product.product_type,
            product.barcode,
            barcode_type_str,
            product.category_id,
            product.item_group_id,
            product.track_inventory,
            tracking_method_str,
            product.default_uom_id,
            product.sale_price,
            product.cost_price,
            product.currency_code,
            product.weight_grams,
            product.dimensions,
            product.attributes,
            product.is_active,
            product.is_sellable,
            product.is_purchaseable,
            product.created_at,
            product.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Product {
            product_id: row.product_id,
            tenant_id: row.tenant_id,
            sku: row.sku,
            name: row.name,
            description: row.description,
            product_type: row.product_type,
            barcode: row.barcode,
            barcode_type: row.barcode_type.and_then(|s| s.parse::<BarcodeType>().ok()),
            category_id: row.category_id,
            item_group_id: row.item_group_id,
            track_inventory: row.track_inventory,
            tracking_method: row
                .tracking_method
                .unwrap_or_else(|| "none".to_string())
                .parse::<ProductTrackingMethod>()
                .unwrap_or(ProductTrackingMethod::None),
            default_uom_id: row.default_uom_id,
            sale_price: row.sale_price,
            cost_price: row.cost_price,
            currency_code: row.currency_code.unwrap_or_else(|| "VND".to_string()),
            weight_grams: row.weight_grams,
            dimensions: row.dimensions,
            attributes: row.attributes,
            is_active: row.is_active,
            is_sellable: row.is_sellable,
            is_purchaseable: row.is_purchaseable,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }

    async fn update(
        &self,
        tenant_id: Uuid,
        product_id: Uuid,
        product: &Product,
    ) -> Result<Product> {
        let tracking_method_str = product.tracking_method.to_string();
        let barcode_type_str = product.barcode_type.as_ref().map(|bt| bt.to_string());

        let row = sqlx::query!(
            r#"
            UPDATE products SET
                name = $3,
                description = $4,
                product_type = $5,
                barcode = $6,
                barcode_type = $7,
                category_id = $8,
                item_group_id = $9,
                track_inventory = $10,
                tracking_method = $11,
                default_uom_id = $12,
                sale_price = $13,
                cost_price = $14,
                currency_code = $15,
                weight_grams = $16,
                dimensions = $17,
                attributes = $18,
                is_active = $19,
                is_sellable = $20,
                is_purchaseable = $21,
                updated_at = $22
            WHERE tenant_id = $1 AND product_id = $2 AND deleted_at IS NULL
            RETURNING
                product_id, tenant_id, sku, name, description,
                product_type, barcode, barcode_type, category_id, item_group_id, track_inventory, tracking_method,
                default_uom_id, sale_price, cost_price, currency_code,
                weight_grams, dimensions, attributes,
                is_active, is_sellable, is_purchaseable,
                created_at, updated_at, deleted_at
            "#,
            tenant_id,
            product_id,
            product.name,
            product.description,
            product.product_type,
            product.barcode,
            barcode_type_str,
            product.category_id,
            product.item_group_id,
            product.track_inventory,
            tracking_method_str,
            product.default_uom_id,
            product.sale_price,
            product.cost_price,
            product.currency_code,
            product.weight_grams,
            product.dimensions,
            product.attributes,
            product.is_active,
            product.is_sellable,
            product.is_purchaseable,
            product.updated_at
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| shared_error::AppError::NotFound("Product not found".to_string()))?;

        Ok(Product {
            product_id: row.product_id,
            tenant_id: row.tenant_id,
            sku: row.sku,
            name: row.name,
            description: row.description,
            product_type: row.product_type,
            barcode: row.barcode,
            barcode_type: row.barcode_type.and_then(|s| s.parse::<BarcodeType>().ok()),
            category_id: row.category_id,
            item_group_id: row.item_group_id,
            track_inventory: row.track_inventory,
            tracking_method: row
                .tracking_method
                .unwrap_or_else(|| "none".to_string())
                .parse::<ProductTrackingMethod>()
                .unwrap_or(ProductTrackingMethod::None),
            default_uom_id: row.default_uom_id,
            sale_price: row.sale_price,
            cost_price: row.cost_price,
            currency_code: row.currency_code.unwrap_or_else(|| "VND".to_string()),
            weight_grams: row.weight_grams,
            dimensions: row.dimensions,
            attributes: row.attributes,
            is_active: row.is_active,
            is_sellable: row.is_sellable,
            is_purchaseable: row.is_purchaseable,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        })
    }

    async fn delete(&self, tenant_id: Uuid, product_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE products
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            product_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========================================================================
    // Analytics and Statistics (Future)
    // ========================================================================

    async fn record_search_analytics(
        &self,
        _tenant_id: Uuid,
        _query: &str,
        _result_count: u32,
        _user_id: Option<Uuid>,
    ) -> Result<()> {
        todo!("Implement search analytics recording")
    }

    async fn get_popular_search_terms(
        &self,
        _tenant_id: Uuid,
        _limit: u32,
    ) -> Result<Vec<(String, u32)>> {
        todo!("Implement popular search terms")
    }

    // ========================================================================
    // Inventory Operations (Future)
    // ========================================================================

    async fn is_in_stock(&self, _tenant_id: Uuid, _product_id: Uuid) -> Result<bool> {
        todo!("Implement inventory check")
    }

    async fn get_inventory_level(&self, _tenant_id: Uuid, _product_id: Uuid) -> Result<i64> {
        todo!("Implement inventory level query")
    }

    // ========================================================================
    // Bulk Operations
    // ========================================================================

    async fn bulk_activate(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<i64> {
        if product_ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query!(
            r#"
            UPDATE products
            SET is_active = true, updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = ANY($2) AND deleted_at IS NULL
            "#,
            tenant_id,
            &product_ids
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    async fn bulk_deactivate(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<i64> {
        if product_ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query!(
            r#"
            UPDATE products
            SET is_active = false, updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = ANY($2) AND deleted_at IS NULL
            "#,
            tenant_id,
            &product_ids
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    async fn bulk_delete(&self, tenant_id: Uuid, product_ids: &[Uuid]) -> Result<i64> {
        if product_ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query!(
            r#"
            UPDATE products
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE tenant_id = $1 AND product_id = ANY($2) AND deleted_at IS NULL
            "#,
            tenant_id,
            &product_ids
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    // ========================================================================
    // Import/Export Operations
    // ========================================================================

    async fn save(&self, product: &Product) -> Result<()> {
        let tracking_method_str = product.tracking_method.to_string();
        let barcode_type_str = product.barcode_type.as_ref().map(|bt| bt.to_string());

        sqlx::query!(
            r#"
            INSERT INTO products (
                product_id, tenant_id, sku, name, description,
                product_type, barcode, barcode_type, category_id, item_group_id, track_inventory, tracking_method,
                default_uom_id, sale_price, cost_price, currency_code,
                weight_grams, dimensions, attributes,
                is_active, is_sellable, is_purchaseable,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16,
                $17, $18, $19,
                $20, $21, $22,
                $23, $24
            )
            ON CONFLICT (tenant_id, sku) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                product_type = EXCLUDED.product_type,
                barcode = EXCLUDED.barcode,
                barcode_type = EXCLUDED.barcode_type,
                category_id = EXCLUDED.category_id,
                item_group_id = EXCLUDED.item_group_id,
                track_inventory = EXCLUDED.track_inventory,
                tracking_method = EXCLUDED.tracking_method,
                default_uom_id = EXCLUDED.default_uom_id,
                sale_price = EXCLUDED.sale_price,
                cost_price = EXCLUDED.cost_price,
                currency_code = EXCLUDED.currency_code,
                weight_grams = EXCLUDED.weight_grams,
                dimensions = EXCLUDED.dimensions,
                attributes = EXCLUDED.attributes,
                is_active = EXCLUDED.is_active,
                is_sellable = EXCLUDED.is_sellable,
                is_purchaseable = EXCLUDED.is_purchaseable,
                updated_at = EXCLUDED.updated_at
            "#,
            product.product_id,
            product.tenant_id,
            product.sku,
            product.name,
            product.description,
            product.product_type,
            product.barcode,
            barcode_type_str,
            product.category_id,
            product.item_group_id,
            product.track_inventory,
            tracking_method_str,
            product.default_uom_id,
            product.sale_price,
            product.cost_price,
            product.currency_code,
            product.weight_grams,
            product.dimensions,
            product.attributes,
            product.is_active,
            product.is_sellable,
            product.is_purchaseable,
            product.created_at,
            product.updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_all_for_export(
        &self,
        tenant_id: Uuid,
        category_id: Option<Uuid>,
        product_type: Option<&str>,
        is_active: Option<bool>,
        search: Option<&str>,
    ) -> Result<Vec<Product>> {
        let mut query_builder = QueryBuilder::new(
            r#"
            SELECT
                product_id, tenant_id, sku, name, description,
                product_type, barcode, barcode_type, category_id, item_group_id, track_inventory, tracking_method,
                default_uom_id, sale_price, cost_price, currency_code,
                weight_grams, dimensions, attributes,
                is_active, is_sellable, is_purchaseable,
                created_at, updated_at, deleted_at
            FROM products
            WHERE tenant_id =
            "#,
        );
        query_builder.push_bind(tenant_id);
        query_builder.push(" AND deleted_at IS NULL");

        if let Some(cat_id) = category_id {
            query_builder.push(" AND category_id = ");
            query_builder.push_bind(cat_id);
        }

        if let Some(ptype) = product_type {
            query_builder.push(" AND product_type = ");
            query_builder.push_bind(ptype);
        }

        if let Some(active) = is_active {
            query_builder.push(" AND is_active = ");
            query_builder.push_bind(active);
        }

        if let Some(s) = search {
            let pattern = format!("%{}%", s);
            query_builder.push(" AND (sku ILIKE ");
            query_builder.push_bind(pattern.clone());
            query_builder.push(" OR name ILIKE ");
            query_builder.push_bind(pattern);
            query_builder.push(")");
        }

        query_builder.push(" ORDER BY sku ASC");

        let rows = query_builder.build().fetch_all(&self.pool).await?;

        let products = rows
            .into_iter()
            .map(|row| Product {
                product_id: row.get("product_id"),
                tenant_id: row.get("tenant_id"),
                sku: row.get("sku"),
                name: row.get("name"),
                description: row.get("description"),
                product_type: row.get("product_type"),
                barcode: row.get("barcode"),
                barcode_type: row
                    .get::<Option<String>, _>("barcode_type")
                    .and_then(|s| s.parse::<BarcodeType>().ok()),
                category_id: row.get("category_id"),
                item_group_id: row.get("item_group_id"),
                track_inventory: row.get("track_inventory"),
                tracking_method: row
                    .get::<Option<String>, _>("tracking_method")
                    .unwrap_or_else(|| "none".to_string())
                    .parse::<ProductTrackingMethod>()
                    .unwrap_or(ProductTrackingMethod::None),
                default_uom_id: row.get("default_uom_id"),
                sale_price: row.get("sale_price"),
                cost_price: row.get("cost_price"),
                currency_code: row
                    .get::<Option<String>, _>("currency_code")
                    .unwrap_or_else(|| "VND".to_string()),
                weight_grams: row.get("weight_grams"),
                dimensions: row.get("dimensions"),
                attributes: row.get("attributes"),
                is_active: row.get("is_active"),
                is_sellable: row.get("is_sellable"),
                is_purchaseable: row.get("is_purchaseable"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                deleted_at: row.get("deleted_at"),
            })
            .collect();

        Ok(products)
    }
}
