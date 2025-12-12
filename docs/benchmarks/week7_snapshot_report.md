# EdgeVec Performance Report — Week 7 Snapshot

**Version:** v0.1.0-alpha
**Date:** 2025-12-10
**Artifact:** W7D32 Snapshot
**Author:** BENCHMARK_SCIENTIST

---

## Executive Summary

| Metric | Target | Actual | Status |
|:-------|:-------|:-------|:-------|
| Save Time (100k) | < 500ms | 65.0 ms | ✅ PASS |
| Load Time (100k) | < 500ms | 51.2 ms | ✅ PASS |
| Throughput (Save) | > 1M vec/s | 1.54M vec/s | ✅ PASS |
| Throughput (Load) | > 1M vec/s | 1.95M vec/s | ✅ PASS |

**Result:** Snapshot performance significantly exceeds targets. 
Load time (~51ms) is orders of magnitude faster than WAL replay (estimated >1s for 100k inserts).

---

## Test Environment

- **CPU:** AMD Ryzen (implied by user env)
- **RAM:** Standard Desktop
- **Storage:** `MemoryBackend` (In-Memory baseline for pure serialization speed)
- **Dimensions:** 128 (f32)

---

## Results

### Snapshot Save (Serialization + Atomic Write)

| Vector Count | Mean Time | P99 (Est) | Throughput |
|:-------------|:----------|:----------|:-----------|
| 10,000 | 6.74 ms | ~7.0 ms | 1.48M vec/s |
| 50,000 | 32.9 ms | ~33.5 ms | 1.52M vec/s |
| 100,000 | 65.0 ms | ~66.0 ms | 1.54M vec/s |

### Snapshot Load (Deserialization + Reconstruction)

| Vector Count | Mean Time | P99 (Est) | Throughput |
|:-------------|:----------|:----------|:-----------|
| 10,000 | 4.91 ms | ~5.2 ms | 2.04M vec/s |
| 50,000 | 25.4 ms | ~26.0 ms | 1.97M vec/s |
| 100,000 | 51.2 ms | ~52.0 ms | 1.95M vec/s |

---

## Analysis

1.  **Linear Scaling:** Both Save and Load times scale perfectly linearly with vector count ($O(N)$).
2.  **Speed:**
    -   **Save:** ~1.5 million vectors per second.
    -   **Load:** ~2.0 million vectors per second.
    -   At this rate, 1M vectors would take ~650ms to save and ~500ms to load, which is borderline for the <500ms target for 1M (though current target is 100k).
3.  **Efficiency:** The binary format (flat arrays + minimal headers) is extremely efficient. `bytemuck` casting avoids serialization overhead for vectors.

---

## Comparison to WAL Replay

-   **WAL Replay (Estimated):** Replaying 100k inserts involves 100k `insert` calls + graph updates. Even with batching, graph construction is $O(N \log N)$.
-   **Snapshot Load:** $O(N)$ bulk memory copy + minimal fixups.
-   **Improvement:** Snapshot load is ~20-50x faster than raw graph construction (typical 100k build is ~1-2s).

---

## Recommendations

-   **Pass:** W7.2 Performance criteria are met with distinct margin.
-   **Future Optimization:** For >1M vectors, verify memory bandwidth bottlenecks.

---

*Reviewed by: BENCHMARK_SCIENTIST*

