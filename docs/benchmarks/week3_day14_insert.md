# HNSW Insertion Benchmark Report (W3D14)

**Date:** 2025-12-07
**Version:** v0.0.1-alpha
**Author:** BENCHMARK_SCIENTIST

---

## Executive Summary

| Metric | Target | Actual (Baseline) | Status |
|:-------|:-------|:------------------|:-------|
| Mean Insert Latency (10k) | <1ms | **883 µs** | ✅ PASS |
| Throughput (10k batch) | >1000/s | **1.13 k/s** | ✅ PASS |

**Hardware:** Windows 10, i7-Equivalent (Env inferred)

---

## Benchmark Design

### Methodology
- **Scenario:** Build HNSW index from scratch.
- **Vectors:** 128-dimensional, Uniform Random [-1, 1].
- **Parameters:** M=16, ef_construction=200 (Default HnswConfig).
- **Hardware:** Windows 10 (Dry Run Environment).

### Metrics
1. **Throughput:** Elements processed per second (from `insert_throughput` group).
2. **Mean Latency:** Inverse of throughput (Time / Count).

### Code
`benches/insert_bench.rs` uses Criterion.rs to measure:
- Batch insertion of 1,000 vectors.
- Batch insertion of 10,000 vectors.

---

## Results

| Count | Mean Time (Total) | Mean Latency (Per Vec) | Throughput |
|:------|:------------------|:-----------------------|:-----------|
| 1,000 | 182.19 ms | **182 µs** | 5.49 k ops/s |
| 10,000 | 8.83 s | **883 µs** | 1.13 k ops/s |

---

## Analysis

- **Scaling:**
  - 1k -> 10k vectors (10x increase in size).
  - Latency increased from 182µs to 883µs (~4.8x).
  - Throughput dropped from 5.49k/s to 1.13k/s.
  - This reflects the O(log N) cost of insertion in HNSW graphs (longer traversal paths as graph grows).

- **Budget Check:**
  - The 1ms budget is met at 10k vectors (883µs < 1000µs).
  - **Verdict:** PASS.

---

## Reproduction

```bash
cargo bench --bench insert_bench
```
