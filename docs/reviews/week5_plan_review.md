# HOSTILE_REVIEWER: Rejection — Week 5 Plan

**Date:** 2025-12-08
**Artifact:** Week 5 Plan
**Author:** PLANNER
**Status:** ❌ REJECTED

---

## Summary

The Week 5 Plan proposes implementing SIMD acceleration, Persistence Hardening, Soft Deletes, and Advanced Fuzzing. The plan contains critical flaws regarding the correctness of the Delete operation (Graph Connectivity) and the feasibility of the SIMD strategy on Stable Rust.

---

## Findings

### Critical Issues: 2

- [C1] **Delete Operation Breaks Graph Connectivity**
  - **Description:** W5D23 Task 3 specifies: "Update `HnswIndex::search` to skip `deleted` nodes."
  - **Evidence:** W5D23.md Line 14.
  - **Impact:** "Skipping" nodes during graph traversal (routing) will break the small-world navigation properties, creating "orphans" and making valid nodes unreachable if the deleted node was a bridge.
  - **Required Action:** Explicitly specify that deleted nodes must remain participants in the routing graph (traversable) but are filtered from the final Candidate Set returned to the user.

- [C2] **Ambiguous SIMD Strategy & Nightly Risk**
  - **Description:** W5D21 proposes: "Use `wide` or `std::simd` (nightly)." and "Decision: Use `wide`... or `portable-simd` if we are already nightly."
  - **Evidence:** W5D21.md Lines 8-9.
  - **Impact:**
    1. The workspace mandates MSRV 1.70 (Stable). `std::simd` requires Nightly.
    2. The `wide` crate has limited/unverified support for `wasm32` SIMD128.
    3. The plan allows for a "Nightly" fallback without a concrete Feature-Gate Strategy (e.g., `cfg(feature = "nightly-simd")`).
  - **Required Action:** Define a deterministic strategy. If `wide` is used, verify WASM support. If `std::simd` is used, it MUST be gated behind a feature flag. If Stable WASM SIMD is required, `std::arch::wasm32` intrinsics should be the primary or fallback implementation.

### Major Issues: 0
*(Persistence Hardening checks passed; W5D22 explicitly includes corruption tests.)*

### Minor Issues: 1
- [m1] **Benchmarks rely on "Extrapolation"**
  - Description: W5D25 relies on extrapolating 1M vector performance from 200k. While acceptable for a "Simulation" task, it risks missing non-linear degradations (e.g., cache thrashing) that occur only at scale.

---

## Verdict

**REJECTED**

This artifact fails 2 critical quality gates and cannot proceed.

---

## Required Actions Before Resubmission

1. [ ] **Refine W5D23 (Delete):** Explicitly distinguish between "traversal visibility" (must include deleted nodes) and "result visibility" (must exclude them).
2. [ ] **Refine W5D21 (SIMD):** Select a concrete SIMD approach. If targeting Stable Rust (as per .cursorrules), use `std::arch::wasm32` or a confirmed stable-compatible crate. If Nightly is allowed, explicitly define the feature-gate strategy.
3. [ ] **Update WEEKLY_TASK_PLAN.md:** Ensure task descriptions match the refined day plans.

---

## Resubmission Process

1. Address ALL critical issues
2. Address ALL major issues
3. Update artifact with `[REVISED]` tag
4. Resubmit for hostile review

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-08*
*Verdict: REJECTED*

