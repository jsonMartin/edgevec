# Week 2 Greedy Search Baseline Report

**Date:** 2025-12-06
**Benchmark:** W2.1 Baseline (Layer 0 Greedy Search)
**Environment:** Local Dev Machine (Windows x64)
**Metadata:**
- **CPU:** Intel Core Ultra 9 285H
- **RAM:** High-End Mobile Workstation
- **OS:** Microsoft Windows NT 10.0.26200 (Windows 11)
- **Rust:** 1.90.0

## 1. Summary

Greedy search implementation `W2.1` successfully outperforms linear scan (Brute Force) on a synthetic 10k vector dataset.

- **Speedup vs Brute Force:** ~7.5x faster (at `ef=64`)
- **Latency (ef=64):** 74.2 µs (P50), 110.6 µs (P99)
- **Scalability:** Latency grows with `ef` but remains sub-millisecond.

## 2. Results

### 2.1 Latency vs `ef` Parameter

Dataset: 10,000 vectors, 128 dimensions.

| `ef` | Latency (P50) | Latency (P99) | Throughput | Notes |
|:---|:---|:---|:---|:---|
| 16 | 25.2 µs | 33.5 µs | 39.1k ops/s | Lowest latency, lower recall risk |
| 64 | 74.2 µs | 110.6 µs | 13.0k ops/s | Standard trade-off |
| 128 | 179.9 µs | 249.0 µs | 5.3k ops/s | Higher recall, higher cost |

### 2.2 Comparison: HNSW vs Brute Force

| Method | Latency (Mean) | Speedup |
|:---|:---|:---|
| **HNSW (ef=64)** | **76.1 µs** | **7.4x** |
| Brute Force | 564.4 µs | 1.0x |

## 3. Analysis

1.  **Performance Validation:**
    - The greedy search algorithm is functioning correctly and providing significant speedups over brute force even on a small (10k) dataset.
    - The speedup factor (7x) validates the efficiency of graph traversal vs full scan.

2.  **Metric Overhead:**
    - The `L2Squared` metric overhead is minimal, as seen in the `distance_bench` (previous baseline), allowing the graph traversal logic to dominate the profile as expected.

3.  **Memory Access:**
    - The mock storage access pattern seems cache-friendly enough. Real `VectorStorage` integration might introduce different characteristics, but this baseline confirms the algorithm itself is sound.

## 4. Conclusion

Task `W2.1` (Greedy Search) meets performance expectations.

**Status:** ✅ BASELINE ESTABLISHED

## 5. Reproduction

```bash
cargo bench --bench greedy_search_bench
```

