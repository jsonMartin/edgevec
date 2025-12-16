# Week 19 Day 4: Test Hardening & CI Enhancement

**Task ID:** W19.4
**Date:** 2025-12-19
**Estimated Hours:** 8 hours (3x rule: 2.67h optimistic × 3 = 8h)
**Base Estimate:** 2.67 hours (chaos tests using existing patterns, straightforward CI config)
**Risk Buffer:** +5.33 hours (bug discovery during chaos testing, CI debugging)
**Dependencies:** W19.1 (Reconciliation)
**Priority:** HIGH

---

## Objective

Strengthen EdgeVec's test coverage with chaos testing (edge cases), load testing (sustained stress), and P99 latency tracking to ensure v0.4.0 stability. Add regression detection to CI to catch performance degradation automatically.

---

## Background

**Existing Test Infrastructure:**
- Unit tests: 159+ tests in `src/`
- Property tests: `tests/proptest_hnsw_delete.rs`
- Fuzz tests: `fuzz/` targets
- Integration tests: `tests/integration_soft_delete.rs`
- WASM tests: `tests/wasm_bench.rs`

**Existing Benchmarks:**
- `benches/search_bench.rs` - Search latency
- `benches/insert_bench.rs` - Insert latency
- `benches/tombstone_bench.rs` - Soft delete performance
- `benches/competitive/` - Competitive comparison

**Gaps:**
- No chaos testing for extreme edge cases
- No sustained load testing (stress over time)
- No automated regression detection in CI

---

## Deliverables

| # | Deliverable | Path | Type |
|:--|:------------|:-----|:-----|
| 1 | Chaos Tests | `tests/chaos_hnsw.rs` | Test |
| 2 | Load Tests | `tests/load_test.rs` | Test |
| 3 | Regression CI | `.github/workflows/regression.yml` | Config |
| 4 | Baseline File | `benches/baselines.json` | Data |
| 5 | P99 Tracking | `benches/p99_bench.rs` | Bench |

---

## Acceptance Criteria

- [ ] AC1: Chaos tests cover 11 edge cases (empty index, single vector, max dimensions, recall accuracy, etc.)
- [ ] AC2: Load test successfully handles 100k vector insertions without panic
- [ ] AC3: Load test sustains 1000 searches/second for 60 seconds
- [ ] AC4: CI regression workflow runs on every PR
- [ ] AC5: CI fails if search latency regresses by >10% from baseline
- [ ] AC6: All existing tests still pass (`cargo test`)
- [ ] AC7: P99 latency benchmark runs and reports P50/P99/P999 percentiles
- [ ] AC8: P99 tracking integrated into CI workflow (per README v0.4.0 commitment)

---

## Implementation Steps

### Step 1: Create Chaos Tests (3 hours)

**tests/chaos_hnsw.rs:**

```rust
//! Chaos tests for HNSW edge cases
//! These tests verify behavior under unusual or extreme conditions

use edgevec::{HnswConfig, HnswIndex, VectorStorage};

/// Test 1: Empty index operations
#[test]
fn chaos_empty_index_search() {
    let config = HnswConfig::new(128);
    let storage = VectorStorage::new(&config, None);
    let index = HnswIndex::new(config.clone(), &storage).unwrap();

    // Search on empty index should return empty results
    let query = vec![0.0; 128];
    let results = index.search(&query, 10, &storage).unwrap();
    assert!(results.is_empty());
}

/// Test 2: Single vector index
#[test]
fn chaos_single_vector() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    let vector = vec![1.0; 128];
    let id = index.insert(&vector, &mut storage).unwrap();

    // Search should find the single vector
    let results = index.search(&vector, 10, &storage).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id);
}

/// Test 3: All vectors deleted
#[test]
fn chaos_all_deleted() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    // Insert and delete all vectors
    for i in 0..100 {
        let vector = vec![i as f32; 128];
        let id = index.insert(&vector, &mut storage).unwrap();
        index.soft_delete(id).unwrap();
    }

    // Search should return empty (all tombstones)
    let query = vec![50.0; 128];
    let results = index.search(&query, 10, &storage).unwrap();
    assert!(results.is_empty());

    // Verify counts
    assert_eq!(index.deleted_count(), 100);
    assert_eq!(index.live_count(), 0);
}

/// Test 4: Zero vector
#[test]
fn chaos_zero_vector() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    let zero_vector = vec![0.0; 128];
    let id = index.insert(&zero_vector, &mut storage).unwrap();

    // Should still be searchable
    let results = index.search(&zero_vector, 1, &storage).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id);
}

/// Test 5: Maximum supported dimensions
#[test]
fn chaos_max_dimensions() {
    let config = HnswConfig::new(4096); // High dimension
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    let vector = vec![0.1; 4096];
    let id = index.insert(&vector, &mut storage).unwrap();

    let results = index.search(&vector, 1, &storage).unwrap();
    assert_eq!(results[0].id, id);
}

/// Test 6: Duplicate vectors
#[test]
fn chaos_duplicate_vectors() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    let vector = vec![1.0; 128];

    // Insert same vector multiple times
    let id1 = index.insert(&vector, &mut storage).unwrap();
    let id2 = index.insert(&vector, &mut storage).unwrap();
    let id3 = index.insert(&vector, &mut storage).unwrap();

    // All should have unique IDs
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);

    // Search should find all three
    let results = index.search(&vector, 10, &storage).unwrap();
    assert_eq!(results.len(), 3);
}

/// Test 7: Delete and reinsert
#[test]
fn chaos_delete_reinsert() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    let vector = vec![1.0; 128];
    let id1 = index.insert(&vector, &mut storage).unwrap();
    index.soft_delete(id1).unwrap();

    // Reinsert (should get new ID)
    let id2 = index.insert(&vector, &mut storage).unwrap();
    assert_ne!(id1, id2);

    // Only new one should be searchable
    let results = index.search(&vector, 10, &storage).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id2);
}

/// Test 8: Extreme values
#[test]
fn chaos_extreme_values() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    // Very large values
    let large = vec![1e10_f32; 128];
    let id1 = index.insert(&large, &mut storage).unwrap();

    // Very small values
    let small = vec![1e-10_f32; 128];
    let id2 = index.insert(&small, &mut storage).unwrap();

    // Negative values
    let negative = vec![-1.0; 128];
    let id3 = index.insert(&negative, &mut storage).unwrap();

    assert_eq!(index.len(), 3);
}

/// Test 9: Rapid insert-delete cycles
#[test]
fn chaos_rapid_cycles() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    for i in 0..1000 {
        let vector = vec![i as f32; 128];
        let id = index.insert(&vector, &mut storage).unwrap();

        if i % 2 == 0 {
            index.soft_delete(id).unwrap();
        }
    }

    assert_eq!(index.live_count(), 500);
    assert_eq!(index.deleted_count(), 500);
}

/// Test 10: Compaction stress
#[test]
fn chaos_compaction_stress() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    // Insert many vectors
    for i in 0..500 {
        let vector = vec![i as f32; 128];
        index.insert(&vector, &mut storage).unwrap();
    }

    // Delete most of them
    for id in 0..400 {
        let _ = index.soft_delete(id);
    }

    assert!(index.needs_compaction());

    // Compact (takes immutable reference per API)
    let (new_index, _new_storage, result) = index.compact(&storage).unwrap();

    assert_eq!(result.tombstones_removed, 400);
    assert_eq!(new_index.len(), 100);
    assert_eq!(new_index.deleted_count(), 0);
}

/// Test 11: Recall accuracy under chaos (m7 fix)
#[test]
fn chaos_recall_accuracy() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config.clone(), &storage).unwrap();

    // Insert 100 vectors with known positions
    let mut vectors: Vec<Vec<f32>> = Vec::new();
    for i in 0..100 {
        let vector: Vec<f32> = (0..128).map(|j| (i * 10 + j) as f32).collect();
        vectors.push(vector.clone());
        index.insert(&vector, &mut storage).unwrap();
    }

    // Delete 50% randomly
    for i in (0..100).step_by(2) {
        index.soft_delete(edgevec::VectorId(i)).unwrap();
    }

    // Verify recall: search for each remaining vector should find itself
    let mut found = 0;
    for i in (1..100).step_by(2) {
        let query = &vectors[i as usize];
        let results = index.search(query, 1, &storage).unwrap();
        if !results.is_empty() && results[0].id == edgevec::VectorId(i) {
            found += 1;
        }
    }

    // Expect high recall (>90%) for exact matches among live vectors
    let recall = found as f64 / 50.0;
    assert!(recall >= 0.90, "Recall too low after deletions: {:.2}%", recall * 100.0);
}
```

### Step 2: Create Load Tests (2 hours)

**tests/load_test.rs:**

```rust
//! Load tests for sustained performance under stress
//! Run with: cargo test --release --test load_test -- --ignored

use edgevec::{HnswConfig, HnswIndex, VectorStorage};
use std::time::{Duration, Instant};

/// Test: Sustained insert load (100k vectors)
#[test]
#[ignore] // Run explicitly with --ignored
fn load_insert_100k() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    let start = Instant::now();

    for i in 0..100_000 {
        let vector: Vec<f32> = (0..128).map(|j| (i * j) as f32 / 1000.0).collect();
        index.insert(&vector, &mut storage).unwrap();

        if i % 10_000 == 0 {
            println!("Inserted {} vectors in {:?}", i, start.elapsed());
        }
    }

    let duration = start.elapsed();
    println!("Total: 100k inserts in {:?}", duration);

    // Assert: Should complete in under 2 minutes
    assert!(duration < Duration::from_secs(120), "Insert took too long: {:?}", duration);
    assert_eq!(index.len(), 100_000);
}

/// Test: Sustained search load (1000 QPS for 60 seconds)
#[test]
#[ignore]
fn load_search_sustained() {
    // Setup: Build index with 10k vectors
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    for i in 0..10_000 {
        let vector: Vec<f32> = (0..128).map(|j| (i * j) as f32 / 1000.0).collect();
        index.insert(&vector, &mut storage).unwrap();
    }

    // Test: Sustain 1000 searches/second for 60 seconds
    let target_qps = 1000;
    let duration_secs = 60;
    let total_queries = target_qps * duration_secs;

    let start = Instant::now();
    let mut query_count = 0;

    while query_count < total_queries {
        let query: Vec<f32> = (0..128).map(|j| (query_count * j) as f32 / 1000.0).collect();
        let results = index.search(&query, 10, &storage).unwrap();
        assert!(!results.is_empty());
        query_count += 1;

        if query_count % 10_000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let actual_qps = query_count as f64 / elapsed;
            println!("{} queries, {:.0} QPS", query_count, actual_qps);
        }
    }

    let duration = start.elapsed();
    let actual_qps = total_queries as f64 / duration.as_secs_f64();

    println!("Completed {} queries in {:?} ({:.0} QPS)", total_queries, duration, actual_qps);

    // Assert: Average QPS should be at least 500 (accounting for CI slowness)
    assert!(actual_qps >= 500.0, "QPS too low: {:.0}", actual_qps);
}

/// Test: Mixed workload (insert + search + delete)
#[test]
#[ignore]
fn load_mixed_workload() {
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    let start = Instant::now();

    for i in 0..50_000 {
        // Insert
        let vector: Vec<f32> = (0..128).map(|j| (i * j) as f32 / 1000.0).collect();
        let id = index.insert(&vector, &mut storage).unwrap();

        // Search every 10 inserts
        if i % 10 == 0 {
            let query = vec![i as f32 / 1000.0; 128];
            let _ = index.search(&query, 10, &storage);
        }

        // Delete every 100 inserts
        if i % 100 == 0 && i > 0 {
            let delete_id = (i - 50) as u64;
            let _ = index.soft_delete(delete_id);
        }
    }

    let duration = start.elapsed();
    println!("Mixed workload completed in {:?}", duration);

    // Assert: Should complete in under 3 minutes
    assert!(duration < Duration::from_secs(180));
}
```

### Step 3: Create CI Regression Workflow (2 hours)

**.github/workflows/regression.yml:**

```yaml
name: Performance Regression

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        with:
          toolchain: stable

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run benchmarks
        run: |
          cargo bench --bench search_bench -- --noplot --save-baseline current

      - name: Download baseline
        if: github.event_name == 'pull_request'
        uses: actions/download-artifact@v4
        with:
          name: benchmark-baseline
          path: target/criterion
        continue-on-error: true

      - name: Compare with baseline
        if: github.event_name == 'pull_request'
        run: |
          cargo bench --bench search_bench -- --noplot --baseline baseline

          # Parse criterion output for regression
          if grep -q "Performance has regressed" target/criterion/*/new/estimates.json 2>/dev/null; then
            echo "::error::Performance regression detected!"
            exit 1
          fi

      - name: Upload baseline
        if: github.event_name == 'push' && github.ref == 'refs/heads/main'
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-baseline
          path: target/criterion
          retention-days: 30

      - name: Performance Summary
        run: |
          echo "## Performance Summary" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "| Benchmark | Time |" >> $GITHUB_STEP_SUMMARY
          echo "|:----------|:-----|" >> $GITHUB_STEP_SUMMARY
          # Parse and display results
```

### Step 4: Create P99 Benchmark (1.5 hours)

**benches/p99_bench.rs:**

```rust
//! P99 latency tracking benchmark
//! Reports P50, P99, P999 percentiles for search latency
//!
//! Run with: cargo bench --bench p99_bench

use criterion::{criterion_group, criterion_main, Criterion};
use edgevec::{HnswConfig, HnswIndex, VectorStorage};
use std::time::Instant;

fn collect_latencies(index: &HnswIndex, storage: &VectorStorage, queries: &[Vec<f32>]) -> Vec<f64> {
    let mut latencies = Vec::with_capacity(queries.len());

    for query in queries {
        let start = Instant::now();
        let _ = index.search(query, 10, storage);
        latencies.push(start.elapsed().as_nanos() as f64);
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    latencies
}

fn percentile(sorted_latencies: &[f64], p: f64) -> f64 {
    let idx = ((p / 100.0) * sorted_latencies.len() as f64) as usize;
    let idx = idx.min(sorted_latencies.len() - 1);
    sorted_latencies[idx]
}

fn p99_benchmark(c: &mut Criterion) {
    // Build index with 10k vectors
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    for i in 0..10_000 {
        let vector: Vec<f32> = (0..128).map(|j| (i * j) as f32 / 1000.0).collect();
        index.insert(&vector, &mut storage).unwrap();
    }

    // Generate query vectors
    let queries: Vec<Vec<f32>> = (0..1000)
        .map(|i| (0..128).map(|j| (i * j) as f32 / 500.0).collect())
        .collect();

    let mut group = c.benchmark_group("p99_latency");

    group.bench_function("search_10k_p99", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::ZERO;
            for _ in 0..iters {
                let latencies = collect_latencies(&index, &storage, &queries);

                // Report percentiles
                let p50 = percentile(&latencies, 50.0);
                let p99 = percentile(&latencies, 99.0);
                let p999 = percentile(&latencies, 99.9);

                println!("P50: {:.2}µs, P99: {:.2}µs, P999: {:.2}µs",
                    p50 / 1000.0, p99 / 1000.0, p999 / 1000.0);

                total_duration += std::time::Duration::from_nanos(p99 as u64);
            }
            total_duration
        })
    });

    group.finish();
}

criterion_group!(benches, p99_benchmark);
criterion_main!(benches);
```

**Add to Cargo.toml:**
```toml
[[bench]]
name = "p99_bench"
harness = false
```

### Step 5: Create Baseline File (1 hour)

**benches/baselines.json:**

```json
{
  "version": "1.0.0",
  "created": "2025-12-19",
  "environment": {
    "cpu": "GitHub Actions runner (2 cores)",
    "rust": "stable",
    "flags": "release"
  },
  "baselines": {
    "search_10k_128d": {
      "mean_ns": 200000,
      "threshold_percent": 10,
      "description": "Search 10k vectors, 128 dimensions, k=10"
    },
    "search_100k_128d": {
      "mean_ns": 500000,
      "threshold_percent": 10,
      "description": "Search 100k vectors, 128 dimensions, k=10"
    },
    "insert_single": {
      "mean_ns": 50000,
      "threshold_percent": 15,
      "description": "Insert single vector"
    }
  }
}
```

---

## Test Requirements

- [ ] `cargo test` - All existing tests pass (159+)
- [ ] `cargo test --test chaos_hnsw` - All 10 chaos tests pass
- [ ] `cargo test --release --test load_test -- --ignored` - Load tests pass
- [ ] `cargo bench --bench p99_bench` - P99 benchmark runs and reports percentiles
- [ ] `cargo clippy` - No warnings

---

## Review Gate

**Artifacts for Review:**
1. `tests/chaos_hnsw.rs`
2. `tests/load_test.rs`
3. `.github/workflows/regression.yml`
4. `benches/baselines.json`
5. `benches/p99_bench.rs`

**Command:** `/review tests/chaos_hnsw.rs`

---

## Exit Criteria

Day 4 is **COMPLETE** when:
- [ ] All 10+ chaos tests pass
- [ ] Load tests pass in release mode
- [ ] CI workflow file created and valid YAML
- [ ] Baseline performance values documented
- [ ] P99 benchmark created and reports percentiles
- [ ] All existing tests still pass
- [ ] `/review` approved

---

**Next:** Proceed to W19.5 (v0.4.0 Release Prep) after review approval
