//! Performance benchmarks for inventory operations
//!
//! Run: cargo bench --package inventory_service_core --bench inventory_benchmarks

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use uuid::Uuid;

// ============================================================================
// UUID Generation Benchmarks
// ============================================================================

fn bench_uuid_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid_generation");

    group.bench_function("uuid_v4", |b| {
        b.iter(|| Uuid::new_v4());
    });

    group.bench_function("uuid_v7", |b| {
        b.iter(|| Uuid::now_v7());
    });

    // Batch UUID generation
    for count in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("uuid_v7_batch", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let mut ids = Vec::with_capacity(count);
                    for _ in 0..count {
                        ids.push(Uuid::now_v7());
                    }
                    black_box(ids)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Category Path Parsing Benchmarks
// ============================================================================

fn bench_category_path_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("category_path");

    // Test parsing paths of different depths
    let paths = vec![
        ("depth_1", "550e8400-e29b-41d4-a716-446655440001"),
        (
            "depth_3",
            "550e8400-e29b-41d4-a716-446655440001/550e8400-e29b-41d4-a716-446655440002/550e8400-e29b-41d4-a716-446655440003",
        ),
        (
            "depth_5",
            "550e8400-e29b-41d4-a716-446655440001/550e8400-e29b-41d4-a716-446655440002/550e8400-e29b-41d4-a716-446655440003/550e8400-e29b-41d4-a716-446655440004/550e8400-e29b-41d4-a716-446655440005",
        ),
        (
            "depth_10",
            "550e8400-e29b-41d4-a716-446655440001/550e8400-e29b-41d4-a716-446655440002/550e8400-e29b-41d4-a716-446655440003/550e8400-e29b-41d4-a716-446655440004/550e8400-e29b-41d4-a716-446655440005/550e8400-e29b-41d4-a716-446655440006/550e8400-e29b-41d4-a716-446655440007/550e8400-e29b-41d4-a716-446655440008/550e8400-e29b-41d4-a716-446655440009/550e8400-e29b-41d4-a716-44665544000a",
        ),
    ];

    for (name, path) in paths {
        group.bench_with_input(
            BenchmarkId::new("parse_path_ids", name),
            &path,
            |b, path| {
                b.iter(|| {
                    let ids: Vec<Uuid> = path
                        .split('/')
                        .filter_map(|s| Uuid::parse_str(s).ok())
                        .collect();
                    black_box(ids)
                });
            },
        );
    }

    // Benchmark path prefix matching (is_ancestor_of check)
    let root_path = "550e8400-e29b-41d4-a716-446655440001";
    let child_path = "550e8400-e29b-41d4-a716-446655440001/550e8400-e29b-41d4-a716-446655440002/550e8400-e29b-41d4-a716-446655440003";

    group.bench_function("path_starts_with", |b| {
        b.iter(|| black_box(child_path).starts_with(black_box(root_path)));
    });

    group.finish();
}

// ============================================================================
// Category Tree Building Benchmarks
// ============================================================================

/// A minimal category structure for benchmarking
#[derive(Clone)]
struct BenchCategory {
    id: Uuid,
    parent_id: Option<Uuid>,
    level: i32,
}

/// A minimal tree node for benchmarking
struct BenchCategoryNode {
    category: BenchCategory,
    children: Vec<BenchCategoryNode>,
}

impl BenchCategoryNode {
    fn new(category: BenchCategory) -> Self {
        Self {
            category,
            children: Vec::new(),
        }
    }

    fn count_descendants(&self) -> usize {
        self.children.len()
            + self.children.iter().map(|c| c.count_descendants()).sum::<usize>()
    }
}

/// Generate test categories with specified depth and width
fn generate_categories(depth: usize, children_per_level: usize) -> Vec<BenchCategory> {
    let mut categories = Vec::new();
    let root = BenchCategory {
        id: Uuid::now_v7(),
        parent_id: None,
        level: 0,
    };
    categories.push(root.clone());

    let mut current_level = vec![root];

    for level in 1..depth {
        let mut next_level = Vec::new();
        for parent in &current_level {
            for _ in 0..children_per_level {
                let cat = BenchCategory {
                    id: Uuid::now_v7(),
                    parent_id: Some(parent.id),
                    level: level as i32,
                };
                categories.push(cat.clone());
                next_level.push(cat);
            }
        }
        current_level = next_level;
    }

    categories
}

/// Build tree from flat category list (simple O(n²) algorithm)
fn build_tree_simple(categories: &[BenchCategory]) -> Vec<BenchCategoryNode> {
    let roots: Vec<_> = categories
        .iter()
        .filter(|c| c.parent_id.is_none())
        .cloned()
        .collect();

    fn add_children(node: &mut BenchCategoryNode, all_categories: &[BenchCategory]) {
        let children: Vec<_> = all_categories
            .iter()
            .filter(|c| c.parent_id == Some(node.category.id))
            .cloned()
            .collect();

        for child_cat in children {
            let mut child_node = BenchCategoryNode::new(child_cat);
            add_children(&mut child_node, all_categories);
            node.children.push(child_node);
        }
    }

    let mut tree = Vec::new();
    for root in roots {
        let mut node = BenchCategoryNode::new(root);
        add_children(&mut node, categories);
        tree.push(node);
    }
    tree
}

/// Build tree from flat category list using HashMap (optimized O(n) algorithm)
fn build_tree_optimized(categories: &[BenchCategory]) -> Vec<BenchCategoryNode> {
    use std::collections::HashMap;

    // Create lookup map
    let mut nodes: HashMap<Uuid, BenchCategoryNode> = categories
        .iter()
        .map(|c| (c.id, BenchCategoryNode::new(c.clone())))
        .collect();

    // Collect parent-child relationships
    let relationships: Vec<(Uuid, Uuid)> = categories
        .iter()
        .filter_map(|c| c.parent_id.map(|p| (p, c.id)))
        .collect();

    // Build tree from bottom up
    // First, collect all root IDs
    let root_ids: Vec<Uuid> = categories
        .iter()
        .filter(|c| c.parent_id.is_none())
        .map(|c| c.id)
        .collect();

    // Group children by parent
    let mut children_by_parent: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for (parent_id, child_id) in relationships {
        children_by_parent
            .entry(parent_id)
            .or_default()
            .push(child_id);
    }

    // Recursive function to build subtree
    fn build_subtree(
        node_id: Uuid,
        nodes: &mut HashMap<Uuid, BenchCategoryNode>,
        children_by_parent: &HashMap<Uuid, Vec<Uuid>>,
    ) -> BenchCategoryNode {
        let mut node = nodes.remove(&node_id).unwrap();
        if let Some(child_ids) = children_by_parent.get(&node_id) {
            for child_id in child_ids {
                let child = build_subtree(*child_id, nodes, children_by_parent);
                node.children.push(child);
            }
        }
        node
    }

    root_ids
        .into_iter()
        .map(|id| build_subtree(id, &mut nodes, &children_by_parent))
        .collect()
}

fn bench_tree_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("category_tree_building");

    // Test different tree configurations: (depth, children_per_level)
    let configs = vec![
        ("small_3x3", 3, 3),     // 13 nodes
        ("medium_4x4", 4, 4),    // 85 nodes
        ("deep_10x2", 10, 2),    // 1023 nodes
        ("wide_3x10", 3, 10),    // 111 nodes
        ("large_5x5", 5, 5),     // 781 nodes
    ];

    for (name, depth, width) in &configs {
        let categories = generate_categories(*depth, *width);
        let cat_count = categories.len();

        group.bench_with_input(
            BenchmarkId::new(format!("simple_{}", name), cat_count),
            &categories,
            |b, cats| {
                b.iter(|| {
                    let tree = build_tree_simple(black_box(cats));
                    black_box(tree)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new(format!("optimized_{}", name), cat_count),
            &categories,
            |b, cats| {
                b.iter(|| {
                    let tree = build_tree_optimized(black_box(cats));
                    black_box(tree)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Tree Traversal Benchmarks
// ============================================================================

fn bench_tree_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_traversal");

    // Build a test tree
    let categories = generate_categories(5, 5); // 781 nodes
    let tree = build_tree_optimized(&categories);

    if let Some(root) = tree.first() {
        group.bench_function("count_descendants", |b| {
            b.iter(|| black_box(root.count_descendants()));
        });
    }

    group.finish();
}

// ============================================================================
// String Operations Benchmarks (for SKU/Code generation)
// ============================================================================

fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // SKU generation patterns
    group.bench_function("sku_format_simple", |b| {
        let prefix = "PRD";
        let id = 12345;
        b.iter(|| format!("{}-{:06}", black_box(prefix), black_box(id)));
    });

    group.bench_function("sku_format_with_uuid", |b| {
        let uuid = Uuid::now_v7();
        b.iter(|| {
            let uuid_str = uuid.to_string();
            format!("SKU-{}", &uuid_str[..8])
        });
    });

    // UUID to string conversions
    let uuid = Uuid::now_v7();
    group.bench_function("uuid_to_string", |b| {
        b.iter(|| black_box(&uuid).to_string());
    });

    group.bench_function("uuid_to_hyphenated", |b| {
        b.iter(|| black_box(&uuid).as_hyphenated().to_string());
    });

    // String comparisons (for search)
    let sku1 = "PRD-000123-VARIANT-A";
    let sku2 = "PRD-000123-VARIANT-B";

    group.bench_function("string_eq", |b| {
        b.iter(|| black_box(sku1) == black_box(sku2));
    });

    let search_term = "000123";
    group.bench_function("string_contains", |b| {
        b.iter(|| black_box(sku1).contains(black_box(search_term)));
    });

    group.bench_function("string_starts_with", |b| {
        b.iter(|| black_box(sku1).starts_with("PRD"));
    });

    group.finish();
}

// ============================================================================
// Collection Operations Benchmarks
// ============================================================================

fn bench_collection_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("collection_operations");

    // Vec allocation for different sizes
    for size in [10, 100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("vec_with_capacity", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let v: Vec<Uuid> = Vec::with_capacity(size);
                    black_box(v)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("vec_fill_uuids", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut v = Vec::with_capacity(size);
                    for _ in 0..size {
                        v.push(Uuid::now_v7());
                    }
                    black_box(v)
                });
            },
        );
    }

    // HashMap operations (for lookups)
    use std::collections::HashMap;

    let mut map: HashMap<Uuid, String> = HashMap::with_capacity(1000);
    for _ in 0..1000 {
        let id = Uuid::now_v7();
        map.insert(id, format!("Product-{}", id));
    }

    let lookup_id = *map.keys().next().unwrap();

    group.bench_function("hashmap_lookup", |b| {
        b.iter(|| map.get(black_box(&lookup_id)));
    });

    group.bench_function("hashmap_insert", |b| {
        b.iter_batched(
            || (Uuid::now_v7(), "New Product".to_string()),
            |(id, name)| {
                let mut m = map.clone();
                m.insert(id, name);
                black_box(m)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// ============================================================================
// Valuation Calculation Benchmarks
// ============================================================================

fn bench_valuation_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("valuation_calculations");

    // FIFO cost layer simulation
    #[derive(Clone)]
    struct CostLayer {
        quantity: i64,
        unit_cost_cents: i64,
    }

    // Generate test cost layers
    let layers: Vec<CostLayer> = (0..100)
        .map(|i| CostLayer {
            quantity: 10 + (i % 20),
            unit_cost_cents: 1000 + (i * 10),
        })
        .collect();

    group.bench_function("fifo_total_value", |b| {
        b.iter(|| {
            let total: i64 = layers
                .iter()
                .map(|l| l.quantity * l.unit_cost_cents)
                .sum();
            black_box(total)
        });
    });

    group.bench_function("fifo_consume_quantity", |b| {
        let quantity_to_consume = 50i64;

        b.iter_batched(
            || layers.clone(),
            |mut layers_copy| {
                let mut remaining = quantity_to_consume;
                let mut total_cost = 0i64;

                for layer in layers_copy.iter_mut() {
                    if remaining <= 0 {
                        break;
                    }
                    let take = remaining.min(layer.quantity);
                    total_cost += take * layer.unit_cost_cents;
                    layer.quantity -= take;
                    remaining -= take;
                }

                black_box(total_cost)
            },
            BatchSize::SmallInput,
        );
    });

    // AVCO (Average Cost) calculation - includes sum calculation for accurate benchmarking
    group.bench_function("avco_calculate", |b| {
        b.iter(|| {
            let total_quantity: i64 = layers.iter().map(|l| l.quantity).sum();
            let total_value: i64 = layers.iter().map(|l| l.quantity * l.unit_cost_cents).sum();
            let avg_cost = if total_quantity > 0 {
                total_value / total_quantity
            } else {
                0
            };
            black_box(avg_cost)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_uuid_generation,
    bench_category_path_operations,
    bench_tree_building,
    bench_tree_traversal,
    bench_string_operations,
    bench_collection_operations,
    bench_valuation_calculations,
);

criterion_main!(benches);
