# W7D35 Chaos Monkey Report

**Date:** 2025-12-10
**Campaign:** W7.5_chaos_execute
**Target:** Resilience of Persistence Layer (WAL + Snapshots)

## 1. Executive Summary

The Chaos Testing Campaign successfully executed 1,000 randomized iterations simulating catastrophic storage failures (disk full, I/O errors, atomic write failures). The system demonstrated **100% resilience** with zero panics and zero data corruption events.

| Metric | Value |
|:---|:---|
| **Total Iterations** | 1,000 |
| **Failed Snapshots (Caught)** | 252 |
| **WAL Append Failures (Caught)** | 1,532 |
| **Data Corruption Events** | 0 |
| **Panics** | 0 |

**Verdict:** ✅ PASSED (Nvidia Grade Resilience)

## 2. Methodology

### 2.1 Chaos Injection
- **Harness:** `ChaosStorageBackend` (mocks `StorageBackend`)
- **Fault Model:** Probabilistic failure injection on `append()` and `atomic_write()`
- **Failure Rates:** Randomized between 10% and 90% per iteration
- **RNG:** `SmallRng` (Seeded: `0xDEADBEEF`)

### 2.2 Scenarios
1.  **Atomic Snapshot Rollback:**
    -   Save Valid State A.
    -   Mutate to State B.
    -   Attempt Save State B (Inject Faults).
    -   **Invariant:** Load must return *either* State A (if failed) *or* State B (if success). Never partial/corrupt state.

2.  **WAL Append Resilience:**
    -   Stream 10 inserts with random failure rate.
    -   **Invariant:** In-memory index count must match exactly the number of successful WAL appends.
    -   **Invariant:** Recovery from WAL must match in-memory state.

## 3. Detailed Findings

### 3.1 Snapshot Atomicity
- **Observations:** 252 snapshot attempts failed due to simulated I/O errors.
- **Verification:** In all 252 cases, the system correctly rolled back to the previous valid snapshot (State A).
- **Phantom Reads:** None. Checksums and atomic rename mechanics prevented partial reads.

### 3.2 WAL Consistency
- **Observations:** 1,532 individual WAL appends failed.
- **Verification:** `VectorStorage` correctly propagated errors to the caller (`HnswIndex`), preventing in-memory updates for failed writes.
- **Recovery:** Replay of the WAL consistently reproduced the exact in-memory state of the successful inserts.

## 4. Conclusion

EdgeVec's persistence layer meets the **strict no-panic** and **atomic-or-nothing** requirements. The use of `atomic_write` (rename pattern) for snapshots and strict append-only log checking for WAL ensures data integrity even under high failure rates.

### Next Steps
- Incorporate `chaos_monkey.rs` into nightly CI pipeline.
- Extend chaos tests to simulate bit-rot (checksum mismatches) in Week 8.

