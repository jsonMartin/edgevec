# EdgeVec Performance Baselines

**Version:** v0.4.0 (target)
**Date:** 2025-12-16
**Author:** W19.2 Benchmark Dashboard

---

## Executive Summary

EdgeVec is a WASM-first vector database optimized for browser and edge deployment. This document establishes performance baselines and targets for v0.4.0 release.

**Key Competitive Position:**
- **24x faster** than voy (another WASM vector library)
- **4x slower** than hnswlib-node (native C++ bindings) — acceptable trade-off for browser compatibility
- **<250KB** WASM bundle (compressed)

---

## Target Metrics

### Search Performance (10,000 vectors, 128 dimensions)

| Metric | Target | Measured | Status |
|:-------|:-------|:---------|:------:|
| Search P50 | <1.0 ms | **0.20 ms** | 5x under |
| Search P99 | <5.0 ms | **0.22 ms** | 23x under |
| Search Mean | <1.0 ms | **0.20 ms** | 5x under |

### Insert Performance

| Metric | Target | Measured | Status |
|:-------|:-------|:---------|:------:|
| Insert P50 | <2.0 ms | **0.83 ms** | 2.4x under |
| Insert P99 | <5.0 ms | **0.85 ms** | 6x under |
| Insert Mean | <2.0 ms | **0.79 ms** | 2.5x under |

### Resource Usage

| Metric | Target | Measured | Status |
|:-------|:-------|:---------|:------:|
| WASM Bundle (gzip) | <500 KB | **~220 KB** | 56% under |
| Memory per 10K vectors | <100 MB | **~2.76 MB** | 27x under |
| Index Load Time | <500 ms | **TBD** | TBD |

---

## Competitive Analysis

### Search Latency Comparison (P50, 10K vectors)

| Library | P50 (ms) | Relative to EdgeVec | Platform |
|:--------|:---------|:--------------------|:---------|
| **hnswlib-node** | 0.05 | 4x faster | Node.js (Native C++) |
| **EdgeVec** | 0.20 | Baseline | Browser WASM |
| **voy** | 4.78 | 24x slower | Browser WASM |

### Insert Latency Comparison (P50, 10K vectors)

| Library | P50 (ms) | Relative to EdgeVec | Platform |
|:--------|:---------|:--------------------|:---------|
| **voy** | 0.03 | 28x faster | Browser WASM |
| **EdgeVec** | 0.83 | Baseline | Browser WASM |
| **hnswlib-node** | 1.56 | 1.9x slower | Node.js (Native C++) |

### Memory Usage Comparison

| Library | Memory (MB) | Notes |
|:--------|:------------|:------|
| **EdgeVec** | ~0.1 | SQ8 quantization enabled |
| **hnswlib-node** | 2.76 | Full precision f32 |
| **voy** | 47.10 | Higher memory overhead |

---

## Benchmark Configuration

```json
{
  "dimensions": 128,
  "vectorCount": 10000,
  "queryCount": 100,
  "k": 10,
  "warmupRuns": 3,
  "measurementRuns": 5,
  "hnsw": {
    "m": 16,
    "efConstruction": 200,
    "efSearch": 50
  }
}
```

---

## Key Insights

### Why EdgeVec is Competitive

1. **SIMD Optimization:** Runtime detection uses AVX2/SSE4.1 when available
2. **Scalar Quantization (SQ8):** 4x memory reduction with <5% recall loss
3. **Efficient HNSW:** Optimized neighbor selection and graph traversal
4. **WASM-Native Design:** Built for browser from the ground up

### Trade-offs

| Trade-off | Impact | Justification |
|:----------|:-------|:--------------|
| Native vs WASM | 4x slower than C++ | Required for browser deployment |
| Insert speed | Slower than voy | Prioritized search performance |
| Memory | Higher than minimal | Enables instant queries |

---

## Performance Regression Thresholds

**CI will fail if:**

| Metric | Regression Threshold |
|:-------|:--------------------|
| Search P50 | >50% regression (>0.30 ms) |
| Search P99 | >50% regression (>0.33 ms) |
| Insert P50 | >50% regression (>1.25 ms) |
| WASM Bundle | >10% increase (>242 KB) |

---

## Benchmark Reproduction

### Running Competitive Benchmarks

```bash
# From project root
cd benches/competitive
npm install
npm run bench

# Results written to: benches/competitive/results/latest.json
```

### Viewing Dashboard

```bash
# From project root
python -m http.server 8000
# Open: http://localhost:8000/wasm/examples/benchmark-dashboard.html
```

### Running Rust Benchmarks

```bash
# Standard benchmarks
cargo bench

# P99 latency benchmarks
cargo bench --bench quant_bench
```

---

## Version History

| Version | Date | Change |
|:--------|:-----|:-------|
| v0.3.0 | 2025-12-15 | Initial baselines established |
| v0.4.0 | 2025-12-16 | Dashboard and documentation added |

---

## References

- **Interactive Dashboard:** `wasm/examples/benchmark-dashboard.html`
- **Detailed Analysis:** `docs/benchmarks/competitive_analysis.md`
- **Benchmark Source:** `benches/competitive/harness.js`
- **Baseline JSON:** `benches/competitive/results/latest.json`

---

**Next Review:** v0.5.0 (after ARM/NEON optimization)
