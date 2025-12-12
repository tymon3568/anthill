//! Common validation functions shared across DTOs and domain entities
//!
//! This module contains validation functions that are used in multiple places
//! to reduce code duplication and ensure consistency.

use serde::{Deserialize, Serialize};
use sqlx::Type;
use validator::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "removal_strategy_type", rename_all = "snake_case")]
pub enum RemovalStrategyType {
    Fifo,
    Lifo,
    Fefo,
    ClosestLocation,
    LeastPackages,
}

/// Validate picking method type (batch, cluster, wave)
/// Case-insensitive validation
pub fn validate_picking_method_type(method_type: &str) -> Result<(), ValidationError> {
    let normalized = method_type.to_ascii_lowercase();
    match normalized.as_str() {
        "batch" | "cluster" | "wave" => Ok(()),
        _ => Err(ValidationError::new("invalid_picking_method_type")),
    }
}

/// Validate warehouse type
pub fn validate_warehouse_type(warehouse_type: &str) -> Result<(), ValidationError> {
    match warehouse_type {
        "main" | "transit" | "quarantine" | "distribution" | "retail" | "satellite" => Ok(()),
        _ => Err(ValidationError::new("invalid_warehouse_type")),
    }
}

/// Validate zone type
pub fn validate_zone_type(zone_type: &str) -> Result<(), ValidationError> {
    match zone_type {
        "storage" | "picking" | "quarantine" | "receiving" | "shipping" | "bulk" | "damaged"
        | "returns" => Ok(()),
        _ => Err(ValidationError::new("invalid_zone_type")),
    }
}

/// Validate location type
pub fn validate_location_type(location_type: &str) -> Result<(), ValidationError> {
    match location_type {
        "bin" | "shelf" | "pallet" | "floor" | "rack" | "container" | "bulk" => Ok(()),
        _ => Err(ValidationError::new("invalid_location_type")),
    }
}

/// Validate product type
pub fn validate_product_type(product_type: &str) -> Result<(), ValidationError> {
    match product_type {
        "goods" | "service" | "consumable" => Ok(()),
        _ => Err(ValidationError::new("invalid_product_type")),
    }
}

/// Validate removal strategy type (fifo, lifo, fefo, closest_location, least_packages)
pub fn validate_removal_strategy_type(strategy_type: &str) -> Result<(), ValidationError> {
    let normalized = strategy_type.to_ascii_lowercase();
    match normalized.as_str() {
        "fifo" | "lifo" | "fefo" | "closest_location" | "least_packages" => Ok(()),
        _ => Err(ValidationError::new("invalid_removal_strategy_type")),
    }
}

/// Validate that JSON config is not null or empty object
pub fn validate_config_not_empty(config: &serde_json::Value) -> Result<(), ValidationError> {
    if config.is_null() {
        return Err(ValidationError::new("config_cannot_be_null"));
    }
    if config.as_object().map(|o| o.is_empty()).unwrap_or(false) {
        return Err(ValidationError::new("config_cannot_be_empty"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // =========================================================================
    // validate_product_type Tests
    // =========================================================================

    #[test]
    fn test_validate_product_type_goods() {
        assert!(validate_product_type("goods").is_ok());
    }

    #[test]
    fn test_validate_product_type_service() {
        assert!(validate_product_type("service").is_ok());
    }

    #[test]
    fn test_validate_product_type_consumable() {
        assert!(validate_product_type("consumable").is_ok());
    }

    #[test]
    fn test_validate_product_type_invalid() {
        let result = validate_product_type("invalid");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "invalid_product_type");
    }

    // =========================================================================
    // validate_warehouse_type Tests
    // =========================================================================

    #[test]
    fn test_validate_warehouse_type_all_valid() {
        let valid_types = [
            "main",
            "transit",
            "quarantine",
            "distribution",
            "retail",
            "satellite",
        ];
        for t in valid_types {
            assert!(validate_warehouse_type(t).is_ok(), "Failed for type: {}", t);
        }
    }

    #[test]
    fn test_validate_warehouse_type_invalid() {
        let result = validate_warehouse_type("unknown");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "invalid_warehouse_type");
    }

    // =========================================================================
    // validate_zone_type Tests
    // =========================================================================

    #[test]
    fn test_validate_zone_type_all_valid() {
        let valid_types = [
            "storage",
            "picking",
            "quarantine",
            "receiving",
            "shipping",
            "bulk",
            "damaged",
            "returns",
        ];
        for t in valid_types {
            assert!(validate_zone_type(t).is_ok(), "Failed for type: {}", t);
        }
    }

    #[test]
    fn test_validate_zone_type_invalid() {
        let result = validate_zone_type("invalid_zone");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "invalid_zone_type");
    }

    // =========================================================================
    // validate_location_type Tests
    // =========================================================================

    #[test]
    fn test_validate_location_type_all_valid() {
        let valid_types = [
            "bin",
            "shelf",
            "pallet",
            "floor",
            "rack",
            "container",
            "bulk",
        ];
        for t in valid_types {
            assert!(validate_location_type(t).is_ok(), "Failed for type: {}", t);
        }
    }

    #[test]
    fn test_validate_location_type_invalid() {
        let result = validate_location_type("drawer");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "invalid_location_type");
    }

    // =========================================================================
    // validate_picking_method_type Tests
    // =========================================================================

    #[test]
    fn test_validate_picking_method_type_valid() {
        assert!(validate_picking_method_type("batch").is_ok());
        assert!(validate_picking_method_type("cluster").is_ok());
        assert!(validate_picking_method_type("wave").is_ok());
    }

    #[test]
    fn test_validate_picking_method_type_case_insensitive() {
        assert!(validate_picking_method_type("BATCH").is_ok());
        assert!(validate_picking_method_type("Cluster").is_ok());
        assert!(validate_picking_method_type("WAVE").is_ok());
    }

    #[test]
    fn test_validate_picking_method_type_invalid() {
        let result = validate_picking_method_type("random");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "invalid_picking_method_type");
    }

    // =========================================================================
    // validate_removal_strategy_type Tests
    // =========================================================================

    #[test]
    fn test_validate_removal_strategy_type_valid() {
        let valid_strategies = ["fifo", "lifo", "fefo", "closest_location", "least_packages"];
        for s in valid_strategies {
            assert!(validate_removal_strategy_type(s).is_ok(), "Failed for strategy: {}", s);
        }
    }

    #[test]
    fn test_validate_removal_strategy_type_case_insensitive() {
        assert!(validate_removal_strategy_type("FIFO").is_ok());
        assert!(validate_removal_strategy_type("Lifo").is_ok());
        assert!(validate_removal_strategy_type("FEFO").is_ok());
    }

    #[test]
    fn test_validate_removal_strategy_type_invalid() {
        let result = validate_removal_strategy_type("random_pick");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "invalid_removal_strategy_type");
    }

    // =========================================================================
    // validate_config_not_empty Tests
    // =========================================================================

    #[test]
    fn test_validate_config_valid_object() {
        let config = json!({"key": "value"});
        assert!(validate_config_not_empty(&config).is_ok());
    }

    #[test]
    fn test_validate_config_null_fails() {
        let config = json!(null);
        let result = validate_config_not_empty(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "config_cannot_be_null");
    }

    #[test]
    fn test_validate_config_empty_object_fails() {
        let config = json!({});
        let result = validate_config_not_empty(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "config_cannot_be_empty");
    }

    #[test]
    fn test_validate_config_array_allowed() {
        let config = json!([1, 2, 3]);
        assert!(validate_config_not_empty(&config).is_ok());
    }

    #[test]
    fn test_validate_config_complex_object() {
        let config = json!({
            "nested": {
                "array": [1, 2, 3],
                "string": "value"
            }
        });
        assert!(validate_config_not_empty(&config).is_ok());
    }
}
