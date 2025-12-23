# Week 27 Day 5: Benchmarks + Validation Tests

**Date:** 2025-12-26
**Focus:** Performance validation and comprehensive testing
**Estimated Duration:** 4 hours
**Phase:** RFC-002 Implementation Phase 2 (Binary Quantization)

---

## Tasks

### W27.5.1: Create Benchmark Suite

**Objective:** Validate BQ performance targets from RFC-002.

**Performance Targets:**
| Metric | Target | Measurement |
|:-------|:-------|:------------|
| BQ memory reduction | 32x vs F32 | `768 × 4 bytes` vs `96 bytes` |
| BQ search speedup | 3-5x vs F32 | Latency benchmark |
| BQ+rescore recall | >0.90 @ k=10 | Recall benchmark |
| SIMD vs scalar | >2x speedup | Popcount benchmark |

**Acceptance Criteria:**
- [ ] `benches/bq_quantize.rs` — quantization throughput
- [ ] `benches/bq_hamming.rs` — Hamming distance throughput
- [ ] `benches/bq_search.rs` — BQ vs F32 search latency
- [ ] `benches/bq_recall.rs` — recall@k measurements
- [ ] README with benchmark instructions
- [ ] Benchmark results logged in review document

**Files:**
- `benches/bq_quantize.rs` (new file)
- `benches/bq_hamming.rs` (new file)
- `benches/bq_search.rs` (new file)
- `benches/bq_recall.rs` (new file)

**Estimated Duration:** 2 hours

**Agent:** BENCHMARK_SCIENTIST

**Benchmark Implementations:**

```rust
// benches/bq_quantize.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use edgevec::quantization::variable::BinaryVector;

fn benchmark_quantize(c: &mut Criterion) {
    let mut group = c.benchmark_group("BQ Quantization");

    for dim in [128, 384, 768, 1024, 1536] {
        let vector: Vec<f32> = (0..dim).map(|i| (i as f32).sin()).collect();

        group.bench_with_input(
            BenchmarkId::new("quantize", dim),
            &vector,
            |b, v| b.iter(|| BinaryVector::quantize(v).unwrap()),
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_quantize);
criterion_main!(benches);
```

```rust
// benches/bq_hamming.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use edgevec::quantization::variable::BinaryVector;
use edgevec::simd::popcount::{simd_popcount_xor, scalar_popcount_xor};

fn benchmark_hamming(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hamming Distance");

    for dim in [128, 384, 768, 1024] {
        let v1: Vec<f32> = (0..dim).map(|i| (i as f32).sin()).collect();
        let v2: Vec<f32> = (0..dim).map(|i| (i as f32).cos()).collect();

        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();

        group.bench_with_input(
            BenchmarkId::new("simd_popcount", dim),
            &(&bv1, &bv2),
            |b, (a, b_vec)| b.iter(|| simd_popcount_xor(a.data(), b_vec.data())),
        );

        group.bench_with_input(
            BenchmarkId::new("scalar_popcount", dim),
            &(&bv1, &bv2),
            |b, (a, b_vec)| b.iter(|| scalar_popcount_xor(a.data(), b_vec.data())),
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_hamming);
criterion_main!(benches);
```

```rust
// benches/bq_search.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use edgevec::hnsw::{HnswConfig, HnswIndex};
use edgevec::storage::VectorStorage;
use rand::Rng;

fn benchmark_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("Search Comparison");

    for n in [1_000, 10_000, 100_000] {
        // Create index with n vectors
        let config = HnswConfig::new(768);
        let mut storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::with_bq(config, &storage).unwrap();

        let mut rng = rand::thread_rng();

        for _ in 0..n {
            let v: Vec<f32> = (0..768).map(|_| rng.gen::<f32>()).collect();
            index.insert_bq(&v, &mut storage).unwrap();
        }

        let query: Vec<f32> = (0..768).map(|_| rng.gen::<f32>()).collect();

        group.bench_with_input(
            BenchmarkId::new("f32_search", n),
            &(&index, &storage, &query),
            |b, (idx, stor, q)| b.iter(|| idx.search(q, 10, stor).unwrap()),
        );

        group.bench_with_input(
            BenchmarkId::new("bq_search", n),
            &(&index, &storage, &query),
            |b, (idx, stor, q)| b.iter(|| idx.search_bq(q, 10, stor).unwrap()),
        );

        group.bench_with_input(
            BenchmarkId::new("bq_rescored", n),
            &(&index, &storage, &query),
            |b, (idx, stor, q)| b.iter(|| idx.search_bq_rescored(q, 10, 3, stor).unwrap()),
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
```

**Dependencies:** W27.1-W27.4 (all BQ components)

---

### W27.5.2: Property Tests for BQ Invariants

**Objective:** Ensure BQ operations are correct under all inputs.

**Invariants to Test:**
1. Hamming distance is symmetric: `d(a, b) == d(b, a)`
2. Hamming distance to self is 0: `d(a, a) == 0`
3. Triangle inequality: `d(a, c) <= d(a, b) + d(b, c)`
4. Quantization is deterministic: `q(v) == q(v)`
5. Dimension preserved: `bv.dimension() == v.len()`
6. BQ similarity in [0, 1]: `0.0 <= sim <= 1.0`

**Acceptance Criteria:**
- [ ] Property tests with proptest
- [ ] Generators for random vectors
- [ ] All invariants verified
- [ ] Edge cases: zero vector, all positive, all negative

**Files:**
- `tests/bq_proptests.rs` (new file)

**Estimated Duration:** 2 hours

**Agent:** TEST_ENGINEER

**Property Tests:**

```rust
// tests/bq_proptests.rs

use proptest::prelude::*;
use edgevec::quantization::variable::BinaryVector;

// Generator for random f32 vectors of given dimension
fn random_vector(dim: usize) -> impl Strategy<Value = Vec<f32>> {
    proptest::collection::vec(any::<f32>().prop_filter("finite", |x| x.is_finite()), dim)
}

// Generator for dimension divisible by 8
fn valid_dimension() -> impl Strategy<Value = usize> {
    (1..=192usize).prop_map(|x| x * 8) // 8 to 1536
}

proptest! {
    /// Hamming distance is symmetric.
    #[test]
    fn prop_hamming_symmetric(
        dim in valid_dimension(),
        a in random_vector(768),
        b in random_vector(768),
    ) {
        let dim = 768; // Use fixed dim for now
        let va: Vec<f32> = a.into_iter().take(dim).collect();
        let vb: Vec<f32> = b.into_iter().take(dim).collect();

        let bva = BinaryVector::quantize(&va).unwrap();
        let bvb = BinaryVector::quantize(&vb).unwrap();

        let d_ab = bva.hamming_distance(&bvb).unwrap();
        let d_ba = bvb.hamming_distance(&bva).unwrap();

        prop_assert_eq!(d_ab, d_ba);
    }

    /// Hamming distance to self is zero.
    #[test]
    fn prop_hamming_identity(v in random_vector(768)) {
        let bv = BinaryVector::quantize(&v).unwrap();
        let d = bv.hamming_distance(&bv).unwrap();
        prop_assert_eq!(d, 0);
    }

    /// Triangle inequality holds.
    #[test]
    fn prop_triangle_inequality(
        a in random_vector(768),
        b in random_vector(768),
        c in random_vector(768),
    ) {
        let bva = BinaryVector::quantize(&a).unwrap();
        let bvb = BinaryVector::quantize(&b).unwrap();
        let bvc = BinaryVector::quantize(&c).unwrap();

        let d_ab = bva.hamming_distance(&bvb).unwrap();
        let d_bc = bvb.hamming_distance(&bvc).unwrap();
        let d_ac = bva.hamming_distance(&bvc).unwrap();

        prop_assert!(d_ac <= d_ab + d_bc);
    }

    /// Quantization is deterministic.
    #[test]
    fn prop_quantize_deterministic(v in random_vector(768)) {
        let bv1 = BinaryVector::quantize(&v).unwrap();
        let bv2 = BinaryVector::quantize(&v).unwrap();
        prop_assert_eq!(bv1, bv2);
    }

    /// Dimension is preserved.
    #[test]
    fn prop_dimension_preserved(dim in valid_dimension()) {
        let v: Vec<f32> = (0..dim).map(|i| (i as f32).sin()).collect();
        let bv = BinaryVector::quantize(&v).unwrap();
        prop_assert_eq!(bv.dimension(), dim);
        prop_assert_eq!(bv.bytes(), dim / 8);
    }

    /// Similarity is bounded [0, 1].
    #[test]
    fn prop_similarity_bounded(
        a in random_vector(768),
        b in random_vector(768),
    ) {
        let bva = BinaryVector::quantize(&a).unwrap();
        let bvb = BinaryVector::quantize(&b).unwrap();

        let sim = bva.similarity(&bvb).unwrap();
        prop_assert!(sim >= 0.0);
        prop_assert!(sim <= 1.0);
    }
}

/// Edge case tests (not property-based)
mod edge_cases {
    use super::*;

    #[test]
    fn test_zero_vector() {
        let v = vec![0.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();
        // All zeros -> all bits should be 0 (0.0 > 0.0 is false)
        assert!(bv.data().iter().all(|&b| b == 0x00));
    }

    #[test]
    fn test_all_positive() {
        let v = vec![1.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();
        // All positive -> all bits should be 1
        assert!(bv.data().iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_all_negative() {
        let v = vec![-1.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();
        // All negative -> all bits should be 0
        assert!(bv.data().iter().all(|&b| b == 0x00));
    }

    #[test]
    fn test_alternating() {
        let v: Vec<f32> = (0..768).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        let bv = BinaryVector::quantize(&v).unwrap();
        // Alternating -> 0b01010101 = 0x55
        assert!(bv.data().iter().all(|&b| b == 0x55));
    }

    #[test]
    fn test_nan_treated_as_non_positive() {
        let mut v = vec![1.0f32; 768];
        v[0] = f32::NAN;
        let bv = BinaryVector::quantize(&v).unwrap();
        // NaN > 0.0 is false, so bit 0 should be 0
        assert_eq!(bv.data()[0] & 0x01, 0);
    }

    #[test]
    fn test_infinity() {
        let mut v = vec![0.0f32; 768];
        v[0] = f32::INFINITY;
        v[1] = f32::NEG_INFINITY;
        let bv = BinaryVector::quantize(&v).unwrap();
        // +Inf > 0.0 is true (bit 0 = 1)
        // -Inf > 0.0 is false (bit 1 = 0)
        assert_eq!(bv.data()[0] & 0x01, 1);
        assert_eq!(bv.data()[0] & 0x02, 0);
    }
}
```

**Dependencies:** W27.1.1 (BinaryVector)

---

## Day 5 Checklist

- [ ] W27.5.1: Benchmark suite created
- [ ] W27.5.2: Property tests written and passing
- [ ] BQ search speedup measured (target: 3-5x)
- [ ] BQ+rescore recall measured (target: >0.90)
- [ ] All tests pass (`cargo test`)
- [ ] All benchmarks run (`cargo bench`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Formatted (`cargo fmt --check`)

## Day 5 Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | CI green |
| `cargo bench` runs | Benchmark results |
| Speedup 3-5x | Benchmark report |
| Recall >0.90 | Recall benchmark |
| Property tests pass | proptest |

## Day 5 Handoff

After completing Day 5:

**Artifacts Generated:**
- `benches/bq_quantize.rs`
- `benches/bq_hamming.rs`
- `benches/bq_search.rs`
- `benches/bq_recall.rs`
- `tests/bq_proptests.rs`

**Status:** PENDING_HOSTILE_REVIEW

**Next:** Week 28 — WASM Bindings + Integration

---

## Week 27 Summary

| Day | Focus | Hours | Key Deliverable |
|:----|:------|:------|:----------------|
| 1 | Variable BQ + SIMD | 12 | `BinaryVector`, popcount |
| 2 | BinaryVectorStorage | 10 | Storage layer |
| 3 | HNSW BQ Search | 14 | `search_bq()` |
| 4 | Rescoring | 8 | `search_bq_rescored()` |
| 5 | Benchmarks | 4 | Performance validation |
| **Total** | | **48** | |

**Week 27 Deliverables:**
- Variable-dimension binary quantization
- SIMD-optimized Hamming distance (x86 + ARM)
- BinaryVectorStorage with tombstone support
- BQ search integrated with HNSW
- F32 rescoring for recall recovery
- Comprehensive benchmark suite
- Property tests for correctness

---

*Agent: PLANNER + RUST_ENGINEER + TEST_ENGINEER + BENCHMARK_SCIENTIST*
*Status: [APPROVED] (2025-12-21)*
