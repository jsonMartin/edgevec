# Week 16 Task Plan — Soft Delete Core Implementation

**Sprint:** Week 16 (Following Week 15 completion)
**Phase:** 4.6 (v0.3.0 Feature Development)
**Status:** [REVISED]
**PLANNER:** Week 16 Planning
**Date Created:** 2025-12-14
**Date Revised:** 2025-12-14

---

## REVISION NOTES (Post-Hostile Review)

**Review Reference:** `docs/reviews/2025-12-14_WEEK16_PLAN_HOSTILE_REVIEW.md`

**Issues Addressed:**

| ID | Issue | Resolution |
|:---|:------|:-----------|
| C-RISK-1 | Storage/Index mismatch after compact | Redesigned `compact()` to return `(HnswIndex, VectorStorage, CompactionResult)` tuple |
| C-DEP-1 | Missing VectorStorage.get_vector() verification | Verified exists at `src/storage.rs:448` |
| C-AC-1 | O(n) complexity documented as O(1) | Fixed: documented as O(n) lookup + O(1) mutation |
| C-AC-2 | Missing memory safety ACs for compact() | Added AC16.4.8-10 |
| M-DEP-1 | get_node() vs get_node_by_vector_id() ambiguity | Clarified: `get_node(NodeId)` exists at line 337, O(1) |
| M-DEP-2 | Incomplete W16.2→W16.3 deliverable list | Added explicit deliverable table |
| M-AC-1 | No performance baseline defined | Added concrete baseline specification |
| M-RISK-1 | insert_with_id collision risk underestimated | Updated R16.5 to MEDIUM probability |

---

## Executive Summary

Week 16 implements the core soft delete functionality per RFC-001 (approved in Week 15). This is the first major feature development for v0.3.0, enabling vector deletion without full index rebuild.

**Key Insight:** The `deleted` field replaces the existing `pad` byte in `HnswNode`, resulting in **zero memory overhead**.

**Goal:** Implement delete(), is_deleted(), compact(), and persistence v3 format.

---

## Week 16 Context

### Previous Week (Week 15) Accomplishments

| Task | Status | Key Deliverable |
|:-----|:-------|:----------------|
| W15.1 | ✅ APPROVED | `src/simd/detect.rs` - Runtime SIMD detection |
| W15.2 | ✅ APPROVED | `benches/recall_bench.rs` - GloVe-100D harness |
| W15.3 | ✅ APPROVED | `docs/rfcs/RFC-001-soft-delete.md` - Zero-overhead design |
| W15.4 | ✅ APPROVED | `docs/BROWSER_COMPATIBILITY.md` - 4-browser matrix |

**Gate Status:** `.claude/GATE_15_COMPLETE.md` ✅

### RFC-001 Design Summary

The approved RFC-001 specifies:

1. **Tombstone Design:** Inline `deleted: u8` field (replaces `pad` byte)
2. **Delete API:** O(1) soft delete by setting byte to 1
3. **Search Behavior:** Filter tombstones during candidate evaluation
4. **Compaction:** Full rebuild when tombstone_ratio > 30%
5. **Persistence:** Snapshot format v3 with `deleted_count` header

### Current HnswNode Structure (v0.2.x)

```rust
pub struct HnswNode {
    pub vector_id: VectorId,     // 8 bytes
    pub neighbor_offset: u32,    // 4 bytes
    pub neighbor_len: u16,       // 2 bytes
    pub max_layer: u8,           // 1 byte
    pub pad: u8,                 // 1 byte ← BECOMES `deleted`
}
```

**Size:** 16 bytes (unchanged after modification)

---

## Task Overview

| Day | Task ID | Focus | Agent | Hours |
|:----|:--------|:------|:------|:------|
| **Day 1** | W16.1 | Rename `pad` → `deleted` + `deleted_count` | RUST_ENGINEER | 4h |
| **Day 2** | W16.2 | Implement `delete()`, `is_deleted()` API | RUST_ENGINEER | 6h |
| **Day 3** | W16.3 | Update search to filter tombstones | RUST_ENGINEER | 6h |
| **Day 4** | W16.4 | Implement `compact()` + `insert_with_id()` | RUST_ENGINEER | 8h |
| **Day 5** | W16.5 | Update persistence format to v3 | RUST_ENGINEER | 6h |

**Total Planned:** 30h + 10h buffer = 40h
**Buffer Allocation:** 25%

---

## Day 1: Field Rename (W16.1)

### W16.1: Rename `pad` → `deleted` in HnswNode

**Agent:** RUST_ENGINEER
**Estimate:** 4h (1.3h base × 3x)
**Priority:** P0 (Foundation)

#### Objective

Rename the `pad` field to `deleted` in `HnswNode` struct and add `deleted_count` to `HnswIndex`. This is a mechanical change that establishes the data model for soft delete.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC16.1.1 | `HnswNode.pad` renamed to `HnswNode.deleted` | `grep -q "pub deleted: u8" src/hnsw/graph.rs` |
| AC16.1.2 | `HnswIndex.deleted_count: usize` field added | Struct definition |
| AC16.1.3 | `HnswNode` size unchanged at 16 bytes | `examples/size_check.rs` passes |
| AC16.1.4 | All existing tests pass | `cargo test --all` |
| AC16.1.5 | Documentation updated | Rustdoc comments |

#### Implementation

```rust
// Before (v0.2.x)
pub struct HnswNode {
    pub vector_id: VectorId,
    pub neighbor_offset: u32,
    pub neighbor_len: u16,
    pub max_layer: u8,
    pub pad: u8,  // ← RENAME
}

// After (v0.3.0)
pub struct HnswNode {
    pub vector_id: VectorId,
    pub neighbor_offset: u32,
    pub neighbor_len: u16,
    pub max_layer: u8,
    pub deleted: u8,  // 0 = live, 1 = deleted
}
```

#### Files to Modify

1. `src/hnsw/graph.rs` - HnswNode struct, HnswIndex struct
2. `src/hnsw/mod.rs` - Exports if needed
3. `examples/size_check.rs` - Verify unchanged size
4. Any tests referencing `pad`

#### Command

```
/rust-implement W16.1
```

**Details:** [DAY_1_TASKS.md](./DAY_1_TASKS.md)

---

## Day 2: Delete API (W16.2)

### W16.2: Implement `delete()` and `is_deleted()` Methods

**Agent:** RUST_ENGINEER
**Estimate:** 6h (2h base × 3x)
**Priority:** P0

#### Objective

Implement the core delete API methods as specified in RFC-001.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC16.2.1 | `delete(VectorId) -> Result<bool>` implemented | Unit test |
| AC16.2.2 | `is_deleted(VectorId) -> Result<bool>` implemented | Unit test |
| AC16.2.3 | `deleted_count() -> usize` implemented | Unit test |
| AC16.2.4 | `tombstone_ratio() -> f64` implemented | Unit test |
| AC16.2.5 | Double-delete returns `Ok(false)` | Unit test |
| AC16.2.6 | Delete of non-existent ID returns error | Unit test |
| AC16.2.7 | `get_node_mut()` helper added | Internal method |

#### API Specification (from RFC-001)

```rust
impl HnswIndex {
    /// Mark a vector as deleted (soft delete)
    pub fn delete(&mut self, vector_id: VectorId) -> Result<bool, EdgeVecError>;

    /// Check if a vector is deleted
    pub fn is_deleted(&self, vector_id: VectorId) -> Result<bool, EdgeVecError>;

    /// Count of deleted (tombstoned) vectors
    pub fn deleted_count(&self) -> usize;

    /// Ratio of deleted to total vectors
    pub fn tombstone_ratio(&self) -> f64;
}
```

#### Command

```
/rust-implement W16.2
```

**Details:** [DAY_2_TASKS.md](./DAY_2_TASKS.md)

---

## Day 3: Search Filtering (W16.3)

### W16.3: Update Search to Filter Tombstones

**Agent:** RUST_ENGINEER
**Estimate:** 6h (2h base × 3x)
**Priority:** P0

#### Objective

Modify the search algorithm to exclude deleted vectors from results. Implement adaptive k adjustment to maintain result quality when tombstone ratio is high.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC16.3.1 | Search excludes deleted vectors | Unit test |
| AC16.3.2 | `adjusted_k()` compensates for tombstones | Unit test |
| AC16.3.3 | Empty result when all matches deleted | Unit test |
| AC16.3.4 | Performance degradation < 20% at 10% tombstones | Benchmark |
| AC16.3.5 | Search still uses deleted nodes for routing | Graph traversal test |

#### Implementation Notes

```rust
fn adjusted_k(&self, k: usize) -> usize {
    if self.deleted_count == 0 {
        return k;
    }
    let ratio = self.tombstone_ratio();
    let multiplier = 1.0 / (1.0 - ratio.min(0.9)); // Cap at 10x
    ((k as f64) * multiplier).ceil() as usize
}
```

**Key Decision:** Deleted nodes remain in graph for routing (HNSW correctness). They are filtered only at result collection.

#### Command

```
/rust-implement W16.3
```

**Details:** [DAY_3_TASKS.md](./DAY_3_TASKS.md)

---

## Day 4: Compaction (W16.4)

### W16.4: Implement `compact()` + `insert_with_id()`

**Agent:** RUST_ENGINEER
**Estimate:** 8h (2.7h base × 3x)
**Priority:** P0

#### Objective

Implement index compaction that removes tombstones by rebuilding the graph. Also implement `insert_with_id()` to preserve vector IDs during rebuild.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC16.4.1 | `compact()` removes all tombstones | Unit test |
| AC16.4.2 | `insert_with_id(id, vec)` preserves IDs | Unit test |
| AC16.4.3 | `needs_compaction()` threshold check | Unit test |
| AC16.4.4 | `set_compaction_threshold(ratio)` | Unit test |
| AC16.4.5 | `CompactionResult` struct returned | API design |
| AC16.4.6 | Compaction preserves search quality | Property test |
| AC16.4.7 | Compaction time O(n log n) for n live vectors | Benchmark |

#### API Specification (REVISED — C-RISK-1 Fix)

```rust
#[derive(Debug, Clone)]
pub struct CompactionResult {
    pub tombstones_removed: usize,
    pub new_size: usize,
    pub duration_ms: u64,
}

impl HnswIndex {
    /// Compact the index by rebuilding without tombstones
    ///
    /// **IMPORTANT (C-RISK-1 FIX):** Returns NEW index AND NEW storage.
    /// Caller MUST replace BOTH to avoid storage/index mismatch.
    ///
    /// ```ignore
    /// let (new_index, new_storage, result) = old_index.compact(&old_storage)?;
    /// index = new_index;
    /// storage = new_storage;
    /// ```
    pub fn compact(
        &self,  // Note: &self not &mut self
        storage: &VectorStorage,
    ) -> Result<(HnswIndex, VectorStorage, CompactionResult), EdgeVecError>;

    /// Insert with specific ID (for compaction rebuild)
    pub fn insert_with_id(
        &mut self,
        id: VectorId,
        vector: &[f32],
        storage: &mut VectorStorage
    ) -> Result<VectorId, EdgeVecError>;

    /// Check if compaction is recommended
    pub fn needs_compaction(&self) -> bool;

    /// Set compaction threshold (default 0.3)
    pub fn set_compaction_threshold(&mut self, ratio: f64);
}
```

#### Command

```
/rust-implement W16.4
```

**Details:** [DAY_4_TASKS.md](./DAY_4_TASKS.md)

---

## Day 5: Persistence v3 (W16.5)

### W16.5: Update Persistence Format to v3

**Agent:** RUST_ENGINEER
**Estimate:** 6h (2h base × 3x)
**Priority:** P0

#### Objective

Update the snapshot format to version 3 to persist the `deleted` field and `deleted_count`. Implement forward migration from v2.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC16.5.1 | Snapshot header version bumped to 3 | Format check |
| AC16.5.2 | `deleted_count` in header | Serialization test |
| AC16.5.3 | `deleted` field serialized per node | Serialization test |
| AC16.5.4 | v2 → v3 migration (set deleted=0) | Migration test |
| AC16.5.5 | Backward compatibility documented | README note |
| AC16.5.6 | CRC32 checksum still valid | Integrity test |

#### Format Specification (from RFC-001)

```
[Header]
  magic: [0xED, 0x6E, 0x56, 0x45]  // "EdnVE"
  version: u32 = 3                  // Bumped from 2
  flags: u32                        // Bit 0: has_tombstones
  node_count: u64
  deleted_count: u64                // NEW

[Nodes]
  For each node:
    vector_id: u64
    neighbor_offset: u32
    neighbor_len: u16
    max_layer: u8
    deleted: u8                     // NEW (was padding)

[Neighbor Pool]
  (unchanged)

[CRC32]
  checksum: u32
```

#### Command

```
/rust-implement W16.5
```

**Details:** [DAY_5_TASKS.md](./DAY_5_TASKS.md)

---

## Risk Register (REVISED)

| ID | Risk | Probability | Impact | Mitigation |
|:---|:-----|:------------|:-------|:-----------|
| R16.1 | `bytemuck` Pod derivation breaks | LOW | HIGH | Test size_check.rs first |
| R16.2 | Search performance degrades > 20% | MEDIUM | MEDIUM | Benchmark before/after with defined baseline |
| R16.3 | Compaction memory spike (2x) | LOW | LOW | Document in API, warn for >500k vectors |
| R16.4 | v2 → v3 migration fails | MEDIUM | HIGH | Extensive migration tests |
| R16.5 | insert_with_id ID collision | **MEDIUM** | MEDIUM | debug_assert in compact(), cannot happen with unique source IDs |
| R16.6 | Storage/Index mismatch after compact | ~~HIGH~~ **MITIGATED** | ~~CRITICAL~~ | **FIXED:** compact() returns (index, storage) tuple |

**Note (R16.5 — M-RISK-1 Fix):** Updated from LOW to MEDIUM. While collision should be impossible during compact() (IDs come from original index), the method is public and could be misused. Added debug_assert for safety.

---

## Dependencies

### Internal Dependencies

| Task | Depends On | Notes |
|:-----|:-----------|:------|
| W16.2 | W16.1 | Needs `deleted` field |
| W16.3 | W16.2 | Needs `is_deleted()` |
| W16.4 | W16.2, W16.3 | Needs delete + search |
| W16.5 | W16.1, W16.2 | Needs field + count |

### Execution Order

```
W16.1 ──► W16.2 ──► W16.3 ──► W16.4
              │                  │
              └──────► W16.5 ◄───┘
```

---

## Success Metrics

### Quality Gates

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| Unit Tests | +20 new tests | `cargo test` |
| Property Tests | 2 new (delete invariants) | `cargo test proptest` |
| Clippy | 0 warnings | `cargo clippy -- -D warnings` |
| Rustfmt | Clean | `cargo fmt -- --check` |

### Performance Targets

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| delete() latency | < 1 μs | Benchmark |
| Search overhead at 10% tombstones | < 20% | Benchmark |
| Compact 10k vectors | < 5 s | Benchmark |

### Functional Targets

| Test | Expected |
|:-----|:---------|
| Delete + search excludes | ✅ |
| Delete + compact removes | ✅ |
| Save + load preserves deletes | ✅ |
| v2 snapshot loads in v3 | ✅ |

---

## HOSTILE_REVIEWER Checkpoints

| Day | Artifact | Review Focus |
|:----|:---------|:-------------|
| Day 1 | `src/hnsw/graph.rs` | Size unchanged, backward compat |
| Day 2 | Delete API | Edge cases, error handling |
| Day 3 | Search filtering | Performance, correctness |
| Day 4 | Compaction | Memory safety, ID preservation |
| Day 5 | Persistence v3 | Migration path, checksum |

---

## Week 17 Preview

**Theme:** WASM Bindings & Testing

**Planned Tasks:**

| Task | Description |
|:-----|:------------|
| W17.1 | WASM bindings for delete(), compact() |
| W17.2 | Property tests for delete invariants |
| W17.3 | Fuzz tests for delete/compact sequences |
| W17.4 | Performance benchmarks |
| W17.5 | Documentation + examples |

**Prerequisite:** Week 16 complete and approved

---

## Appendix: Detailed Task Files

- [DAY_1_TASKS.md](./DAY_1_TASKS.md) — Field Rename
- [DAY_2_TASKS.md](./DAY_2_TASKS.md) — Delete API
- [DAY_3_TASKS.md](./DAY_3_TASKS.md) — Search Filtering
- [DAY_4_TASKS.md](./DAY_4_TASKS.md) — Compaction
- [DAY_5_TASKS.md](./DAY_5_TASKS.md) — Persistence v3

---

## Revision History

| Version | Date | Author | Changes |
|:--------|:-----|:-------|:--------|
| 1.0 | 2025-12-14 | PLANNER | Initial Week 16 plan based on RFC-001 |

---

**Status:** [PROPOSED]
**Next:** HOSTILE_REVIEWER approval → Week 16 execution
