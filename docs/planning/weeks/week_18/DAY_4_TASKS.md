# Day 4: Batch Delete API — Rust Core (W18.4)

**Date:** Week 18, Day 4
**Task ID:** W18.4
**Agent:** RUST_ENGINEER
**Status:** [REVISED]
**Revision:** v1.1 — Addresses C4 (error reporting), C5 (atomicity)

---

## Objective

Implement efficient batch deletion API that marks multiple vectors as deleted in a single operation. This addresses the "Multi-vector delete API" item from CHANGELOG "Unreleased".

---

## Context

### Current State

v0.3.0 provides single-vector soft delete:
```rust
pub fn soft_delete(&mut self, id: VectorId) -> Result<bool, EdgeVecError>;
```

### User Need

Deleting 1000 vectors requires 1000 function calls. A batch API provides:
1. Cleaner API for bulk operations
2. Potential future optimizations (batch tombstone writes)
3. Progress tracking for large deletions

---

## Deliverables

### 1. BatchDeleteResult Struct

**[C4 FIX]** Added detailed error reporting with `errors` field.

```rust
// src/hnsw/graph.rs

/// Error type for individual batch delete failures
/// [C4 FIX] Enables caller to distinguish failure reasons
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchDeleteError {
    /// Vector ID not found in index
    NotFound(VectorId),
    /// Vector was already deleted (idempotent, not an error)
    AlreadyDeleted(VectorId),
    /// Internal error during deletion
    InternalError(VectorId, String),
}

/// Result of a batch delete operation
/// [C4 FIX] Includes detailed error information
#[derive(Debug, Clone)]
pub struct BatchDeleteResult {
    /// Number of vectors successfully deleted
    pub deleted: usize,
    /// Number of vectors that were already deleted (idempotent)
    pub already_deleted: usize,
    /// Number of invalid IDs (not found in index)
    pub invalid_ids: usize,
    /// Total vectors processed
    pub total: usize,
    /// [C4 FIX] Detailed errors for failed IDs
    pub errors: Vec<BatchDeleteError>,
}

impl BatchDeleteResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self {
            deleted: 0,
            already_deleted: 0,
            invalid_ids: 0,
            total: 0,
            errors: Vec::new(),
        }
    }

    /// Check if all operations succeeded (no invalid IDs)
    pub fn all_valid(&self) -> bool {
        self.invalid_ids == 0
    }

    /// Check if any deletions occurred
    pub fn any_deleted(&self) -> bool {
        self.deleted > 0
    }

    /// Check if there were any errors (not including already-deleted)
    pub fn has_errors(&self) -> bool {
        self.invalid_ids > 0 || !self.errors.iter().all(|e| matches!(e, BatchDeleteError::AlreadyDeleted(_)))
    }
}

impl Default for BatchDeleteResult {
    fn default() -> Self {
        Self::new()
    }
}
```

### 2. soft_delete_batch Method

**[C5 FIX]** Two-phase approach with pre-validation for safety.

```rust
// src/hnsw/graph.rs

impl HnswIndex {
    /// Delete multiple vectors in a single operation
    ///
    /// **[C5 FIX] Two-Phase Implementation:**
    /// 1. Pre-validation: Check all IDs exist and are not already deleted
    /// 2. Execution: Apply deletions (guaranteed to succeed after validation)
    ///
    /// This prevents partial failures from leaving the index in an inconsistent state.
    ///
    /// # Arguments
    /// * `ids` - Slice of VectorId values to delete
    ///
    /// # Returns
    /// * `BatchDeleteResult` with counts and detailed errors
    ///
    /// # Example
    /// ```
    /// use edgevec::hnsw::{HnswConfig, HnswIndex, VectorId};
    /// use edgevec::storage::VectorStorage;
    ///
    /// let config = HnswConfig::new(4);
    /// let mut storage = VectorStorage::new(&config, None);
    /// let mut index = HnswIndex::new(config, &storage).unwrap();
    ///
    /// // Insert some vectors
    /// for i in 0..10 {
    ///     index.insert(&vec![i as f32; 4], &mut storage).unwrap();
    /// }
    ///
    /// // Batch delete
    /// let ids = vec![VectorId(1), VectorId(3), VectorId(5)];
    /// let result = index.soft_delete_batch(&ids);
    ///
    /// assert_eq!(result.deleted, 3);
    /// assert_eq!(result.total, 3);
    /// assert!(result.all_valid());
    /// ```
    pub fn soft_delete_batch(&mut self, ids: &[VectorId]) -> BatchDeleteResult {
        let mut result = BatchDeleteResult {
            deleted: 0,
            already_deleted: 0,
            invalid_ids: 0,
            total: ids.len(),
            errors: Vec::new(),
        };

        // [C5 FIX] Phase 1: Pre-validation
        // Check all IDs and categorize them BEFORE making any changes
        let mut valid_ids = Vec::with_capacity(ids.len());
        let mut already_deleted_ids = Vec::new();

        for &id in ids {
            match self.is_deleted(id) {
                Ok(true) => {
                    // Already deleted - not an error, just skip
                    already_deleted_ids.push(id);
                    result.errors.push(BatchDeleteError::AlreadyDeleted(id));
                }
                Ok(false) => {
                    // Valid and not deleted - queue for deletion
                    valid_ids.push(id);
                }
                Err(_) => {
                    // ID not found
                    result.invalid_ids += 1;
                    result.errors.push(BatchDeleteError::NotFound(id));
                }
            }
        }

        result.already_deleted = already_deleted_ids.len();

        // [C5 FIX] Phase 2: Execution
        // All IDs in valid_ids are guaranteed to exist and not be deleted
        // This phase should not fail
        for &id in &valid_ids {
            match self.soft_delete(id) {
                Ok(true) => result.deleted += 1,
                Ok(false) => {
                    // Should not happen after validation, but handle gracefully
                    result.already_deleted += 1;
                }
                Err(e) => {
                    // Should not happen after validation
                    result.errors.push(BatchDeleteError::InternalError(
                        id,
                        format!("Unexpected error after validation: {:?}", e),
                    ));
                }
            }
        }

        result
    }

    /// Delete multiple vectors with progress callback
    ///
    /// Callback is invoked approximately every 10% of progress.
    /// Useful for UI updates during large batch operations.
    ///
    /// # Arguments
    /// * `ids` - Slice of VectorId values to delete
    /// * `callback` - Function called with (processed, total) counts
    ///
    /// # Example
    /// ```
    /// use edgevec::hnsw::{HnswConfig, HnswIndex, VectorId};
    /// use edgevec::storage::VectorStorage;
    ///
    /// let config = HnswConfig::new(4);
    /// let mut storage = VectorStorage::new(&config, None);
    /// let mut index = HnswIndex::new(config, &storage).unwrap();
    ///
    /// // Insert vectors
    /// for i in 0..100 {
    ///     index.insert(&vec![i as f32; 4], &mut storage).unwrap();
    /// }
    ///
    /// // Batch delete with progress
    /// let ids: Vec<VectorId> = (1..=50).map(VectorId).collect();
    /// let result = index.soft_delete_batch_with_progress(&ids, |processed, total| {
    ///     println!("Progress: {}/{}", processed, total);
    /// });
    /// ```
    pub fn soft_delete_batch_with_progress<F>(
        &mut self,
        ids: &[VectorId],
        mut callback: F,
    ) -> BatchDeleteResult
    where
        F: FnMut(usize, usize),
    {
        let total = ids.len();
        let mut result = BatchDeleteResult {
            deleted: 0,
            already_deleted: 0,
            invalid_ids: 0,
            total,
        };

        if total == 0 {
            return result;
        }

        // Calculate progress interval (~10% increments, minimum 1)
        let interval = (total / 10).max(1);
        let mut last_callback = 0;

        for (i, &id) in ids.iter().enumerate() {
            match self.soft_delete(id) {
                Ok(true) => result.deleted += 1,
                Ok(false) => result.already_deleted += 1,
                Err(_) => result.invalid_ids += 1,
            }

            // Fire callback at ~10% intervals
            if i + 1 - last_callback >= interval || i + 1 == total {
                callback(i + 1, total);
                last_callback = i + 1;
            }
        }

        result
    }
}
```

### 3. Exports

Update `src/lib.rs`:
```rust
pub use hnsw::{BatchDeleteResult, /* existing exports */};
```

### 4. Tests

Create `tests/batch_delete.rs`:

```rust
//! Batch Delete API Tests

use edgevec::hnsw::{BatchDeleteResult, HnswConfig, HnswIndex, VectorId};
use edgevec::storage::VectorStorage;

fn create_test_index(count: usize) -> (HnswIndex, VectorStorage) {
    let config = HnswConfig::new(4);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    for i in 0..count {
        index.insert(&vec![i as f32; 4], &mut storage).unwrap();
    }

    (index, storage)
}

#[test]
fn test_batch_delete_all_valid() {
    let (mut index, _storage) = create_test_index(10);

    let ids: Vec<VectorId> = vec![VectorId(1), VectorId(3), VectorId(5)];
    let result = index.soft_delete_batch(&ids);

    assert_eq!(result.deleted, 3);
    assert_eq!(result.already_deleted, 0);
    assert_eq!(result.invalid_ids, 0);
    assert_eq!(result.total, 3);
    assert!(result.all_valid());
    assert!(result.any_deleted());
}

#[test]
fn test_batch_delete_mixed_results() {
    let (mut index, _storage) = create_test_index(10);

    // Delete one first
    index.soft_delete(VectorId(1)).unwrap();

    // Batch with: already deleted, valid, invalid
    let ids = vec![VectorId(1), VectorId(2), VectorId(999)];
    let result = index.soft_delete_batch(&ids);

    assert_eq!(result.deleted, 1);        // VectorId(2)
    assert_eq!(result.already_deleted, 1); // VectorId(1)
    assert_eq!(result.invalid_ids, 1);     // VectorId(999)
    assert_eq!(result.total, 3);
    assert!(!result.all_valid());
}

#[test]
fn test_batch_delete_empty() {
    let (mut index, _storage) = create_test_index(10);

    let result = index.soft_delete_batch(&[]);

    assert_eq!(result.deleted, 0);
    assert_eq!(result.total, 0);
    assert!(result.all_valid());
    assert!(!result.any_deleted());
}

#[test]
fn test_batch_delete_idempotent() {
    let (mut index, _storage) = create_test_index(10);

    let ids: Vec<VectorId> = (1..=5).map(VectorId).collect();

    // First batch delete
    let result1 = index.soft_delete_batch(&ids);
    assert_eq!(result1.deleted, 5);

    // Second batch delete (same IDs)
    let result2 = index.soft_delete_batch(&ids);
    assert_eq!(result2.deleted, 0);
    assert_eq!(result2.already_deleted, 5);
}

#[test]
fn test_batch_delete_with_progress() {
    let (mut index, _storage) = create_test_index(100);

    let ids: Vec<VectorId> = (1..=50).map(VectorId).collect();
    let mut progress_calls = 0;
    let mut last_processed = 0;

    let result = index.soft_delete_batch_with_progress(&ids, |processed, total| {
        progress_calls += 1;
        assert!(processed > last_processed);
        assert_eq!(total, 50);
        last_processed = processed;
    });

    assert_eq!(result.deleted, 50);
    assert!(progress_calls >= 5); // At least 5 callbacks for 50 items at ~10% intervals
}

#[test]
fn test_batch_delete_updates_counts() {
    let (mut index, _storage) = create_test_index(100);

    assert_eq!(index.deleted_count(), 0);
    assert_eq!(index.live_count(), 100);

    let ids: Vec<VectorId> = (1..=30).map(VectorId).collect();
    index.soft_delete_batch(&ids);

    assert_eq!(index.deleted_count(), 30);
    assert_eq!(index.live_count(), 70);
}

#[test]
fn test_batch_delete_result_default() {
    let result = BatchDeleteResult::default();

    assert_eq!(result.deleted, 0);
    assert_eq!(result.already_deleted, 0);
    assert_eq!(result.invalid_ids, 0);
    assert_eq!(result.total, 0);
}
```

---

## Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC18.4.1 | `soft_delete_batch(&[VectorId]) -> BatchDeleteResult` | Unit test |
| AC18.4.2 | Returns count of successfully deleted vectors | Unit test |
| AC18.4.3 | Partial failure handling (some IDs invalid) | Unit test |
| AC18.4.4 | Progress callback support (optional) | Unit test |
| AC18.4.5 | Performance: batch no slower than N individual calls | Benchmark |
| AC18.4.6 | `BatchDeleteResult` struct exported | `cargo doc` |
| AC18.4.7 | Rustdoc examples compile and pass | `cargo test --doc` |

---

## Implementation Plan

### Step 1: Add BatchDeleteResult Struct

Add to `src/hnsw/graph.rs`.

### Step 2: Implement soft_delete_batch

Add methods to `impl HnswIndex`.

### Step 3: Update Exports

Add to `src/lib.rs`.

### Step 4: Create Tests

Add `tests/batch_delete.rs`.

### Step 5: Run Tests

```bash
cargo test batch_delete
cargo test --doc
```

---

## Files to Create/Modify

| File | Action | Description |
|:-----|:-------|:------------|
| `src/hnsw/graph.rs` | MODIFY | Add BatchDeleteResult + methods |
| `src/lib.rs` | MODIFY | Export BatchDeleteResult |
| `tests/batch_delete.rs` | CREATE | Test suite |

---

## Verification Commands

```bash
# Run batch delete tests
cargo test batch_delete --verbose

# Run doc tests
cargo test --doc

# Check exports
cargo doc --open
# Search for "BatchDeleteResult"
```

---

## Handoff

**On Completion:**
- Mark W18.4 as COMPLETE
- Submit for hostile review
- Proceed to W18.5 (WASM bindings)
