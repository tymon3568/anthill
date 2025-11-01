// Performance benchmarks for password operations
// Run: cargo bench --package user_service_core --bench password_benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use user_service_core::domains::auth::utils::password_validator::{
    validate_password_strength, PasswordStrength,
};

fn bench_password_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_validation");

    // Test different password lengths
    let passwords = vec![
        ("short", "Pass1!"),
        ("medium", "MyP@ssw0rd123"),
        ("long", "ThisIsAVeryLongAndSecureP@ssw0rd123456"),
        ("very_long", "ThisIsAnExtremelyLongPasswordThatShouldBenchmarkPerformanceUnderStress123!@#"),
    ];

    for (name, password) in passwords {
        group.bench_with_input(
            BenchmarkId::new("validate_strength", name),
            &password,
            |b, pwd| {
                b.iter(|| {
                    validate_password_strength(black_box(pwd), &[])
                });
            },
        );
    }

    group.finish();
}

fn bench_password_with_user_info(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_with_user_info");

    let test_cases = vec![
        ("no_user_info", "SecureP@ssw0rd123", vec![]),
        ("with_email", "johndoePass123!", vec!["john.doe@example.com"]),
        ("with_username", "P@ssw0rd123", vec!["johndoe"]),
        ("with_both", "MySecureP@ss123", vec!["john.doe@example.com", "johndoe"]),
    ];

    for (name, password, user_inputs) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("validate_with_context", name),
            &(password, user_inputs),
            |b, (pwd, inputs)| {
                let inputs_refs: Vec<&str> = inputs.iter().map(|s| s.as_ref()).collect();
                b.iter(|| {
                    validate_password_strength(black_box(pwd), &inputs_refs)
                });
            },
        );
    }

    group.finish();
}

fn bench_password_strength_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_strength_detection");

    let passwords = vec![
        ("weak", "pass123"),
        ("medium", "MyP@ssw0rd"),
        ("strong", "MyP@ssw0rd123!"),
        ("very_strong", "Th1s!sAV3ryStr0ngP@ssw0rd"),
    ];

    for (strength, password) in passwords {
        group.bench_with_input(
            BenchmarkId::new("strength_level", strength),
            &password,
            |b, pwd| {
                b.iter(|| {
                    validate_password_strength(black_box(pwd), &[])
                });
            },
        );
    }

    group.finish();
}

fn bench_common_password_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("common_patterns");

    let passwords = vec![
        ("sequential", "abc123def456"),
        ("repeated", "aaaaaa1111"),
        ("keyboard", "qwerty123"),
        ("dictionary", "password123"),
        ("mixed", "MyP@ssw0rd123!"),
    ];

    for (pattern, password) in passwords {
        group.bench_with_input(
            BenchmarkId::new("pattern_detection", pattern),
            &password,
            |b, pwd| {
                b.iter(|| {
                    validate_password_strength(black_box(pwd), &[])
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_password_validation,
    bench_password_with_user_info,
    bench_password_strength_detection,
    bench_common_password_patterns,
);

criterion_main!(benches);
