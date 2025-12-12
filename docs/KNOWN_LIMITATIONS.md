# EdgeVec v0.2.0-alpha.1 — Known Limitations

**Version:** 0.2.0-alpha.1
**Last Updated:** 2025-12-12
**Document Owner:** DOCWRITER

This document describes known limitations in the alpha release and planned resolutions.

---

## Overview

EdgeVec is an alpha release optimized for **search latency** and **memory efficiency**. Some features are intentionally deferred to maintain focus on core performance. This document provides:

- Current behavior and impact
- Workarounds for alpha users
- Planned resolutions with target versions

---

## 1. Insert/Build Time Not Optimized

### Current Behavior

| Operation | 10k Vectors | 50k Vectors | 100k Vectors |
|:----------|:------------|:------------|:-------------|
| Float32 Build | 6.3s | 55s | 116s |
| Quantized Build | 2.6s | 24s | 63s |

Build time scales approximately O(n log n) due to HNSW graph construction.

### Impact

| Severity | Use Cases Affected |
|:---------|:-------------------|
| **MEDIUM** | Real-time ingestion, streaming data, high-throughput batch loading |

### Root Cause

- HNSW neighbor selection algorithm has high constant factor
- Each insert triggers graph maintenance across multiple layers
- No batch optimization or parallel construction implemented

### Workaround (Alpha)

1. **Build offline**: Pre-build index during deployment, load at runtime
2. **Batch client-side**: Accumulate vectors, insert during idle time
3. **Use Quantized mode**: 2-3x faster build than Float32

```javascript
// Example: Batch insertion with idle callback
const vectors = collectVectors(); // Accumulate
requestIdleCallback(() => {
    vectors.forEach(v => client.insert(v));
});
```

```rust
// Example: Offline index building (Rust)
use edgevec::{HnswIndex, IndexConfig};

// Build during deployment/CI
fn build_offline_index(vectors: &[Vec<f32>]) -> Result<(), Error> {
    let config = IndexConfig::default();
    let mut index = HnswIndex::new(config);

    for vec in vectors {
        index.insert(vec)?;
    }

    // Save to disk for later loading
    index.save("prebuilt-index.edgevec")?;
    Ok(())
}

// Load at runtime (fast)
fn load_prebuilt() -> Result<HnswIndex, Error> {
    HnswIndex::load("prebuilt-index.edgevec")
}
```

### Planned Resolution

| Version | Improvement |
|:--------|:------------|
| **v0.3.0** | Parallel bulk insert API (10x throughput for batch inserts) |
| **v0.4.0** | Streaming insert with background graph maintenance |

---

## 2. No Delete/Update Operations

### Current Behavior

- Vectors can only be **inserted**, not deleted or updated
- Vector IDs are assigned sequentially and cannot be reused
- Index rebuild required to remove vectors

### Impact

| Severity | Use Cases Affected |
|:---------|:-------------------|
| **MEDIUM** | Dynamic datasets, content moderation, user data deletion |

### Root Cause

- HNSW graph structure makes deletion complex
- Removing a node requires reconnecting neighbors
- Tombstone-based deletion adds memory overhead

### Workaround (Alpha)

1. **Client-side tracking**: Maintain a "deleted IDs" set, filter results post-search
2. **Periodic rebuild**: Export valid vectors, rebuild index
3. **Version-based indexes**: Create new index for each version, swap atomically

```javascript
// Example: Client-side deletion tracking
const deletedIds = new Set();

function softDelete(id) {
    deletedIds.add(id);
}

function search(query, k) {
    const results = client.search(query, k + deletedIds.size);
    return results.filter(r => !deletedIds.has(r.id)).slice(0, k);
}
```

### Planned Resolution

| Version | Improvement |
|:--------|:------------|
| **v0.3.0** | Soft delete with tombstones + periodic compaction |
| **v0.4.0** | In-place update support (re-quantize without rebuild) |

---

## 3. Single-Threaded WASM Execution

### Current Behavior

- WASM execution is single-threaded in browsers
- No parallel search or batch operations
- CPU-bound operations block the main thread

### Impact

| Severity | Use Cases Affected |
|:---------|:-------------------|
| **LOW** | Very large datasets (>500k), real-time UI responsiveness |

### Root Cause

- `SharedArrayBuffer` and Web Workers require specific security headers
- Cross-origin isolation (COOP/COEP) complicates deployment
- `wasm-bindgen-rayon` adds complexity and bundle size

### Workaround (Alpha)

1. **Web Workers**: Run searches in dedicated Worker threads
2. **Index sharding**: Split large indexes across multiple Worker instances
3. **Chunked operations**: Break batch operations into smaller chunks with `requestAnimationFrame`

```javascript
// Example: Web Worker for search
// worker.js
import { EdgeVecClient } from '@edgevec/core';
const client = await EdgeVecClient.load('my-index', { dimensions: 768 });
self.onmessage = async (e) => {
    const results = client.search(e.data.query, e.data.k);
    self.postMessage(results);
};

// main.js
const worker = new Worker('worker.js', { type: 'module' });
worker.postMessage({ query: queryVector, k: 10 });
```

### Planned Resolution

| Version | Improvement |
|:--------|:------------|
| **v0.4.0** | Optional `wasm-bindgen-rayon` support with feature flag |
| **v1.0.0** | Thread pool for parallel batch operations |

---

## 4. Compiler Optimization Required

### Current Behavior

- **With optimization flags**: 88-572 µs search latency (10k-100k)
- **Without optimization flags**: 395-1,267 µs search latency (60-78% slower)

### Impact

| Severity | Use Cases Affected |
|:---------|:-------------------|
| **HIGH** | All Rust users building from source |

### Root Cause

- AVX2 SIMD instructions require `-C target-cpu=native` compiler flag
- Default Rust builds use generic x86_64 without SIMD
- Quantized distance calculations fall back to scalar code without AVX2

### Required Configuration

Create `.cargo/config.toml` in your project root:

```toml
[build]
rustflags = [
    "-C", "target-cpu=native",
    "-C", "opt-level=3",
]

[profile.release]
lto = "fat"
codegen-units = 1
```

### Verification

```bash
# Build and verify AVX2 instructions are present
cargo build --release
objdump -d target/release/your-binary | grep -E "vpmaddubsw|vpsadbw"
# If you see output, AVX2 is enabled
```

### Planned Resolution

| Version | Improvement |
|:--------|:------------|
| **v0.3.0** | Runtime SIMD detection with warnings if not enabled |
| **v0.4.0** | Pre-built binaries with AVX2 enabled |

---

## 5. SQ8 Quantization Trade-offs

### Current Behavior

| Metric | Float32 | Quantized (SQ8) | Delta |
|:-------|:--------|:----------------|:------|
| Memory | 3,176 bytes/vec | 872 bytes/vec | **3.6x smaller** |
| Search (100k) | 572 µs | 329 µs | **1.7x faster** |
| Precision | 100% | ~98% recall¹ | **~2% loss** |

¹ *Recall measured on 768-dimensional normalized embeddings at k=10, ef_search=200. Results may vary with different dimensionalities and data distributions. Formal recall benchmarks planned for v0.3.0.*

### Impact

| Severity | Use Cases Affected |
|:---------|:-------------------|
| **LOW** | Extremely high-precision search requirements |

### Root Cause

- 8-bit quantization introduces rounding errors in distance calculations
- Affects vectors with similar distances (edge cases)
- Trade-off is intentional: memory/speed vs. precision

### Workaround (Alpha)

1. **Use Float32 mode**: For accuracy-critical applications
2. **Increase k**: Request more results, re-rank with exact distances
3. **Tune ef_search**: Higher values improve recall at cost of latency

```javascript
// Example: Quantized search with re-ranking
const candidates = client.search(query, k * 2); // Over-fetch
const reranked = candidates
    .map(c => ({ ...c, exactDist: computeExactDistance(query, c.vector) }))
    .sort((a, b) => a.exactDist - b.exactDist)
    .slice(0, k);
```

### Planned Resolution

| Version | Improvement |
|:--------|:------------|
| **v0.4.0** | Two-phase search (SQ8 first pass, F32 re-ranking) |
| **v1.0.0** | Product Quantization (PQ) for better recall at same memory |

---

## 6. Memory Does Not Shrink (WASM)

### Current Behavior

- WASM linear memory grows but never shrinks
- Deleted vectors (when implemented) will leave gaps
- Long-running applications may accumulate memory

### Impact

| Severity | Use Cases Affected |
|:---------|:-------------------|
| **LOW** | Long-running browser applications with frequent index rebuilds |

### Root Cause

- WebAssembly linear memory can only grow, per spec
- JavaScript garbage collector cannot reclaim WASM memory
- Memory fragmentation accumulates over time

### Workaround (Alpha)

1. **Periodic reload**: Save to IndexedDB, reload to defragment
2. **Index rotation**: Create new index, swap, discard old
3. **Monitor memory**: Use `performance.memory` to track growth

```javascript
// Example: Memory monitoring and reload
if (performance.memory?.usedJSHeapSize > threshold) {
    await client.save('backup');
    client = await EdgeVecClient.load('backup', config);
}
```

### Planned Resolution

| Version | Improvement |
|:--------|:------------|
| **v0.4.0** | Memory defragmentation on snapshot load |
| **v1.0.0** | Memory-mapped backend for native builds |

---

## Limitation Tracking Summary

| # | Limitation | Severity | Workaround | Target Version |
|:--|:-----------|:---------|:-----------|:---------------|
| 1 | Insert/build time | MEDIUM | Batch offline | v0.3.0 |
| 2 | No delete/update | MEDIUM | Client-side tracking | v0.3.0 |
| 3 | Single-threaded WASM | LOW | Web Workers | v0.4.0 |
| 4 | Compiler flags required | HIGH | Document config | v0.3.0 |
| 5 | SQ8 precision loss | LOW | Use Float32 | v0.4.0 |
| 6 | Memory doesn't shrink | LOW | Periodic reload | v0.4.0 |

---

## Stability Guarantees

### What Won't Change in v0.x

1. **Core API**: `EdgeVecClient.create()`, `.insert()`, `.search()`, `.save()`, `.load()`
2. **Persistence format**: Snapshots created in v0.2.x will load in v0.x
3. **Search semantics**: Same query returns same results (deterministic)

### What May Change

1. **Performance characteristics**: Future versions may be faster/slower for specific operations
2. **Memory layout**: Internal struct layouts may change (no ABI stability)
3. **Default parameters**: `m`, `ef_construction`, `ef_search` defaults may be tuned

---

## Reporting Issues

If you encounter a limitation not documented here, please report it:

1. **GitHub Issues**: [https://github.com/anthropics/edgevec/issues](https://github.com/anthropics/edgevec/issues)
2. **Include**: EdgeVec version, platform, reproduction steps, expected vs actual behavior

---

**Document Status:** [APPROVED]
**Last Reviewed:** 2025-12-12
**Next Review:** v0.3.0 Release
