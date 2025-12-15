# RFC-001: Soft Delete with Tombstones

**Status:** APPROVED
**Author:** META_ARCHITECT
**Date:** 2025-12-14
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

Real-world use cases requiring delete:

1. **GDPR/CCPA compliance:** User requests data deletion
2. **Semantic search:** Remove outdated embeddings
3. **Recommendation systems:** Remove unavailable items
4. **RAG pipelines:** Update document embeddings

---

## Design

### Tombstone Structure

**Option A: Inline Tombstone Bit (CHOSEN)**

Add a single byte to `HnswNode` indicating deleted status. This replaces the existing padding byte:

```rust
/// Current HnswNode structure (16 bytes)
pub struct HnswNode {
    pub vector_id: VectorId,     // 8 bytes
    pub neighbor_offset: u32,    // 4 bytes
    pub neighbor_len: u16,       // 2 bytes
    pub max_layer: u8,           // 1 byte
    pub pad: u8,                 // 1 byte (padding)
}

/// Proposed HnswNode structure (16 bytes - NO SIZE CHANGE)
pub struct HnswNode {
    pub vector_id: VectorId,     // 8 bytes
    pub neighbor_offset: u32,    // 4 bytes
    pub neighbor_len: u16,       // 2 bytes
    pub max_layer: u8,           // 1 byte
    pub deleted: u8,             // 1 byte (0 = live, 1 = deleted) [WAS: pad]
}
```

**Memory overhead: 0 bytes** (reuses existing padding!)
**Lookup cost:** O(1) via node access

**Option B: Separate Tombstone Set (REJECTED)**

```rust
pub struct TombstoneSet {
    deleted: HashSet<VectorId>,
}
```

Memory overhead: ~24 bytes per deleted vector (HashSet entry)
Lookup cost: O(1) but separate hash lookup

**Decision:** Option A chosen for:
- Zero additional memory overhead
- Single memory access during search
- Simpler persistence
- No HashSet allocation for empty index

### Search Behavior

```rust
impl HnswIndex {
    pub fn search(&self, query: &[f32], k: usize, storage: &VectorStorage)
        -> Result<Vec<SearchResult>, EdgeVecError>
    {
        // Existing search logic with one change:
        // Filter out deleted nodes during neighbor collection

        // Over-fetch to compensate for tombstones
        let fetch_k = self.adjusted_k(k);

        let mut results = Vec::with_capacity(k);
        for candidate in candidates.iter().take(fetch_k) {
            let node = self.nodes.get(candidate.id)?;
            if node.deleted == 0 {  // Skip tombstones
                results.push(candidate.clone());
            }
            if results.len() >= k {
                break;
            }
        }
        Ok(results)
    }

    /// Adjust k based on tombstone ratio to maintain result quality
    fn adjusted_k(&self, k: usize) -> usize {
        if self.deleted_count == 0 {
            return k;
        }
        let ratio = self.tombstone_ratio();
        let multiplier = 1.0 / (1.0 - ratio.min(0.9)); // Cap at 10x
        ((k as f64) * multiplier).ceil() as usize
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
    ///
    /// # Complexity
    /// - Time: O(1)
    /// - Space: O(1)
    ///
    /// # Persistence
    /// **IMPORTANT:** Delete operations are in-memory only until `save()` is called.
    /// If the process crashes before `save()`, the delete will be lost. For durability:
    /// 1. Call `save()` after critical deletes
    /// 2. Or batch deletes and save periodically
    ///
    /// # Example
    /// ```
    /// let deleted = index.delete(VectorId(42))?;
    /// assert!(deleted);
    /// assert!(index.is_deleted(VectorId(42))?);
    /// index.save("my-index")?; // Persist the delete!
    /// ```
    pub fn delete(&mut self, vector_id: VectorId) -> Result<bool, EdgeVecError> {
        let node = self.get_node_mut(vector_id)?;
        if node.deleted != 0 {
            return Ok(false);
        }
        node.deleted = 1;
        self.deleted_count += 1;
        Ok(true)
    }

    /// Check if a vector is deleted
    pub fn is_deleted(&self, vector_id: VectorId) -> Result<bool, EdgeVecError> {
        Ok(self.get_node(vector_id)?.deleted != 0)
    }

    /// Count of deleted (tombstoned) vectors
    pub fn deleted_count(&self) -> usize {
        self.deleted_count
    }

    /// Ratio of deleted to total vectors
    pub fn tombstone_ratio(&self) -> f64 {
        if self.node_count() == 0 {
            return 0.0;
        }
        self.deleted_count as f64 / self.node_count() as f64
    }

    /// Check if compaction is recommended
    pub fn needs_compaction(&self) -> bool {
        self.tombstone_ratio() > self.compaction_threshold
    }
}
```

### Compaction Strategy

When `tombstone_ratio() > threshold` (default 0.3):

```rust
/// Result of compaction operation
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// Number of tombstones removed
    pub tombstones_removed: usize,
    /// New index size (live vectors)
    pub new_size: usize,
    /// Time taken in milliseconds
    pub duration_ms: u64,
}

impl HnswIndex {
    /// Compact the index by rebuilding without tombstones
    ///
    /// This operation:
    /// 1. Collects all live vector IDs
    /// 2. Creates a new index with only live vectors
    /// 3. Re-inserts vectors maintaining optimal graph structure
    /// 4. Swaps new index in place
    ///
    /// # Performance
    /// - Time: O(n log n) where n = live vector count
    /// - Space: 2x index size during compaction (temporary)
    ///
    /// # Blocking
    /// This is a blocking operation. For WASM, consider running
    /// during idle time or on user action.
    ///
    /// # Implementation Note
    /// Requires `insert_with_id()` method (to be added in Week 16).
    /// This preserves original vector IDs during rebuild.
    pub fn compact(&mut self, storage: &VectorStorage) -> Result<CompactionResult, EdgeVecError> {
        let start = std::time::Instant::now();
        let original_deleted = self.deleted_count;

        // Collect live vector IDs
        let live_ids: Vec<VectorId> = self.nodes.iter()
            .filter(|node| node.deleted == 0)
            .map(|node| node.vector_id)
            .collect();

        let new_size = live_ids.len();

        // Build new index
        let config = self.config.clone();
        let mut new_index = HnswIndex::new(config)?;

        for id in live_ids {
            let vector = storage.get(id)?;
            new_index.insert_with_id(id, &vector, storage)?;
        }

        // Swap
        *self = new_index;

        Ok(CompactionResult {
            tombstones_removed: original_deleted,
            new_size,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Set compaction threshold (default 0.3 = 30% tombstones)
    pub fn set_compaction_threshold(&mut self, ratio: f64) {
        self.compaction_threshold = ratio.clamp(0.01, 0.99);
    }
}
```

### Persistence Changes

**Snapshot Format (v3)**

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

**Migration Path**

1. **v0.2.x → v0.3.0 (read):**
   - Detect version 2 snapshot
   - Set `deleted = 0` for all nodes during load
   - Convert to version 3 on next save

2. **v0.3.0 → v0.2.x (incompatible):**
   - Version 3 snapshots cannot be read by v0.2.x
   - Downgrade requires re-indexing

### WASM API

```typescript
interface EdgeVec {
    // Existing methods...

    // New in v0.3.0

    /**
     * Soft delete a vector
     * @param vectorId The ID of the vector to delete
     * @returns true if deleted, false if already deleted
     * @throws if vectorId not found
     */
    delete(vectorId: bigint): boolean;

    /**
     * Check if a vector is deleted
     */
    isDeleted(vectorId: bigint): boolean;

    /**
     * Get count of deleted vectors
     */
    deletedCount(): number;

    /**
     * Get ratio of deleted to total vectors
     */
    tombstoneRatio(): number;

    /**
     * Check if compaction is recommended
     */
    needsCompaction(): boolean;

    /**
     * Compact the index (blocking operation)
     * @returns CompactionResult with statistics
     */
    compact(): CompactionResult;
}

interface CompactionResult {
    tombstonesRemoved: number;
    newSize: number;
    durationMs: number;
}
```

---

## Memory Impact

| Scenario | Overhead per Vector | 1M Vectors |
|:---------|:--------------------|:-----------|
| v0.2.x (current) | 0 bytes | 0 MB |
| v0.3.0 (with deleted field) | **0 bytes** | **0 MB** |

**Key insight:** The `deleted` field replaces the existing padding byte in `HnswNode`, resulting in **zero additional memory overhead**.

### Memory During Compaction

During compaction, temporary space is needed:

| Live Vectors | Temporary Space |
|:-------------|:----------------|
| 100k | ~50 MB |
| 500k | ~250 MB |
| 1M | ~500 MB |

This is acceptable for browser environments with typical 1-4GB heap limits.

---

## Performance Impact

### Search

| Tombstone Ratio | Impact | Mitigation |
|:----------------|:-------|:-----------|
| 0% | None | - |
| 10% | ~10% over-fetch | Dynamic k adjustment |
| 30% | ~30% over-fetch | Trigger compaction warning |
| 50% | ~50% over-fetch | Force compaction recommended |

The over-fetch is compensated by `adjusted_k()` which inflates the internal search limit based on tombstone ratio.

### Delete Operation

| Operation | Time Complexity | Notes |
|:----------|:----------------|:------|
| `delete()` | O(1) | Just set byte |
| `is_deleted()` | O(1) | Just read byte |
| `compact()` | O(n log n) | Full rebuild |

### Benchmarks (Expected)

| Operation | Time (100k vectors) |
|:----------|:--------------------|
| Delete single | <1 μs |
| Delete 1000 | <1 ms |
| Compact (30% deleted) | ~5-10 s |

---

## Alternatives Considered

### A: Hard Delete (REJECTED)

Immediately remove vector and repair graph connections.

**Pros:**
- No tombstone overhead
- No compaction needed

**Cons:**
- O(M × neighbors) graph repair per delete
- Complex consistency during concurrent access
- Potential graph quality degradation
- Risk of disconnected subgraphs

### B: Log-Structured Merge (REJECTED)

Maintain multiple index segments, merge periodically.

**Pros:**
- Append-only writes
- Good for high write rates

**Cons:**
- Complex query spanning multiple segments
- Higher memory for segment metadata
- Overkill for expected delete rates (<1% of inserts)

### C: Bitmap Index (REJECTED)

Separate bitvec for deleted flags.

**Pros:**
- Very compact (1 bit per vector)

**Cons:**
- Additional data structure to maintain
- Cache miss on every search candidate check
- More complex persistence

---

## Testing Strategy

### 1. Unit Tests

```rust
#[test]
fn test_delete_marks_node() {
    let mut index = create_test_index();
    let id = VectorId(1);

    assert!(!index.is_deleted(id).unwrap());
    assert!(index.delete(id).unwrap());
    assert!(index.is_deleted(id).unwrap());
}

#[test]
fn test_delete_idempotent() {
    let mut index = create_test_index();
    let id = VectorId(1);

    assert!(index.delete(id).unwrap());  // First delete returns true
    assert!(!index.delete(id).unwrap()); // Second delete returns false
}

#[test]
fn test_search_excludes_deleted() {
    let mut index = create_test_index();
    insert_vector(&mut index, VectorId(1), &[1.0, 0.0]);
    insert_vector(&mut index, VectorId(2), &[0.9, 0.1]);

    index.delete(VectorId(1)).unwrap();

    let results = index.search(&[1.0, 0.0], 10, &storage).unwrap();
    assert!(!results.iter().any(|r| r.vector_id == VectorId(1)));
}

#[test]
fn test_compaction_removes_tombstones() {
    let mut index = create_test_index();
    for i in 0..100 {
        insert_vector(&mut index, VectorId(i), &random_vector());
    }

    for i in 0..30 {
        index.delete(VectorId(i)).unwrap();
    }

    assert_eq!(index.deleted_count(), 30);

    let result = index.compact(&storage).unwrap();

    assert_eq!(result.tombstones_removed, 30);
    assert_eq!(index.deleted_count(), 0);
    assert_eq!(index.node_count(), 70);
}
```

### 2. Property Tests

```rust
proptest! {
    #[test]
    fn prop_deleted_never_in_results(
        insertions in vec(any::<Vec<f32>>(), 10..100),
        deletions in vec(0usize..100, 0..50),
        query in any::<Vec<f32>>()
    ) {
        let mut index = create_index();
        let mut deleted_ids = HashSet::new();

        for (i, vec) in insertions.iter().enumerate() {
            index.insert_with_id(VectorId(i as u64), vec, &storage).unwrap();
        }

        for idx in deletions {
            if idx < insertions.len() {
                index.delete(VectorId(idx as u64)).unwrap();
                deleted_ids.insert(idx as u64);
            }
        }

        let results = index.search(&query, 10, &storage).unwrap();

        for result in results {
            prop_assert!(!deleted_ids.contains(&result.vector_id.0));
        }
    }

    #[test]
    fn prop_compact_removes_all_tombstones(
        operations in vec(Operation::arbitrary(), 50..200)
    ) {
        let mut index = create_index();
        apply_operations(&mut index, operations);

        let pre_deleted = index.deleted_count();
        index.compact(&storage).unwrap();

        prop_assert_eq!(index.deleted_count(), 0);
        prop_assert!(pre_deleted >= 0);
    }
}
```

### 3. Fuzz Tests

```rust
fuzz_target!(|data: &[u8]| {
    let mut index = create_index();
    let mut cursor = Cursor::new(data);

    while let Ok(op) = parse_operation(&mut cursor) {
        match op {
            Op::Insert(id, vec) => { index.insert_with_id(id, &vec, &storage).ok(); }
            Op::Delete(id) => { index.delete(id).ok(); }
            Op::Search(query, k) => { index.search(&query, k, &storage).ok(); }
            Op::Compact => { index.compact(&storage).ok(); }
        }
    }

    // Invariant: deleted_count <= node_count
    assert!(index.deleted_count() <= index.node_count());
});
```

---

## Implementation Plan

### Week 16: Core Implementation

| Day | Task | Effort |
|:----|:-----|:-------|
| W16.1 | Rename `pad` → `deleted` in HnswNode | 2h |
| W16.2 | Implement `delete()`, `is_deleted()` | 4h |
| W16.3 | Update search to filter tombstones | 3h |
| W16.4 | Implement `compact()` + `insert_with_id()` | 6h |
| W16.5 | Update persistence format to v3 | 4h |

**Note (W16.4):** The `compact()` method requires a new `insert_with_id()` API to preserve vector IDs during rebuild. This method will be added as part of W16.4.

### Week 17: WASM & Testing

| Day | Task | Effort |
|:----|:-----|:-------|
| W17.1 | WASM bindings for delete API | 4h |
| W17.2 | Property tests | 4h |
| W17.3 | Fuzz tests | 4h |
| W17.4 | Performance benchmarks | 3h |
| W17.5 | Documentation | 3h |

---

## Open Questions

1. **Auto-compaction:** Should compaction trigger automatically when threshold exceeded?
   - **Proposal:** No. Compaction should be explicit to avoid unexpected latency spikes.

2. **Compaction locking:** Block writes during compaction or copy-on-write?
   - **Proposal:** Blocking. Copy-on-write adds complexity. Users should schedule compaction during quiet periods.

3. **WASM compaction:** Allow in browser or recommend offline compaction?
   - **Proposal:** Allow in browser for small indices (<100k). Warn for larger indices. Provide progress callback.

4. **Delete during search:** What if a vector is deleted mid-search?
   - **Proposal:** Accept eventual consistency. The result may include vectors deleted during search. This is acceptable for soft delete semantics.

---

## Security Considerations

1. **DoS via tombstone flooding:** Malicious actors could insert-then-delete to bloat index.
   - **Mitigation:** Rate limiting at application layer. Compaction alerts.

2. **Information leakage:** Deleted vectors still exist in memory until compaction.
   - **Mitigation:** Document this behavior. For sensitive data, require immediate compaction.

---

## References

- [HNSW Paper](https://arxiv.org/abs/1603.09320) - Section on dynamic updates
- [Faiss DeleteById](https://github.com/facebookresearch/faiss/wiki/FAQ#can-i-delete-vectors-from-the-index) - Similar tombstone approach
- [Milvus Soft Delete](https://milvus.io/docs/delete_data.md) - Production example
- [Pinecone Delete](https://docs.pinecone.io/docs/delete-data) - Cloud vector DB approach

---

**Status:** APPROVED
**Next:** HOSTILE_REVIEWER approval → Week 16 implementation
