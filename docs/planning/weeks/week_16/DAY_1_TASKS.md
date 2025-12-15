# Week 16 — Day 1 Tasks

**Date:** Day 1 of Week 16
**Focus:** Field Rename (`pad` → `deleted`)
**Agent:** RUST_ENGINEER
**Status:** [REVISED]

---

## Day Objective

Rename the `pad` field to `deleted` in `HnswNode` struct and add `deleted_count` tracking to `HnswIndex`. This establishes the data model for soft delete functionality.

**Success Criteria:**
- `HnswNode.deleted` field exists
- `HnswIndex.deleted_count` field exists
- All existing tests pass
- Struct size unchanged at 16 bytes

---

## Tasks

### W16.1: Rename `pad` → `deleted` in HnswNode

**Priority:** P0 (Foundation - blocks all other Week 16 tasks)
**Estimate:** 4h (1.3h base × 3x)
**Agent:** RUST_ENGINEER

#### Scope

- [x] **AC16.1.1:** Rename `pub pad: u8` to `pub deleted: u8` in HnswNode
- [x] **AC16.1.2:** Add `deleted_count: usize` field to HnswIndex
- [x] **AC16.1.3:** Verify HnswNode size unchanged (16 bytes)
- [x] **AC16.1.4:** Update all code referencing `pad` field
- [x] **AC16.1.5:** Update documentation comments in the following locations:
  - `HnswNode` struct doc comment: Add "# Soft Delete (v0.3.0)" section (lines 128-138 in graph.rs)
  - `deleted` field doc comment: Document semantic values 0=live, 1=deleted
  - `HnswIndex` struct doc comment: Add soft delete usage note
  - `deleted_count` field doc comment: Explain tracking purpose

#### Implementation Specification

**File:** `src/hnsw/graph.rs`

##### Before (v0.2.x)

```rust
/// A node in the HNSW graph with its adjacency information.
///
/// # Layout
///
/// Total size: 16 bytes
/// Alignment: 8 bytes
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct HnswNode {
    /// The vector ID this node represents
    pub vector_id: VectorId,

    /// Offset into COMPRESSED neighbor pool
    pub neighbor_offset: u32,

    /// Length of neighbor data in bytes (Allocated Capacity)
    pub neighbor_len: u16,

    /// The maximum layer this node appears in
    pub max_layer: u8,

    /// Explicit padding byte (always zero-initialized for Pod safety)
    pub pad: u8,
}
```

##### After (v0.3.0)

```rust
/// A node in the HNSW graph with its adjacency information.
///
/// # Layout
///
/// Total size: 16 bytes
/// Alignment: 8 bytes
///
/// # Soft Delete (v0.3.0)
///
/// The `deleted` field enables O(1) soft delete. Deleted nodes remain in
/// the graph for routing but are excluded from search results.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct HnswNode {
    /// The vector ID this node represents
    pub vector_id: VectorId,

    /// Offset into COMPRESSED neighbor pool
    pub neighbor_offset: u32,

    /// Length of neighbor data in bytes (Allocated Capacity)
    pub neighbor_len: u16,

    /// The maximum layer this node appears in
    pub max_layer: u8,

    /// Soft delete flag: 0 = live, 1 = deleted (v0.3.0)
    /// This field replaces the padding byte from v0.2.x.
    pub deleted: u8,
}
```

##### HnswIndex Changes

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HnswIndex {
    /// Algorithm configuration
    pub config: HnswConfig,

    /// Node metadata (fixed-size per node)
    pub(crate) nodes: Vec<HnswNode>,

    /// Compressed neighbor lists
    pub(crate) neighbors: NeighborPool,

    /// Entry point (highest layer node)
    pub(crate) entry_point: Option<NodeId>,

    /// Maximum layer in the graph
    pub(crate) max_layer: u8,

    /// Level probability multiplier (1/ln(M))
    pub(crate) level_mult: f32,

    /// Deterministic RNG state
    rng: ChaCha8Rng,

    /// Count of soft-deleted vectors (v0.3.0)
    pub(crate) deleted_count: usize,
}
```

#### Code Changes Required

1. **`src/hnsw/graph.rs`**
   - Rename `pad` → `deleted` in HnswNode
   - Add `deleted_count: usize` to HnswIndex
   - Initialize `deleted_count: 0` in `HnswIndex::new()`
   - Update `add_node()` to set `deleted: 0`

2. **Search for `pad` references**
   ```bash
   grep -r "\.pad" src/ tests/ benches/
   ```

3. **`examples/size_check.rs`**
   - Update to check `deleted` field instead of `pad`

#### Verification Commands

```bash
# Verify struct size unchanged
cargo run --example size_check

# Run all tests
cargo test --all

# Check for any remaining `pad` references
grep -r "\.pad" src/ tests/

# Clippy check
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

#### Test Cases

```rust
#[test]
fn test_hnsw_node_has_deleted_field() {
    let node = HnswNode {
        vector_id: VectorId(1),
        neighbor_offset: 0,
        neighbor_len: 0,
        max_layer: 0,
        deleted: 0,  // New field
    };
    assert_eq!(node.deleted, 0);
}

#[test]
fn test_hnsw_index_tracks_deleted_count() {
    let config = HnswConfig::new(128);
    let storage = VectorStorage::new(&config, None);
    let index = HnswIndex::new(config, &storage).unwrap();
    assert_eq!(index.deleted_count, 0);
}

#[test]
fn test_hnsw_node_size_unchanged() {
    assert_eq!(std::mem::size_of::<HnswNode>(), 16);
    assert_eq!(std::mem::align_of::<HnswNode>(), 8);
}
```

#### Risks

- **R16.1.1:** `bytemuck::Pod` derivation may fail if field order changes
  - **Mitigation:** Only rename, don't reorder fields
- **R16.1.2:** Serialization format may break
  - **Mitigation:** `pad` was always 0, `deleted` starts as 0

#### Rollback Plan

If implementation fails or introduces regressions:

1. **Immediate:** `git checkout src/hnsw/graph.rs examples/size_check.rs`
2. **Verify:** `cargo test --all` passes after rollback
3. **Document:** Create `docs/reviews/[DATE]_W16.1_ROLLBACK.md` with failure reason
4. **Escalate:** Report to HOSTILE_REVIEWER for root cause analysis

**Recovery Point:** Commit `49f6b4b` (W15.1 Runtime SIMD detection system)

---

## Day 1 Summary

**Total Effort:** 4h scheduled

**Deliverables:**
1. `src/hnsw/graph.rs` with renamed field
2. Updated `examples/size_check.rs`
3. All tests passing

**Day 2 Preview:**
- Implement `delete()`, `is_deleted()` methods
- Error handling for delete operations

---

## HOSTILE_REVIEWER Pre-Flight

Before end of day:

- [ ] `HnswNode.deleted` field exists
- [ ] `HnswIndex.deleted_count` field exists
- [ ] `examples/size_check.rs` passes (16 bytes)
- [ ] `cargo test --all` passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] No remaining `.pad` references in code

---

**Status:** [APPROVED] ✅
**Implementation:** COMPLETE (2025-12-14)
**Review:** `docs/reviews/2025-12-14_W16.1_HOSTILE_REVIEW.md`
**Next:** Proceed to Day 2 → `/rust-implement W16.2`
