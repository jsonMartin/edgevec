# EdgeVec Performance Report — Soft Delete (Tombstones)

**Date:** 2025-12-08
**Version:** v0.0.1-alpha (W5.3)
**Author:** BENCHMARK_SCIENTIST

---

## Executive Summary

| Metric | Target | Actual | Status |
|:-------|:-------|:-------|:-------|
| 50% Deleted Overhead | < 2.0x latency | ~1.95x (136µs vs 70µs) | ✅ PASS |
| Ghost Traversal | Visible | Yes | ✅ PASS |

**Verdict:** The soft delete implementation behaves exactly as expected for a tombstone-based system. We traverse ~2x as many nodes to find `k` valid neighbors when density is 50%, resulting in a proportional ~2x latency increase.

---

## Benchmark Scenario

- **Dataset:** 10,000 vectors (32 dimensions).
- **Metric:** L2 Squared.
- **Search:** `k=10`.
- **Delete Ratio:** 0% vs 50% (5,000 deletions).
- **Method:** `delete(i)` called on every even index.

### Why 32 Dimensions?
We reduced dimensions to 32 (from 128) to emphasize graph traversal overhead over distance calculation overhead. This makes the "ghost node" penalty more visible.

---

## Results

| Scenario | P50 Latency | Throughput | Overhead Factor |
|:---------|:------------|:-----------|:----------------|
| **0% Deleted** | 69.6 µs | 14,362 ops/s | 1.0x (Baseline) |
| **50% Deleted** | 135.7 µs | 7,367 ops/s | 1.95x |

### Analysis

1.  **Linear Penalty:** The overhead is almost perfectly linear with the deletion ratio. To find `k` valid neighbors in a field of 50% ghosts, the search algorithm must inspect roughly `2*k` candidates.
2.  **BitVec Efficiency:** checking `is_deleted` (bit access) is negligible compared to the distance calculation and memory fetch of the vector itself. The slowdown comes from *computing distances to ghosts*, not checking the flag.
3.  **Routing Integrity:** The search successfully completes, proving that ghost nodes effectively bridge the graph.

---

## Recommendations

1.  **Periodic Compaction:** While soft delete is fast (O(1)), the search penalty accumulates. We need a background `vacuum` process (Planned for W5.5 or Week 6) to rebuild the graph and remove ghosts.
2.  **Adaptive `ef`:** The `ef_search` parameter might need dynamic adjustment based on the deletion ratio (`tombstones / total`) to maintain recall, though HNSW is generally robust.

---

## Reproducibility

```bash
cargo bench --bench delete_bench
```

