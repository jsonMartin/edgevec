# Day 1 Tasks — W17.1: WASM Soft Delete Bindings

**Date:** Week 17, Day 1
**Task ID:** W17.1
**Agent:** WASM_SPECIALIST
**Estimate:** 8h (2.7h base × 3x)
**Priority:** P0 (Critical — Deferred from W16)
**Status:** PENDING

---

## Objective

Expose all Week 16 soft delete APIs via WASM bindings, matching the TypeScript interface specified in RFC-001. This is the critical path item deferred from Week 16.

---

## Prerequisites

- [x] Week 16 complete (Rust APIs implemented)
- [x] `wasm-pack` installed and working
- [x] Existing WASM bindings functional (`insert`, `search`, `save`, `load`)
- [x] Browser compatibility matrix from W15.4

---

## Implementation Checklist

### 1. Add WASM Bindings to `src/wasm/mod.rs`

```rust
// File: src/wasm/mod.rs

use wasm_bindgen::prelude::*;
use crate::hnsw::{HnswIndex, VectorId, CompactionResult};

#[wasm_bindgen]
impl WasmIndex {
    // Existing bindings (v0.2.x)...

    // NEW: v0.3.0 Soft Delete API

    /// Soft delete a vector by ID
    /// Returns true if deleted, false if already deleted
    /// Throws if vector ID not found
    #[wasm_bindgen(js_name = softDelete)]
    pub fn soft_delete(&mut self, vector_id: u64) -> Result<bool, JsValue> {
        let id = VectorId(vector_id);
        self.index
            .soft_delete(id)
            .map(|_| true)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Check if a vector is deleted
    #[wasm_bindgen(js_name = isDeleted)]
    pub fn is_deleted(&self, vector_id: u64) -> Result<bool, JsValue> {
        let id = VectorId(vector_id);
        self.index
            .is_deleted(id)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get count of deleted vectors
    #[wasm_bindgen(js_name = deletedCount)]
    pub fn deleted_count(&self) -> usize {
        self.index.deleted_count()
    }

    /// Get count of live (non-deleted) vectors
    #[wasm_bindgen(js_name = liveCount)]
    pub fn live_count(&self) -> usize {
        self.index.live_count()
    }

    /// Get ratio of deleted to total vectors (0.0 - 1.0)
    #[wasm_bindgen(js_name = tombstoneRatio)]
    pub fn tombstone_ratio(&self) -> f64 {
        self.index.tombstone_ratio()
    }

    /// Check if compaction is recommended (tombstone ratio > threshold)
    #[wasm_bindgen(js_name = needsCompaction)]
    pub fn needs_compaction(&self) -> bool {
        self.index.needs_compaction()
    }

    /// Get compaction warning message if threshold exceeded, null otherwise
    #[wasm_bindgen(js_name = compactionWarning)]
    pub fn compaction_warning(&self) -> Option<String> {
        self.index.compaction_warning()
    }

    /// Compact the index, removing all tombstones
    /// Returns CompactionResult with statistics
    /// WARNING: This is a blocking operation
    #[wasm_bindgen]
    pub fn compact(&mut self) -> Result<WasmCompactionResult, JsValue> {
        let (new_index, new_storage, result) = self.index
            .compact(&self.storage)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.index = new_index;
        self.storage = new_storage;

        Ok(WasmCompactionResult {
            tombstones_removed: result.tombstones_removed as u32,
            new_size: result.new_size as u32,
            duration_ms: result.duration_ms as u32,
        })
    }

    /// Set compaction threshold (default 0.3 = 30%)
    #[wasm_bindgen(js_name = setCompactionThreshold)]
    pub fn set_compaction_threshold(&mut self, ratio: f64) {
        self.index.set_compaction_threshold(ratio);
    }
}

/// Result of compaction operation
#[wasm_bindgen]
pub struct WasmCompactionResult {
    #[wasm_bindgen(readonly)]
    pub tombstones_removed: u32,
    #[wasm_bindgen(readonly)]
    pub new_size: u32,
    #[wasm_bindgen(readonly)]
    pub duration_ms: u32,
}
```

### 2. Update TypeScript Definitions (`pkg/edgevec.d.ts`)

```typescript
// File: pkg/edgevec.d.ts (additions)

export class WasmIndex {
    // Existing methods...

    // v0.3.0 Soft Delete API
    softDelete(vectorId: bigint): boolean;
    isDeleted(vectorId: bigint): boolean;
    deletedCount(): number;
    liveCount(): number;
    tombstoneRatio(): number;
    needsCompaction(): boolean;
    compactionWarning(): string | null;
    compact(): WasmCompactionResult;
    setCompactionThreshold(ratio: number): void;
}

export class WasmCompactionResult {
    readonly tombstones_removed: number;
    readonly new_size: number;
    readonly duration_ms: number;
}
```

### 3. Build and Verify

```bash
# Build WASM package
wasm-pack build --target web --release

# Verify bundle size
ls -la pkg/edgevec_bg.wasm
# Target: < 500KB

# Verify TypeScript compiles
cd pkg && npx tsc --noEmit edgevec.d.ts
```

### 4. Update `pkg/README.md`

Add documentation for new APIs with usage examples.

---

## Acceptance Criteria Verification

| AC | Verification Command | Expected |
|:---|:---------------------|:---------|
| AC17.1.1 | TypeScript compiles | No errors |
| AC17.1.2 | TypeScript compiles | No errors |
| AC17.1.3 | `npm test` | PASS |
| AC17.1.4 | `npm test` | PASS |
| AC17.1.5 | `npm test` | PASS |
| AC17.1.6 | `npm test` | PASS |
| AC17.1.7 | Integration test | PASS |
| AC17.1.8 | `npm test` | PASS |
| AC17.1.9 | `ls -la pkg/*.wasm` | < 500KB |
| AC17.1.10 | Manual review | Complete |

---

## Test Cases

### Unit Tests (Rust)

```rust
#[cfg(test)]
mod wasm_tests {
    use super::*;

    #[test]
    fn test_wasm_soft_delete() {
        let mut index = WasmIndex::new(128, 16, 200).unwrap();
        let id = index.insert(&[1.0; 128]).unwrap();

        assert!(!index.is_deleted(id).unwrap());
        assert!(index.soft_delete(id).unwrap());
        assert!(index.is_deleted(id).unwrap());
    }

    #[test]
    fn test_wasm_deleted_count() {
        let mut index = WasmIndex::new(128, 16, 200).unwrap();
        let id = index.insert(&[1.0; 128]).unwrap();

        assert_eq!(index.deleted_count(), 0);
        index.soft_delete(id).unwrap();
        assert_eq!(index.deleted_count(), 1);
    }

    #[test]
    fn test_wasm_compact() {
        let mut index = WasmIndex::new(128, 16, 200).unwrap();

        for i in 0..10 {
            index.insert(&[i as f32 / 10.0; 128]).unwrap();
        }

        for i in 0..3 {
            index.soft_delete(i).unwrap();
        }

        let result = index.compact().unwrap();
        assert_eq!(result.tombstones_removed, 3);
        assert_eq!(index.deleted_count(), 0);
    }
}
```

---

## Risks and Mitigations

| Risk | Mitigation |
|:-----|:-----------|
| WASM memory allocation during compact | Test with 10k vectors in browser |
| BigInt handling for vector IDs | Use u64 with JsValue conversion |
| Error propagation to JS | Map all Rust errors to JsValue strings |

---

## Output

### Artifacts Generated

- [ ] `src/wasm/mod.rs` — Updated with soft delete bindings
- [ ] `pkg/edgevec.d.ts` — Updated TypeScript definitions
- [ ] `pkg/edgevec_bg.wasm` — Rebuilt WASM bundle
- [ ] `pkg/README.md` — Updated documentation

### Status After Completion

```
✅ W17.1 COMPLETE
Next: W17.2 (Integration Tests)
```

---

**Status:** PENDING
**Next:** `/wasm-bind soft_delete`
