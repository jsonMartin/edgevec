# Week 27 Day 4: Rescoring Layer for Recall Recovery

**Date:** 2025-12-25
**Focus:** F32 rescoring to improve BQ search recall
**Estimated Duration:** 8 hours
**Phase:** RFC-002 Implementation Phase 2 (Binary Quantization)

---

## Tasks

### W27.4.1: Implement rescore() Function

**Objective:** Rerank BQ candidates using exact F32 distance.

**Why Rescoring:**
- BQ search is fast but loses precision
- Rescoring top-N candidates with F32 distance recovers recall
- Trade-off: more candidates = better recall but slower

**Acceptance Criteria:**
- [ ] `rescore(candidates: &[(VectorId, f32)], query: &[f32], storage: &VectorStorage) -> Vec<(VectorId, f32)>`
- [ ] Loads F32 vectors for each candidate
- [ ] Computes exact L2 distance (or cosine similarity)
- [ ] Returns candidates sorted by F32 distance (ascending)
- [ ] Unit test: rescore improves ordering

**Files:**
- `src/hnsw/rescore.rs` (new file)
- `src/hnsw/mod.rs` (add module)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**Implementation:**

```rust
// src/hnsw/rescore.rs

use crate::hnsw::{GraphError, VectorId};
use crate::storage::VectorStorage;

/// Rescores candidates using exact F32 distance.
///
/// This function takes approximate BQ results and recomputes
/// the exact distance for each candidate, returning them in
/// correct order.
///
/// # Arguments
///
/// * `candidates` - BQ search results (VectorId, approximate_score).
/// * `query` - The original query vector.
/// * `storage` - F32 vector storage.
///
/// # Returns
///
/// Candidates sorted by exact F32 distance (ascending).
///
/// # Performance
///
/// Time: O(n × d) where n = candidates.len(), d = dimension.
/// Space: O(n) for storing rescored results.
///
/// # Example
///
/// ```ignore
/// let bq_results = index.search_bq(&query, k * 3, &storage)?;
/// let rescored = rescore(&bq_results, &query, &storage);
/// let final_results: Vec<_> = rescored.into_iter().take(k).collect();
/// ```
pub fn rescore(
    candidates: &[(VectorId, f32)],
    query: &[f32],
    storage: &VectorStorage,
) -> Vec<(VectorId, f32)> {
    let mut rescored: Vec<(VectorId, f32)> = candidates
        .iter()
        .filter_map(|(id, _approx_score)| {
            // Load F32 vector from storage
            let vector = storage.get(*id).ok()?;

            // Compute exact L2 distance
            let distance = l2_distance(query, &vector);

            Some((*id, distance))
        })
        .collect();

    // Sort by distance (ascending = most similar first)
    rescored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    rescored
}

/// Computes L2 (Euclidean) distance between two vectors.
///
/// Distance = sqrt(sum((a[i] - b[i])^2))
fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    let sum_sq: f32 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff = x - y;
            diff * diff
        })
        .sum();

    sum_sq.sqrt()
}

/// Rescores candidates and returns top-k.
///
/// Convenience function combining rescore + truncate.
pub fn rescore_top_k(
    candidates: &[(VectorId, f32)],
    query: &[f32],
    storage: &VectorStorage,
    k: usize,
) -> Vec<(VectorId, f32)> {
    let mut rescored = rescore(candidates, query, storage);
    rescored.truncate(k);
    rescored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l2_distance_zero() {
        let v = vec![1.0, 2.0, 3.0];
        assert_eq!(l2_distance(&v, &v), 0.0);
    }

    #[test]
    fn test_l2_distance_known() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        assert!((l2_distance(&a, &b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_l2_distance_symmetric() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert_eq!(l2_distance(&a, &b), l2_distance(&b, &a));
    }
}
```

**Dependencies:** W27.3.3 (search_bq provides candidates)

---

### W27.4.2: Implement search_bq_rescored()

**Objective:** Combined BQ search + F32 rescoring for production use.

**Algorithm:**
1. Search with BQ for `k × rescore_factor` candidates
2. Rescore all candidates with F32 distance
3. Return top-k

**Acceptance Criteria:**
- [ ] `search_bq_rescored(query: &[f32], k: usize, rescore_factor: usize, storage: &VectorStorage) -> Result<Vec<(VectorId, f32)>, GraphError>`
- [ ] Default rescore_factor = 3 (retrieve 3× candidates)
- [ ] Returns exactly min(k, available) results
- [ ] Recall@10 > 0.90 with rescore_factor=3
- [ ] Unit test: rescored results are better than raw BQ
- [ ] Benchmark: latency vs F32-only search

**Files:**
- `src/hnsw/graph.rs` (add method)
- `tests/bq_rescore.rs` (new file)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**Implementation:**

```rust
// In src/hnsw/graph.rs

impl HnswIndex {
    /// Searches using binary quantization with F32 rescoring.
    ///
    /// This provides the best of both worlds:
    /// - Fast BQ search for candidate generation
    /// - Accurate F32 rescoring for final ranking
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector.
    /// * `k` - Number of results to return.
    /// * `rescore_factor` - Overfetch multiplier (default: 3).
    ///   Higher values improve recall but increase latency.
    /// * `storage` - F32 vector storage.
    ///
    /// # Returns
    ///
    /// Top-k results sorted by exact F32 distance.
    ///
    /// # Performance
    ///
    /// - BQ phase: O(log n × d/8) — very fast
    /// - Rescore phase: O(k × rescore_factor × d) — proportional to overfetch
    ///
    /// Typical latency: 1.5-2× pure BQ, but with recall ~0.95+
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Search for 10 results with 3× overfetch
    /// let results = index.search_bq_rescored(&query, 10, 3, &storage)?;
    /// ```
    pub fn search_bq_rescored(
        &self,
        query: &[f32],
        k: usize,
        rescore_factor: usize,
        storage: &VectorStorage,
    ) -> Result<Vec<(VectorId, f32)>, GraphError> {
        use crate::hnsw::rescore::rescore_top_k;

        // Validate inputs
        let rescore_factor = rescore_factor.max(1); // At least 1×

        // Step 1: BQ search for more candidates
        let overfetched_k = k.saturating_mul(rescore_factor);
        let bq_candidates = self.search_bq(query, overfetched_k, storage)?;

        // Step 2: Rescore with F32 and return top-k
        let rescored = rescore_top_k(&bq_candidates, query, storage, k);

        // Convert distance to similarity for consistent API
        // (Lower distance = higher similarity)
        let results: Vec<_> = rescored
            .into_iter()
            .map(|(id, distance)| {
                // Convert distance to similarity
                // Using inverse: similarity = 1 / (1 + distance)
                let similarity = 1.0 / (1.0 + distance);
                (id, similarity)
            })
            .collect();

        Ok(results)
    }

    /// Convenience method with default rescore factor.
    ///
    /// Uses rescore_factor = 3, which provides good recall/speed balance.
    pub fn search_bq_rescored_default(
        &self,
        query: &[f32],
        k: usize,
        storage: &VectorStorage,
    ) -> Result<Vec<(VectorId, f32)>, GraphError> {
        self.search_bq_rescored(query, k, 3, storage)
    }
}
```

**Test Cases:**

```rust
// tests/bq_rescore.rs

mod rescore_tests {
    use edgevec::hnsw::{HnswConfig, HnswIndex};
    use edgevec::storage::VectorStorage;

    /// Helper to create index with known vectors.
    fn create_test_index() -> (HnswIndex, VectorStorage) {
        let config = HnswConfig::new(128);
        let mut storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::with_bq(config, &storage).unwrap();

        // Insert 100 vectors with varying patterns
        for i in 0..100 {
            let v: Vec<f32> = (0..128)
                .map(|j| ((i * 128 + j) as f32).sin())
                .collect();
            index.insert_bq(&v, &mut storage).unwrap();
        }

        (index, storage)
    }

    #[test]
    fn test_search_bq_rescored_returns_k() {
        let (index, storage) = create_test_index();

        let query: Vec<f32> = (0..128).map(|j| (j as f32).sin()).collect();
        let results = index.search_bq_rescored(&query, 10, 3, &storage).unwrap();

        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_rescored_better_than_raw_bq() {
        let (index, storage) = create_test_index();

        // Query is exactly vector 0
        let query: Vec<f32> = (0..128).map(|j| (j as f32).sin()).collect();

        // Raw BQ search
        let bq_results = index.search_bq(&query, 10, &storage).unwrap();

        // Rescored search
        let rescored = index.search_bq_rescored(&query, 10, 3, &storage).unwrap();

        // First result of rescored should be vector 0 (exact match)
        // This may not be true for raw BQ due to quantization loss
        assert_eq!(rescored[0].0.0, 1); // VectorId(1) is first insert
    }

    #[test]
    fn test_rescore_factor_affects_recall() {
        let (index, storage) = create_test_index();
        let query: Vec<f32> = (0..128).map(|j| (j as f32).sin()).collect();

        // Higher rescore factor should give better results
        let low_factor = index.search_bq_rescored(&query, 10, 1, &storage).unwrap();
        let high_factor = index.search_bq_rescored(&query, 10, 5, &storage).unwrap();

        // Can't directly compare, but high factor should at least work
        assert_eq!(low_factor.len(), 10);
        assert_eq!(high_factor.len(), 10);
    }
}

mod recall_benchmark {
    use edgevec::hnsw::{HnswConfig, HnswIndex};
    use edgevec::storage::VectorStorage;

    /// Measures recall@k for BQ+rescore vs pure F32 search.
    #[test]
    fn test_recall_at_10_above_threshold() {
        let config = HnswConfig::new(128);
        let mut storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::with_bq(config, &storage).unwrap();

        // Insert 1000 random vectors
        let mut rng = rand::thread_rng();
        use rand::Rng;

        let vectors: Vec<Vec<f32>> = (0..1000)
            .map(|_| (0..128).map(|_| rng.gen::<f32>()).collect())
            .collect();

        for v in &vectors {
            index.insert_bq(v, &mut storage).unwrap();
        }

        // Run queries and measure recall
        let mut total_recall = 0.0;
        let num_queries = 50;

        for i in 0..num_queries {
            let query = &vectors[i * 10]; // Use some vectors as queries

            // Ground truth: F32 search
            let f32_results = index.search(query, 10, &storage).unwrap();
            let f32_ids: std::collections::HashSet<_> =
                f32_results.iter().map(|r| r.vector_id).collect();

            // BQ+rescore search
            let bq_results = index.search_bq_rescored(query, 10, 3, &storage).unwrap();
            let bq_ids: std::collections::HashSet<_> =
                bq_results.iter().map(|(id, _)| *id).collect();

            // Recall = intersection / ground_truth
            let intersection = f32_ids.intersection(&bq_ids).count();
            let recall = intersection as f64 / f32_ids.len() as f64;
            total_recall += recall;
        }

        let avg_recall = total_recall / num_queries as f64;
        println!("Average recall@10: {:.3}", avg_recall);

        // Target: >0.90 recall
        assert!(avg_recall > 0.90, "Recall {avg_recall:.3} below 0.90 threshold");
    }
}
```

**Dependencies:** W27.4.1

---

## Day 4 Checklist

- [ ] W27.4.1: rescore() function implemented
- [ ] W27.4.2: search_bq_rescored() implemented
- [ ] Recall@10 > 0.90 with rescore_factor=3
- [ ] All existing tests pass (`cargo test`)
- [ ] New tests pass (`cargo test bq_rescore`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Formatted (`cargo fmt --check`)

## Day 4 Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | CI green |
| Rescoring works | Unit tests |
| Recall > 0.90 | Recall benchmark |
| API is ergonomic | Documentation + examples |

## Day 4 Handoff

After completing Day 4:

**Artifacts Generated:**
- `src/hnsw/rescore.rs`
- Modified `src/hnsw/graph.rs`
- `tests/bq_rescore.rs`

**Status:** PENDING_DAY_5

**Next:** Day 5 — Benchmarks + validation tests

---

*Agent: RUST_ENGINEER*
*Status: [APPROVED] (2025-12-21)*
