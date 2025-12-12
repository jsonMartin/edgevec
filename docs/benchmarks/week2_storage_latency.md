# Benchmark: Storage Insert Latency (W2.10)

**Date:** 2025-12-07
**Author:** BENCHMARK_SCIENTIST
**Status:** ✅ PASS

## 1. Executive Summary

| Metric | Target | Result | Status |
|:-------|:-------|:-------|:-------|
| Insert (Memory) | Baseline | 325 ns | N/A |
| Insert (WAL) | < 1 ms | 5.1 µs | ✅ PASS |

**Conclusion:** Write-Ahead Logging adds ~4.8µs overhead per insert, remaining well within the 1ms budget.

## 2. Methodology

- **Framework:** Criterion.rs
- **Scenarios:**
  - `insert_memory_only`: `VectorStorage` with `wal: None`.
  - `insert_with_wal`: `VectorStorage` with `wal: Some(WalAppender)` backed by a temporary file.
- **Hardware:**
  - Disk: [Local Temporary Directory] (Likely NVMe SSD or OS Cache)
- **Note on Persistence:**
  - Current implementation uses `Write::flush()`. On standard file systems, this flushes user-space buffers to the OS page cache but does not guarantee physical disk persistence (fsync).
  - The measured 5.1µs latency reflects the cost of serialization + syscalls + OS cache write.
  - Strict `fsync` would likely increase latency to 100µs–1ms range (SSD).

## 3. Results

### 3.1 In-Memory Baseline
- **Mean:** 325 ns
- **Throughput:** ~3.0 Million ops/sec

### 3.2 With WAL (Durable)
- **Mean:** 5.13 µs
- **Throughput:** ~195,000 ops/sec
- **Overhead:** ~15x slower than in-memory, but negligible in absolute terms (< 0.01 ms).

## 4. Analysis

The overhead comes from:
1. **Serialization:** Copying bytes to payload buffer.
2. **CRC32:** Calculating checksum.
3. **Syscall:** Writing to the file descriptor.

## 5. Recommendations

1. **Accept W2.10:** The implementation meets the performance budget.
2. **Future Work:** If strict power-loss protection is required, upgrading `WalAppender` to support `fsync` (and batching commits) should be considered, though it will impact latency.

## 6. Reproducibility

```bash
cargo bench --bench storage_bench
```

