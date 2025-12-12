# HOSTILE_REVIEWER: Rejection — W3D11 HNSW Infrastructure

**Date:** 2025-12-07
**Artifact:** W3D11 Deliverables (HNSW Init)
**Author:** RUST_ENGINEER
**Status:** ❌ REJECTED

---

## Summary

The review covered the initial HNSW infrastructure code, including `HnswIndex`, `HnswNode`, and configuration logic. The deliverable was expected to contain ONLY the structural skeleton and layer generation logic (W3.1), without search or insertion algorithms.

---

## Findings

### Critical Issues: 3

- [C1] **Compilation Failure**
  - Description: `cargo test --lib` fails to compile. The file `src/hnsw/heuristic.rs` contains tests referencing a `Heuristic` struct that is not defined or imported. The file appears to contain *only* the test module and no implementation.
  - Evidence: `error[E0433]: failed to resolve: use of undeclared type Heuristic` in `src/hnsw/heuristic.rs:41`.
  - Impact: The code does not build. Blocked.
  - Required Action: Implement `Heuristic` struct or remove the broken tests/file if it's for a future task.

- [C2] **Forbidden `unwrap()` in Library Code**
  - Description: `unwrap()` is used in `src/hnsw/search.rs` inside the `search_layer` function.
  - Evidence: `src/hnsw/search.rs:256`: `let bytes: [u8; 4] = chunk.try_into().unwrap();`
  - Impact: Violates "No `unwrap()` in library code" rule. Panics are unacceptable.
  - Required Action: Use `try_into().expect("msg")` if provably safe, or handle the error gracefully.

- [C3] **Scope Violation: Premature Implementation**
  - Description: `src/hnsw/search.rs` implements `search_layer` (Greedy Search). This logic belongs to Task W3.3, not W3.1 (Infrastructure). The instructions explicitly asked: "Are insert/search truly absent?". They are present.
  - Evidence: `src/hnsw/search.rs` exists and contains full search logic.
  - Impact: Violates "Code Without Plan" and strict scope control. Premature code is often untested or buggy (as seen with the unwrap).
  - Required Action: Remove `search_layer` logic until W3.3 is active, or if intended to be fast-tracked, it must be fully compliant (no unwraps) and planned. Given the unwrap, it should be removed or fixed.

### Major Issues: 0

### Minor Issues: 2

- [m1] **Data Layout Deviation**
  - Description: `HnswIndex` struct in `src/hnsw/graph.rs` includes `max_layer` and `level_mult` fields, which are not specified in `DATA_LAYOUT.md`.
  - Required Action: Add these runtime fields to `DATA_LAYOUT.md` or mark them as transient/ignored in serialization docs.

- [m2] **Unused Benchmark Code**
  - Description: `benches/hnsw_init_bench.rs` contains an unused function `measure_empty_index_size`.
  - Required Action: Remove dead code.

---

## Verdict

**REJECTED**

This artifact fails 3 critical quality gates (Compilation, Safety, Scope) and cannot proceed.

---

## Required Actions Before Resubmission

1. [ ] **Fix Compilation:** Ensure `cargo test --lib` passes. Implement or remove `Heuristic`.
2. [ ] **Remove `unwrap()`:** Fix line 256 in `src/hnsw/search.rs` (or remove the file if removing search logic).
3. [ ] **Scope Check:** Remove `search_layer` logic if it is not part of the approved W3.1 task. If it stays, it must be perfect.
4. [ ] **Run Tests:** Verify `cargo test` passes on all modules.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-07*
*Verdict: REJECTED*


