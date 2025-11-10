//! Product repository implementation
//!
//! PostgreSQL implementation of the ProductRepository trait.

use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder, Row};
use uuid::Uuid;

use inventory_service_core::domains::inventory::dto::search_dto::{
    AppliedFilters, PaginationInfo, ProductSearchRequest, ProductSearchResponse,
    ProductSearchResult, ProductSortBy, SearchFacets, SearchMeta, SearchSuggestion,
    SearchSuggestionsRequest, SearchSuggestionsResponse, SortOrder, SuggestionType,
};
use inventory_service_core::domains::inventory::product::Product;
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
            PaginationInfo, ProductSearchResult, SearchFacets, SearchMeta,
        };

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
        if request.active_only.unwrap_or(true) {
            query_builder.push(" AND p.is_active = true");
        }
        if request.sellable_only.unwrap_or(true) {
            query_builder.push(" AND p.is_sellable = true");
        }

        // Add in-stock filter
        if request.in_stock_only.unwrap_or(false) {
            // Since inventory_levels table doesn't exist yet, use track_inventory logic:
            // - If track_inventory = false: always in stock
            // - If track_inventory = true: currently consider out of stock (until inventory table is implemented)
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
                    in_stock: Some(!row.get::<bool, _>("track_inventory")), // If not tracking inventory, consider in stock
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

        if request.active_only.unwrap_or(true) {
            count_builder.push(" AND p.is_active = true");
        }
        if request.sellable_only.unwrap_or(true) {
            count_builder.push(" AND p.is_sellable = true");
        }

        let total_count: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;

        // Build pagination info
        let total_pages = ((total_count as u32 + limit - 1) / limit).max(1);
        let pagination = PaginationInfo {
            page,
            limit,
            total_count: total_count as u64,
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

        // Combine and limit
        let mut suggestions: Vec<SearchSuggestion> = vec![];

        for row in product_suggestions {
            suggestions.push(SearchSuggestion {
                text: row.text,
                product_count: row.count as u32,
                suggestion_type: SuggestionType::ProductName,
            });
        }

        for row in sku_suggestions {
            suggestions.push(SearchSuggestion {
                text: row.text,
                product_count: row.count as u32,
                suggestion_type: SuggestionType::Sku,
            });
        }

        // Sort by count desc, then limit
        suggestions.sort_by(|a, b| b.product_count.cmp(&a.product_count));
        suggestions.truncate(limit as usize);

        Ok(SearchSuggestionsResponse { suggestions })
    }

    // ========================================================================
    // CRUD Operations (Future)
    // ========================================================================

    async fn find_by_id(&self, _tenant_id: Uuid, _product_id: Uuid) -> Result<Option<Product>> {
        todo!("Implement find_by_id")
    }

    async fn find_by_sku(&self, _tenant_id: Uuid, _sku: &str) -> Result<Option<Product>> {
        todo!("Implement find_by_sku")
    }

    async fn create(&self, _product: &Product) -> Result<Product> {
        todo!("Implement create")
    }

    async fn update(
        &self,
        _tenant_id: Uuid,
        _product_id: Uuid,
        _product: &Product,
    ) -> Result<Product> {
        todo!("Implement update")
    }

    async fn delete(&self, _tenant_id: Uuid, _product_id: Uuid) -> Result<bool> {
        todo!("Implement delete")
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
}
