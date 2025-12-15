# Week 16 — Day 2 Tasks

**Date:** Day 2 of Week 16
**Focus:** Delete API Implementation
**Agent:** RUST_ENGINEER
**Status:** [REVISED]

---

## REVISION NOTES (Post-Hostile Review)

**Addressed Issues:**
- C-AC-1: Corrected complexity documentation — O(n) lookup + O(1) mutation, NOT O(1)
- M-DEP-2: Listed specific deliverables required by W16.3

---

## Day Objective

Implement the core delete API methods: `delete()`, `is_deleted()`, `deleted_count()`, and `tombstone_ratio()`. These provide O(1) soft delete functionality.

**Success Criteria:**
- `delete(VectorId)` marks vector as deleted
- `is_deleted(VectorId)` checks deletion status
- `deleted_count()` and `tombstone_ratio()` work correctly
- Comprehensive unit tests pass

---

## Tasks

### W16.2: Implement Delete API Methods

**Priority:** P0 (Core API)
**Estimate:** 6h (2h base × 3x)
**Agent:** RUST_ENGINEER
**Depends On:** W16.1 (deleted field)

#### Scope

- [x] **AC16.2.1:** `soft_delete(VectorId) -> Result<bool, GraphError>` (renamed from `delete()`)
- [x] **AC16.2.2:** `is_deleted(VectorId) -> Result<bool, GraphError>`
- [x] **AC16.2.3:** `deleted_count() -> usize`
- [x] **AC16.2.4:** `tombstone_ratio() -> f64`
- [x] **AC16.2.5:** Double-delete returns `Ok(false)`
- [x] **AC16.2.6:** Delete of non-existent ID returns error
- [x] **AC16.2.7:** `get_node_mut()` helper added

#### Implementation Specification

**File:** `src/hnsw/graph.rs`

##### Helper Method: get_node_mut

```rust
impl HnswIndex {
    /// Get mutable reference to a node by VectorId
    fn get_node_mut(&mut self, vector_id: VectorId) -> Result<&mut HnswNode, GraphError> {
        // Find node with matching vector_id
        self.nodes
            .iter_mut()
            .find(|n| n.vector_id == vector_id)
            .ok_or(GraphError::InvalidVectorId)
    }

    /// Get immutable reference to a node by VectorId
    fn get_node_by_vector_id(&self, vector_id: VectorId) -> Result<&HnswNode, GraphError> {
        self.nodes
            .iter()
            .find(|n| n.vector_id == vector_id)
            .ok_or(GraphError::InvalidVectorId)
    }
}
```

##### Delete Methods

```rust
impl HnswIndex {
    /// Mark a vector as deleted (soft delete)
    ///
    /// The vector remains in the graph for routing but is excluded from
    /// search results. Space is reclaimed via `compact()`.
    ///
    /// # Arguments
    ///
    /// * `vector_id` - The ID of the vector to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Vector was deleted
    /// * `Ok(false)` - Vector was already deleted
    /// * `Err(InvalidVectorId)` - Vector ID not found
    ///
    /// # Complexity (REVISED — C-AC-1 Fix)
    ///
    /// * Time: **O(n)** for VectorId → NodeId lookup via linear scan
    ///         + O(1) for setting the deleted byte
    /// * Space: O(1)
    ///
    /// **Note:** The O(n) lookup is a known limitation in v0.3.0.
    /// A HashMap<VectorId, NodeId> index could provide O(1) lookup
    /// but is deferred to v0.4.0 to avoid scope creep.
    ///
    /// # Persistence
    ///
    /// **IMPORTANT:** Delete operations are in-memory only until `save()` is called.
    /// If the process crashes before `save()`, the delete will be lost.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let deleted = index.delete(VectorId(42))?;
    /// assert!(deleted);
    /// assert!(index.is_deleted(VectorId(42))?);
    /// ```
    pub fn delete(&mut self, vector_id: VectorId) -> Result<bool, GraphError> {
        let node = self.get_node_mut(vector_id)?;

        if node.deleted != 0 {
            return Ok(false); // Already deleted
        }

        node.deleted = 1;
        self.deleted_count += 1;
        Ok(true)
    }

    /// Check if a vector is marked as deleted
    ///
    /// # Arguments
    ///
    /// * `vector_id` - The ID of the vector to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Vector is deleted
    /// * `Ok(false)` - Vector is live
    /// * `Err(InvalidVectorId)` - Vector ID not found
    pub fn is_deleted(&self, vector_id: VectorId) -> Result<bool, GraphError> {
        let node = self.get_node_by_vector_id(vector_id)?;
        Ok(node.deleted != 0)
    }

    /// Get the count of deleted (tombstoned) vectors
    ///
    /// # Returns
    ///
    /// Number of vectors marked as deleted
    #[must_use]
    pub fn deleted_count(&self) -> usize {
        self.deleted_count
    }

    /// Get the ratio of deleted vectors to total vectors
    ///
    /// # Returns
    ///
    /// A value between 0.0 and 1.0 representing the tombstone ratio.
    /// Returns 0.0 if the index is empty.
    #[must_use]
    pub fn tombstone_ratio(&self) -> f64 {
        let total = self.node_count();
        if total == 0 {
            return 0.0;
        }
        self.deleted_count as f64 / total as f64
    }

    /// Get count of live (non-deleted) vectors
    #[must_use]
    pub fn live_count(&self) -> usize {
        self.node_count().saturating_sub(self.deleted_count)
    }
}
```

#### Error Handling

Update `src/hnsw/graph.rs` GraphError:

```rust
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    // ... existing variants ...

    /// Vector ID not found in index
    #[error("vector id not found: {0:?}")]
    VectorNotFound(VectorId),
}
```

#### Test Cases

**File:** `src/hnsw/graph.rs` (in `mod tests`)

```rust
mod delete_tests {
    use super::*;

    fn create_test_index() -> (HnswIndex, VectorStorage) {
        let config = HnswConfig::new(4);
        let storage = VectorStorage::new(&config, None);
        let index = HnswIndex::new(config, &storage).unwrap();
        (index, storage)
    }

    #[test]
    fn test_delete_marks_node() {
        let (mut index, mut storage) = create_test_index();
        let vec = vec![1.0, 2.0, 3.0, 4.0];
        let id = index.insert(&vec, &mut storage).unwrap();

        assert!(!index.is_deleted(id).unwrap());
        assert!(index.delete(id).unwrap());
        assert!(index.is_deleted(id).unwrap());
    }

    #[test]
    fn test_delete_idempotent() {
        let (mut index, mut storage) = create_test_index();
        let vec = vec![1.0, 2.0, 3.0, 4.0];
        let id = index.insert(&vec, &mut storage).unwrap();

        assert!(index.delete(id).unwrap());  // First: true
        assert!(!index.delete(id).unwrap()); // Second: false
        assert_eq!(index.deleted_count(), 1); // Still 1
    }

    #[test]
    fn test_delete_nonexistent_fails() {
        let (mut index, _storage) = create_test_index();
        let result = index.delete(VectorId(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_deleted_count() {
        let (mut index, mut storage) = create_test_index();

        // Insert 3 vectors
        let id1 = index.insert(&[1.0, 0.0, 0.0, 0.0], &mut storage).unwrap();
        let id2 = index.insert(&[0.0, 1.0, 0.0, 0.0], &mut storage).unwrap();
        let _id3 = index.insert(&[0.0, 0.0, 1.0, 0.0], &mut storage).unwrap();

        assert_eq!(index.deleted_count(), 0);
        assert_eq!(index.node_count(), 3);

        // Delete 2
        index.delete(id1).unwrap();
        index.delete(id2).unwrap();

        assert_eq!(index.deleted_count(), 2);
        assert_eq!(index.live_count(), 1);
    }

    #[test]
    fn test_tombstone_ratio() {
        let (mut index, mut storage) = create_test_index();

        // Empty index
        assert_eq!(index.tombstone_ratio(), 0.0);

        // Insert 4 vectors
        for i in 0..4 {
            index.insert(&[i as f32; 4], &mut storage).unwrap();
        }

        assert_eq!(index.tombstone_ratio(), 0.0);

        // Delete 1 of 4 = 25%
        index.delete(VectorId(1)).unwrap();
        assert!((index.tombstone_ratio() - 0.25).abs() < 0.01);

        // Delete 2 of 4 = 50%
        index.delete(VectorId(2)).unwrap();
        assert!((index.tombstone_ratio() - 0.50).abs() < 0.01);
    }

    #[test]
    fn test_is_deleted_nonexistent_fails() {
        let (index, _storage) = create_test_index();
        let result = index.is_deleted(VectorId(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_live_count() {
        let (mut index, mut storage) = create_test_index();

        // Insert 5, delete 2
        for i in 0..5 {
            index.insert(&[i as f32; 4], &mut storage).unwrap();
        }

        index.delete(VectorId(1)).unwrap();
        index.delete(VectorId(2)).unwrap();

        assert_eq!(index.node_count(), 5);
        assert_eq!(index.deleted_count(), 2);
        assert_eq!(index.live_count(), 3);
    }
}
```

#### Verification Commands

```bash
# Run delete tests
cargo test delete_tests

# Run all tests
cargo test --all

# Clippy check
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

---

## Day 2 Summary

**Total Effort:** 6h scheduled

**Deliverables:**
1. `delete()` method
2. `is_deleted()` method
3. `deleted_count()` method
4. `tombstone_ratio()` method
5. `live_count()` method
6. Comprehensive unit tests

**Day 3 Preview:**
- Update search to filter tombstones
- Implement `adjusted_k()` for result quality

---

## Deliverables Required by W16.3 (M-DEP-2 Fix)

W16.3 (Search Filtering) depends on these specific W16.2 deliverables:

| Deliverable | Method | Used In W16.3 For |
|:------------|:-------|:------------------|
| `deleted` field | `HnswNode.deleted` | Filter check in search |
| `deleted_count` field | `HnswIndex.deleted_count` | adjusted_k() calculation |
| `tombstone_ratio()` | `HnswIndex::tombstone_ratio()` | adjusted_k() multiplier |
| `is_deleted()` | `HnswIndex::is_deleted()` | Test verification |

**W16.3 MUST NOT start until all above are implemented and tested.**

---

## HOSTILE_REVIEWER Pre-Flight

Before end of day:

- [x] `soft_delete()` returns `Ok(true)` on first delete
- [x] `soft_delete()` returns `Ok(false)` on double-delete
- [x] `soft_delete()` returns `Err` on non-existent ID
- [x] `is_deleted()` correctly reports status
- [x] `deleted_count()` increments correctly
- [x] `tombstone_ratio()` calculated correctly
- [x] All new tests pass (11 delete tests)
- [x] Clippy clean

---

**Status:** [IMPLEMENTED]
**Implementation Date:** 2025-12-14
**Next:** `/review W16.2`
