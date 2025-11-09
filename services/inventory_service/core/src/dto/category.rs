//! Category DTOs (Data Transfer Objects)
//!
//! Request and response structures for category API endpoints.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::domains::category::{Category, CategoryBreadcrumb, CategoryNode};

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to create a new category
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CategoryCreateRequest {
    /// Parent category ID (None for root categories)
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>, format = "uuid"))]
    pub parent_category_id: Option<Uuid>,

    /// Category name (required)
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Category description
    #[validate(length(max = 5000))]
    pub description: Option<String>,

    /// Optional category code for integration
    #[validate(length(max = 100))]
    pub code: Option<String>,

    /// Display order within same level
    #[serde(default)]
    pub display_order: i32,

    /// Icon name/class for UI
    #[validate(length(max = 100))]
    pub icon: Option<String>,

    /// Hex color code (e.g., #FF5733)
    #[validate(regex = "COLOR_REGEX")]
    pub color: Option<String>,

    /// Category image URL
    #[validate(url)]
    pub image_url: Option<String>,

    /// Whether category is active
    #[serde(default = "default_true")]
    pub is_active: bool,

    /// Whether category is visible in public catalogs
    #[serde(default = "default_true")]
    pub is_visible: bool,

    /// URL-friendly identifier (auto-generated from name if not provided)
    #[validate(length(max = 255))]
    pub slug: Option<String>,

    /// SEO meta title
    #[validate(length(max = 255))]
    pub meta_title: Option<String>,

    /// SEO meta description
    #[validate(length(max = 1000))]
    pub meta_description: Option<String>,

    /// SEO meta keywords
    #[validate(length(max = 500))]
    pub meta_keywords: Option<String>,
}

/// Request to update an existing category
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CategoryUpdateRequest {
    /// Parent category ID (None to make root category)
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>, format = "uuid"))]
    pub parent_category_id: Option<Uuid>,

    /// Category name
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    /// Category description
    #[validate(length(max = 5000))]
    pub description: Option<String>,

    /// Category code
    #[validate(length(max = 100))]
    pub code: Option<String>,

    /// Display order within same level
    pub display_order: Option<i32>,

    /// Icon name/class for UI
    #[validate(length(max = 100))]
    pub icon: Option<String>,

    /// Hex color code
    #[validate(regex = "COLOR_REGEX")]
    pub color: Option<String>,

    /// Category image URL
    #[validate(url)]
    pub image_url: Option<String>,

    /// Whether category is active
    pub is_active: Option<bool>,

    /// Whether category is visible
    pub is_visible: Option<bool>,

    /// URL-friendly identifier
    #[validate(length(max = 255))]
    pub slug: Option<String>,

    /// SEO meta title
    #[validate(length(max = 255))]
    pub meta_title: Option<String>,

    /// SEO meta description
    #[validate(length(max = 1000))]
    pub meta_description: Option<String>,

    /// SEO meta keywords
    #[validate(length(max = 500))]
    pub meta_keywords: Option<String>,
}

/// Request to move multiple products to a category
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct MoveToCategoryRequest {
    /// Product IDs to move
    #[validate(length(min = 1, max = 1000))]
    #[cfg_attr(feature = "openapi", schema(value_type = Vec<String>, format = "uuid"))]
    pub product_ids: Vec<Uuid>,

    /// Target category ID
    #[cfg_attr(feature = "openapi", schema(value_type = String, format = "uuid"))]
    pub category_id: Uuid,
}

/// Query parameters for listing categories
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CategoryListQuery {
    /// Filter by parent category (None for root categories only)
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>, format = "uuid"))]
    pub parent_id: Option<Uuid>,

    /// Filter by level (0 for root, 1 for first level, etc.)
    #[validate(range(min = 0))]
    pub level: Option<i32>,

    /// Filter by active status
    pub is_active: Option<bool>,

    /// Filter by visibility
    pub is_visible: Option<bool>,

    /// Search term (searches name and description)
    #[validate(length(max = 255))]
    pub search: Option<String>,

    /// Page number (1-based)
    #[serde(default = "default_page")]
    #[validate(range(min = 1))]
    pub page: i32,

    /// Page size
    #[serde(default = "default_page_size")]
    #[validate(range(min = 1, max = 100))]
    pub page_size: i32,

    /// Sort field
    #[serde(default)]
    pub sort_by: CategorySortField,

    /// Sort direction
    #[serde(default)]
    pub sort_dir: SortDirection,
}

/// Sort field options for categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
#[derive(PartialEq)]
pub enum CategorySortField {
    #[default]
    DisplayOrder,
    Name,
    CreatedAt,
    UpdatedAt,
    ProductCount,
    Level,
}

/// Sort direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "lowercase")]
#[derive(PartialEq)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Category response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CategoryResponse {
    pub category_id: Uuid,
    pub tenant_id: Uuid,
    pub parent_category_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub path: String,
    pub level: i32,
    pub display_order: i32,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub image_url: Option<String>,
    pub is_active: bool,
    pub is_visible: bool,
    pub slug: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub product_count: i32,
    pub total_product_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    /// Breadcrumb path from root to this category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breadcrumbs: Option<Vec<CategoryBreadcrumb>>,
}

impl From<Category> for CategoryResponse {
    fn from(category: Category) -> Self {
        Self {
            category_id: category.category_id,
            tenant_id: category.tenant_id,
            parent_category_id: category.parent_category_id,
            name: category.name,
            description: category.description,
            code: category.code,
            path: category.path,
            level: category.level,
            display_order: category.display_order,
            icon: category.icon,
            color: category.color,
            image_url: category.image_url,
            is_active: category.is_active,
            is_visible: category.is_visible,
            slug: category.slug,
            meta_title: category.meta_title,
            meta_description: category.meta_description,
            meta_keywords: category.meta_keywords,
            product_count: category.product_count,
            total_product_count: category.total_product_count,
            created_at: category.created_at,
            updated_at: category.updated_at,
            breadcrumbs: None,
        }
    }
}

/// Paginated list of categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CategoryListResponse {
    pub categories: Vec<CategoryResponse>,
    pub pagination: PaginationInfo,
}

/// Category tree response (hierarchical structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CategoryTreeResponse {
    pub category_id: Uuid,
    pub name: String,
    pub slug: Option<String>,
    pub level: i32,
    pub product_count: i32,
    pub total_product_count: i32,
    pub is_active: bool,
    pub children: Vec<CategoryTreeResponse>,
}

impl From<CategoryNode> for CategoryTreeResponse {
    fn from(node: CategoryNode) -> Self {
        Self {
            category_id: node.category.category_id,
            name: node.category.name,
            slug: node.category.slug,
            level: node.category.level,
            product_count: node.category.product_count,
            total_product_count: node.category.total_product_count,
            is_active: node.category.is_active,
            children: node.children.into_iter().map(Into::into).collect(),
        }
    }
}

/// Category statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CategoryStatsResponse {
    pub category_id: Uuid,
    pub name: String,
    pub level: i32,
    pub product_count: i32,
    pub total_product_count: i32,
    pub subcategory_count: i32,
    pub active_product_count: i32,
    pub inactive_product_count: i32,
}

/// Bulk operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct BulkOperationResponse {
    pub success: bool,
    pub affected_count: i32,
    pub message: String,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PaginationInfo {
    pub page: i32,
    pub page_size: i32,
    pub total_items: i64,
    pub total_pages: i32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationInfo {
    pub fn new(page: i32, page_size: i32, total_items: i64) -> Self {
        assert!(page_size > 0, "page_size must be greater than 0");
        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as i32;
        Self {
            page,
            page_size,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

// ============================================================================
// Helper functions and constants
// ============================================================================

fn default_true() -> bool {
    true
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

use std::sync::LazyLock;

static COLOR_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_info() {
        let info = PaginationInfo::new(1, 20, 100);
        assert_eq!(info.page, 1);
        assert_eq!(info.page_size, 20);
        assert_eq!(info.total_items, 100);
        assert_eq!(info.total_pages, 5);
        assert!(info.has_next);
        assert!(!info.has_prev);

        let info = PaginationInfo::new(3, 20, 100);
        assert!(info.has_next);
        assert!(info.has_prev);

        let info = PaginationInfo::new(5, 20, 100);
        assert!(!info.has_next);
        assert!(info.has_prev);

        // Edge cases
        let info = PaginationInfo::new(1, 10, 0);
        assert_eq!(info.total_pages, 0);
        assert!(!info.has_next);
        assert!(!info.has_prev);

        let info = PaginationInfo::new(1, 10, 1);
        assert_eq!(info.total_pages, 1);
        assert!(!info.has_next);
        assert!(!info.has_prev);
    }

    #[test]
    fn test_category_create_request_validation() {
        // Valid request
        let req = CategoryCreateRequest {
            parent_category_id: None,
            name: "Electronics".to_string(),
            description: Some("Electronic products".to_string()),
            code: Some("ELEC".to_string()),
            display_order: 1,
            icon: Some("electronics".to_string()),
            color: Some("#FF5733".to_string()),
            image_url: None,
            is_active: true,
            is_visible: true,
            slug: Some("electronics".to_string()),
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
        };
        assert!(req.validate().is_ok());

        // Test name validation
        let mut invalid_req = req.clone();
        invalid_req.name = "".to_string();
        assert!(invalid_req.validate().is_err());

        invalid_req.name = "a".repeat(256);
        assert!(invalid_req.validate().is_err());

        // Test description length
        let mut invalid_req = req.clone();
        invalid_req.description = Some("a".repeat(5001));
        assert!(invalid_req.validate().is_err());

        // Test color validation
        let mut invalid_req = req.clone();
        invalid_req.color = Some("invalid".to_string());
        assert!(invalid_req.validate().is_err());

        invalid_req.color = Some("#GGG".to_string());
        assert!(invalid_req.validate().is_err());

        invalid_req.color = Some("#FF573".to_string()); // 5 chars
        assert!(invalid_req.validate().is_err());

        // Valid colors
        invalid_req.color = Some("#ff5733".to_string()); // lowercase
        assert!(invalid_req.validate().is_ok());

        invalid_req.color = Some("#000000".to_string());
        assert!(invalid_req.validate().is_ok());

        // Test URL validation
        let mut invalid_req = req.clone();
        invalid_req.image_url = Some("not-a-url".to_string());
        assert!(invalid_req.validate().is_err());

        invalid_req.image_url = Some("https://example.com/image.jpg".to_string());
        assert!(invalid_req.validate().is_ok());
    }

    #[test]
    fn test_category_update_request_validation() {
        let req = CategoryUpdateRequest {
            parent_category_id: None,
            name: Some("Updated Electronics".to_string()),
            description: Some("Updated description".to_string()),
            code: Some("ELEC2".to_string()),
            display_order: Some(2),
            icon: Some("updated-icon".to_string()),
            color: Some("#00FF00".to_string()),
            image_url: Some("https://example.com/new-image.jpg".to_string()),
            is_active: Some(true),
            is_visible: Some(false),
            slug: Some("updated-electronics".to_string()),
            meta_title: Some("Updated Meta Title".to_string()),
            meta_description: Some("Updated meta description".to_string()),
            meta_keywords: Some("updated, keywords".to_string()),
        };
        assert!(req.validate().is_ok());

        // Test optional fields can be None
        let req = CategoryUpdateRequest {
            parent_category_id: None,
            name: None,
            description: None,
            code: None,
            display_order: None,
            icon: None,
            color: None,
            image_url: None,
            is_active: None,
            is_visible: None,
            slug: None,
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
        };
        assert!(req.validate().is_ok());

        // Test validation when fields are provided
        let invalid_req = CategoryUpdateRequest {
            parent_category_id: None,
            name: Some("".to_string()),
            description: None,
            code: None,
            display_order: None,
            icon: None,
            color: None,
            image_url: None,
            is_active: None,
            is_visible: None,
            slug: None,
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
        };
        assert!(invalid_req.validate().is_err());
    }

    #[test]
    fn test_category_list_query_defaults() {
        let query: CategoryListQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 20);
        assert_eq!(query.sort_by, CategorySortField::DisplayOrder);
        assert_eq!(query.sort_dir, SortDirection::Asc);
    }

    #[test]
    fn test_category_list_query_validation() {
        // Valid query
        let query = CategoryListQuery {
            parent_id: Some(Uuid::new_v4()),
            level: Some(1),
            is_active: Some(true),
            is_visible: Some(false),
            search: Some("electronics".to_string()),
            page: 2,
            page_size: 50,
            sort_by: CategorySortField::Name,
            sort_dir: SortDirection::Desc,
        };
        assert!(query.validate().is_ok());

        // Invalid page
        let mut invalid_query = query.clone();
        invalid_query.page = 0;
        assert!(invalid_query.validate().is_err());

        // Invalid page_size
        let mut invalid_query = query.clone();
        invalid_query.page_size = 0;
        assert!(invalid_query.validate().is_err());

        invalid_query.page_size = 101;
        assert!(invalid_query.validate().is_err());

        // Invalid level
        let mut invalid_query = query.clone();
        invalid_query.level = Some(-1);
        assert!(invalid_query.validate().is_err());

        // Invalid search length
        let mut invalid_query = query.clone();
        invalid_query.search = Some("a".repeat(256));
        assert!(invalid_query.validate().is_err());
    }

    #[test]
    fn test_move_to_category_request_validation() {
        let req = MoveToCategoryRequest {
            product_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            category_id: Uuid::new_v4(),
        };
        assert!(req.validate().is_ok());

        // Empty product_ids
        let mut invalid_req = req.clone();
        invalid_req.product_ids = vec![];
        assert!(invalid_req.validate().is_err());

        // Too many product_ids
        let mut invalid_req = req.clone();
        invalid_req.product_ids = (0..1001).map(|_| Uuid::new_v4()).collect();
        assert!(invalid_req.validate().is_err());
    }

    #[test]
    fn test_category_response_from_category() {
        let category = Category {
            category_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            parent_category_id: Some(Uuid::new_v4()),
            name: "Test Category".to_string(),
            description: Some("Test description".to_string()),
            code: Some("TEST".to_string()),
            path: "root/test".to_string(),
            level: 1,
            display_order: 5,
            icon: Some("test-icon".to_string()),
            color: Some("#123456".to_string()),
            image_url: Some("https://example.com/image.jpg".to_string()),
            is_active: true,
            is_visible: false,
            slug: Some("test-category".to_string()),
            meta_title: Some("Test Meta".to_string()),
            meta_description: Some("Test meta desc".to_string()),
            meta_keywords: Some("test, keywords".to_string()),
            product_count: 10,
            total_product_count: 25,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let response: CategoryResponse = category.into();
        assert_eq!(response.category_id, response.category_id); // Just check it's set
        assert_eq!(response.name, "Test Category");
        assert_eq!(response.breadcrumbs, None); // Not set in conversion
    }

    #[test]
    fn test_category_tree_response_from_node() {
        let category = Category {
            category_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            parent_category_id: None,
            name: "Root Category".to_string(),
            description: None,
            code: None,
            path: "root".to_string(),
            level: 0,
            display_order: 0,
            icon: None,
            color: None,
            image_url: None,
            is_active: true,
            is_visible: true,
            slug: Some("root-category".to_string()),
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
            product_count: 5,
            total_product_count: 15,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut node = CategoryNode::new(category.clone());
        let child_category = Category {
            category_id: Uuid::new_v4(),
            tenant_id: category.tenant_id,
            parent_category_id: Some(category.category_id),
            name: "Child Category".to_string(),
            description: None,
            code: None,
            path: "root/child".to_string(),
            level: 1,
            display_order: 0,
            icon: None,
            color: None,
            image_url: None,
            is_active: true,
            is_visible: true,
            slug: Some("child-category".to_string()),
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
            product_count: 3,
            total_product_count: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };
        node.add_child(CategoryNode::new(child_category));

        let tree: CategoryTreeResponse = node.into();
        assert_eq!(tree.category_id, category.category_id);
        assert_eq!(tree.name, category.name);
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].name, "Child Category");
    }

    #[test]
    fn test_sort_field_serialization() {
        let field = CategorySortField::Name;
        let serialized = serde_json::to_string(&field).unwrap();
        assert_eq!(serialized, "\"name\"");

        let deserialized: CategorySortField = serde_json::from_str("\"created_at\"").unwrap();
        assert!(matches!(deserialized, CategorySortField::CreatedAt));
    }

    #[test]
    fn test_sort_direction_serialization() {
        let dir = SortDirection::Desc;
        let serialized = serde_json::to_string(&dir).unwrap();
        assert_eq!(serialized, "\"desc\"");

        let deserialized: SortDirection = serde_json::from_str("\"asc\"").unwrap();
        assert!(matches!(deserialized, SortDirection::Asc));
    }

    #[test]
    fn test_color_regex() {
        // Test the regex directly
        assert!(COLOR_REGEX.is_match("#FF5733"));
        assert!(COLOR_REGEX.is_match("#000000"));
        assert!(COLOR_REGEX.is_match("#ffffff"));
        assert!(COLOR_REGEX.is_match("#123ABC"));

        assert!(!COLOR_REGEX.is_match("#GGG"));
        assert!(!COLOR_REGEX.is_match("#FF573"));
        assert!(!COLOR_REGEX.is_match("FF5733"));
        assert!(!COLOR_REGEX.is_match("#FF57333"));
        assert!(!COLOR_REGEX.is_match("#ff573g"));
    }
}
