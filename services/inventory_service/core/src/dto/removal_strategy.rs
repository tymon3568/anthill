//! Data Transfer Objects for removal strategies
//!
//! This module contains request and response structures for removal strategy operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domains::inventory::dto::common::validate_removal_strategy_type;
use crate::dto::PaginationInfo;

/// Information about available stock in a location
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockLocationInfo {
    pub location_id: Uuid,
    pub location_code: String,
    pub available_quantity: i64,
    pub lot_serial_id: Option<Uuid>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub last_receipt_date: Option<DateTime<Utc>>,
}

/// Request to create a new removal strategy
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RemovalStrategyCreateRequest {
    /// Strategy name (unique per tenant)
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    /// Removal strategy type
    #[validate(custom(function = "validate_removal_strategy_type"))]
    pub strategy_type: String,

    /// Optional warehouse scope (null for global)
    pub warehouse_id: Option<Uuid>,
    pub product_id: Option<Uuid>,

    /// Configuration
    pub config: serde_json::Value,
}

/// Request to update an existing removal strategy
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RemovalStrategyUpdateRequest {
    /// Strategy name (unique per tenant)
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    /// Removal strategy type
    #[validate(custom(function = "validate_removal_strategy_type"))]
    pub strategy_type: Option<String>,

    /// Optional warehouse scope (null for global)
    pub warehouse_id: Option<Uuid>,

    /// Optional product scope (null for all products)
    pub product_id: Option<Uuid>,

    /// Active status
    pub active: Option<bool>,

    /// Strategy configuration (JSON)
    pub config: Option<serde_json::Value>,
}

/// Query parameters for listing removal strategies
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RemovalStrategyListQuery {
    /// Filter by warehouse
    pub warehouse_id: Option<Uuid>,

    /// Filter by product
    pub product_id: Option<Uuid>,

    /// Filter by strategy type
    pub strategy_type: Option<String>,

    /// Filter by active status
    pub active: Option<bool>,

    /// Search by name
    pub search: Option<String>,

    /// Pagination
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Response for a single removal strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovalStrategyResponse {
    pub strategy_id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub strategy_type: String,
    pub strategy_type_display: String,
    pub warehouse_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub active: bool,
    pub config: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Response for listing removal strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovalStrategyListResponse {
    pub strategies: Vec<RemovalStrategyResponse>,
    pub pagination: PaginationInfo,
}

/// Request to suggest optimal stock for picking
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SuggestRemovalRequest {
    /// Warehouse to pick from
    pub warehouse_id: Uuid,

    /// Product to pick
    pub product_id: Uuid,

    /// Required quantity
    #[validate(range(min = 1))]
    pub quantity: i64,

    /// Optional location preferences
    pub preferred_location_ids: Option<Vec<Uuid>>,

    /// Force specific strategy (override automatic selection)
    pub force_strategy_id: Option<Uuid>,
}

/// Suggested stock location for picking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockSuggestion {
    pub location_id: Uuid,
    pub location_code: String,
    pub available_quantity: i64,
    pub suggested_quantity: i64,
    pub lot_serial_id: Option<Uuid>,
    pub expiry_date: Option<chrono::DateTime<chrono::Utc>>,
    pub strategy_used: String,
    pub strategy_reason: String,
}

/// Response for removal suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestRemovalResponse {
    pub suggestions: Vec<StockSuggestion>,
    pub total_suggested: i64,
    pub strategy_applied: String,
    pub can_fulfill: bool,
}

/// Analytics for strategy performance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyAnalyticsResponse {
    pub strategy_id: Uuid,
    pub strategy_name: String,
    pub strategy_type: String,
    pub usage_count: i64,
    pub average_pick_time: Option<f64>,
    pub success_rate: f64,
    pub total_picked_quantity: i64,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
}

// Default implementations for optional fields
// Removed unused default functions

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_removal_strategy_create_request_validation() {
        // Valid request
        let request = RemovalStrategyCreateRequest {
            name: "FIFO Strategy".to_string(),
            strategy_type: "fifo".to_string(),
            warehouse_id: Some(Uuid::new_v4()),
            product_id: None,
            config: json!({"priority": "oldest"}),
        };
        assert!(request.validate().is_ok());

        // Invalid strategy type
        let invalid_request = RemovalStrategyCreateRequest {
            name: "Invalid Strategy".to_string(),
            strategy_type: "invalid".to_string(),
            warehouse_id: None,
            product_id: None,
            config: json!({}),
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_pagination_info() {
        let info = PaginationInfo::new(1, 10, 25);
        assert_eq!(info.page, 1);
        assert_eq!(info.page_size, 10);
        assert_eq!(info.total_items, 25);
        assert_eq!(info.total_pages, 3);
        assert!(info.has_next);
        assert!(!info.has_prev);
    }

    #[test]
    fn test_suggest_removal_request_validation() {
        let request = SuggestRemovalRequest {
            warehouse_id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            quantity: 10,
            preferred_location_ids: Some(vec![Uuid::new_v4()]),
            force_strategy_id: None,
        };
        assert!(request.validate().is_ok());

        // Invalid quantity
        let invalid_request = SuggestRemovalRequest {
            warehouse_id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            quantity: 0,
            preferred_location_ids: None,
            force_strategy_id: None,
        };
        assert!(invalid_request.validate().is_err());
    }
}
