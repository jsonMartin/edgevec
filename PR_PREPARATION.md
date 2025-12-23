# EdgeVec PR Preparation: BinaryFlatIndex + WASM SIMD

## Overview

This document outlines the steps needed to prepare a PR that adds:

1. **BinaryFlatIndex** - Brute force search for binary vectors
2. **WASM SIMD128 Hamming** - 2.6x faster than scalar on WASM

## Current State

### Your Branch: `feat/binary-vector-support`

| Component                                  | Status      | Lines |
| ------------------------------------------ | ----------- | ----- |
| `src/flat/mod.rs`                          | ✅ Complete | 396   |
| `src/metric/simd.rs` (WASM + AVX2 Hamming) | ✅ Complete | 1111  |
| `src/filter/filtered_search.rs`            | ✅ Complete | 260   |
| `src/hnsw/insert.rs` (binary support)      | ✅ Complete | 413+  |
| `src/hnsw/search.rs` (binary search)       | ✅ Complete | 330+  |
| `src/wasm/mod.rs` (bindings)               | ✅ Complete | 800+  |
| `tests/integration_binary.rs`              | ✅ Complete | 311   |

### Official v0.6.0 (upstream)

New features in v0.6.0 that your branch is missing:

- `src/hnsw/rescore.rs` - BQ rescoring pipeline
- `src/hnsw/search_bq.rs` - `searchBQ()` with automatic rescoring
- `src/storage/binary.rs` - Refactored binary storage (907 lines)
- `src/quantization/variable.rs` - Variable quantization support
- `src/simd/popcount.rs` - Different SIMD popcount implementation

## Key Conflicts to Resolve

### 1. SIMD Hamming Distance

**Your implementation** (`src/metric/simd.rs`):

- Full WASM SIMD128 with LUT popcount
- AVX2 with LUT popcount
- 4-way unrolling for ILP
- Dispatcher function `hamming_distance()`

**v0.6.0 implementation** (`src/simd/popcount.rs`):

- Different structure
- May not have WASM SIMD128

**Resolution**: Your WASM SIMD is superior for browser use. Keep your implementation but ensure it integrates with v0.6.0's dispatcher pattern.

### 2. Binary Storage

**Your implementation** (`src/storage.rs`):

- Inline binary support in VectorStorage

**v0.6.0 implementation** (`src/storage/binary.rs`):

- Separate BinaryVectorStorage module (907 lines)

**Resolution**: Adopt v0.6.0's structure, ensure BinaryFlatIndex works with it.

### 3. Filter Strategy

**Your implementation** (`src/filter/filtered_search.rs`):

- Full pre/post/hybrid strategy with binary search support

**v0.6.0 implementation** (`src/filter/strategy.rs`):

- May have been refactored (170 lines removed in diff)

**Resolution**: Merge carefully, preserve binary filtered search capability.

## Step-by-Step PR Preparation

### Step 1: Merge v0.6.0

```bash
# Fetch upstream
git fetch upstream

# Create a new branch for the merge
git checkout feat/binary-vector-support
git checkout -b pr/binary-flat-index

# Merge v0.6.0
git merge v0.6.0 --no-commit

# Expect conflicts in:
# - src/metric/simd.rs (your SIMD vs their simd/)
# - src/storage.rs vs src/storage/binary.rs
# - src/wasm/mod.rs (extensive changes both sides)
```

### Step 2: Resolve Conflicts

Priority order:

1. **Keep your WASM SIMD128 Hamming** - v0.6.0 doesn't have it
2. **Adopt v0.6.0's storage structure** - More modular
3. **Keep your BinaryFlatIndex** - v0.6.0 doesn't have brute force
4. **Merge WASM bindings carefully** - Both have extensive changes

### Step 3: Integrate BinaryFlatIndex with Rescoring

v0.6.0 has a `searchBQ()` method that:

1. Searches with binary vectors (fast)
2. Rescores with f32 vectors (accurate)

Add the same for BinaryFlatIndex:

```rust
// Add to src/flat/mod.rs or new file
impl BinaryFlatIndex {
    /// Search with rescoring for improved accuracy.
    pub fn search_with_rescore(
        &self,
        query_binary: &[u8],
        query_f32: &[f32],
        k: usize,
        oversample: usize,
        f32_storage: &VectorStorage,
    ) -> Vec<SearchResult> {
        // 1. Binary search with oversample
        let candidates = self.search(query_binary, k * oversample);

        // 2. Rescore with f32 (reuse v0.6.0's rescore logic)
        rescore_candidates(&candidates, query_f32, f32_storage, k)
    }
}
```

### Step 4: Add Force Pre-filter for Flat Index

For brute force, pre-filter is always optimal. Add option:

```rust
// In FilteredSearcher or new FlatFilteredSearcher
impl FlatFilteredSearcher {
    pub fn search_filtered(
        &self,
        query: &[u8],
        k: usize,
        filter: &FilterExpr,
    ) -> Vec<SearchResult> {
        // Always pre-filter for flat index
        let matching_ids = self.prefilter_metadata(filter);
        self.search_subset(query, k, &matching_ids)
    }
}
```

### Step 5: WASM Bindings

Add new methods to `EdgeVecDb`:

```rust
#[wasm_bindgen]
impl EdgeVecDb {
    /// Create a flat (brute force) binary index.
    #[wasm_bindgen(js_name = "createFlatIndex")]
    pub fn create_flat_index(&mut self, dimensions: u32) -> Result<(), JsValue> {
        // ...
    }

    /// Search flat index with optional filter.
    #[wasm_bindgen(js_name = "searchFlat")]
    pub fn search_flat(&self, query: Uint8Array, k: u32) -> Result<Array, JsValue> {
        // ...
    }

    /// Search flat index with BQ rescoring.
    #[wasm_bindgen(js_name = "searchFlatBQ")]
    pub fn search_flat_bq(&self, query: Float32Array, k: u32) -> Result<Array, JsValue> {
        // ...
    }
}
```

### Step 6: Testing

```bash
# Run all tests
cargo test

# Run binary-specific tests
cargo test binary

# Run clippy
cargo clippy -- -D warnings

# Build WASM
wasm-pack build --target web

# Test in browser
# Open examples/browser/binary-test.html
```

### Step 7: Benchmarks

Add benchmarks comparing:

- HNSW binary search vs BinaryFlatIndex
- With and without WASM SIMD

```bash
cargo bench --bench binary_search
```

### Step 8: PR Description

```markdown
## Summary

This PR adds brute force binary vector search, optimized for datasets < 1M vectors.

### Features

1. **BinaryFlatIndex** - O(1) insert, O(n) SIMD-accelerated search
2. **WASM SIMD128 Hamming** - 2.6x faster than scalar in browser
3. **AVX2 Hamming** - Native performance on x86_64
4. **Filtered search** - DSL pre-filter with flat index
5. **BQ rescoring** - Binary search + f32 rescore for 95%+ recall

### When to use

| Index Type | Insert             | Search (1M) | Recall   | Use Case                     |
| ---------- | ------------------ | ----------- | -------- | ---------------------------- |
| HNSW       | O(log n) ~64 vec/s | ~2ms        | 90-95%   | Large datasets, batch insert |
| **Flat**   | **O(1) instant**   | ~5-10ms     | **100%** | Real-time apps, < 1M vectors |

### Benchmarks

| Metric      | HNSW   | BinaryFlatIndex |
| ----------- | ------ | --------------- |
| Insert 100K | 26 min | **< 1 sec**     |
| Query 100K  | 2ms    | 5ms             |
| Recall@10   | 95%    | **100%**        |
```

## Files to Submit in PR

### New files (from your branch)

- `src/flat/mod.rs`
- `src/metric/simd.rs` (WASM SIMD additions)
- `tests/integration_binary.rs` (if not duplicating v0.6.0 tests)

### Modified files

- `src/lib.rs` (add `pub mod flat`)
- `src/wasm/mod.rs` (add flat index bindings)
- `src/metric/mod.rs` (export SIMD functions)

### Do NOT submit

- `.claude/` directory
- `BENCHMARK_PLAN.md`
- `BINARY_SUPPORT_PR_PLAN.md`
- Other planning/documentation files

## Timeline Estimate

| Task                | Effort    |
| ------------------- | --------- |
| Merge v0.6.0        | 2-4 hours |
| Resolve conflicts   | 2-4 hours |
| Integrate rescoring | 1-2 hours |
| Add WASM bindings   | 1-2 hours |
| Testing             | 2-4 hours |
| Benchmarks          | 1-2 hours |
| PR writeup          | 1 hour    |

**Total: 10-20 hours of focused work**
