# Day 5 Tasks — W17.5: Documentation + Publish

**Date:** Week 17, Day 5
**Task ID:** W17.5
**Agent:** DOCWRITER
**Estimate:** 4h (1.3h base × 3x)
**Priority:** P0
**Status:** PENDING

---

## Objective

Complete v0.3.0 documentation and publish to crates.io and npm.

---

## Prerequisites

- [ ] W17.4 complete (Release prep)
- [ ] All validation checks pass
- [ ] Credentials for crates.io and npm verified
- [x] GitHub repository access

---

## Implementation Checklist

### 1. Update README.md

Add soft delete section to main README:

```markdown
## Soft Delete (v0.3.0+)

EdgeVec supports soft deletion of vectors without rebuilding the index:

### Rust API

```rust
use edgevec::{HnswIndex, VectorId};

// Delete a vector (O(1) operation)
let deleted = index.soft_delete(VectorId(42))?;

// Check if deleted
let is_deleted = index.is_deleted(VectorId(42))?;

// Get statistics
let deleted_count = index.deleted_count();
let live_count = index.live_count();
let ratio = index.tombstone_ratio();

// Check if compaction recommended (default threshold: 30%)
if index.needs_compaction() {
    let (new_index, new_storage, result) = index.compact(&storage)?;
    println!("Removed {} tombstones", result.tombstones_removed);
}
```

### JavaScript/TypeScript API

```typescript
import init, { WasmIndex } from 'edgevec';

await init();
const index = new WasmIndex(128, 16, 200);

// Insert vectors
const id = index.insert(new Float32Array(128).fill(1.0));

// Soft delete
index.softDelete(id);

// Check status
console.log('Deleted:', index.isDeleted(id));
console.log('Live count:', index.liveCount());
console.log('Tombstone ratio:', index.tombstoneRatio());

// Compact when needed
if (index.needsCompaction()) {
    const result = index.compact();
    console.log(`Removed ${result.tombstones_removed} tombstones`);
}
```

### Performance Characteristics

| Operation | Time Complexity | Notes |
|:----------|:----------------|:------|
| `soft_delete()` | O(1) | Set tombstone byte |
| `is_deleted()` | O(1) | Read tombstone byte |
| `search()` | O(log n) + filtering | Excludes tombstones |
| `compact()` | O(n log n) | Full rebuild |

### When to Compact

- Default threshold: 30% tombstones
- Compaction rebuilds the entire index
- Memory: ~2x during compaction
- Recommended: Compact during low-traffic periods

See [MIGRATION.md](docs/MIGRATION.md) for upgrade instructions.
```

### 2. Update `docs/API_REFERENCE.md`

Add comprehensive API documentation for all new methods.

### 3. Update `pkg/README.md` for npm

```markdown
# EdgeVec

High-performance embedded vector database for browser and Node.js.

## Installation

```bash
npm install edgevec
```

## Quick Start

```typescript
import init, { WasmIndex } from 'edgevec';

async function main() {
    await init();

    // Create index (dimension, M, ef_construction)
    const index = new WasmIndex(128, 16, 200);

    // Insert vectors
    const vector = new Float32Array(128).fill(0.5);
    const id = index.insert(vector);

    // Search
    const results = index.search(vector, 10);
    console.log('Top result:', results[0]);

    // Soft delete (v0.3.0+)
    index.softDelete(id);

    // Compact
    if (index.needsCompaction()) {
        const result = index.compact();
        console.log('Compacted:', result.tombstones_removed);
    }

    // Save/Load
    const data = index.save();
    const loaded = WasmIndex.load(data);

    // Clean up
    index.free();
}

main();
```

## API Reference

### Constructor

- `new WasmIndex(dimension, m, ef_construction)` — Create new index

### Insert & Search

- `insert(vector: Float32Array): bigint` — Insert vector, returns ID
- `search(query: Float32Array, k: number): SearchResult[]` — Find k nearest

### Soft Delete (v0.3.0+)

- `softDelete(vectorId: bigint): boolean` — Mark as deleted
- `isDeleted(vectorId: bigint): boolean` — Check if deleted
- `deletedCount(): number` — Count of tombstones
- `liveCount(): number` — Count of live vectors
- `tombstoneRatio(): number` — Ratio (0.0 - 1.0)

### Compaction (v0.3.0+)

- `needsCompaction(): boolean` — Check threshold
- `compactionWarning(): string | null` — Get warning if needed
- `compact(): CompactionResult` — Rebuild without tombstones
- `setCompactionThreshold(ratio: number)` — Set threshold

### Persistence

- `save(): Uint8Array` — Serialize to bytes
- `WasmIndex.load(data: Uint8Array): WasmIndex` — Deserialize

### Cleanup

- `free()` — Release WASM memory

## Browser Support

- Chrome 90+
- Firefox 90+
- Safari 15+
- Edge 90+

## License

MIT OR Apache-2.0
```

### 4. Complete Rustdoc Comments

Ensure all public APIs have complete rustdoc comments:

```rust
/// Soft delete a vector by marking it as a tombstone.
///
/// # Arguments
///
/// * `id` - The vector ID to delete
///
/// # Returns
///
/// * `Ok(())` - Vector was deleted (or already deleted)
/// * `Err(GraphError::VectorNotFound)` - Vector ID doesn't exist
///
/// # Complexity
///
/// * Time: O(1)
/// * Space: O(1)
///
/// # Example
///
/// ```
/// use edgevec::{HnswIndex, VectorId};
///
/// let mut index = HnswIndex::new(128, 16, 200);
/// // ... insert vectors ...
///
/// index.soft_delete(VectorId(42))?;
/// assert!(index.is_deleted(VectorId(42))?);
/// ```
///
/// # Thread Safety
///
/// This method requires `&mut self`. For concurrent access, wrap in
/// appropriate synchronization (e.g., `Mutex`).
pub fn soft_delete(&mut self, id: VectorId) -> Result<(), GraphError> {
    // ...
}
```

### 5. Publish to crates.io

```bash
# Final dry run
cargo publish --dry-run

# Publish
cargo publish

# Verify
cargo search edgevec
```

### 6. Publish to npm

```bash
# Build WASM
wasm-pack build --target web --release

# Final dry run
cd pkg && npm publish --dry-run

# Publish
npm publish

# Verify
npm info edgevec
```

### 7. Create GitHub Release

```bash
# Create tag
git tag -a v0.3.0 -m "Release v0.3.0: Soft Delete & Compaction"

# Push tag
git push origin v0.3.0
```

Create GitHub release with:
- Title: `v0.3.0: Soft Delete & Compaction`
- Body: Copy from CHANGELOG.md v0.3.0 section
- Attach: WASM bundle (optional)

---

## Acceptance Criteria Verification

| AC | Verification | Expected |
|:---|:-------------|:---------|
| AC17.5.1 | Review README.md | Complete |
| AC17.5.2 | Review API_REFERENCE.md | Complete |
| AC17.5.3 | Review pkg/README.md | Complete |
| AC17.5.4 | `cargo doc` | Complete |
| AC17.5.5 | `cargo publish --dry-run` | Success |
| AC17.5.6 | `npm publish --dry-run` | Success |
| AC17.5.7 | `git tag -l` | v0.3.0 |
| AC17.5.8 | GitHub UI | Release exists |

---

## Post-Publish Verification

After publishing, verify:

```bash
# crates.io
cargo add edgevec@0.3.0 --dry-run
# Should resolve successfully

# npm
npm install edgevec@0.3.0 --dry-run
# Should resolve successfully
```

---

## Output

### Artifacts Generated

- [ ] `README.md` — Updated with soft delete docs
- [ ] `docs/API_REFERENCE.md` — Complete API docs
- [ ] `pkg/README.md` — npm package docs
- [ ] crates.io: edgevec v0.3.0 published
- [ ] npm: edgevec v0.3.0 published
- [ ] GitHub: v0.3.0 release created

### Status After Completion

```
✅ W17.5 COMPLETE
✅ WEEK 17 COMPLETE
✅ v0.3.0 RELEASED

Next: Week 18 Planning (v0.4.0 features)
```

---

**Status:** PENDING
**Next:** `/doc-readme v0.3.0`
