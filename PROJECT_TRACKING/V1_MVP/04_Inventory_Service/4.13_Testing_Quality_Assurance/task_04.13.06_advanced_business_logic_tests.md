# Task: Implement Tests for Advanced Business Logic

**Task ID:** V1_MVP/04_Inventory_Service/4.13_Testing_Quality_Assurance/task_04.13.06_advanced_business_logic_tests.md
**Version:** V1_MVP
**Phase:** 04_Inventory_Service
**Module:** 4.13_Testing_Quality_Assurance
**Priority:** Medium
**Status:** NeedsReview
**Assignee:** Agent
**Created Date:** 2025-12-12
**Last Updated:** 2025-12-13

## Detailed Description:
Implement unit tests for advanced inventory business logic including FIFO/AVCO valuation, stock reservation, and reorder rules.

## Specific Sub-tasks:
- [x] 1. FIFO/AVCO Valuation Tests (13 tests implemented)
    - [x] 1.1. FIFO cost calculation on receipts (layer creation)
    - [x] 1.2. FIFO cost calculation on deliveries (layer consumption)
    - [x] 1.3. AVCO recalculation on receipts (weighted average)
    - [x] 1.4. Mixed scenarios (multiple receipts with different costs)
    - [x] 1.5. Edge cases (zero qty, cost adjustments, revaluation)
- [ ] 2. Stock Reservation Tests (BLOCKED - Feature not implemented)
    - [ ] 2.1. Reserve stock for order
    - [ ] 2.2. Release reserved stock
    - [ ] 2.3. Partial reservation
    - [ ] 2.4. Reservation expiry
- [x] 3. Reorder Rules Tests (9 tests implemented)
    - [x] 3.1. Reorder point trigger detection
    - [x] 3.2. Reorder quantity calculation (respects min/max)
    - [x] 3.3. Lead time in rule creation
    - [x] 3.4. Safety stock calculations

## Acceptance Criteria:
- [x] Tests cover all valuation calculation scenarios (FIFO, AVCO, Standard)
- [ ] Reservation logic tested with various edge cases (BLOCKED)
- [x] Reorder triggers tested with realistic scenarios
- [x] All business rules from specifications validated

## Test Files Created:
* `services/inventory_service/api/tests/valuation_business_logic_tests.rs` (13 tests)
* `services/inventory_service/api/tests/reorder_rules_tests.rs` (9 tests)

## Dependencies:
* task_04.06.01_implement_valuation.md (Completed - implementation exists)
* task_04.07.01_implement_automated_replenishment.md (NeedsReview)

## Related Documents:
* Valuation methodology documentation
* Reorder rules specification

## Notes / Discussion:
---
* Stock Reservation tests remain BLOCKED until the feature is implemented
* Total of 22 integration tests added covering valuation and reorder rules business logic
