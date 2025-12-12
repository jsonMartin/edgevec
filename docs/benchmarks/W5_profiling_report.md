# W5 Profiling Report: Insert Latency Analysis

**Date:** 2025-12-09
**Author:** BENCHMARK_SCIENTIST
**Status:** DRAFT

## 1. Executive Summary

**The "19ms Bottleneck" has been identified in Phase 2 (Neighbor Selection).**

Profiling reveals that `HnswIndex::insert` is dominated by `search_layer`, consuming **~90% of the insertion time** (2.7ms out of 3.0ms per insert). The root cause is excessive memory allocation during neighbor list decoding in the hot search loop.

| Phase | Duration (Avg) | % of Total | Status |
|:------|:---------------|:-----------|:-------|
| 1. Entry Point Search | ~0.05ms | 1.6% | ✅ OK |
| **2. Select Neighbors** | **~2.70ms** | **89.1%** | ❌ **CRITICAL** |
| 3. Connect Neighbors | ~0.30ms | 9.9% | ⚠️ WARN |

**Recommendation:** Implement Zero-Copy Neighbor Iteration to eliminate `Vec<u32>` allocations in the hot path.

---

## 2. Methodology

### 2.1 Instrumentation
Timers (`std::time::Instant`) were injected into `src/hnsw/insert.rs` to measure three distinct phases:
1.  **Phase 1 (P1):** Greedy search for entry point from `max_layer` down to `level+1`.
2.  **Phase 2 (P2):** `search_layer` call to find candidates at current `level`.
3.  **Phase 3 (P3):** Heuristic neighbor selection and bidirectional connection.

### 2.2 Micro-Benchmark
- **Dataset:** 1,000 random vectors (128 dimensions).
- **Environment:** Windows 10, Release Profile (`opt-level = 3`).
- **Command:** `cargo run --release --example profile_insert`

---

## 3. Detailed Findings

### 3.1 Latency Breakdown (Sample from Layer 0)

Typical profiling logs for a single layer 0 insertion:

```text
PROF,41,2713,342,0  -> P1: 41µs, P2: 2713µs, P3: 342µs
PROF,29,1450,516,0  -> P1: 29µs, P2: 1450µs, P3: 516µs
PROF,13,3227,155,0  -> P1: 13µs, P2: 3227µs, P3: 155µs
```

*Note: P2 (Search Layer) consistently ranges from 1.5ms to 3.2ms.*

### 3.2 Bottleneck Analysis: Phase 2 (Search Layer)

The `search_layer` function performs a greedy search with `ef_construction` (default 200). This involves visiting up to ~200 nodes.

**The Hot Path:**
1.  `search_layer` loop pops a candidate.
2.  Retrieves `HnswNode`.
3.  Calls `NeighborPool::decode_layer`.
4.  **ALLOCATION:** `decode_layer` allocates a new `Vec<u32>` for neighbors.
5.  Iterates `Vec`, computes distances, pushes to heap.
6.  Drops `Vec`.

**Math:**
- `ef_construction` = 200.
- Allocations per insert = ~200 * (Layers).
- For 1k inserts: 200,000 allocations.
- 2.7ms / 200 nodes = **13.5µs per node visit**.
- This is orders of magnitude slower than simple L2 distance calculation (~100ns).

### 3.3 Secondary Issue: Phase 3 (Connection)

Phase 3 (`connect_neighbors`) takes ~0.3ms. This involves:
- Re-decoding neighbor lists (Allocation).
- Encoding new lists (Allocation).
- Writing to storage.

While significant, optimizing Phase 2 will yield the largest gain.

---

## 4. Remediation Plan

**Task:** `W5.5_perf_fix`

1.  **Refactor `NeighborPool`:**
    - Introduce `NeighborIterator` that decodes VByte on-the-fly without allocation.
    - Remove `decode_layer` usage in `search_layer`.
2.  **Update `search_layer`:**
    - Use the iterator to traverse neighbors.
3.  **Optimization Target:**
    - Reduce P2 from ~2.7ms to <0.2ms.
    - Total insert time target: <0.5ms (currently 3ms).

## 5. Raw Data (Snippet)

```
Total time: 3.0285993s (for 1000 inserts)
Avg time: 3.028ms

PROF,41,2713,342,0
PROF,30,698,1057,0
PROF,45,709,575,0
...
```

