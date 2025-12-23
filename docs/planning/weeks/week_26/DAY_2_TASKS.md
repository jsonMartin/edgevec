# Week 26 Day 2: soft_delete + compact + search_filtered

**Date:** 2025-12-22
**Focus:** Metadata lifecycle operations and basic filtered search
**Estimated Duration:** 8 hours
**Phase:** RFC-002 Core Metadata (Phase 1)

---

## Tasks

### W26.2.1: Modify soft_delete() for metadata cleanup

**Objective:** Automatically remove metadata when vector is soft-deleted per RFC-002 §2.3.

**Acceptance Criteria:**
- [ ] `soft_delete()` calls `self.metadata.remove(id)` after marking tombstone
- [ ] No orphaned metadata after delete (metadata removed even if vector didn't have any)
- [ ] Unit test verifies metadata removal
- [ ] Existing soft_delete tests still pass

**Files:**
- `src/hnsw/operations.rs` (primary — modify soft_delete method)
- `tests/metadata_delete.rs` (new file — deletion tests)

**Estimated Duration:** 2 hours

**Agent:** RUST_ENGINEER

**Implementation Notes:**
1. Add `self.metadata.remove(id)` at end of soft_delete() success path
2. Remove should be idempotent (no error if metadata doesn't exist)
3. Ensure deletion happens AFTER tombstone is set (for consistency)

**Dependencies:** W26.1.1 (metadata field must exist in HnswIndex)

---

### W26.2.2: Modify compact() for metadata

**Objective:** Remove metadata for compacted (tombstoned) vectors per RFC-002 §2.3.

**Acceptance Criteria:**
- [ ] `compact()` collects all deleted IDs during compaction
- [ ] Calls `self.metadata.remove(id)` for each compacted ID
- [ ] Metadata store shrinks after compaction (no orphaned entries)
- [ ] Unit test verifies metadata compaction
- [ ] Existing compact tests still pass

**Files:**
- `src/hnsw/operations.rs` (primary — modify compact method)
- `tests/metadata_compact.rs` (new file — compaction tests)

**Estimated Duration:** 2 hours

**Agent:** RUST_ENGINEER

**Implementation Notes:**
1. During compact(), track which IDs are being removed
2. After successful compaction, batch-remove metadata for those IDs
3. Consider memory: collect deleted IDs first, then remove in bulk
4. Verify: after compact(), metadata.len() equals remaining vector count

**Dependencies:** W26.1.1 (metadata field must exist in HnswIndex)

---

### W26.2.3: Implement search_filtered() basic

**Objective:** Post-filtering with adaptive overfetch per RFC-002 §3.2.

**Acceptance Criteria:**
- [ ] `search_filtered(&self, storage: &VectorStorage, query: &[f32], filter: &str, k: usize) -> Result<Vec<(VectorId, f32)>, GraphError>` implemented
- [ ] Parses filter using existing `Filter::parse()` from filter module
- [ ] Applies overfetch factor: `min(10, max(2, 1 / selectivity))` with default selectivity 0.50
- [ ] Returns top-k results passing filter (sorted by distance ascending)
- [ ] Empty result if no vectors pass filter (not an error)
- [ ] Unit tests for various filter expressions

**Files:**
- `src/hnsw/search.rs` (primary — add search_filtered method)
- `tests/metadata_search.rs` (new file — filtered search tests)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**API Signature (from RFC-002 §3.1):**
```rust
impl HnswIndex {
    /// Search with filter expression evaluated in Rust.
    ///
    /// # Arguments
    /// * `storage` - The vector storage
    /// * `query` - The query vector
    /// * `filter` - Filter expression string (e.g., "price < 50")
    /// * `k` - Number of results
    ///
    /// # Returns
    /// Vector of (VectorId, distance) tuples for matching vectors.
    ///
    /// # Algorithm
    /// 1. Parse filter expression
    /// 2. Estimate selectivity (default 0.50)
    /// 3. Overfetch k * overfetch_factor candidates
    /// 4. Filter candidates using metadata
    /// 5. Return top-k passing filter
    pub fn search_filtered(
        &self,
        storage: &VectorStorage,
        query: &[f32],
        filter: &str,
        k: usize,
    ) -> Result<Vec<(VectorId, f32)>, GraphError>;
}
```

**Implementation Notes:**
1. Use existing `Filter::parse(filter)` to parse expression
2. Default selectivity = 0.50 (will be refined in W26.3.1)
3. Call existing `search()` with `k * overfetch_factor`
4. For each result: lookup metadata, evaluate filter, collect passing
5. Return first k passing results (already sorted by distance)
6. Handle edge case: if fewer than k pass filter, return what we have

**Dependencies:**
- W26.1.1 (metadata field must exist)
- W26.1.2 (insert_with_metadata for test data)
- Existing Filter module (`src/filter/`)

---

## Day 2 Checklist

- [x] W26.2.1: soft_delete() calls metadata.remove()
- [x] W26.2.2: compact() removes metadata for compacted IDs
- [x] W26.2.3: search_filtered() implemented with post-filtering
- [x] All existing tests pass (`cargo test`)
- [x] New tests pass (`cargo test metadata_delete metadata_compact metadata_search`)
- [x] Clippy clean (`cargo clippy -- -D warnings`)
- [x] Formatted (`cargo fmt --check`)

## Day 2 Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | CI green |
| No orphaned metadata | Unit tests verify |
| search_filtered returns correct results | Filter tests pass |
| No breaking changes | Existing code compiles |

## Day 2 Handoff

After completing Day 2:

**Artifacts Generated:**
- Modified `src/hnsw/operations.rs` (soft_delete, compact)
- Modified `src/hnsw/search.rs` (search_filtered)
- New `tests/metadata_delete.rs`
- New `tests/metadata_compact.rs`
- New `tests/metadata_search.rs`

**Status:** APPROVED (2025-12-21)

**Next:** Day 3 — Selectivity estimation + comprehensive unit tests

---

*Agent: RUST_ENGINEER*
*Status: [APPROVED]*
