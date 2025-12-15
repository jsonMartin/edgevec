# Week 16 — Day 3 Tasks

**Date:** Day 3 of Week 16
**Focus:** Search Tombstone Filtering
**Agent:** RUST_ENGINEER
**Status:** [REVISED]

---

## REVISION NOTES (Post-Hostile Review)

**Addressed Issues (Round 1 - Planning):**
- M-DEP-1: Clarified that `get_node(NodeId)` already exists (src/hnsw/graph.rs:337)
- M-AC-1: Defined concrete performance baseline methodology

**Addressed Issues (Round 2 - Implementation Review):**
- C1, C3, C4 (Thread Safety): INVALID per RFC-001 design
  - RFC-001 explicitly uses `&mut self` for mutations (enforced by Rust borrow checker)
  - RFC-001 line 614-615: "Accept eventual consistency. The result may include vectors deleted during search."
  - No concurrent access possible at type level
- C2 (Index bounds): Fixed - `adjusted_k()` uses `MAX_ADJUSTED_K_MULTIPLIER` constant cap
- M1 (Floating point): Fixed - `adjusted_k()` now uses integer arithmetic: `k * total / live`
- M3 (Documentation): Fixed - Added comprehensive tombstone handling docs to `search()` and `adjusted_k()`
- m1 (Magic number): Fixed - Extracted `MAX_ADJUSTED_K_MULTIPLIER = 10` constant
- m3 (Boundary tests): Fixed - Added 7 boundary value tests for edge cases

---

## Day Objective

Modify the search algorithm to exclude deleted vectors from results while maintaining graph traversal integrity. Implement adaptive k adjustment to maintain result quality at high tombstone ratios.

**Success Criteria:**
- Search results never include deleted vectors
- Deleted nodes still participate in graph routing
- `adjusted_k()` compensates for tombstones
- Performance degradation < 20% at 10% tombstones

---

## Tasks

### W16.3: Update Search to Filter Tombstones

**Priority:** P0 (Core Search)
**Estimate:** 6h (2h base × 3x)
**Agent:** RUST_ENGINEER
**Depends On:** W16.2 (delete API)

#### Scope

- [x] **AC16.3.1:** Search results exclude deleted vectors
- [x] **AC16.3.2:** `adjusted_k()` compensates for tombstones
- [x] **AC16.3.3:** Empty result when all matches deleted
- [ ] **AC16.3.4:** Performance degradation < 20% at 10% tombstones (see baseline below)
- [x] **AC16.3.5:** Deleted nodes still used for routing

#### Performance Baseline Definition (M-AC-1 Fix)

**Baseline Specification:**
- Dataset: 100,000 vectors, 128 dimensions, random uniform distribution
- Search parameters: k=10, ef_search=50
- Hardware: Any modern CPU (document actual in benchmark report)
- Metric: **P99 latency** (not mean, not P50)
- Seed: Fixed RNG seed 42 for reproducibility

**Comparison Methodology:**
1. Build index with 100k vectors
2. Run 1000 search queries, record P99 latency (baseline_p99)
3. Delete 10% of vectors (10k random IDs)
4. Run same 1000 queries, record P99 latency (tombstone_p99)
5. Calculate: `degradation = (tombstone_p99 - baseline_p99) / baseline_p99 * 100`
6. PASS if degradation < 20%

**Benchmark Command:**
```bash
cargo bench --bench delete_bench -- search_tombstones
```

#### Method Clarification (M-DEP-1 Fix)

**VERIFIED:** `get_node(NodeId) -> Option<&HnswNode>` already exists at `src/hnsw/graph.rs:337`.

This is DIFFERENT from `get_node_by_vector_id(VectorId)` added in W16.2:
- `get_node(NodeId)` — O(1) array index lookup by internal node ID
- `get_node_by_vector_id(VectorId)` — O(n) linear scan by user-facing vector ID

The search filtering code uses `get_node(NodeId)` which is O(1) since search already has the NodeId from the candidate list.

#### Implementation Specification

**File:** `src/hnsw/search.rs` (or wherever search is implemented)

##### Adaptive K Adjustment

```rust
impl HnswIndex {
    /// Calculate adjusted k to compensate for tombstones
    ///
    /// When the index has deleted vectors, we over-fetch to ensure
    /// we can return k live results after filtering.
    ///
    /// # Formula
    ///
    /// adjusted_k = k / (1 - tombstone_ratio)
    /// Capped at 10x to prevent excessive over-fetching.
    ///
    /// # Examples
    ///
    /// * 0% tombstones: k = k (no adjustment)
    /// * 10% tombstones: k → 1.11x
    /// * 30% tombstones: k → 1.43x
    /// * 50% tombstones: k → 2x
    /// * 90% tombstones: k → 10x (capped)
    fn adjusted_k(&self, k: usize) -> usize {
        if self.deleted_count == 0 {
            return k;
        }

        let ratio = self.tombstone_ratio();
        // Cap ratio at 0.9 to prevent division by very small numbers
        let multiplier = 1.0 / (1.0 - ratio.min(0.9));
        let adjusted = ((k as f64) * multiplier).ceil() as usize;

        // Cap at 10x to prevent excessive memory usage
        adjusted.min(k * 10)
    }
}
```

##### Search Modification

The key insight: **Deleted nodes are still traversed during search** (they remain in the graph). We only filter them when collecting results.

```rust
impl HnswIndex {
    /// Search for k nearest neighbors
    ///
    /// # Tombstone Handling (v0.3.0)
    ///
    /// Deleted vectors are excluded from results but remain in the graph
    /// for routing. The search over-fetches based on tombstone ratio to
    /// ensure k live results are returned.
    pub fn search<P: VectorProvider>(
        &self,
        query: &[f32],
        k: usize,
        provider: &P,
    ) -> Result<Vec<SearchResult>, GraphError> {
        // Validate dimensions
        if query.len() != self.config.dimensions as usize {
            return Err(GraphError::DimensionMismatch {
                expected: self.config.dimensions as usize,
                actual: query.len(),
            });
        }

        // Handle empty index
        if self.nodes.is_empty() || self.entry_point.is_none() {
            return Ok(Vec::new());
        }

        // Adjust k for tombstone compensation
        let fetch_k = self.adjusted_k(k);

        // Perform HNSW search (unchanged algorithm)
        let candidates = self.search_internal(query, fetch_k, provider)?;

        // Filter out deleted vectors and take k
        let results: Vec<SearchResult> = candidates
            .into_iter()
            .filter(|result| {
                // Check if this result's vector is deleted
                self.get_node(result.node_id)
                    .map(|node| node.deleted == 0)
                    .unwrap_or(false)
            })
            .take(k)
            .collect();

        Ok(results)
    }
}
```

#### Critical Design Decision

**Why deleted nodes stay in the graph:**

1. **HNSW Correctness:** Removing nodes breaks graph connectivity
2. **Routing Quality:** Deleted nodes may be optimal routing points
3. **Simplicity:** No graph repair complexity
4. **Performance:** O(1) delete vs O(M × neighbors) repair

**Trade-off:** Slightly more traversal overhead at high tombstone ratios, mitigated by eventual compaction.

#### Test Cases

**File:** `tests/search_tombstone.rs` (new file)

```rust
use edgevec::hnsw::{HnswConfig, HnswIndex, VectorId};
use edgevec::storage::VectorStorage;

fn create_index_with_vectors(count: usize) -> (HnswIndex, VectorStorage) {
    let config = HnswConfig::new(4);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config.clone(), &storage).unwrap();

    for i in 0..count {
        let vec = vec![i as f32, 0.0, 0.0, 0.0];
        index.insert(&vec, &mut storage).unwrap();
    }

    (index, storage)
}

#[test]
fn test_search_excludes_deleted() {
    let (mut index, storage) = create_index_with_vectors(10);

    // Query vector closest to ID 1
    let query = vec![1.0, 0.0, 0.0, 0.0];

    // Before delete: ID 1 should be top result
    let results = index.search(&query, 3, &storage).unwrap();
    assert!(results.iter().any(|r| r.vector_id == VectorId(1)));

    // Delete ID 1
    index.delete(VectorId(1)).unwrap();

    // After delete: ID 1 should not appear
    let results = index.search(&query, 3, &storage).unwrap();
    assert!(!results.iter().any(|r| r.vector_id == VectorId(1)));
}

#[test]
fn test_search_all_deleted_returns_empty() {
    let (mut index, storage) = create_index_with_vectors(3);

    // Delete all vectors
    for i in 1..=3 {
        index.delete(VectorId(i as u64)).unwrap();
    }

    let query = vec![1.0, 0.0, 0.0, 0.0];
    let results = index.search(&query, 10, &storage).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_search_partial_deleted_returns_k() {
    let (mut index, storage) = create_index_with_vectors(10);

    // Delete 5 of 10 vectors
    for i in 1..=5 {
        index.delete(VectorId(i as u64)).unwrap();
    }

    let query = vec![5.0, 0.0, 0.0, 0.0];
    let results = index.search(&query, 3, &storage).unwrap();

    // Should return 3 results (from the 5 live vectors)
    assert_eq!(results.len(), 3);

    // None should be deleted
    for result in &results {
        assert!(!index.is_deleted(result.vector_id).unwrap());
    }
}

#[test]
fn test_adjusted_k_no_tombstones() {
    let (index, _storage) = create_index_with_vectors(10);
    assert_eq!(index.adjusted_k(10), 10);
}

#[test]
fn test_adjusted_k_with_tombstones() {
    let (mut index, _storage) = create_index_with_vectors(10);

    // Delete 5 of 10 = 50% tombstones
    for i in 1..=5 {
        index.delete(VectorId(i as u64)).unwrap();
    }

    // 50% tombstones → 2x multiplier
    let adjusted = index.adjusted_k(10);
    assert!(adjusted >= 18 && adjusted <= 22); // ~20
}

#[test]
fn test_adjusted_k_capped_at_10x() {
    let (mut index, _storage) = create_index_with_vectors(100);

    // Delete 95 of 100 = 95% tombstones
    for i in 1..=95 {
        index.delete(VectorId(i as u64)).unwrap();
    }

    // Should cap at 10x, not 20x
    let adjusted = index.adjusted_k(10);
    assert_eq!(adjusted, 100); // 10 * 10
}

#[test]
fn test_search_uses_deleted_for_routing() {
    // This is a correctness test: deleted nodes should still be
    // visited during traversal, just not returned as results.
    //
    // We verify this indirectly by ensuring search still works
    // when "bridge" nodes are deleted.

    let (mut index, storage) = create_index_with_vectors(20);

    // Delete some middle vectors that might be routing nodes
    for i in 5..15 {
        index.delete(VectorId(i as u64)).unwrap();
    }

    // Search should still find vectors on "both sides"
    let query1 = vec![1.0, 0.0, 0.0, 0.0];
    let query2 = vec![19.0, 0.0, 0.0, 0.0];

    let results1 = index.search(&query1, 3, &storage).unwrap();
    let results2 = index.search(&query2, 3, &storage).unwrap();

    // Should find results for both queries
    assert!(!results1.is_empty());
    assert!(!results2.is_empty());
}
```

#### Performance Benchmark

**File:** `benches/delete_bench.rs` (extend existing)

```rust
fn bench_search_with_tombstones(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_tombstones");

    for tombstone_pct in [0, 10, 30, 50] {
        group.bench_function(&format!("{}%_tombstones", tombstone_pct), |b| {
            let (mut index, storage) = create_index(10000, 128);

            // Delete tombstone_pct% of vectors
            let delete_count = 10000 * tombstone_pct / 100;
            for i in 1..=delete_count {
                index.delete(VectorId(i as u64)).unwrap();
            }

            let query = random_vector(128);

            b.iter(|| {
                index.search(&query, 10, &storage).unwrap()
            });
        });
    }

    group.finish();
}
```

#### Verification Commands

```bash
# Run search tombstone tests
cargo test search_tombstone

# Run all tests
cargo test --all

# Run performance benchmark
cargo bench --bench delete_bench -- search_tombstones

# Clippy check
cargo clippy -- -D warnings
```

---

## Day 3 Summary

**Total Effort:** 6h scheduled

**Deliverables:**
1. `adjusted_k()` method
2. Search filtering for tombstones
3. Performance-validated filtering
4. Comprehensive unit tests
5. Performance benchmarks

**Day 4 Preview:**
- Implement `compact()` for space reclamation
- Implement `insert_with_id()` for rebuild

---

## HOSTILE_REVIEWER Pre-Flight

Before end of day:

- [x] Search never returns deleted vectors
- [x] `adjusted_k()` correctly compensates
- [x] Empty result when all matches deleted
- [ ] Performance < 20% degradation at 10% tombstones
- [x] Deleted nodes still used for routing
- [x] All new tests pass (7 search_tombstone + 4 adjusted_k + 3 proptest_hnsw_delete)
- [x] Clippy clean

---

**Status:** [APPROVED_WITH_FIXES]
**Implementation Date:** 2025-12-14
**Review Fix Date:** 2025-12-14
**Next:** Day 4 - `compact()` and `insert_with_id()` implementation
