//! Benchmark comparing different Hamming distance implementations.
//!
//! Compares:
//! 1. `metric::simd::hamming_distance` - Our compile-time dispatcher
//! 2. `simd::popcount::simd_popcount_xor` - Upstream's runtime dispatcher
//! 3. Scalar baseline
//!
//! Run with: `cargo bench --bench simd_comparison`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Generate a random byte vector of given length.
fn generate_byte_vector(bytes: usize, seed: u64) -> Vec<u8> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..bytes).map(|_| rng.gen()).collect()
}

/// Scalar hamming distance (baseline).
fn scalar_hamming(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}

fn bench_hamming_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("hamming_comparison");

    // Common embedding dimensions → byte counts
    // 128D → 16B, 384D → 48B, 768D → 96B, 1024D → 128B, 1536D → 192B
    for bytes in [16, 48, 96, 128, 192] {
        let a = generate_byte_vector(bytes, 42);
        let b = generate_byte_vector(bytes, 43);

        group.throughput(Throughput::Bytes((bytes * 2) as u64));

        // 1. Our compile-time dispatcher (uses WASM SIMD128 or AVX2)
        group.bench_with_input(
            BenchmarkId::new("metric_simd", bytes),
            &bytes,
            |bench, _| {
                bench.iter(|| edgevec::metric::simd::hamming_distance(black_box(&a), black_box(&b)))
            },
        );

        // 2. Upstream's runtime dispatcher (uses AVX2/popcnt/NEON/scalar)
        group.bench_with_input(
            BenchmarkId::new("popcount_simd", bytes),
            &bytes,
            |bench, _| {
                bench.iter(|| {
                    edgevec::simd::popcount::simd_popcount_xor(black_box(&a), black_box(&b))
                })
            },
        );

        // 3. Scalar baseline
        group.bench_with_input(BenchmarkId::new("scalar", bytes), &bytes, |bench, _| {
            bench.iter(|| scalar_hamming(black_box(&a), black_box(&b)))
        });
    }

    group.finish();
}

fn bench_768d_focused(c: &mut Criterion) {
    let mut group = c.benchmark_group("hamming_768d");
    group.sample_size(200);

    // 768D = 96 bytes (most common embedding dimension)
    let a = generate_byte_vector(96, 42);
    let b = generate_byte_vector(96, 43);

    group.throughput(Throughput::Bytes(192));

    group.bench_function("metric_simd", |bench| {
        bench.iter(|| edgevec::metric::simd::hamming_distance(black_box(&a), black_box(&b)))
    });

    group.bench_function("popcount_simd", |bench| {
        bench.iter(|| edgevec::simd::popcount::simd_popcount_xor(black_box(&a), black_box(&b)))
    });

    group.bench_function("scalar", |bench| {
        bench.iter(|| scalar_hamming(black_box(&a), black_box(&b)))
    });

    group.finish();
}

criterion_group!(benches, bench_hamming_comparison, bench_768d_focused);
criterion_main!(benches);
