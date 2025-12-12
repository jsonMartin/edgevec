# HOSTILE_REVIEWER: Rejection — W3D14_Artifacts_v4

**Date:** 2025-12-07
**Artifact:** W3D14_Artifacts_v4 (Codebase)
**Author:** RUST_ENGINEER
**Status:** ❌ REJECTED

---

## Summary

Gate 3 Re-check for Week 3 Day 14 deliverables (HNSW Insert Logic). The review focused on verifying benchmark suite health and ensuring 100% test passage. While the benchmark suite compilation issues were resolved, a critical correctness bug was discovered via property testing.

---

## Findings

### Critical Issues: 1
- [C1] **Graph Integrity Violation (Buffer Reuse)**
  - **Description:** The `add_connection` logic in `src/hnsw/insert.rs` incorrectly manages memory when `NeighborPool` returns a reused slot larger than the requested size. The code sets `node.neighbor_len` to the full `capacity` of the slot but only writes `new_size` bytes. The remaining bytes (tail) contain garbage from previous usage. Because `scan_blob_for_layer` iterates based on `neighbor_len`, it interprets this garbage as valid VByte-encoded layers/neighbors, resulting in "ghost" neighbors and violating the `M_MAX` invariant.
  - **Evidence:** 
    - `tests/proptest_hnsw_insert.rs` failed with `Test failed: Node NodeId(49) at layer 2 has 9 neighbors, max 5`.
    - `src/hnsw/insert.rs:397` sets `node.neighbor_len = new_capacity;`.
    - `src/hnsw/neighbor.rs:60` (`alloc`) does not clear reused memory.
    - No code in `add_connection` zeros out the range `new_size..new_capacity`.
  - **Impact:** Graph corruption. Search correctness is compromised as nodes effectively have random neighbors added.
  - **Required Action:** In `add_connection`, zero-fill the unused portion of the allocated buffer (`new_size..new_capacity`) before updating the node.

### Major Issues: 0

### Minor Issues: 0
- Benchmarks show regressions ("Performance has regressed") but remain within absolute constraints (<1ms insert mean).

---

## Verdict

**REJECTED**

This artifact fails **1** critical quality gate (Broken Tests / Graph Corruption) and cannot proceed.

---

## Required Actions Before Resubmission

1. [ ] **Fix [C1]:** Modify `src/hnsw/insert.rs` to zero-out unused buffer capacity in `add_connection`.
2. [ ] **Verify:** Ensure `tests/proptest_hnsw_insert.rs` passes consistently.
3. [ ] **Resubmit:** Run `cargo test` and `cargo bench` before requesting re-review.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-07*
*Verdict: REJECTED*

