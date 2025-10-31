# User Service Core Tests

## Overview

Unit tests for business logic without external dependencies.

## Test Utilities

- `test_utils.rs` - Builders for test data (UserBuilder, TenantBuilder)
- `mocks.rs` - Mock repositories (MockUserRepo, MockTenantRepo, MockSessionRepo)

## Usage Example

```rust
use user_service_core::tests::{UserBuilder, MockUserRepo};
use mockall::predicate::*;

#[tokio::test]
async fn test_example() {
    // Create test data
    let user = UserBuilder::new()
        .with_email("test@example.com")
        .with_role("admin")
        .build();
    
    // Use mocks
    let mut mock_repo = MockUserRepo::new();
    mock_repo
        .expect_find_by_email()
        .with(eq("test@example.com"), any())
        .returning(move |_, _| Ok(Some(user.clone())));
}
```

## Running Tests

```bash
# All core tests
cargo test --package user_service_core

# With output
cargo test --package user_service_core -- --nocapture
```
