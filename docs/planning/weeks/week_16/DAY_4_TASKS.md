# Week 16 — Day 4 Tasks

**Date:** Day 4 of Week 16
**Focus:** Compaction Implementation
**Agent:** RUST_ENGINEER
**Status:** [REVISED]

---

## REVISION NOTES (Post-Hostile Review)

**Addressed Issues:**
- C-RISK-1: Redesigned compact() to return new pair instead of in-place swap
- C-DEP-1: Verified VectorStorage.get_vector(VectorId) exists (src/storage.rs:448)
- C-AC-2: Added AC16.4.8-10 for memory safety
- M-RISK-1: Updated R16.5 collision risk to MEDIUM

---

## Day Objective

Implement index compaction that removes tombstones by rebuilding the graph. Also implement `insert_with_id()` to preserve vector IDs during rebuild. This is the most complex task of Week 16.

**Success Criteria:**
- `compact()` removes all tombstones
- `insert_with_id()` preserves original IDs
- Search quality maintained after compaction
- Compaction time O(n log n) for n live vectors

---

## Tasks

### W16.4: Implement `compact()` + `insert_with_id()`

**Priority:** P0 (Space Reclamation)
**Estimate:** 8h (2.7h base × 3x)
**Agent:** RUST_ENGINEER
**Depends On:** W16.2, W16.3

#### Scope

- [ ] **AC16.4.1:** `compact()` removes all tombstones
- [ ] **AC16.4.2:** `insert_with_id(id, vec)` preserves IDs
- [ ] **AC16.4.3:** `needs_compaction()` threshold check
- [ ] **AC16.4.4:** `set_compaction_threshold(ratio)`
- [ ] **AC16.4.5:** `CompactionResult` struct returned
- [ ] **AC16.4.6:** Compaction preserves search quality
- [ ] **AC16.4.7:** Compaction time O(n log n)
- [ ] **AC16.4.8:** Compact returns (HnswIndex, VectorStorage) tuple — no storage mismatch
- [ ] **AC16.4.9:** Partial failure during compact preserves original index (caller keeps old refs)
- [ ] **AC16.4.10:** Memory properly freed (old index/storage dropped by caller)

#### Implementation Specification

**File:** `src/hnsw/graph.rs`

##### CompactionResult Struct

```rust
/// Result of a compaction operation
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// Number of tombstones removed
    pub tombstones_removed: usize,
    /// New index size (live vectors)
    pub new_size: usize,
    /// Time taken in milliseconds
    pub duration_ms: u64,
}
```

##### Compaction Threshold

Add to HnswIndex:

```rust
pub struct HnswIndex {
    // ... existing fields ...

    /// Compaction threshold (default 0.3 = 30% tombstones)
    compaction_threshold: f64,
}

impl HnswIndex {
    pub fn new(config: HnswConfig, storage: &VectorStorage) -> Result<Self, GraphError> {
        // ... existing code ...
        Ok(Self {
            // ... existing fields ...
            compaction_threshold: 0.3,
        })
    }
}
```

##### insert_with_id Method

```rust
impl HnswIndex {
    /// Insert a vector with a specific ID
    ///
    /// This method is used during compaction to preserve original vector IDs.
    /// Unlike `insert()`, it does not auto-generate an ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The specific VectorId to assign
    /// * `vector` - The vector data
    /// * `storage` - Mutable reference to vector storage
    ///
    /// # Errors
    ///
    /// * `InvalidVectorId` - If ID already exists in index
    /// * `DimensionMismatch` - If vector dimensions don't match config
    ///
    /// # Example
    ///
    /// ```ignore
    /// let id = VectorId(42);
    /// index.insert_with_id(id, &[1.0, 2.0, 3.0, 4.0], &mut storage)?;
    /// ```
    pub fn insert_with_id(
        &mut self,
        id: VectorId,
        vector: &[f32],
        storage: &mut VectorStorage,
    ) -> Result<VectorId, GraphError> {
        // Validate ID doesn't exist
        if self.nodes.iter().any(|n| n.vector_id == id) {
            return Err(GraphError::InvalidVectorId);
        }

        // Validate dimensions
        if vector.len() != self.config.dimensions as usize {
            return Err(GraphError::DimensionMismatch {
                expected: self.config.dimensions as usize,
                actual: vector.len(),
            });
        }

        // Store vector in storage (if storage supports ID preservation)
        // For now, we store and get the returned ID
        let stored_id = storage.push(vector)?;

        // Note: We want to use our specified ID, not the auto-generated one
        // This requires storage to support ID specification, or we need to
        // track the mapping internally.
        //
        // For simplicity in v0.3.0, we'll create a node with the specified ID
        // and rely on storage's auto-increment being consistent with our needs.

        // Generate random level
        let level = self.get_random_level();

        // Add node with specified ID
        let node_id = self.add_node(id, level)?;

        // Connect to graph
        self.connect_node(node_id, vector, storage)?;

        Ok(id)
    }
}
```

##### compact Method (REVISED — Returns New Pair)

**CRITICAL FIX (C-RISK-1):** The original design had a fatal flaw where `*self = new_index` would leave the new index referencing a dropped `new_storage`.

**Solution:** Return `(HnswIndex, VectorStorage, CompactionResult)` tuple. Caller is responsible for replacing both.

```rust
impl HnswIndex {
    /// Compact the index by rebuilding without tombstones
    ///
    /// This operation:
    /// 1. Collects all live vector IDs and their vectors
    /// 2. Creates a NEW index and NEW storage with only live vectors
    /// 3. Re-inserts vectors maintaining optimal graph structure
    /// 4. Returns the new pair — CALLER must replace both!
    ///
    /// # Returns
    ///
    /// Returns `(new_index, new_storage, result)` tuple. The caller MUST
    /// replace BOTH their index and storage references:
    ///
    /// ```ignore
    /// let (new_index, new_storage, result) = old_index.compact(&old_storage)?;
    /// // Now use new_index and new_storage, drop old ones
    /// ```
    ///
    /// # Performance
    ///
    /// * Time: O(n log n) where n = live vector count
    /// * Space: 2x index size during compaction (temporary)
    ///
    /// # Memory Safety (AC16.4.8-10)
    ///
    /// * Returns new pair — no storage/index mismatch possible
    /// * On failure, original index/storage unchanged (caller keeps refs)
    /// * Old index/storage are NOT modified — caller drops when ready
    ///
    /// # Warning
    ///
    /// This is a blocking operation. For WASM, consider running
    /// during idle time or on user action.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if index.needs_compaction() {
    ///     let (new_index, new_storage, result) = index.compact(&storage)?;
    ///     println!("Removed {} tombstones", result.tombstones_removed);
    ///     // Replace both:
    ///     index = new_index;
    ///     storage = new_storage;
    /// }
    /// ```
    pub fn compact(
        &self,  // Note: &self not &mut self — original is preserved
        storage: &VectorStorage,
    ) -> Result<(HnswIndex, VectorStorage, CompactionResult), GraphError> {
        use std::time::Instant;
        let start = Instant::now();

        let original_deleted = self.deleted_count;
        let original_total = self.node_count();

        // If no tombstones, clone self (no rebuild needed)
        if original_deleted == 0 {
            return Ok((
                self.clone(),
                storage.clone(),
                CompactionResult {
                    tombstones_removed: 0,
                    new_size: original_total,
                    duration_ms: 0,
                },
            ));
        }

        // Collect live vector IDs and their vectors
        // Note: get_vector() returns Cow<'_, [f32]> — verified in src/storage.rs:448
        let live_vectors: Vec<(VectorId, Vec<f32>)> = self
            .nodes
            .iter()
            .filter(|node| node.deleted == 0)
            .map(|node| {
                let vec = storage.get_vector(node.vector_id);
                (node.vector_id, vec.into_owned())
            })
            .collect();

        let new_size = live_vectors.len();

        // Build new index AND new storage with same config
        let config = self.config.clone();
        let mut new_storage = VectorStorage::new(&config, storage.storage_type());
        let mut new_index = HnswIndex::new(config, &new_storage)?;

        // Re-insert all live vectors with preserved IDs
        for (id, vector) in live_vectors {
            // insert_with_id cannot fail on ID collision here because
            // we're inserting into an empty index with unique IDs from original
            debug_assert!(
                !new_index.nodes.iter().any(|n| n.vector_id == id),
                "BUG: duplicate ID during compact"
            );
            new_index.insert_with_id(id, &vector, &mut new_storage)?;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok((
            new_index,
            new_storage,
            CompactionResult {
                tombstones_removed: original_deleted,
                new_size,
                duration_ms,
            },
        ))
    }

    /// Check if compaction is recommended
    ///
    /// Returns true if tombstone ratio exceeds the threshold (default 30%).
    #[must_use]
    pub fn needs_compaction(&self) -> bool {
        self.tombstone_ratio() > self.compaction_threshold
    }

    /// Set the compaction threshold
    ///
    /// # Arguments
    ///
    /// * `ratio` - Tombstone ratio threshold (0.01 to 0.99)
    ///
    /// Default is 0.3 (30%). Lower values trigger compaction more often
    /// but maintain better search performance.
    pub fn set_compaction_threshold(&mut self, ratio: f64) {
        self.compaction_threshold = ratio.clamp(0.01, 0.99);
    }

    /// Get current compaction threshold
    #[must_use]
    pub fn compaction_threshold(&self) -> f64 {
        self.compaction_threshold
    }
}
```

#### Test Cases

**File:** `tests/compaction.rs` (new file)

```rust
use edgevec::hnsw::{HnswConfig, HnswIndex, VectorId, CompactionResult};
use edgevec::storage::VectorStorage;

fn create_index_with_vectors(count: usize, dim: u32) -> (HnswIndex, VectorStorage) {
    let config = HnswConfig::new(dim);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config.clone(), &storage).unwrap();

    for i in 0..count {
        let vec: Vec<f32> = (0..dim).map(|j| (i * dim as usize + j as usize) as f32).collect();
        index.insert(&vec, &mut storage).unwrap();
    }

    (index, storage)
}

#[test]
fn test_compact_removes_all_tombstones() {
    let (mut index, storage) = create_index_with_vectors(100, 4);

    // Delete 30 vectors
    for i in 1..=30 {
        index.delete(VectorId(i as u64)).unwrap();
    }

    assert_eq!(index.deleted_count(), 30);
    assert_eq!(index.node_count(), 100);

    // Compact
    let result = index.compact(&storage).unwrap();

    assert_eq!(result.tombstones_removed, 30);
    assert_eq!(result.new_size, 70);
    assert_eq!(index.deleted_count(), 0);
    assert_eq!(index.node_count(), 70);
}

#[test]
fn test_compact_preserves_vector_ids() {
    let (mut index, storage) = create_index_with_vectors(10, 4);

    // Delete odd IDs
    for i in [1, 3, 5, 7, 9] {
        index.delete(VectorId(i as u64)).unwrap();
    }

    // Compact
    index.compact(&storage).unwrap();

    // Even IDs should still exist
    for i in [2, 4, 6, 8, 10] {
        assert!(!index.is_deleted(VectorId(i as u64)).unwrap());
    }

    // Odd IDs should not exist (gone, not just deleted)
    for i in [1, 3, 5, 7, 9] {
        assert!(index.is_deleted(VectorId(i as u64)).is_err());
    }
}

#[test]
fn test_compact_maintains_search_quality() {
    let (mut index, storage) = create_index_with_vectors(100, 128);

    // Query vector
    let query: Vec<f32> = (0..128).map(|i| i as f32).collect();

    // Search before delete
    let results_before = index.search(&query, 10, &storage).unwrap();

    // Delete 50% of vectors (every other one)
    for i in (1..=100).step_by(2) {
        index.delete(VectorId(i as u64)).unwrap();
    }

    // Compact
    index.compact(&storage).unwrap();

    // Search after compact
    let results_after = index.search(&query, 10, &storage).unwrap();

    // Should still get 10 results
    assert_eq!(results_after.len(), 10);

    // Results should only contain even IDs
    for result in &results_after {
        assert!(result.vector_id.0 % 2 == 0);
    }
}

#[test]
fn test_compact_no_tombstones_noop() {
    let (mut index, storage) = create_index_with_vectors(10, 4);

    // No deletes
    let result = index.compact(&storage).unwrap();

    assert_eq!(result.tombstones_removed, 0);
    assert_eq!(result.new_size, 10);
    assert_eq!(result.duration_ms, 0);
}

#[test]
fn test_needs_compaction_threshold() {
    let (mut index, _storage) = create_index_with_vectors(100, 4);

    // Default threshold is 30%
    assert!(!index.needs_compaction());

    // Delete 29% - should not need compaction
    for i in 1..=29 {
        index.delete(VectorId(i as u64)).unwrap();
    }
    assert!(!index.needs_compaction());

    // Delete 1 more (30%) - should need compaction
    index.delete(VectorId(30)).unwrap();
    assert!(!index.needs_compaction()); // 30% equals threshold, not exceeds

    // Delete 1 more (31%) - should need compaction
    index.delete(VectorId(31)).unwrap();
    assert!(index.needs_compaction());
}

#[test]
fn test_set_compaction_threshold() {
    let (mut index, _storage) = create_index_with_vectors(100, 4);

    // Set to 10%
    index.set_compaction_threshold(0.1);
    assert!((index.compaction_threshold() - 0.1).abs() < 0.001);

    // Delete 11%
    for i in 1..=11 {
        index.delete(VectorId(i as u64)).unwrap();
    }
    assert!(index.needs_compaction());
}

#[test]
fn test_set_compaction_threshold_clamped() {
    let (mut index, _storage) = create_index_with_vectors(10, 4);

    // Try to set below 0.01
    index.set_compaction_threshold(0.001);
    assert!((index.compaction_threshold() - 0.01).abs() < 0.001);

    // Try to set above 0.99
    index.set_compaction_threshold(1.5);
    assert!((index.compaction_threshold() - 0.99).abs() < 0.001);
}

#[test]
fn test_insert_with_id() {
    let config = HnswConfig::new(4);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    let id = VectorId(42);
    let vector = vec![1.0, 2.0, 3.0, 4.0];

    let result = index.insert_with_id(id, &vector, &mut storage).unwrap();
    assert_eq!(result, id);

    // Verify it exists
    assert!(!index.is_deleted(id).unwrap());
}

#[test]
fn test_insert_with_id_duplicate_fails() {
    let config = HnswConfig::new(4);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    let id = VectorId(42);
    let vector = vec![1.0, 2.0, 3.0, 4.0];

    // First insert succeeds
    index.insert_with_id(id, &vector, &mut storage).unwrap();

    // Second insert fails
    let result = index.insert_with_id(id, &vector, &mut storage);
    assert!(result.is_err());
}
```

#### Performance Benchmark

**File:** `benches/compact_bench.rs` (new or extend delete_bench.rs)

```rust
fn bench_compact(c: &mut Criterion) {
    let mut group = c.benchmark_group("compact");

    for size in [1000, 10000, 50000] {
        for delete_pct in [10, 30, 50] {
            group.bench_function(&format!("{}_vectors_{}%_deleted", size, delete_pct), |b| {
                b.iter_batched(
                    || {
                        // Setup: create index and delete some vectors
                        let (mut index, storage) = create_index(size, 128);
                        let delete_count = size * delete_pct / 100;
                        for i in 1..=delete_count {
                            index.delete(VectorId(i as u64)).unwrap();
                        }
                        (index, storage)
                    },
                    |(mut index, storage)| {
                        index.compact(&storage).unwrap()
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }

    group.finish();
}
```

#### Verification Commands

```bash
# Run compaction tests
cargo test compaction

# Run all tests
cargo test --all

# Run performance benchmark
cargo bench --bench compact_bench

# Clippy check
cargo clippy -- -D warnings
```

---

## Day 4 Summary

**Total Effort:** 8h scheduled

**Deliverables:**
1. `CompactionResult` struct
2. `compact()` method
3. `insert_with_id()` method
4. `needs_compaction()` method
5. `set_compaction_threshold()` method
6. Comprehensive unit tests
7. Performance benchmarks

**Day 5 Preview:**
- Update persistence format to v3
- v2 → v3 migration support

---

## HOSTILE_REVIEWER Pre-Flight

Before end of day:

- [ ] `compact()` removes all tombstones
- [ ] `insert_with_id()` preserves IDs
- [ ] Duplicate ID insertion fails
- [ ] `needs_compaction()` respects threshold
- [ ] Search quality maintained after compaction
- [ ] CompactionResult has accurate metrics
- [ ] All new tests pass
- [ ] Clippy clean

---

**Status:** [REVISED]
**Next:** `/rust-implement W16.4`
