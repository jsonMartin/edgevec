# HNSW Baseline Report (Week 3 Day 11)

**Date:** 2025-12-07
**Component:** `HnswIndex` (Initialization & Structure)
**Commit:** [Current]

---

## 1. Executive Summary

| Metric | Target | Actual | Status |
|:-------|:-------|:-------|:-------|
| Init Latency (Empty) | <100ns | 8.5ns | ✅ PASS |
| Stack Memory | <512 bytes | 432 bytes | ✅ PASS |
| Heap Overhead (Empty) | ~0 bytes | 0 bytes | ✅ PASS |

**Conclusion:**
The HNSW infrastructure is extremely lightweight. Initialization is effectively free (nanosecond scale), and the memory footprint is well within embedded constraints.

---

## 2. Detailed Metrics

### 2.1 Initialization Latency

**Benchmark:** `init_empty_index`
**Input:** `M=16, ef=100` (Standard Config)

| Run | Time (ns) |
|:----|:----------|
| Mean | **8.46 ns** |
| Min | 8.37 ns |
| Max | 8.56 ns |

**Analysis:**
Initialization primarily involves allocating empty vectors and seeding the RNG. The 8.5ns result confirms that no heavy pre-allocation or expensive computation occurs during `new()`.

### 2.2 Memory Layout

**Struct Size:** `432 bytes`

**Breakdown:**
- `rng` (ChaCha8Rng): ~360 bytes (bulk of the size)
- `config`: 32 bytes
- `nodes` (Vec): 24 bytes
- `neighbors` (NeighborPool): 24 bytes + size
- `entry_point` (Option<NodeId>): 8 bytes (padded)
- `max_layer`: 1 byte
- `level_mult`: 4 bytes

**Note:** The large RNG state is expected for a high-quality CSPRNG like ChaCha8. This state is per-index, which is acceptable.

---

## 3. Recommendations

1.  **RNG Optimization (Future):** If 432 bytes is too large for specific embedded use cases, we could switch to a smaller PCG or Xoshiro generator, but ChaCha8 is safer and preferred for now.
2.  **Proceed:** No performance blockers found.

---

## 4. Reproducibility

```bash
cargo bench --bench hnsw_init_bench
cargo test --test size_check -- --nocapture
```

