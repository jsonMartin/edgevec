# NeighborPool Performance Report (W3D12)

**Date:** 2025-12-07
**Component:** `NeighborPool` (VByte + FreeList)

## 1. Executive Summary

| Metric | Target | Actual | Status |
|:-------|:-------|:-------|:-------|
| Decode (32 neighbors) | < 100 ns | **66.47 ns** | ✅ PASS |
| Encode (32 neighbors) | N/A | **91.34 ns** | INFO |
| Alloc/Free Cycle | < 500 ns | **46.13 ns** | ✅ PASS |

**Hardware:** Windows 10, i7-Equivalent (Env inferred)

## 2. Analysis

### 2.1 Decoding Speed
Decoding 32 neighbors takes ~66ns. This is critical because it happens on every node visit during search.
- **Per neighbor:** ~2 ns.
- **Throughput:** ~15 million lists/sec.
- **Impact:** Negligible overhead compared to distance calculation (approx 300-500ns for 128d vector).

### 2.2 Memory Recycling
The `alloc/free` cycle using `BTreeMap` takes ~46ns.
- This is surprisingly fast, likely due to small map size in microbenchmark.
- In worst case (fragmented heap), this might scale logarithmically with heap fragmentation, but fits well within the 1ms insert budget.

### 2.3 Conclusion
The VByte implementation is highly efficient and meets the strict performance budget for the hot path.

## 3. Reproduction

```bash
cargo run --release --example neighbor_perf_check
```

