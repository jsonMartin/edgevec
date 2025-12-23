# Week 27 Day 3: HNSW BQ Search Integration

**Date:** 2025-12-24
**Focus:** Integrate binary quantization with HNSW search
**Estimated Duration:** 14 hours
**Phase:** RFC-002 Implementation Phase 2 (Binary Quantization)

---

## Tasks

### W27.3.1: Add BQ Storage Field to HnswIndex

**Objective:** Enable dual-mode operation (F32 + BQ) in HnswIndex.

**Design Decision:**
- BQ storage is **optional** — index can operate F32-only
- When BQ is enabled, vectors are stored in both F32 and BQ format
- F32 storage remains authoritative (for rescoring)
- BQ storage provides fast approximate search

**Acceptance Criteria:**
- [ ] `bq_storage: Option<BinaryVectorStorage>` field added to `HnswIndex`
- [ ] `HnswIndex::new()` initializes `bq_storage = None`
- [ ] `HnswIndex::with_bq(config, storage)` constructor enables BQ
- [ ] `enable_bq()` method to enable BQ on existing index
- [ ] `has_bq() -> bool` accessor
- [ ] All existing tests still pass

**Files:**
- `src/hnsw/graph.rs` (modify struct + constructors)
- `src/hnsw/mod.rs` (re-exports if needed)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**Implementation:**

```rust
// In src/hnsw/graph.rs

pub struct HnswIndex {
    // Existing fields...
    pub config: HnswConfig,
    pub(crate) nodes: Vec<HnswNode>,
    pub(crate) neighbors: NeighborPool,
    pub(crate) entry_point: Option<NodeId>,
    pub(crate) max_layer: u8,
    pub(crate) level_mult: f32,
    rng: ChaCha8Rng,
    pub(crate) deleted_count: usize,
    compaction_threshold: f64,
    pub(crate) metadata: MetadataStore,

    // NEW: Binary quantization storage (optional)
    pub(crate) bq_storage: Option<BinaryVectorStorage>,
}

impl HnswIndex {
    /// Creates a new index with binary quantization enabled.
    ///
    /// BQ provides 32x memory reduction and 3-5x search speedup
    /// at the cost of some recall (recovered via rescoring).
    pub fn with_bq(
        config: HnswConfig,
        storage: &VectorStorage,
    ) -> Result<Self, GraphError> {
        let mut index = Self::new(config, storage)?;
        let dimension = config.dimensions as usize;

        // Validate dimension is compatible with BQ (divisible by 8)
        if dimension % 8 != 0 {
            return Err(GraphError::InvalidDimension {
                expected: "divisible by 8",
                actual: dimension,
            });
        }

        index.bq_storage = Some(
            BinaryVectorStorage::new(dimension)
                .map_err(|e| GraphError::Storage(e.to_string()))?
        );

        Ok(index)
    }

    /// Enables binary quantization on an existing index.
    ///
    /// This creates a new BQ storage and quantizes all existing vectors.
    /// Time complexity: O(n × d) where n = vector count, d = dimension.
    ///
    /// # Errors
    ///
    /// Returns error if dimension is not divisible by 8.
    pub fn enable_bq(&mut self, storage: &VectorStorage) -> Result<(), GraphError> {
        let dimension = self.config.dimensions as usize;

        if dimension % 8 != 0 {
            return Err(GraphError::InvalidDimension {
                expected: "divisible by 8",
                actual: dimension,
            });
        }

        let mut bq_storage = BinaryVectorStorage::new(dimension)
            .map_err(|e| GraphError::Storage(e.to_string()))?;

        // Quantize all existing vectors
        for node in &self.nodes {
            if node.deleted != 0 {
                continue;
            }

            let vector = storage.get(node.vector_id)?;
            let bv = BinaryVector::quantize(&vector)
                .map_err(|e| GraphError::Quantization(e.to_string()))?;
            bq_storage.insert(&bv)
                .map_err(|e| GraphError::Storage(e.to_string()))?;
        }

        self.bq_storage = Some(bq_storage);
        Ok(())
    }

    /// Returns true if binary quantization is enabled.
    pub fn has_bq(&self) -> bool {
        self.bq_storage.is_some()
    }

    /// Returns a reference to the BQ storage, if enabled.
    pub fn bq_storage(&self) -> Option<&BinaryVectorStorage> {
        self.bq_storage.as_ref()
    }
}
```

**Dependencies:** W27.2.1 (BinaryVectorStorage)

---

### W27.3.2: Implement insert_bq()

**Objective:** Add vector with automatic binary quantization.

**Behavior:**
- Insert vector into F32 storage (existing behavior)
- Insert into HNSW graph (existing behavior)
- If BQ enabled: quantize and insert into BQ storage

**Acceptance Criteria:**
- [ ] `insert_bq(vector: &[f32], storage: &mut VectorStorage) -> Result<VectorId, GraphError>`
- [ ] Validates dimension before any mutation
- [ ] Atomically updates F32 storage, HNSW graph, and BQ storage
- [ ] Returns same VectorId as `insert()`
- [ ] Unit test: insert with BQ enabled
- [ ] Unit test: insert without BQ (falls back to insert())

**Files:**
- `src/hnsw/graph.rs` (add method)
- `tests/bq_insert.rs` (new file)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**Implementation:**

```rust
impl HnswIndex {
    /// Inserts a vector with automatic binary quantization.
    ///
    /// If BQ is enabled, the vector is stored in both F32 and BQ format.
    /// If BQ is disabled, this behaves identically to `insert()`.
    ///
    /// # Arguments
    ///
    /// * `vector` - The vector to insert.
    /// * `storage` - The F32 vector storage.
    ///
    /// # Returns
    ///
    /// The assigned vector ID.
    ///
    /// # Errors
    ///
    /// - `GraphError::DimensionMismatch` if vector dimension is wrong.
    /// - `GraphError::Quantization` if BQ quantization fails.
    pub fn insert_bq(
        &mut self,
        vector: &[f32],
        storage: &mut VectorStorage,
    ) -> Result<VectorId, GraphError> {
        // Step 1: Validate dimension
        let expected_dim = self.config.dimensions as usize;
        if vector.len() != expected_dim {
            return Err(GraphError::DimensionMismatch {
                expected: expected_dim,
                actual: vector.len(),
            });
        }

        // Step 2: Insert into F32 storage and HNSW graph
        let vector_id = self.insert(vector, storage)?;

        // Step 3: If BQ enabled, quantize and insert
        if let Some(ref mut bq_storage) = self.bq_storage {
            let bv = BinaryVector::quantize(vector)
                .map_err(|e| GraphError::Quantization(e.to_string()))?;

            // BQ storage uses same ID scheme as F32 storage
            bq_storage.insert(&bv)
                .map_err(|e| GraphError::Storage(e.to_string()))?;
        }

        Ok(vector_id)
    }
}
```

**Test Cases:**

```rust
// tests/bq_insert.rs

mod insert_bq {
    use edgevec::hnsw::{HnswConfig, HnswIndex};
    use edgevec::storage::VectorStorage;

    #[test]
    fn test_insert_bq_enabled() {
        let config = HnswConfig::new(128); // 128D, divisible by 8
        let mut storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::with_bq(config, &storage).unwrap();

        let v = vec![1.0f32; 128];
        let id = index.insert_bq(&v, &mut storage).unwrap();

        assert_eq!(id.0, 1);
        assert!(index.has_bq());
        assert_eq!(index.bq_storage().unwrap().len(), 1);
    }

    #[test]
    fn test_insert_bq_disabled_fallback() {
        let config = HnswConfig::new(128);
        let mut storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::new(config, &storage).unwrap();

        // No BQ enabled
        assert!(!index.has_bq());

        let v = vec![1.0f32; 128];
        let id = index.insert_bq(&v, &mut storage).unwrap();

        // Should still work, just no BQ storage
        assert_eq!(id.0, 1);
        assert!(index.bq_storage().is_none());
    }

    #[test]
    fn test_insert_bq_multiple() {
        let config = HnswConfig::new(768);
        let mut storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::with_bq(config, &storage).unwrap();

        for i in 0..100 {
            let v: Vec<f32> = (0..768).map(|j| (i * 768 + j) as f32).collect();
            index.insert_bq(&v, &mut storage).unwrap();
        }

        assert_eq!(index.len(), 100);
        assert_eq!(index.bq_storage().unwrap().len(), 100);
    }
}
```

**Dependencies:** W27.3.1

---

### W27.3.3: Implement search_bq()

**Objective:** Fast approximate search using Hamming distance.

**Algorithm:**
1. Quantize query vector to binary
2. Traverse HNSW graph using Hamming distance instead of F32
3. Return top-k candidates sorted by Hamming distance
4. Convert Hamming distance to approximate similarity

**Acceptance Criteria:**
- [ ] `search_bq(query: &[f32], k: usize, storage: &VectorStorage) -> Result<Vec<(VectorId, f32)>, GraphError>`
- [ ] Returns error if BQ not enabled
- [ ] Uses Hamming distance for graph traversal
- [ ] Returns approximate similarity scores (normalized Hamming)
- [ ] Unit test: search returns correct order
- [ ] Unit test: search on empty index returns empty

**Files:**
- `src/hnsw/search_bq.rs` (new file)
- `src/hnsw/mod.rs` (add module)
- `tests/bq_search.rs` (new file)

**Estimated Duration:** 6 hours

**Agent:** RUST_ENGINEER

**Implementation:**

```rust
// src/hnsw/search_bq.rs

use crate::hnsw::{GraphError, HnswIndex, VectorId};
use crate::quantization::variable::BinaryVector;
use crate::storage::VectorStorage;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

impl HnswIndex {
    /// Searches the index using binary quantization (Hamming distance).
    ///
    /// This is faster than F32 search but has lower recall.
    /// Use `search_bq_rescored()` for better recall.
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector.
    /// * `k` - Number of results to return.
    /// * `storage` - The F32 storage (needed for entry point lookup).
    ///
    /// # Returns
    ///
    /// Top-k results sorted by approximate similarity (higher is better).
    ///
    /// # Errors
    ///
    /// - `GraphError::BqNotEnabled` if BQ storage is not initialized.
    /// - `GraphError::DimensionMismatch` if query dimension is wrong.
    pub fn search_bq(
        &self,
        query: &[f32],
        k: usize,
        storage: &VectorStorage,
    ) -> Result<Vec<(VectorId, f32)>, GraphError> {
        // Validate BQ is enabled
        let bq_storage = self.bq_storage.as_ref()
            .ok_or(GraphError::BqNotEnabled)?;

        // Validate dimension
        let expected_dim = self.config.dimensions as usize;
        if query.len() != expected_dim {
            return Err(GraphError::DimensionMismatch {
                expected: expected_dim,
                actual: query.len(),
            });
        }

        // Check for empty index
        if self.entry_point.is_none() {
            return Ok(Vec::new());
        }

        // Quantize query
        let query_bq = BinaryVector::quantize(query)
            .map_err(|e| GraphError::Quantization(e.to_string()))?;

        // Use existing HNSW traversal with Hamming distance
        let candidates = self.search_bq_internal(&query_bq, k, bq_storage)?;

        // Convert to similarity scores
        let dimension = expected_dim as u32;
        let results: Vec<_> = candidates
            .into_iter()
            .map(|(id, hamming_dist)| {
                // Similarity = 1 - (hamming_dist / dimension)
                let similarity = 1.0 - (hamming_dist as f32 / dimension as f32);
                (id, similarity)
            })
            .collect();

        Ok(results)
    }

    /// Internal BQ search using Hamming distance.
    fn search_bq_internal(
        &self,
        query_bq: &BinaryVector,
        k: usize,
        bq_storage: &BinaryVectorStorage,
    ) -> Result<Vec<(VectorId, u32)>, GraphError> {
        // Similar to regular HNSW search but using Hamming distance
        // Start from entry point, descend layers, collect neighbors

        let entry = self.entry_point.ok_or(GraphError::EmptyIndex)?;
        let mut current = entry;

        // Descend from max_layer to layer 1
        for layer in (1..=self.max_layer).rev() {
            current = self.greedy_bq(current, query_bq, layer, bq_storage)?;
        }

        // Search layer 0 for k nearest
        let candidates = self.search_layer_bq(current, query_bq, k, 0, bq_storage)?;

        Ok(candidates)
    }

    /// Greedy descent using Hamming distance.
    fn greedy_bq(
        &self,
        start: NodeId,
        query_bq: &BinaryVector,
        layer: u8,
        bq_storage: &BinaryVectorStorage,
    ) -> Result<NodeId, GraphError> {
        let mut current = start;
        let mut current_dist = self.hamming_to_node(query_bq, current, bq_storage)?;

        loop {
            let mut changed = false;

            for neighbor in self.get_neighbors(current, layer) {
                let dist = self.hamming_to_node(query_bq, neighbor, bq_storage)?;
                if dist < current_dist {
                    current = neighbor;
                    current_dist = dist;
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }

        Ok(current)
    }

    /// Hamming distance from query to a node.
    fn hamming_to_node(
        &self,
        query_bq: &BinaryVector,
        node: NodeId,
        bq_storage: &BinaryVectorStorage,
    ) -> Result<u32, GraphError> {
        let vector_id = self.nodes[node.0 as usize].vector_id;
        let node_data = bq_storage.get_raw(vector_id.0)
            .ok_or(GraphError::InvalidVectorId)?;

        // Direct XOR popcount without constructing BinaryVector
        let dist = crate::simd::popcount::simd_popcount_xor(query_bq.data(), node_data);
        Ok(dist)
    }
}
```

**Test Cases:**

```rust
// tests/bq_search.rs

mod search_bq {
    use edgevec::hnsw::{HnswConfig, HnswIndex};
    use edgevec::storage::VectorStorage;

    #[test]
    fn test_search_bq_empty_index() {
        let config = HnswConfig::new(128);
        let storage = VectorStorage::new(&config, None);
        let index = HnswIndex::with_bq(config, &storage).unwrap();

        let query = vec![1.0f32; 128];
        let results = index.search_bq(&query, 10, &storage).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_search_bq_finds_similar() {
        let config = HnswConfig::new(128);
        let mut storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::with_bq(config, &storage).unwrap();

        // Insert vectors
        let v1 = vec![1.0f32; 128]; // All positive
        let v2 = vec![-1.0f32; 128]; // All negative
        let v3: Vec<f32> = (0..128).map(|i| if i < 64 { 1.0 } else { -1.0 }).collect(); // Half

        index.insert_bq(&v1, &mut storage).unwrap();
        index.insert_bq(&v2, &mut storage).unwrap();
        index.insert_bq(&v3, &mut storage).unwrap();

        // Query similar to v1
        let query = vec![1.0f32; 128];
        let results = index.search_bq(&query, 3, &storage).unwrap();

        // v1 should be most similar (Hamming = 0)
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0.0, 1); // v1 has VectorId(1)
        assert!((results[0].1 - 1.0).abs() < 0.01); // Similarity ~1.0
    }

    #[test]
    fn test_search_bq_not_enabled_error() {
        let config = HnswConfig::new(128);
        let storage = VectorStorage::new(&config, None);
        let index = HnswIndex::new(config, &storage).unwrap();

        let query = vec![1.0f32; 128];
        let result = index.search_bq(&query, 10, &storage);

        assert!(result.is_err());
    }
}
```

**Dependencies:** W27.3.1, W27.3.2

---

## Day 3 Checklist

- [x] W27.3.1: BQ storage field added to HnswIndex
- [x] W27.3.2: insert_bq() implemented
- [x] W27.3.3: search_bq() implemented
- [x] All existing tests pass (`cargo test`)
- [x] New tests pass (`cargo test search_bq`)
- [x] Clippy clean (`cargo clippy -- -D warnings`)
- [x] Formatted (`cargo fmt --check`)

## Day 3 Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | 656 tests passed |
| BQ insert works | Tests with 128D vectors |
| BQ search returns results | 6 search_bq tests pass |
| Existing search still works | All regression tests pass |

## Day 3 Handoff

After completing Day 3:

**Artifacts Generated:**
- Modified `src/hnsw/graph.rs` (BQ storage field, with_bq, enable_bq, has_bq, bq_storage)
- Modified `src/hnsw/insert.rs` (insert_bq method)
- `src/hnsw/search_bq.rs` (new file - BQ search implementation)
- Modified `src/hnsw/mod.rs` (added search_bq module)

**Status:** COMPLETE

**Next:** Day 4 — Rescoring layer for recall recovery

---

*Agent: RUST_ENGINEER*
*Status: [REVISED] (2025-12-21)*

## Implementation Notes (2025-12-21)

### Changes Made

**[C1] Added BQ storage to HnswIndex struct**
- Added `bq_storage: Option<BinaryVectorStorage>` field with `#[serde(skip)]`
- Added `BqNotEnabled` and `Quantization` error variants to `GraphError`
- Added imports for `BinaryVector` and `BinaryVectorStorage`

**[C2] Implemented BQ constructors and accessors**
- `with_bq(config, storage)` - creates index with BQ enabled
- `enable_bq(storage)` - enables BQ on existing index (quantizes all vectors)
- `has_bq() -> bool` - checks if BQ is enabled
- `bq_storage() -> Option<&BinaryVectorStorage>` - returns BQ storage reference

**[C3] Implemented insert_bq()**
- Validates dimension before any mutation
- Inserts into F32 storage and HNSW graph via `insert()`
- If BQ enabled, quantizes and inserts into BQ storage
- Falls back to regular insert if BQ disabled

**[C4] Implemented search_bq() in new search_bq.rs module**
- Validates BQ is enabled, returns `BqNotEnabled` error if not
- Quantizes query to binary
- Traverses HNSW graph using Hamming distance via `simd_popcount_xor`
- Returns similarity scores (1.0 - hamming_dist / dimension)
- Includes 6 unit tests covering edge cases

### Validation Results

| Check | Result |
|:------|:-------|
| `cargo test --lib` | 656 tests passed |
| `cargo test --doc` | 114 tests passed |
| `cargo test search_bq` | 6 tests passed |
| `cargo clippy -- -D warnings` | Clean |
| `cargo fmt --check` | Clean |

## Revision Notes (2025-12-21)

### HOSTILE_REVIEWER Rejection: 2025-12-21_W27.3_REJECTED.md

**[C1] Fixed: Removed `unwrap()` in production code**
- **Location:** `search_bq.rs:325`
- **Before:**
  ```rust
  .map(|c| {
      let node = self.nodes.get(c.node_id.0 as usize).unwrap();
      (node.vector_id, c.distance)
  })
  ```
- **After:**
  ```rust
  .filter_map(|c| {
      let node = self.nodes.get(c.node_id.0 as usize)?;
      Some((node.vector_id, c.distance))
  })
  ```
- **Rationale:** Eliminates potential panic if node index is invalid, now gracefully filters out invalid references

### Post-Revision Validation

| Check | Result |
|:------|:-------|
| `cargo test --lib` | 656 tests passed |
| `cargo test search_bq` | 6 tests passed |
| `cargo clippy -- -D warnings` | Clean |
| `cargo fmt --check` | Clean |
| No `unwrap()` in production | Verified (only in docs/tests) |
