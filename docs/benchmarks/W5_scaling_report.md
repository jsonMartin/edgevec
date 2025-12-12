# W5.5 Scaling Benchmark Report (Regression Check)

**Date:** 2025-12-09
**Version:** v0.0.2-regression
**Author:** BENCHMARK_SCIENTIST

---

## Executive Summary

| Metric | Target | Result (N=10k) | Status |
|:---|:---|:---|:---|
| **Insert Latency (Mean)** | **< 1ms** | **4.73ms** | ❌ **CRITICAL FAIL** |
| Search Latency (Mean) | < 1ms | 1.05ms | ❌ **FAIL (Fast)** |
| Memory Per Vector | < 100 bytes | 79 bytes | ✅ PASS |

**Verdict:** **ABORTED**. Benchmarks terminated early due to Fail-Fast trigger on Search Latency (>1ms). Insert latency is also severely degraded (4.73ms vs 1ms target).

---

## Detailed Results

### 1. Build Time (Insert Latency)

Vectors: 1536 dimensions, Random uniform distribution.

| N Vectors | Total Build Time | Mean Insert Latency |
|:---|:---|:---|
| 10,000 | 47.26s | **4.73ms** |
| 50,000 | - | *Aborted* |

**Analysis:**
- Insert latency is ~4.7x over budget.
- Memory usage is stable (79 bytes/vec graph overhead), but compute is the bottleneck.
- SIMD optimization attempts in W5.5 (SearchContext reuse) did not recover performance to <1ms levels.

### 2. Search Latency (Fail-Fast Check)

| N Vectors | Mean Latency | Status |
|:---|:---|:---|
| 10,000 | 1.05ms | **> 1.0ms Threshold** |

**Fail-Fast Trigger:**
> "10k Scaling Failed (>1ms). Aborting larger scales to save compute."

---

## Regression Notes

Compared to W5 Final Report (v0.0.1):
- **Insert:** 0.99ms (v0.0.1) -> 4.73ms (Current). **~370% Regression.**
- **Search:** 0.50ms (v0.0.1) -> 1.05ms (Current). **~110% Regression.**

**Hypothesis:**
1.  SIMD overhead for 1536d might be misconfigured or not activating (AVX2 detection).
2.  SearchContext reuse might be adding check overhead that outweighs allocation savings for small N, though N=10k should benefit.
3.  "Scalar fallback" logic added in W5.5 applies to <256d. 1536d uses SIMD.

---

## Next Steps

1.  **HOSTILE_REVIEWER** must decide whether to debug the regression or proceed with Week 6 Quantization (which avoids float32 entirely).
2.  Verify `RUSTFLAGS="-C target-cpu=native"` usage in previous benchmarks.

---
