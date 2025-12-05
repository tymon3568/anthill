//! Putaway service implementation
//!
//! Implementation of the PutawayService trait with business logic for putaway rule evaluation.

use async_trait::async_trait;
use regex::Regex;
use std::collections::HashMap;
use uuid::Uuid;

use inventory_service_core::models::{
    ConfirmPutawayRequest, ConfirmPutawayResponse, CreateStockMoveRequest, PutawayRequest,
    PutawayRule, PutawaySuggestion, StorageLocation,
};
use inventory_service_core::repositories::putaway::{PutawayRepository, PutawayService};
use inventory_service_core::repositories::StockMoveRepository;
use shared_error::AppError;

use crate::repositories::stock::PgStockMoveRepository;

/// Implementation of PutawayService
pub struct PgPutawayService<R: PutawayRepository + Send + Sync> {
    putaway_repo: R,
    stock_move_repo: PgStockMoveRepository,
}

impl<R: PutawayRepository + Send + Sync> PgPutawayService<R> {
    /// Create new service instance
    pub fn new(putaway_repo: R, stock_move_repo: PgStockMoveRepository) -> Self {
        Self {
            putaway_repo,
            stock_move_repo,
        }
    }
}

#[async_trait]
impl<R: PutawayRepository + Send + Sync> PutawayService for PgPutawayService<R> {
    async fn suggest_putaway_locations(
        &self,
        tenant_id: &Uuid,
        request: &PutawayRequest,
    ) -> Result<Vec<PutawaySuggestion>, AppError> {
        // Get active putaway rules
        let rules = self.putaway_repo.get_active_rules(tenant_id).await?;

        // Get available locations for the warehouse
        let warehouse_id = request.warehouse_id.ok_or_else(|| {
            AppError::ValidationError(
                "warehouse_id is required for putaway suggestions".to_string(),
            )
        })?;

        let mut locations = self
            .putaway_repo
            .get_available_locations(
                tenant_id,
                &warehouse_id,
                request.preferred_location_type.as_deref(),
            )
            .await?;

        // Filter locations with capacity
        locations.retain(|loc| {
            if let Some(capacity) = loc.capacity {
                loc.current_stock < capacity
            } else {
                true // No capacity limit
            }
        });

        if locations.is_empty() {
            return Ok(vec![]);
        }

        // Evaluate rules and score locations
        let mut scored_locations = Vec::new();

        for location in locations {
            let score = self
                .evaluate_location_score(&rules, &location, request)
                .await?;
            if score > 0 {
                scored_locations.push((location, score));
            }
        }

        // Sort by score descending
        scored_locations.sort_by(|a, b| b.1.cmp(&a.1));

        // Convert to suggestions
        let suggestions = scored_locations
            .into_iter()
            .map(|(location, score)| {
                let available_capacity = location.capacity.map(|cap| cap - location.current_stock);

                PutawaySuggestion {
                    location_id: location.location_id,
                    location_code: location.location_code,
                    warehouse_id: location.warehouse_id,
                    zone: location.zone,
                    aisle: location.aisle,
                    rack: location.rack,
                    level: location.level,
                    position: location.position,
                    available_capacity,
                    current_stock: location.current_stock,
                    score,
                    rule_applied: Some("Rule-based scoring".to_string()), // TODO: track which rule
                }
            })
            .collect();

        Ok(suggestions)
    }

    async fn confirm_putaway(
        &self,
        tenant_id: &Uuid,
        request: &ConfirmPutawayRequest,
        user_id: &Uuid,
    ) -> Result<ConfirmPutawayResponse, AppError> {
        let mut stock_moves_created = Vec::new();
        let mut total_quantity = 0i64;

        // Aggregate quantities per location to prevent capacity bypass
        let mut per_location_qty: HashMap<Uuid, i64> = HashMap::new();

        for allocation in &request.allocations {
            total_quantity = total_quantity
                .checked_add(allocation.quantity)
                .ok_or_else(|| {
                    AppError::ValidationError("Total putaway quantity overflow".to_string())
                })?;

            let entry = per_location_qty.entry(allocation.location_id).or_insert(0);
            *entry = entry.checked_add(allocation.quantity).ok_or_else(|| {
                AppError::ValidationError(format!(
                    "Putaway quantity overflow for location {}",
                    allocation.location_id
                ))
            })?;
        }

        // Validate capacity for each unique location
        for (location_id, alloc_qty) in per_location_qty {
            let location = self
                .putaway_repo
                .get_location_by_id(tenant_id, &location_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("Storage location {} not found", location_id))
                })?;

            if let Some(capacity) = location.capacity {
                if location.current_stock + alloc_qty > capacity {
                    return Err(AppError::ValidationError(format!(
                        "Location {} does not have enough capacity. Current: {}, Requested: {}, Capacity: {}",
                        location.location_code, location.current_stock, alloc_qty, capacity
                    )));
                }
            }
        }

        // Create stock moves for each allocation
        // TODO: Implement transactional atomicity for all-or-nothing updates

        for allocation in &request.allocations {
            let stock_move = CreateStockMoveRequest {
                product_id: request.product_id, // Fixed: Use actual product_id from request
                source_location_id: None,       // Putaway from receiving area
                destination_location_id: Some(allocation.location_id),
                move_type: "putaway".to_string(),
                quantity: allocation.quantity,
                unit_cost: None, // TODO: Get from GRN
                reference_type: request.reference_type.clone(),
                reference_id: request.reference_id,
                lot_serial_id: None,
                idempotency_key: Uuid::now_v7().to_string(),
                move_reason: Some("Putaway confirmation".to_string()),
                batch_info: None,
                metadata: None,
            };

            let created_move = self.stock_move_repo.create(&stock_move, *tenant_id).await?;
            stock_moves_created.push(created_move.move_id);

            // Update location stock
            let location = self
                .putaway_repo
                .get_location_by_id(tenant_id, &allocation.location_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!(
                        "Storage location {} was deleted during putaway",
                        allocation.location_id
                    ))
                })?;

            let new_stock = location.current_stock + allocation.quantity;
            self.putaway_repo
                .update_location_stock(tenant_id, &allocation.location_id, new_stock)
                .await?;
        }

        Ok(ConfirmPutawayResponse {
            stock_moves_created,
            total_quantity_putaway: total_quantity,
        })
    }

    async fn validate_location_capacity(
        &self,
        tenant_id: &Uuid,
        location_id: &Uuid,
        quantity: i64,
    ) -> Result<bool, AppError> {
        let location = self
            .putaway_repo
            .get_location_by_id(tenant_id, location_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Storage location {} not found", location_id))
            })?;

        let has_capacity = if let Some(capacity) = location.capacity {
            location.current_stock + quantity <= capacity
        } else {
            true
        };

        Ok(has_capacity)
    }
}

impl<R: PutawayRepository + Send + Sync> PgPutawayService<R> {
    /// Evaluate how well a location matches the putaway rules for a request
    async fn evaluate_location_score(
        &self,
        rules: &[PutawayRule],
        location: &StorageLocation,
        request: &PutawayRequest,
    ) -> Result<i32, AppError> {
        let mut total_score = 0i32;

        for rule in rules {
            let rule_score = self
                .rule_applies_to_request(rule, request, location)
                .await?;
            if rule_score > 0 {
                total_score += self.score_location_preferences(rule, location)?;
                total_score += self.validate_rule_quantities(rule, request, location)?;
            }
        }

        // Add base score for location type preference
        if let Some(pref_type) = &request.preferred_location_type {
            if location.location_type == *pref_type {
                total_score += 10;
            }
        }

        // Penalize locations that are close to capacity
        if let Some(capacity) = location.capacity {
            let utilization_ratio = location.current_stock as f64 / capacity as f64;
            if utilization_ratio > 0.9 {
                total_score -= 20; // High penalty for near-full locations
            } else if utilization_ratio > 0.7 {
                total_score -= 10; // Medium penalty
            }
        }

        Ok(total_score.max(0)) // Ensure non-negative score
    }

    /// Check if a rule applies to the request and location
    async fn rule_applies_to_request(
        &self,
        rule: &PutawayRule,
        request: &PutawayRequest,
        location: &StorageLocation,
    ) -> Result<i32, AppError> {
        use inventory_service_core::models::PutawayRuleType;

        let applies = match rule.rule_type {
            PutawayRuleType::Product => {
                if let Some(product_id) = rule.product_id {
                    product_id == request.product_id
                } else {
                    false
                }
            },
            PutawayRuleType::Category => {
                if let Some(rule_cat_id) = rule.product_category_id {
                    if let Some(req_cat_id) = request.product_category_id {
                        rule_cat_id == req_cat_id
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            PutawayRuleType::Attribute => {
                if let Some(conditions) = &rule.conditions {
                    if let Some(attrs) = &request.attributes {
                        // Check if conditions (JSON object) is a subset of attrs
                        if let (Some(cond_obj), Some(attrs_obj)) =
                            (conditions.as_object(), attrs.as_object())
                        {
                            cond_obj
                                .iter()
                                .all(|(key, cond_val)| attrs_obj.get(key) == Some(cond_val))
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            PutawayRuleType::Fifo | PutawayRuleType::Fefo => {
                // FIFO/FEFO rules apply to all products
                true
            },
        };

        if !applies {
            return Ok(0);
        }

        // Check warehouse match
        if let Some(rule_warehouse) = rule.warehouse_id {
            if rule_warehouse != location.warehouse_id {
                return Ok(0);
            }
        }

        Ok(1) // Rule applies
    }

    /// Score location preferences for a rule
    fn score_location_preferences(
        &self,
        rule: &PutawayRule,
        location: &StorageLocation,
    ) -> Result<i32, AppError> {
        let mut score = 0i32;

        // Apply location preferences
        if let Some(pref_type) = &rule.preferred_location_type {
            if location.location_type == *pref_type {
                score += rule.priority_score;
            }
        }

        if let Some(pref_zone) = &rule.preferred_zone {
            if let Some(loc_zone) = &location.zone {
                if self.matches_pattern(pref_zone, loc_zone, &rule.match_mode)? {
                    score += rule.priority_score;
                }
            }
        }

        if let Some(pref_aisle) = &rule.preferred_aisle {
            if let Some(loc_aisle) = &location.aisle {
                if self.matches_pattern(pref_aisle, loc_aisle, &rule.match_mode)? {
                    score += rule.priority_score;
                }
            }
        }

        Ok(score)
    }

    /// Validate rule quantities and capacity
    fn validate_rule_quantities(
        &self,
        rule: &PutawayRule,
        request: &PutawayRequest,
        location: &StorageLocation,
    ) -> Result<i32, AppError> {
        // Check quantity constraints
        if let Some(min_qty) = rule.min_quantity {
            if request.quantity < min_qty {
                return Ok(0);
            }
        }

        if let Some(max_qty) = rule.max_quantity {
            if request.quantity > max_qty {
                return Ok(0);
            }
        }

        // Check location capacity
        if let Some(capacity) = location.capacity {
            if location.current_stock + request.quantity > capacity {
                return Ok(0); // Cannot fit
            }
        }

        Ok(rule.priority_score) // Rule matches, add priority score
    }

    /// Check if a value matches a pattern based on match mode
    fn matches_pattern(
        &self,
        pattern: &str,
        value: &str,
        match_mode: &inventory_service_core::models::PutawayMatchMode,
    ) -> Result<bool, AppError> {
        use inventory_service_core::models::PutawayMatchMode;

        match match_mode {
            PutawayMatchMode::Exact => Ok(pattern == value),
            PutawayMatchMode::Contains => Ok(value.contains(pattern)),
            PutawayMatchMode::Regex => {
                let re = Regex::new(pattern).map_err(|e| {
                    AppError::ValidationError(format!("Invalid regex pattern: {}", e))
                })?;
                Ok(re.is_match(value))
            },
        }
    }
}
