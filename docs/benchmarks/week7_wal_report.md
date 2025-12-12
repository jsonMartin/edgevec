# EdgeVec Performance Report — Week 7 (WAL)

**Version:** v0.0.1-alpha
**Author:** BENCHMARK_SCIENTIST
**Date:** 2025-12-10

---

## Executive Summary

| Metric | Target | Actual | Status |
|:-------|:-------|:-------|:-------|
| WAL Append Latency (Memory) | < 1 µs | 0.057 µs | ✅ PASS |
| WAL Append Latency (File) | < 100 µs | 247 µs | ⚠️ WARN |
| Throughput (Memory) | > 10M ops/s | 17.3M ops/s | ✅ PASS |
| Throughput (File) | > 10k ops/s | 4.0k ops/s | ⚠️ WARN |

**Note:** File latency is higher than target due to synchronous `sync_all` on every append in the current `FileBackend` implementation. This is robust but slow.

---

## Test Environment

| Component | Specification |
|:----------|:--------------|
| OS | Windows 10 |
| Rust | 1.83.0 |
| Backend | `std::fs::File` |

---

## Detailed Results

### 1. Memory Backend (Serialization Overhead)
- **Time per 10k appends:** 579 µs
- **Time per append:** ~58 ns
- **Throughput:** 17.3 Million entries/sec
- **Conclusion:** Serialization and CRC32 calculation overhead is negligible. The core logic is extremely fast.

### 2. File Backend (I/O Overhead)
- **Time per 10k appends:** 2.47 s
- **Time per append:** ~247 µs
- **Throughput:** ~4,046 entries/sec
- **Analysis:**
    - The current implementation calls `file.sync_all()` on *every* append.
    - 247 µs is typical for an SSD fsync latency.
    - For higher throughput, we must implement **group commit** or **buffered writing** (sync every N ms).

---

## Recommendations

1.  **Optimize FileBackend:** Introduce a buffered mode where `sync_all` is called only on `flush()` or periodically, rather than on every `append()`.
2.  **Group Commit:** For high-throughput ingestion, allow batching multiple vector inserts into a single WAL entry or sync once per batch.
3.  **Acceptance:** The *logic* is efficient (Memory benchmark proves it). The *I/O strategy* is safe but conservative.

---

## Reproducibility

```bash
cargo bench --bench persistence_bench
```

