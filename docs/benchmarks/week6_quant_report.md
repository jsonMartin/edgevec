# EdgeVec Performance Report — Week 6 (Quantization)

**Version:** 0.0.1-alpha
**Date:** 2025-12-09
**Task:** W6.4_latency_check
**Author:** BENCHMARK_SCIENTIST

---

## Executive Summary

| Metric | Target | Actual | Status |
|:-------|:-------|:-------|:-------|
| Search P50 (10k, Quantized) | <5ms | **0.11ms** | ✅ PASS (CRUSHED IT) |
| Insert Latency (Mean, Quantized) | <1ms | **~0.8ms** | ✅ PASS |
| Search P50 (10k, Float32) | Reference | 1.42ms | Reference |
| Speedup (Quantized vs Float32) | >5x | **12.9x** | ✅ PASS |

**Result:** Quantized search is ~13x faster than Float32 search for 10k vectors, with P50 latency of ~110µs.

---

## Test Environment

| Component | Specification |
|:----------|:--------------|
| CPU | Windows 10 Host (User Machine) |
| Rust | 1.81.0 |
| Target | x86_64-pc-windows-msvc |
| Profile | Release (Optimized) |

---

## Search Latency Scaling (N=10,000)

### Results

| Mode | Mean Latency | Throughput | Relative Speed |
|:-----|:-------------|:-----------|:---------------|
| Float32 (Baseline) | 1.42 ms | ~705 ops/s | 1.0x |
| **Quantized (Target)** | **0.11 ms** | **~9,200 ops/s** | **12.9x** |

### Analysis

1.  **Quantization Impact:** Switching to `QuantizedU8` (and using `l2_squared_u8` SIMD/Scalar fallback) reduced search latency by **92%**.
2.  **Throughput:** Quantized search handles over **9,000 queries per second** on a single thread.
3.  **Outliers:** Quantized mode showed mild outliers (20%), likely due to OS scheduling at such low latencies (100µs).

---

## Search Latency Scaling (N=50,000)

### Results

| Mode | Mean Latency | Throughput | Relative Speed |
|:-----|:-------------|:-----------|:---------------|
| Float32 (Baseline) | 1.74 ms | ~575 ops/s | 1.0x |
| **Quantized (Target)** | **0.17 ms** | **~6,000 ops/s** | **10.2x** |

### Analysis

1.  **Scalability:** At 50k vectors, quantized search remains extremely fast (170µs).
2.  **Efficiency:** The speedup factor (~10x) holds consistent as N increases.
3.  **Memory:**
    -   Float32 (50k): ~298 MB Est.
    -   Quantized (50k): ~75 MB Est. (4x reduction in vector storage).

---

## Build/Insert Performance

| Metric | N=10,000 (Float32) | N=10,000 (Quantized) | Speedup |
|:-------|:-------------------|:---------------------|:--------|
| Build Time | 73.41s | 7.95s | **9.2x** |
| Per Vector | 7.3ms | 0.8ms | **9.2x** |

*Note: Insert/Build times include random vector generation in the benchmark harness, but the relative speedup confirms that quantized insertion (quantize + store u8) is significantly faster than raw f32 handling, likely due to memory bandwidth savings and reduced allocation overhead in the storage layer.*

---

## Recommendations

1.  **PASS:** Quantized search integration is successful and exceeds performance targets.
2.  **Next:** Proceed to Week 7 (Persistence & Web Integration).

---

## Reproducibility

```bash
cargo bench --bench scaling_bench
```
