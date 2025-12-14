# Week 15 — Day 3 Tasks (Wednesday, Jan 1)

**Date:** 2025-01-01
**Focus:** Soft Delete Architecture Design (v0.3.0 Preparation)
**Agent:** META_ARCHITECT, RUST_ENGINEER
**Status:** [PROPOSED]

---

## Day Objective

Design the architecture for soft delete with tombstones, addressing limitation #2: "No delete/update operations". This is a v0.3.0 feature that requires careful design before implementation.

**Success Criteria:**
- RFC document for soft delete system
- Data structure designs with size calculations
- WAL extension design for delete operations
- Migration path for existing indices
- HOSTILE_REVIEWER approval of architecture

---

## Tasks

### W15.3: Soft Delete Architecture RFC

**Priority:** P0 (Critical Path for v0.3.0)
**Estimate:** 6h (base: 2h × 3x)
**Agent:** META_ARCHITECT

#### Scope

- [ ] **AC15.3.1:** Create RFC document in `docs/rfcs/`
- [ ] **AC15.3.2:** Design tombstone data structure
- [ ] **AC15.3.3:** Design compaction strategy
- [ ] **AC15.3.4:** Design WAL extension for DELETE operations
- [ ] **AC15.3.5:** Calculate memory overhead per deleted vector
- [ ] **AC15.3.6:** Define API changes for `HnswIndex`

#### Implementation Specification

**File:** `docs/rfcs/RFC-001-soft-delete.md`

```markdown
# RFC-001: Soft Delete with Tombstones

**Status:** PROPOSED
**Author:** META_ARCHITECT
**Date:** 2025-01-01
**Target:** v0.3.0

---

## Summary

Add soft delete capability to EdgeVec using a tombstone-based approach. Deleted vectors are marked as inactive but remain in the graph until compaction. This enables:

1. Vector deletion without index rebuild
2. Update-by-delete-then-insert pattern
3. Eventual space reclamation via compaction

---

## Motivation

Current limitation: EdgeVec is append-only. Users cannot remove vectors from the index. This forces workarounds:

- Client-side filtering of deleted IDs (performance penalty)
- Periodic full index rebuilds (downtime)
- Oversized indices with stale data

---

## Design

### Tombstone Structure

```rust
/// Tombstone marker for deleted vectors
#[derive(Debug, Clone, Copy)]
pub struct Tombstone {
    /// ID of deleted vector
    pub vector_id: VectorId,
    /// Timestamp of deletion (for ordering)
    pub deleted_at: u64,
}

// Size: 8 (VectorId) + 8 (timestamp) = 16 bytes per tombstone
```

### Storage Changes

**Option A: Inline Tombstone Bit (Chosen)**

Add a single bit to `HnswNode` indicating deleted status:

```rust
pub struct HnswNode {
    pub vector_id: VectorId,
    pub level: u8,
    pub deleted: bool,  // NEW: 1 bit (padded to 1 byte)
    // ... existing fields
}
```

Memory overhead: 1 byte per vector (padded)
Lookup cost: O(1) via node access

**Option B: Separate Tombstone Set**

```rust
pub struct TombstoneSet {
    deleted: HashSet<VectorId>,
}
```

Memory overhead: ~24 bytes per deleted vector (HashSet entry)
Lookup cost: O(1) but separate hash lookup

**Decision:** Option A (inline) chosen for:
- Lower memory overhead
- Single memory access during search
- Simpler persistence

### Search Behavior

```rust
impl HnswIndex {
    pub fn search(&self, query: &[f32], k: usize, storage: &VectorStorage)
        -> Result<Vec<SearchResult>, EdgeVecError>
    {
        // Existing search logic with one change:
        // Filter out deleted nodes during neighbor collection

        let mut results = Vec::new();
        for candidate in candidates.iter().take(k * 2) {  // Over-fetch
            let node = self.nodes.get(candidate.id)?;
            if !node.deleted {  // NEW: Skip tombstones
                results.push(candidate);
            }
            if results.len() >= k {
                break;
            }
        }
        Ok(results)
    }
}
```

### Delete API

```rust
impl HnswIndex {
    /// Mark a vector as deleted (soft delete)
    ///
    /// The vector remains in the graph but is excluded from search results.
    /// Space is reclaimed during compaction.
    ///
    /// # Returns
    /// - `Ok(true)` if vector was deleted
    /// - `Ok(false)` if vector was already deleted
    /// - `Err(NotFound)` if vector ID doesn't exist
    pub fn delete(&mut self, vector_id: VectorId) -> Result<bool, EdgeVecError> {
        let node = self.nodes.get_mut(vector_id)?;
        if node.deleted {
            return Ok(false);
        }
        node.deleted = true;
        self.deleted_count += 1;
        self.wal.append(WalEntry::Delete { vector_id })?;
        Ok(true)
    }

    /// Check if a vector is deleted
    pub fn is_deleted(&self, vector_id: VectorId) -> Result<bool, EdgeVecError> {
        Ok(self.nodes.get(vector_id)?.deleted)
    }

    /// Count of deleted (tombstoned) vectors
    pub fn deleted_count(&self) -> usize {
        self.deleted_count
    }

    /// Ratio of deleted to total vectors
    pub fn tombstone_ratio(&self) -> f64 {
        self.deleted_count as f64 / self.node_count() as f64
    }
}
```

### WAL Extension

```rust
#[derive(Debug, Clone)]
pub enum WalEntry {
    Insert { vector_id: VectorId, vector: Vec<f32> },
    Delete { vector_id: VectorId },  // NEW
    Checkpoint { snapshot_id: u64 },
}
```

WAL format:
```
[Entry Type: 1 byte][Payload Length: 4 bytes][Payload][CRC32: 4 bytes]

Delete entry payload:
[vector_id: 8 bytes]
```

### Compaction Strategy

When `tombstone_ratio() > threshold` (default 0.3):

```rust
impl HnswIndex {
    /// Compact the index by rebuilding without tombstones
    ///
    /// This operation:
    /// 1. Creates a new index with only live vectors
    /// 2. Re-inserts vectors maintaining optimal graph structure
    /// 3. Swaps new index in place
    ///
    /// # Performance
    /// - Time: O(n log n) where n = live vector count
    /// - Space: 2x index size during compaction
    pub fn compact(&mut self, storage: &mut VectorStorage) -> Result<CompactionResult, EdgeVecError> {
        let live_ids: Vec<VectorId> = self.nodes.iter()
            .filter(|(_, node)| !node.deleted)
            .map(|(id, _)| id)
            .collect();

        // Build new index
        let mut new_index = HnswIndex::new(self.config.clone(), storage)?;
        for id in live_ids {
            let vector = storage.get(id)?;
            new_index.insert(&vector, storage)?;
        }

        // Swap
        *self = new_index;
        Ok(CompactionResult { vectors_removed: self.deleted_count })
    }
}
```

### Migration Path

Existing indices (v0.2.x) lack the `deleted` field. Migration:

1. **On load:** Set `deleted = false` for all existing nodes
2. **Backward compatibility:** v0.3.0 can read v0.2.x snapshots
3. **Forward compatibility:** v0.2.x cannot read v0.3.0 snapshots with deletions

Snapshot format version bump: 2 → 3

---

## API Changes

### New Methods

```rust
// Delete operations
fn delete(&mut self, vector_id: VectorId) -> Result<bool, EdgeVecError>;
fn is_deleted(&self, vector_id: VectorId) -> Result<bool, EdgeVecError>;
fn deleted_count(&self) -> usize;
fn tombstone_ratio(&self) -> f64;

// Compaction
fn compact(&mut self, storage: &mut VectorStorage) -> Result<CompactionResult, EdgeVecError>;
fn needs_compaction(&self) -> bool;
fn set_compaction_threshold(&mut self, ratio: f64);
```

### WASM API

```typescript
interface EdgeVec {
    // Existing...

    // New in v0.3.0
    delete(vectorId: number): boolean;
    isDeleted(vectorId: number): boolean;
    deletedCount(): number;
    tombstoneRatio(): number;
    compact(): CompactionResult;
    needsCompaction(): boolean;
}
```

---

## Memory Impact

| Scenario | Overhead per Vector | 1M Vectors |
|:---------|:--------------------|:-----------|
| No deletions | 0 bytes | 0 MB |
| 10% deleted | 1 byte per vector | 1 MB |
| 30% deleted (compact threshold) | 1 byte per vector | 1 MB |

The inline `deleted` boolean adds 1 byte overhead regardless of deletion state (due to struct padding). This is acceptable.

---

## Performance Impact

### Search

| Scenario | Impact | Mitigation |
|:---------|:-------|:-----------|
| 0% deleted | None | - |
| 10% deleted | ~10% over-fetch | Dynamic k adjustment |
| 30% deleted | ~30% over-fetch | Trigger compaction |

### Delete

| Operation | Time | Notes |
|:----------|:-----|:------|
| Delete | O(1) | Just set boolean + WAL append |
| Compaction | O(n log n) | Full rebuild, run async |

---

## Alternatives Considered

### A: Hard Delete (Rejected)

Immediately remove vector and repair graph connections.

Pros:
- No tombstone overhead
- No compaction needed

Cons:
- O(M × neighbors) graph repair per delete
- Complex consistency during concurrent access
- Potential graph degradation

### B: Log-Structured Merge (Rejected)

Maintain multiple index segments, merge periodically.

Pros:
- Append-only writes
- Good for high write rates

Cons:
- Complex query spanning multiple segments
- Higher memory for segment metadata
- Overkill for expected delete rates

---

## Testing Strategy

1. **Unit Tests**
   - Delete marks node correctly
   - Search excludes deleted nodes
   - WAL replays delete correctly
   - Compaction removes tombstones

2. **Property Tests**
   - `forall deleted. search never returns deleted`
   - `forall compact. deleted_count == 0`
   - `forall delete(id). is_deleted(id) == true`

3. **Fuzz Tests**
   - Random insert/delete/search sequences
   - Crash recovery with pending deletes

---

## Implementation Plan

**Week 16:** Implementation
- W16.1: Add `deleted` field to HnswNode
- W16.2: Implement delete() and search filtering
- W16.3: Extend WAL for delete entries
- W16.4: Implement compaction
- W16.5: WASM bindings

**Week 17:** Testing & Polish
- Property tests
- Fuzz tests
- Performance benchmarks
- Documentation

---

## Open Questions

1. **Auto-compaction:** Should compaction trigger automatically when threshold exceeded?
2. **Compaction locking:** Block writes during compaction or copy-on-write?
3. **WASM compaction:** Allow in browser or recommend offline compaction?

---

## References

- [HNSW Paper](https://arxiv.org/abs/1603.09320) - Section on dynamic updates
- [Faiss DeleteById](https://github.com/facebookresearch/faiss/wiki/FAQ#can-i-delete-vectors-from-the-index) - Similar approach
- [Milvus Soft Delete](https://milvus.io/docs/delete_data.md) - Production example

---

**Status:** PROPOSED
**Next:** HOSTILE_REVIEWER approval → W16 implementation
```

#### Verification Commands

```bash
# Verify RFC is well-formed
cat docs/rfcs/RFC-001-soft-delete.md | head -100

# Check all sections present
grep "^## " docs/rfcs/RFC-001-soft-delete.md
```

#### Dependencies

- None (design document)

#### Risks

- **R15.3.1:** Design may not account for all edge cases
  - **Mitigation:** HOSTILE_REVIEWER review before implementation

---

### W15.3b: Size Calculations Validation

**Priority:** P1 (Validates RFC)
**Estimate:** 2h (base: 0.7h × 3x)
**Agent:** RUST_ENGINEER

#### Scope

- [ ] **AC15.3b.1:** Write test program to verify struct sizes
- [ ] **AC15.3b.2:** Measure actual memory overhead with `deleted` field
- [ ] **AC15.3b.3:** Document findings in RFC

#### Implementation Specification

**File:** `examples/size_check.rs`

```rust
//! Verify struct sizes for RFC-001

use std::mem::size_of;

#[repr(C)]
struct HnswNodeCurrent {
    vector_id: u64,
    level: u8,
    // ... other fields
}

#[repr(C)]
struct HnswNodeWithDelete {
    vector_id: u64,
    level: u8,
    deleted: bool,
    // ... other fields
}

fn main() {
    println!("Struct size analysis:");
    println!("  HnswNode (current): {} bytes", size_of::<HnswNodeCurrent>());
    println!("  HnswNode (with deleted): {} bytes", size_of::<HnswNodeWithDelete>());
    println!("  Overhead: {} bytes",
        size_of::<HnswNodeWithDelete>() - size_of::<HnswNodeCurrent>());
}
```

---

## Day 3 Summary

**Total Effort:** 8h scheduled

**Deliverables:**
1. `docs/rfcs/RFC-001-soft-delete.md` — Complete RFC
2. `examples/size_check.rs` — Size validation
3. Memory overhead calculations
4. Implementation plan for Week 16

**Day 4 Preview:**
- Browser compatibility matrix testing
- Safari/Firefox/Edge verification

---

## HOSTILE_REVIEWER Pre-Flight

Before end of day:

- [ ] RFC covers all sections
- [ ] Memory calculations are accurate
- [ ] API is consistent with existing style
- [ ] Migration path is defined
- [ ] Open questions are documented

---

**Status:** [PROPOSED]
**Next:** `/architect-design soft-delete`
