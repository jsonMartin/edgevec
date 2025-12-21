# Week 26 Day 1: HnswIndex Metadata Integration

**Date:** 2025-12-21
**Focus:** Add metadata field to HnswIndex and implement insert_with_metadata()
**Estimated Duration:** 8 hours
**Phase:** RFC-002 Core Metadata (Phase 1)

---

## Tasks

### W26.1.1: Add metadata field to HnswIndex

**Objective:** Integrate existing MetadataStore into HnswIndex struct per RFC-002 §2.1.

**Acceptance Criteria:**
- [ ] `metadata: MetadataStore` field added to `HnswIndex` struct
- [ ] `HnswIndex::new()` initializes empty MetadataStore
- [ ] `HnswIndex::with_config()` initializes empty MetadataStore
- [ ] `HnswIndex::with_metadata()` constructor added for initialization with metadata
- [ ] Existing tests still pass (`cargo test`)
- [ ] No breaking changes to public API

**Files:**
- `src/hnsw/graph.rs` (primary — add field to struct)
- `src/hnsw/mod.rs` (re-export if needed)
- `src/lib.rs` (re-export if needed)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**Constraints (from RFC-002 §2.1):**
```rust
pub struct HnswIndex {
    // Existing fields (unchanged)
    pub config: HnswConfig,
    pub(crate) nodes: Vec<HnswNode>,
    pub(crate) neighbors: NeighborPool,
    pub(crate) entry_point: Option<NodeId>,
    pub(crate) max_layer: u8,
    pub(crate) level_mult: f32,
    rng: ChaCha8Rng,
    pub(crate) deleted_count: usize,
    compaction_threshold: f64,

    // NEW: Integrated metadata storage
    pub(crate) metadata: MetadataStore,
}
```

**Thread Safety Notes (from RFC-002 §2.1.1):**
- `MetadataStore` is `Send + Sync` when all contained types are `Send + Sync`
- `String` and `MetadataValue` are both `Send + Sync`
- Concurrent modification requires external synchronization (matches existing pattern)

**Dependencies:** None

---

### W26.1.2: Implement insert_with_metadata()

**Objective:** Atomic vector + metadata insert with fail-fast validation per RFC-002 §3.1.

**Acceptance Criteria:**
- [ ] `insert_with_metadata(&mut self, storage: &mut VectorStorage, vector: &[f32], metadata: HashMap<String, MetadataValue>) -> Result<VectorId, GraphError>` implemented
- [ ] Validates metadata limits BEFORE any mutation:
  - 64 keys per vector maximum
  - 256 bytes per key name maximum
  - 64KB per string value maximum
  - 1024 elements per string array maximum
- [ ] On validation failure: returns `GraphError::MetadataValidation` (no partial state)
- [ ] On success: both vector and metadata atomically inserted
- [ ] Unit tests cover:
  - Success path with valid metadata
  - Failure path: too many keys
  - Failure path: key name too long
  - Failure path: value too large
  - Verify rollback on failure (no orphaned data)
- [ ] Documentation with examples

**Files:**
- `src/hnsw/operations.rs` (primary — implement method)
- `src/error.rs` (add `MetadataValidation` variant if not exists)
- `tests/metadata_insert.rs` (new file — unit tests)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**API Signature (from RFC-002 §3.1):**
```rust
impl HnswIndex {
    /// Insert a vector with metadata atomically.
    ///
    /// # Arguments
    /// * `storage` - The vector storage
    /// * `vector` - The vector data
    /// * `metadata` - Key-value metadata pairs
    ///
    /// # Returns
    /// The assigned vector ID
    ///
    /// # Errors
    /// Returns `GraphError::MetadataValidation` if metadata validation fails.
    /// The index remains unchanged on error.
    pub fn insert_with_metadata(
        &mut self,
        storage: &mut VectorStorage,
        vector: &[f32],
        metadata: HashMap<String, MetadataValue>,
    ) -> Result<VectorId, GraphError>;
}
```

**Implementation Notes:**
1. Validate metadata FIRST (before calling `insert()`)
2. Call existing `insert()` to add vector
3. Add metadata to MetadataStore with returned VectorId
4. On any error: ensure no partial state (currently insert is atomic)

**Dependencies:** W26.1.1 (metadata field must exist)

---

## Day 1 Checklist

- [ ] W26.1.1: metadata field added to HnswIndex
- [ ] W26.1.2: insert_with_metadata() implemented
- [ ] All existing tests pass (`cargo test`)
- [ ] New tests pass (`cargo test metadata_insert`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Formatted (`cargo fmt --check`)

## Day 1 Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | CI green |
| New tests added | `tests/metadata_insert.rs` exists |
| No breaking changes | Existing code compiles |
| Documentation | Rustdoc for new methods |

## Day 1 Handoff

After completing Day 1:

**Artifacts Generated:**
- Modified `src/hnsw/graph.rs` (metadata field)
- Modified `src/hnsw/operations.rs` (insert_with_metadata)
- New `tests/metadata_insert.rs`

**Status:** PENDING_DAY_2

**Next:** Day 2 — soft_delete cleanup + compact + search_filtered

---

*Agent: RUST_ENGINEER*
*Status: [PROPOSED]*
