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
