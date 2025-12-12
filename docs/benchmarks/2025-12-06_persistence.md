# EdgeVec Persistence Benchmarks - Week 1 Day 5

**Date:** 2025-12-06
**Version:** v0.1.0 (Persistence Alpha)
**Hardware:** Windows 10 (User Env)
**Status:** ✅ PASSED

---

## Executive Summary

| Metric | Target | Actual | Status |
|:-------|:-------|:-------|:-------|
| Header Write Latency | < 1000 ns | 40.2 ns | ✅ PASS |
| Header Read Latency | < 1000 ns | 18.4 ns | ✅ PASS |
| Write Throughput | > 1 GB/s | 1.48 GB/s | ✅ PASS |
| Read Throughput | > 1 GB/s | 3.24 GB/s | ✅ PASS |

**Conclusion:**
Header serialization/deserialization overhead is negligible (< 0.1 μs). This confirms that the custom binary format with direct byte mapping (via `copy_from_slice` and `from_le_bytes`) is extremely efficient.

---

## Detailed Results

### 1. Write Header (Serialization)
- **Mean Time:** 40.25 ns
- **Throughput:** 1.48 GB/s
- **Analysis:** Dominated by memory copies and CRC32 calculation. The fixed-size 64-byte buffer fits in L1 cache, making this operation effectively free compared to disk IO.

### 2. Read Header (Deserialization)
- **Mean Time:** 18.42 ns
- **Throughput:** 3.24 GB/s
- **Analysis:** Faster than write because CRC validation is highly optimized in `crc32fast`, and reading is just parsing bytes.

---

## Reproducibility

```bash
cargo bench --bench persistence_bench
```

---

## Methodology

- **Tool:** Criterion.rs
- **Iterations:** 100 samples
- **Warmup:** 3 seconds
- **Data:** 64-byte fixed buffer
- **Validation:** `black_box` usage to prevent compiler optimization of unused results.

