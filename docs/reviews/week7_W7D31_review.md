# HOSTILE_REVIEWER: Rejection — W7D31 (WAL Implementation)

**Date:** 2025-12-10
**Artifact:** W7D31 (WAL)
**Author:** RUST_ENGINEER
**Status:** ❌ REJECTED

---

## Summary

Review of the Write-Ahead Log (WAL) implementation in `src/persistence/wal.rs` and associated tests/benchmarks. The implementation provides sequential append-only logging with CRC32 checksums.

---

## Findings

### Critical Issues: 1
- [C1] **Magic Numbers without Constants**
  - Description: The values `16` (Header Size) and `4` (CRC Size) are hardcoded in multiple locations.
  - Evidence: 
    - `src/persistence/wal.rs:93`: `[0u8; 16]`
    - `src/persistence/wal.rs:130`: `[0u8; 4]`
    - `src/persistence/wal.rs:209`: `Vec::with_capacity(16 + payload_len + 4)`
  - Impact: Violation of Quality Standard "No magic numbers". Tight coupling between `WalEntry` size and buffer allocation/reading logic creates maintenance risk. If `WalEntry` size changes, these magic numbers must be updated manually, or code will panic/fail.
  - Required Action: Define `WAL_HEADER_SIZE` (derived from `size_of::<WalEntry>()` or constant) and `WAL_CRC_SIZE`. Use them throughout.

### Major Issues: 1
- [M1] **Persistence Format Endianness Inconsistency**
  - Description: The code explicitly handles Endianness for CRC (`u32::from_le_bytes`), but uses `bytemuck` for `WalEntry`.
  - Evidence:
    - `src/persistence/wal.rs:141`: `u32::from_le_bytes(crc_bytes)` (Explicit LE)
    - `src/persistence/wal.rs:104`: `entry_bytes.copy_from_slice(&header_bytes)` (Native Endian via `bytemuck`)
  - Impact: If `WalEntry` contains multi-byte integers (e.g. `sequence: u64`), the on-disk format becomes platform-dependent. While WASM is LE, a "Persistence Format" should be robust or explicitly defined as LE.
  - Required Action: Explicitly document that `WalEntry` is serialized in Native Endian (and thus format is platform-specific) OR enforce LE for `WalEntry` fields. Given `edgevec` constraints, explicit LE is preferred for the binary format.

### Minor Issues: 2
- [m1] **Generic Backend vs Box**
  - `WalAppender` uses `Box<dyn StorageBackend>`. While acceptable, a generic `WalAppender<S: StorageBackend>` would avoid virtual dispatch. Accepted for now as performance is dominated by I/O.
- [m2] **Hardcoded Validation Logic**
  - `WalIterator` assumes `WalEntry` is the first 16 bytes. If `WalEntry` structure changes, the logic in `next()` (reading 16 bytes then casting) breaks without compile-time safety (runtime panic on copy or misinterpretation).

---

## Verdict

**REJECTED**

This artifact fails strict quality gates (Magic Numbers, Endianness Consistency) and cannot proceed.

---

## Required Actions Before Resubmission

1. [ ] Define `WAL_HEADER_SIZE` and `WAL_CRC_SIZE` constants.
2. [ ] Update `WalIterator` and `WalAppender` to use these constants.
3. [ ] Verify or enforce Endianness for `WalEntry` (or explicitly document the platform dependency).

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-10*
*Verdict: REJECTED*

