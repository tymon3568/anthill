//! Category domain entity
//!
//! Represents a product category in the hierarchical category system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Product category entity
///
/// Represents a category in the hierarchical product organization system.
/// Uses materialized path pattern for efficient tree queries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Category {
    /// Unique category identifier (UUID v7)
    pub category_id: Uuid,

    /// Tenant identifier for multi-tenancy
    pub tenant_id: Uuid,

    /// Parent category ID (None for root categories)
    pub parent_category_id: Option<Uuid>,

    /// Category name
    pub name: String,

    /// Category description
    pub description: Option<String>,

    /// Optional category code for integration
    pub code: Option<String>,

    /// Materialized path (e.g., "uuid1/uuid2/uuid3")
    pub path: String,

    /// Depth in hierarchy (0 for root, 1 for first level, etc.)
    pub level: i32,

    /// Display order within same level
    pub display_order: i32,

    /// Icon name/class for UI
    pub icon: Option<String>,

    /// Hex color code (e.g., #FF5733)
    pub color: Option<String>,

    /// Category image URL
    pub image_url: Option<String>,

    /// Whether category is active
    pub is_active: bool,

    /// Whether category is visible in public catalogs
    pub is_visible: bool,

    /// URL-friendly identifier
    pub slug: Option<String>,

    /// SEO meta title
    pub meta_title: Option<String>,

    /// SEO meta description
    pub meta_description: Option<String>,

    /// SEO meta keywords
    pub meta_keywords: Option<String>,

    /// Number of direct products in this category
    pub product_count: i32,

    /// Total products including subcategories
    pub total_product_count: i32,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Soft delete timestamp
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Category {
    /// Check if this is a root category
    pub fn is_root(&self) -> bool {
        self.parent_category_id.is_none()
    }

    /// Check if category is deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// Check if category can have products
    pub fn can_have_products(&self) -> bool {
        self.is_active && !self.is_deleted()
    }

    /// Get breadcrumb path as vector of UUIDs
    pub fn get_path_ids(&self) -> Vec<Uuid> {
        self.path
            .split('/')
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect()
    }

    /// Check if this category is an ancestor of another category
    pub fn is_ancestor_of(&self, other: &Category) -> bool {
        other.path.starts_with(&self.path) && self.category_id != other.category_id
    }

    /// Check if this category is a descendant of another category
    pub fn is_descendant_of(&self, other: &Category) -> bool {
        self.path.starts_with(&other.path) && self.category_id != other.category_id
    }

    /// Validate color format (hex color code)
    pub fn is_valid_color(&self) -> bool {
        static COLOR_REGEX: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap());

        if let Some(ref color) = self.color {
            COLOR_REGEX.is_match(color)
        } else {
            true
        }
    }
}

/// Category node for tree representation
///
/// Represents a category with its children in a tree structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct CategoryNode {
    /// Category data
    #[serde(flatten)]
    pub category: Category,

    /// Child categories
    pub children: Vec<CategoryNode>,
}

impl CategoryNode {
    /// Create a new category node without children
    pub fn new(category: Category) -> Self {
        Self {
            category,
            children: Vec::new(),
        }
    }

    /// Create a new category node with children
    pub fn with_children(category: Category, children: Vec<CategoryNode>) -> Self {
        Self { category, children }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: CategoryNode) {
        self.children.push(child);
    }

    /// Get total number of descendants
    pub fn count_descendants(&self) -> usize {
        self.children.len()
            + self
                .children
                .iter()
                .map(|c| c.count_descendants())
                .sum::<usize>()
    }

    /// Find a node by category_id
    pub fn find_node(&self, category_id: Uuid) -> Option<&CategoryNode> {
        if self.category.category_id == category_id {
            return Some(self);
        }

        for child in &self.children {
            if let Some(node) = child.find_node(category_id) {
                return Some(node);
            }
        }

        None
    }

    /// Flatten tree to list of categories
    pub fn flatten(&self) -> Vec<Category> {
        let mut categories = vec![self.category.clone()];
        for child in &self.children {
            categories.extend(child.flatten());
        }
        categories
    }
}

/// Category breadcrumb item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[derive(PartialEq)]
pub struct CategoryBreadcrumb {
    pub category_id: Uuid,
    pub name: String,
    pub slug: Option<String>,
    pub level: i32,
}

impl From<Category> for CategoryBreadcrumb {
    fn from(category: Category) -> Self {
        Self {
            category_id: category.category_id,
            name: category.name,
            slug: category.slug,
            level: category.level,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_category(id: &str, parent_id: Option<&str>, path: &str, level: i32) -> Category {
        Category {
            category_id: Uuid::parse_str(id).unwrap(),
            tenant_id: Uuid::new_v4(),
            parent_category_id: parent_id.map(|p| Uuid::parse_str(p).unwrap()),
            name: "Test Category".to_string(),
            description: None,
            code: None,
            path: path.to_string(),
            level,
            display_order: 0,
            icon: None,
            color: Some("#FF5733".to_string()),
            image_url: None,
            is_active: true,
            is_visible: true,
            slug: None,
            meta_title: None,
            meta_description: None,
            meta_keywords: None,
            product_count: 0,
            total_product_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    #[test]
    fn test_is_root() {
        let root = create_test_category(
            "00000000-0000-0000-0000-000000000001",
            None,
            "00000000-0000-0000-0000-000000000001",
            0,
        );
        assert!(root.is_root());

        let child = create_test_category(
            "00000000-0000-0000-0000-000000000002",
            Some("00000000-0000-0000-0000-000000000001"),
            "00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000002",
            1,
        );
        assert!(!child.is_root());
    }

    #[test]
    fn test_is_ancestor_of() {
        let root = create_test_category(
            "00000000-0000-0000-0000-000000000001",
            None,
            "00000000-0000-0000-0000-000000000001",
            0,
        );

        let child = create_test_category(
            "00000000-0000-0000-0000-000000000002",
            Some("00000000-0000-0000-0000-000000000001"),
            "00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000002",
            1,
        );

        let grandchild = create_test_category(
            "00000000-0000-0000-0000-000000000003",
            Some("00000000-0000-0000-0000-000000000002"),
            "00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000002/00000000-0000-0000-0000-000000000003",
            2,
        );

        assert!(root.is_ancestor_of(&child));
        assert!(root.is_ancestor_of(&grandchild));
        assert!(child.is_ancestor_of(&grandchild));
        assert!(!child.is_ancestor_of(&root));
        assert!(!root.is_ancestor_of(&root)); // Not ancestor of itself
    }

    #[test]
    fn test_is_descendant_of() {
        let root = create_test_category(
            "00000000-0000-0000-0000-000000000001",
            None,
            "00000000-0000-0000-0000-000000000001",
            0,
        );

        let child = create_test_category(
            "00000000-0000-0000-0000-000000000002",
            Some("00000000-0000-0000-0000-000000000001"),
            "00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000002",
            1,
        );

        let grandchild = create_test_category(
            "00000000-0000-0000-0000-000000000003",
            Some("00000000-0000-0000-0000-000000000002"),
            "00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000002/00000000-0000-0000-0000-000000000003",
            2,
        );

        assert!(child.is_descendant_of(&root));
        assert!(grandchild.is_descendant_of(&root));
        assert!(grandchild.is_descendant_of(&child));
        assert!(!root.is_descendant_of(&child));
        assert!(!root.is_descendant_of(&root)); // Not descendant of itself
    }

    #[test]
    fn test_is_valid_color() {
        let mut category = create_test_category(
            "00000000-0000-0000-0000-000000000001",
            None,
            "00000000-0000-0000-0000-000000000001",
            0,
        );

        // Valid colors
        category.color = Some("#FF5733".to_string());
        assert!(category.is_valid_color());

        category.color = Some("#ff5733".to_string());
        assert!(category.is_valid_color());

        category.color = Some("#000000".to_string());
        assert!(category.is_valid_color());

        category.color = Some("#FFFFFF".to_string());
        assert!(category.is_valid_color());

        // Invalid colors
        category.color = Some("FF5733".to_string());
        assert!(!category.is_valid_color());

        category.color = Some("#GGG".to_string());
        assert!(!category.is_valid_color());

        category.color = Some("#FF573".to_string()); // Too short
        assert!(!category.is_valid_color());

        category.color = Some("#FF57333".to_string()); // Too long
        assert!(!category.is_valid_color());

        category.color = Some("#ff573g".to_string()); // Invalid char
        assert!(!category.is_valid_color());

        category.color = Some("invalid".to_string());
        assert!(!category.is_valid_color());

        // None is valid (no color set)
        category.color = None;
        assert!(category.is_valid_color());
    }

    #[test]
    fn test_category_node_operations() {
        let root = create_test_category(
            "00000000-0000-0000-0000-000000000001",
            None,
            "00000000-0000-0000-0000-000000000001",
            0,
        );

        let child1 = create_test_category(
            "00000000-0000-0000-0000-000000000002",
            Some("00000000-0000-0000-0000-000000000001"),
            "00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000002",
            1,
        );

        let child2 = create_test_category(
            "00000000-0000-0000-0000-000000000003",
            Some("00000000-0000-0000-0000-000000000001"),
            "00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000003",
            1,
        );

        let grandchild = create_test_category(
            "00000000-0000-0000-0000-000000000004",
            Some("00000000-0000-0000-0000-000000000002"),
            "00000000-0000-0000-0000-000000000001/00000000-0000-0000-0000-000000000002/00000000-0000-0000-0000-000000000004",
            2,
        );

        let mut root_node = CategoryNode::new(root);
        let mut child1_node = CategoryNode::new(child1);
        child1_node.add_child(CategoryNode::new(grandchild));
        root_node.add_child(child1_node);
        root_node.add_child(CategoryNode::new(child2));

        // Test count_descendants
        assert_eq!(root_node.count_descendants(), 3);

        // Test that children have correct counts
        assert_eq!(root_node.children[0].count_descendants(), 1); // child1 has 1 grandchild
        assert_eq!(root_node.children[1].count_descendants(), 0); // child2 has no children

        // Test node creation
        let node = CategoryNode::new(create_test_category(
            "00000000-0000-0000-0000-000000000005",
            None,
            "00000000-0000-0000-0000-000000000005",
            0,
        ));
        assert!(node.children.is_empty());
        assert_eq!(node.count_descendants(), 0);
    }

    #[test]
    fn test_category_breadcrumb() {
        let breadcrumb = CategoryBreadcrumb {
            category_id: Uuid::new_v4(),
            name: "Electronics".to_string(),
            slug: Some("electronics".to_string()),
            level: 0,
        };

        assert_eq!(breadcrumb.name, "Electronics");
        assert_eq!(breadcrumb.level, 0);
    }

    #[test]
    fn test_category_creation_with_various_fields() {
        let now = Utc::now();
        let category = Category {
            category_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            parent_category_id: Some(Uuid::new_v4()),
            name: "Test Category".to_string(),
            description: Some("A test category".to_string()),
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
            meta_title: Some("Test Meta Title".to_string()),
            meta_description: Some("Test meta description".to_string()),
            meta_keywords: Some("test, category, keywords".to_string()),
            product_count: 10,
            total_product_count: 25,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        assert_eq!(category.name, "Test Category");
        assert_eq!(category.level, 1);
        assert_eq!(category.product_count, 10);
        assert_eq!(category.total_product_count, 25);
        assert!(category.is_active);
        assert!(!category.is_visible);
        assert!(category.deleted_at.is_none());
    }

    #[test]
    fn test_category_with_soft_delete() {
        let mut category = create_test_category(
            "00000000-0000-0000-0000-000000000001",
            None,
            "00000000-0000-0000-0000-000000000001",
            0,
        );

        // Initially not deleted
        assert!(category.deleted_at.is_none());

        // Mark as deleted
        category.deleted_at = Some(Utc::now());
        assert!(category.deleted_at.is_some());
    }
}
