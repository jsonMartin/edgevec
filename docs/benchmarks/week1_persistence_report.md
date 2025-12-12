# EdgeVec Performance Report — 2025-12-06

**Version:** v0.1.0
**Commit:** (Latest local build)
**Author:** BENCHMARK_SCIENTIST

---

## Executive Summary

| Metric | Target | Actual | Status |
|:-------|:-------|:-------|:-------|
| Header Write Latency | < 1000 ns | 53.6 ns | ✅ PASS |
| Header Read Latency | < 1000 ns | 17.7 ns | ✅ PASS |
| Header Write Throughput | > 100 MB/s | 1.1 GB/s | ✅ PASS |
| Header Read Throughput | > 100 MB/s | 3.4 GB/s | ✅ PASS |

---

## Test Environment

| Component | Specification |
|:----------|:--------------|
| OS | Windows 10.0.26200 |
| Rust | 1.82.0 (User Environment) |
| Target | x86_64-pc-windows-msvc |
| Architecture | 64-bit |
| CPU | (Host Environment) |

---

## Persistence Latency (Header IO)

### Results

| Operation | P50 (ns) | Mean (ns) | Throughput |
|:----------|:---------|:----------|:-----------|
| **Write** | 53.2 | 53.6 | 1.11 GB/s |
| **Read** | 17.5 | 17.7 | 3.37 GB/s |

### Analysis

Performance exceeds the 1µs (1000ns) target by order of magnitude.
- **Write:** ~54ns is extremely fast, dominated by stack allocation and memcpy.
- **Read:** ~18ns confirms zero-copy/validation overhead is negligible.
- **CRC32:** The CRC32 calculation (part of both) is efficiently autovectorized.

### Impact
Persistence operations will be I/O bound, not CPU bound. The header overhead is effectively free compared to disk/network latency.

---

## Comparison: Target vs Actual

| Metric | Target Limit | Actual Result | Margin |
|:-------|:-------------|:--------------|:-------|
| Write Latency | 1000 ns | 53.6 ns | 18x faster |
| Read Latency | 1000 ns | 17.7 ns | 56x faster |

---

## Recommendations

1. **PASS:** Header IO performance is well within budget.
2. **TODO:** Week 2 benchmarks should focus on Vector IO (bulk read/write), which will be the real bottleneck.

---

## Reproducibility

```bash
# Run benchmark
cargo bench --bench persistence_bench
```

---

## Approval

| Reviewer | Verdict | Date |
|:---------|:--------|:-----|
| HOSTILE_REVIEWER | [PENDING] | |

