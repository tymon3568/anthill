// Performance benchmarks for core operations
// Run: cargo bench --package user_service_core --bench database_benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use uuid::Uuid;

fn bench_uuid_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid_generation");

    group.bench_function("uuid_v7_single", |b| {
        b.iter(|| {
            Uuid::now_v7()
        });
    });

    group.bench_function("uuid_v7_batch_10", |b| {
        b.iter(|| {
            let mut uuids = Vec::with_capacity(10);
            for _ in 0..10 {
                uuids.push(Uuid::now_v7());
            }
            uuids
        });
    });

    group.bench_function("uuid_v7_batch_100", |b| {
        b.iter(|| {
            let mut uuids = Vec::with_capacity(100);
            for _ in 0..100 {
                uuids.push(Uuid::now_v7());
            }
            uuids
        });
    });

    group.bench_function("uuid_v7_batch_1000", |b| {
        b.iter(|| {
            let mut uuids = Vec::with_capacity(1000);
            for _ in 0..1000 {
                uuids.push(Uuid::now_v7());
            }
            uuids
        });
    });

    group.finish();
}

fn bench_uuid_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid_operations");

    let uuid = Uuid::now_v7();

    group.bench_function("uuid_to_string", |b| {
        b.iter(|| {
            black_box(uuid).to_string()
        });
    });

    group.bench_function("uuid_comparison", |b| {
        let uuid2 = Uuid::now_v7();
        b.iter(|| {
            black_box(uuid) == black_box(uuid2)
        });
    });

    group.bench_function("uuid_sorting_100", |b| {
        b.iter(|| {
            let mut uuids: Vec<Uuid> = (0..100).map(|_| Uuid::now_v7()).collect();
            uuids.sort();
            uuids
        });
    });

    group.finish();
}

fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    group.bench_function("email_validation", |b| {
        let email = "user@example.com";
        b.iter(|| {
            black_box(email).contains('@') && black_box(email).contains('.')
        });
    });

    group.bench_function("slug_generation", |b| {
        let name = "Test Tenant Corporation";
        b.iter(|| {
            black_box(name)
                .to_lowercase()
                .replace(' ', "-")
        });
    });

    group.bench_function("string_concat_small", |b| {
        b.iter(|| {
            format!("user-{}-{}", black_box("test"), black_box(123))
        });
    });

    group.bench_function("string_concat_large", |b| {
        b.iter(|| {
            format!(
                "tenant-{}-user-{}-session-{}-{}",
                black_box(Uuid::now_v7()),
                black_box(Uuid::now_v7()),
                black_box(Uuid::now_v7()),
                black_box(chrono::Utc::now())
            )
        });
    });

    group.finish();
}

fn bench_collection_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("collection_operations");

    group.bench_function("vec_allocation_small", |b| {
        b.iter(|| {
            let v: Vec<Uuid> = Vec::with_capacity(10);
            v
        });
    });

    group.bench_function("vec_allocation_large", |b| {
        b.iter(|| {
            let v: Vec<Uuid> = Vec::with_capacity(1000);
            v
        });
    });

    group.bench_function("vec_push_10", |b| {
        b.iter(|| {
            let mut v = Vec::new();
            for _ in 0..10 {
                v.push(Uuid::now_v7());
            }
            v
        });
    });

    group.bench_function("vec_with_capacity_push_10", |b| {
        b.iter(|| {
            let mut v = Vec::with_capacity(10);
            for _ in 0..10 {
                v.push(Uuid::now_v7());
            }
            v
        });
    });

    group.finish();
}

fn bench_timestamp_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamp_operations");

    group.bench_function("current_timestamp", |b| {
        b.iter(|| {
            chrono::Utc::now()
        });
    });

    group.bench_function("timestamp_to_string", |b| {
        let now = chrono::Utc::now();
        b.iter(|| {
            black_box(now).to_rfc3339()
        });
    });

    group.bench_function("timestamp_arithmetic", |b| {
        let now = chrono::Utc::now();
        b.iter(|| {
            black_box(now) + chrono::Duration::days(7)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_uuid_generation,
    bench_uuid_operations,
    bench_string_operations,
    bench_collection_operations,
    bench_timestamp_operations,
);

criterion_main!(benches);
