# HOSTILE_REVIEWER: Rejection — W7D33 (Atomic Persistence)

**Date:** 2025-12-10
**Artifact:** W7D33 (Atomic Persistence)
**Author:** RUST_ENGINEER
**Status:** ❌ REJECTED

---

## Summary

Review of Task W7D33 "Atomic Save". The task aimed to implement `atomic_write` in the `StorageBackend` trait and provide implementations for File and IndexedDB backends, ensuring data safety during crashes.

---

## Findings

### Critical Issues: 1
- [C1] **Missing Atomic Write Failure Verification**
  - **Description:** The Plan `W7D33.md` explicitly required a unit test: `test_atomic_overwrite` with `MemoryBackend` (simulate failure). This test is **MISSING**.
  - **Evidence:** `tests/unit_atomic_save.rs` contains positive tests (`creates_file`, `overwrites`, `no_temp_left`) but no test that simulates a crash/failure during the write process (e.g., failure between write and rename, or failure of the rename itself). The "Faulty Backend" test (referring to `CorruptedBackend` in `proptest_persistence.rs`) validates *read integrity*, not *write atomicity* under failure.
  - **Impact:** We are relying on the theoretical correctness of `fsync` + `rename` without empirical verification in the test suite that the system behaves correctly (i.e., preserves old data) when the operation is interrupted.
  - **Required Action:** Implement a test using a Mock/Faulty backend that fails during the `atomic_write` sequence (e.g., fails the `rename` or the `write`) and asserts that the *original* data at the key remains unchanged.

### Major Issues: 1
- [M1] **Incorrect `IndexedDbBackend::append` Implementation**
  - **Description:** `IndexedDbBackend::append` is implemented to delegate to `atomic_write`, which **overwrites** the data instead of appending.
  - **Evidence:** `src/persistence/storage.rs:336`: `// We delegate to atomic_write on default_key with overwrite, which is INCORRECT for append`.
  - **Impact:** This breaks the `WriteAheadLog` (WAL) functionality on WASM, as WAL relies on appending. While W7D33 focuses on `atomic_write`, the `StorageBackend` trait implementation must be coherent.
  - **Required Action:** Implement `append` correctly for IDB (e.g., read-modify-write in transaction, or throw a distinct `NotImplemented` error if WAL is not supported on IDB yet), or strictly document that WAL is not supported on WASM in this iteration.

### Minor Issues: 1
- [m1] **Synchronous `read` in WASM Backend**
  - **Description:** `IndexedDbBackend::read` returns `Err` because synchronous read is impossible in IDB.
  - **Evidence:** `src/persistence/storage.rs:352`.
  - **Impact:** `load_snapshot` will fail on WASM. This limits the usability of the shared architecture. Accepted for now as this may be addressed in "Full WASM Bindings" (Deferred), but it leaves the WASM implementation in a broken state.

---

## Verdict

**REJECTED**

This artifact fails **1** critical quality gate (Missing Verification) and **1** major quality gate (Incorrect Implementation). It cannot proceed to W7D34.

---

## Required Actions Before Resubmission

1. [ ] Implement `test_atomic_failure_preserves_data` (or similar) using a Mock/Faulty backend to verify 2PC/Atomic property.
2. [ ] Fix or explicitly explicitly mark/gate `IndexedDbBackend::append` to prevent data loss (overwrite) if called.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-10*
*Verdict: REJECTED*

