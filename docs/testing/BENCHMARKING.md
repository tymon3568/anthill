# Performance Benchmarking Guide

## Overview

This guide covers performance benchmarking for the Anthill project using Criterion.rs. Benchmarks help identify performance bottlenecks and track performance regressions over time.

## Quick Start

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --workspace

# Run specific package benchmarks
cargo bench --package user_service_core

# Run specific benchmark
cargo bench --package user_service_core --bench password_benchmarks

# Quick run with fewer samples (for testing)
cargo bench --package user_service_core --bench password_benchmarks -- --sample-size 10 --warm-up-time 1
```

### Benchmark Output

Criterion generates:
- **Console output**: Timing results with statistics
- **HTML reports**: `target/criterion/` directory
- **Plots**: Performance graphs and comparisons
- **Baseline comparisons**: Track performance over time

---

## Available Benchmarks

### 1. Password Benchmarks

**File**: `services/user_service/core/benches/password_benchmarks.rs`

**Benchmarks**:

| Group | Test | Description |
|-------|------|-------------|
| `password_validation` | `validate_strength/*` | Password validation for different lengths (short, medium, long, very_long) |
| `password_with_user_info` | `validate_with_context/*` | Validation with user context (no_user_info, with_email, with_username, with_both) |
| `password_strength_detection` | `strength_level/*` | Strength detection (weak, medium, strong, very_strong) |
| `common_patterns` | `pattern_detection/*` | Pattern detection (sequential, repeated, keyboard, dictionary, mixed) |

**Run**:
```bash
cargo bench --package user_service_core --bench password_benchmarks
```

**Expected Performance** (approximate):
- Short password validation: ~30-50 µs
- Long password validation: ~100-150 µs
- With user info: +10-20 µs overhead
- Pattern detection: ~50-100 µs

---

### 2. Database Operation Benchmarks

**File**: `services/user_service/core/benches/database_benchmarks.rs`

**Benchmarks**:

| Group | Test | Description |
|-------|------|-------------|
| `uuid_generation` | `uuid_v7_*` | UUID v7 generation (single, batch 10/100/1000) |
| `uuid_operations` | `uuid_*` | UUID operations (to_string, comparison, sorting) |
| `string_operations` | `*` | String ops (email validation, slug generation, concatenation) |
| `collection_operations` | `vec_*` | Collection allocations and operations |
| `timestamp_operations` | `timestamp_*` | Timestamp creation and arithmetic |

**Run**:
```bash
cargo bench --package user_service_core --bench database_benchmarks
```

**Expected Performance** (approximate):
- UUID v7 generation: ~200-500 ns
- UUID to string: ~100-200 ns
- String concat small: ~50-100 ns
- Vec allocation: ~10-50 ns
- Timestamp creation: ~50-100 ns

---

### 3. Inventory Benchmarks

**File**: `services/inventory_service/core/benches/inventory_benchmarks.rs`

**Benchmarks**:

| Group | Test | Description |
|-------|------|-------------|
| `uuid_generation` | `uuid_v4`, `uuid_v7`, `uuid_v7_batch/*` | UUID generation (single, batch 10/100/1000) |
| `category_path` | `parse_path_ids/*`, `path_starts_with` | Category path parsing (depth 1/3/5/10) |
| `category_tree_building` | `simple_*`, `optimized_*` | Tree building algorithms (various sizes) |
| `tree_traversal` | `count_descendants` | Tree traversal operations |
| `string_operations` | `sku_format_*`, `uuid_to_string`, etc. | SKU and string operations |
| `collection_operations` | `vec_*`, `hashmap_*` | Collection allocations and lookups |
| `valuation_calculations` | `fifo_*`, `avco_*` | Inventory valuation (FIFO, AVCO) |

**Run**:
```bash
cargo bench --package inventory_service_core --bench inventory_benchmarks
```

**Expected Performance** (approximate):
- UUID v7 generation: ~200-500 ns
- Category path parsing (depth 5): ~1-2 µs
- Tree building (781 nodes, optimized): ~50-100 µs
- FIFO total value: ~40 ns
- AVCO calculation: ~500 ps
- HashMap lookup: ~17 ns
- Vec with capacity: ~20-70 ns

---

### 4. k6 Load Tests (Inventory Service)

**Location**: `services/inventory_service/load_tests/`

**Scripts**:

| Script | Target | Description |
|--------|--------|-------------|
| `product_search.js` | 500 req/s | Product search endpoint load test |
| `stock_moves.js` | 100 req/s | Stock operations (reserve, adjust, query) |
| `mixed_workload.js` | Mixed | 70% reads / 30% writes simulation |

**Run**:
```bash
# Smoke test
k6 run --env BASE_URL=http://localhost:8082 services/inventory_service/load_tests/product_search.js

# Load test
k6 run --env STAGE_TYPE=load services/inventory_service/load_tests/mixed_workload.js
```

**Thresholds**:
- Product search: p95 < 200ms, error rate < 1%
- Stock moves: p95 < 500ms, error rate < 0.5%

## Writing Benchmarks

### Basic Structure

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // Function to benchmark
            black_box(my_function(black_box(input)))
        });
    });
}

criterion_group!(benches, bench_my_function);
criterion_main!(benches);
```

### Benchmark Groups

For testing multiple scenarios:

```rust
use criterion::{BenchmarkId, Criterion};

fn bench_with_different_inputs(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_group");

    for size in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("operation", size),
            &size,
            |b, &s| {
                b.iter(|| {
                    my_function(black_box(s))
                });
            },
        );
    }

    group.finish();
}
```

### Using `black_box`

**Always use `black_box`** to prevent compiler optimizations from eliminating code:

```rust
// ❌ BAD: May be optimized away
b.iter(|| {
    expensive_function()
});

// ✅ GOOD: Prevents optimization
b.iter(|| {
    black_box(expensive_function(black_box(input)))
});
```

---

## Benchmark Configuration

### Cargo.toml Setup

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "my_benchmarks"
harness = false  # Required for Criterion
```

### Custom Configuration

```rust
use criterion::{Criterion, criterion_group, criterion_main};

fn custom_criterion() -> Criterion {
    Criterion::default()
        .sample_size(100)           // Number of samples
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(5))
        .confidence_level(0.95)
}

criterion_group! {
    name = benches;
    config = custom_criterion();
    targets = bench_my_function
}

criterion_main!(benches);
```

---

## Analyzing Results

### Console Output

```
my_function             time:   [123.45 µs 125.67 µs 128.90 µs]
                        change: [-5.23% -2.11% +1.45%] (p = 0.12 > 0.05)
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
```

**Interpretation**:
- **time**: Median time with confidence interval
- **change**: Comparison to baseline (if exists)
- **p-value**: Statistical significance (< 0.05 = significant)
- **outliers**: Measurements outside normal distribution

### HTML Reports

Open `target/criterion/<benchmark_name>/report/index.html` for:
- Interactive plots
- Distribution graphs
- Comparison charts
- Historical trends

---

## Best Practices

### 1. Isolate Benchmarked Code

```rust
// ❌ BAD: Includes setup time
b.iter(|| {
    let data = prepare_data();
    process(data)
});

// ✅ GOOD: Setup outside benchmark
let data = prepare_data();
b.iter(|| {
    process(black_box(&data))
});
```

### 2. Use Realistic Data

```rust
// ❌ BAD: Trivial input
b.iter(|| validate_password(black_box("abc")));

// ✅ GOOD: Realistic password
b.iter(|| validate_password(black_box("MyP@ssw0rd123")));
```

### 3. Benchmark Hot Paths Only

Focus on code that runs frequently or is performance-critical:
- ✅ Password hashing/validation
- ✅ Database query building
- ✅ UUID generation
- ✅ JWT encoding/decoding
- ❌ Error message formatting
- ❌ Logging statements

### 4. Compare Alternatives

```rust
fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid_generation");

    group.bench_function("uuid_v4", |b| {
        b.iter(|| Uuid::new_v4());
    });

    group.bench_function("uuid_v7", |b| {
        b.iter(|| Uuid::now_v7());
    });

    group.finish();
}
```

### 5. Set Baseline for Tracking

```bash
# Save current performance as baseline
cargo bench --package user_service_core --bench password_benchmarks -- --save-baseline main

# Compare against baseline later
cargo bench --package user_service_core --bench password_benchmarks -- --baseline main
```

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Performance Benchmarks

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  benchmark:
    name: Run Benchmarks
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: |
          cargo bench --workspace --no-run
          cargo bench --workspace -- --sample-size 10

      - name: Upload benchmark results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: target/criterion/
```

### Performance Regression Detection

```bash
# In CI: Compare to main branch
git fetch origin main
git checkout origin/main
cargo bench --bench password_benchmarks -- --save-baseline main

git checkout -
cargo bench --bench password_benchmarks -- --baseline main
```

If performance degrades > 10%, fail the build.

---

## Troubleshooting

### Benchmark Takes Too Long

```bash
# Reduce sample size and warm-up time
cargo bench -- --sample-size 10 --warm-up-time 1 --measurement-time 2
```

### Inconsistent Results

**Causes**:
- Background processes (close unnecessary apps)
- CPU frequency scaling (disable Turbo Boost)
- Thermal throttling (ensure cooling)

**Solutions**:
```bash
# Linux: Disable CPU frequency scaling
sudo cpupower frequency-set --governor performance

# Run multiple times to verify
cargo bench --bench my_benchmark
cargo bench --bench my_benchmark
cargo bench --bench my_benchmark
```

### High Variance

**Symptoms**: Large confidence intervals, many outliers

**Solutions**:
- Increase sample size: `--sample-size 200`
- Increase measurement time: `--measurement-time 10`
- Reduce system load
- Use dedicated benchmark machine

---

## Performance Targets

### User Service Core

| Operation | Target | Current | Status |
|-----------|--------|---------|--------|
| Password validation (short) | < 50 µs | ~35-40 µs | ✅ |
| Password validation (long) | < 200 µs | ~130-160 µs | ✅ |
| UUID v7 generation | < 1 µs | ~200-500 ns | ✅ |
| String concatenation | < 100 ns | ~50-100 ns | ✅ |
| Vec allocation (100 items) | < 50 ns | ~10-20 ns | ✅ |

### Adding New Targets

When adding new benchmarks, establish baseline:

1. Run benchmark 10 times:
   ```bash
   for i in {1..10}; do cargo bench --bench new_benchmark; done
   ```

2. Calculate median time

3. Set target as median + 20% buffer

4. Document in table above

---

## Examples

### Example 1: Database Query Builder

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_query_builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_builder");

    group.bench_function("simple_select", |b| {
        b.iter(|| {
            format!(
                "SELECT * FROM users WHERE tenant_id = '{}' AND email = '{}'",
                black_box("tenant-123"),
                black_box("user@example.com")
            )
        });
    });

    group.bench_function("complex_join", |b| {
        b.iter(|| {
            format!(
                "SELECT u.*, t.name FROM users u JOIN tenants t ON u.tenant_id = t.tenant_id WHERE u.user_id = '{}'",
                black_box("user-456")
            )
        });
    });

    group.finish();
}

criterion_group!(benches, bench_query_builder);
criterion_main!(benches);
```

### Example 2: JSON Serialization

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json;

#[derive(serde::Serialize)]
struct User {
    id: String,
    email: String,
    name: String,
}

fn bench_serialization(c: &mut Criterion) {
    let user = User {
        id: "user-123".to_string(),
        email: "user@example.com".to_string(),
        name: "Test User".to_string(),
    };

    c.bench_function("json_serialize", |b| {
        b.iter(|| {
            serde_json::to_string(black_box(&user))
        });
    });
}

criterion_group!(benches, bench_serialization);
criterion_main!(benches);
```

---

## Related Documentation

- **[Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)**
- **[CI/CD Pipeline Guide](./CI_CD_PIPELINE.md)** - Running benchmarks in CI
- **[Testing Guide](../TESTING_GUIDE.md)** - Overall testing strategy

---

## Maintenance

### Weekly Tasks

- [ ] Review benchmark results for regressions
- [ ] Update baselines if intentional changes made
- [ ] Check for outliers in reports

### Monthly Tasks

- [ ] Add benchmarks for new hot paths
- [ ] Remove benchmarks for deprecated code
- [ ] Update performance targets table
- [ ] Archive old benchmark data

### Quarterly Tasks

- [ ] Full benchmark suite run on dedicated hardware
- [ ] Compare performance trends over 3 months
- [ ] Update performance optimization roadmap

---

## Support

- **Issues**: https://github.com/tymon3568/anthill/issues
- **Criterion Docs**: https://docs.rs/criterion/
- **Performance Tips**: https://doc.rust-lang.org/book/ch12-06-writing-to-stderr-instead-of-stdout.html
