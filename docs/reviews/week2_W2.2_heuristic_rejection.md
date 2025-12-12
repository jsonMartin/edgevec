# HOSTILE_REVIEWER: Rejection — W2.2 Heuristic Selection

**Date:** 2025-12-06
**Artifact:** W2.2 Heuristic Selection (heuristic.rs, benchmarks)
**Author:** RUST_ENGINEER / BENCHMARK_SCIENTIST
**Status:** ❌ REJECTED

---

## Summary

The heuristic neighbor selection implementation (W2.2) was reviewed. It implements the diversity check from HNSW Section 4 but fails strict correctness checks regarding equality handling, contains forbidden `unwrap()` calls in the hot loop, and the accompanying benchmark incorrectly establishes the baseline by failing to measure true "Simple" selection.

---

## Findings

### Critical Issues: 2
- [C1] **Benchmark Integrity Failure**
  - Description: `week2_heuristic.md` compares "Heuristic (No Extend)" vs "Heuristic (Extend)" but labels the former as "Simple/No-Extend".
  - Evidence: `heuristic.rs` unconditionally executes the diversity check (lines 139-182). There is no code path for "Simple" selection (Top-M without diversity).
  - Impact: The "Overhead < 5x Simple" constraint cannot be verified because "Simple" selection was never measured. The report measures the cost of *Extension*, not the cost of the *Heuristic*.
  - Required Action: Implement true Simple selection (or a flag to bypass diversity check) and re-run benchmarks to establish valid overhead data.

- [C2] **Logic Deviation (Equality Handling)**
  - Description: The implementation keeps a candidate if it is equidistant to `query` and `neighbor` ($d(c,q) == d(c,r)$), violating the strict "closer" requirement.
  - Evidence: `heuristic.rs:170` uses `if dist_c_r < cand.distance` to discard. This allows candidates where `dist_c_r == cand.distance` to remain.
  - Impact: Deviates from HNSW Paper Section 4 ("only if it is closer"). Reduces diversity in geometric edge cases (e.g., equilateral triangles).
  - Required Action: Change condition to `dist_c_r <= cand.distance` to strictly enforce "closer" requirement.

### Major Issues: 2
- [M1] **Forbidden `unwrap()` in Library Code**
  - Description: Usage of `unwrap()` in a hot loop.
  - Evidence: `heuristic.rs:110`: `let bytes: [u8; 4] = chunk.try_into().unwrap();`.
  - Required Action: Remove `unwrap()`. While `chunks_exact(4)` makes it theoretically safe, library code must use `try_into().expect()` with a message or handle the error to satisfy the "No unwrap" rule.

- [M2] **Unnecessary Allocation in Hot Path**
  - Description: The `working_set` is cloned entirely when `extend_candidates` is true.
  - Evidence: `heuristic.rs:103`: `let initial_candidates = working_set.clone();`.
  - Required Action: Refactor to iterate by index or other means to avoid allocating a new vector on every extended search.

### Minor Issues: 1
- [m1] **Docstring Mismatch**
  - Description: Docstring claims to support "both simple selection ... and heuristic selection", but the method only supports heuristic selection.

---

## Verdict

**REJECTED**

This artifact fails 2 critical quality gates and cannot proceed.

---

## Required Actions Before Resubmission

1. [ ] Fix the diversity check logic to handle equality correctly (strictly closer).
2. [ ] Remove `unwrap()` from `heuristic.rs`.
3. [ ] Optimize `extend_candidates` to remove the `clone()`.
4. [ ] Update `select_neighbors` to support a true "Simple" mode (bypass diversity) OR update benchmarks to measure against a separate Simple implementation.
5. [ ] Re-run benchmarks with valid "Simple" baseline.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-06*
*Verdict: REJECTED*

