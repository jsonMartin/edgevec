# HOSTILE_REVIEWER: Rejection — Week 3 Implementation Plan

**Date:** 2025-12-07
**Artifact:** Week 3 Implementation Plan (Week 3 Plan + Daily Plans)
**Author:** PLANNER
**Status:** ❌ REJECTED

---

## Summary

The Week 3 plan aims to implement the core HNSW algorithms (Insert, Search) and the compressed `NeighborPool`. While the algorithmic steps are largely aligned with the architecture, a critical contradiction regarding memory management renders the plan unacceptable in its current form.

---

## Findings

### Critical Issues: 1
- [C1] **Memory Budget Contradiction (W3D14 vs Architecture)**
  - **Description:** W3D14 (Section 2.2) explicitly allows "orphaned lists" in the `NeighborPool` ("orphaned lists are acceptable for now, compaction is later"). However, `WEEKLY_TASK_PLAN.md` (Validation Criteria) requires demonstrating "<100 bytes/vector overhead", and `ARCHITECTURE.md` (R5) sets this as a HIGH priority requirement.
  - **Evidence:** W3D14 Line 45 vs WEEKLY_TASK_PLAN.md Line 54.
  - **Impact:** With "append-only" updates, every edge update in the HNSW graph (which happens frequently during insertion) will allocate new memory. For 10k vectors, estimated churn could easily exceed 600 bytes/vector (assuming ~10 updates per insert), violating the 100-byte budget by 6x. This fails the "Validation Criteria" defined in the plan itself.
  - **Required Action:** Either:
    1. Implement a basic compaction or free-list mechanism in W3D12/W3D14.
    2. OR, explicitly relax the Week 3 Validation Criteria to allow for higher memory usage (e.g., "Verification of compression logic, ignoring fragmentation overhead"), acknowledging this as technical debt to be paid in Week 4.

### Major Issues: 1
- [M1] **Vague Error Handling Specification in Insert**
  - **Description:** W3D14 defines `HnswIndex::insert` but does not specify the return type or error handling strategy for internal failures (e.g., allocation failure, invalid config). `ARCHITECTURE.md` (INV-API-1) requires all public methods to be FFI-safe (Result types).
  - **Required Action:** Explicitly define the signature of `insert` in W3D14 to return `Result<VectorId, EdgeVecError>` and specify how internal errors (like pool exhaustion) are propagated.

### Minor Issues: 1
- [m1] **Recall Benchmark Details**
  - **Description:** W3D15 specifies "Recall > 0.95" but doesn't specify the dataset distribution (Uniform? Gaussian?). HNSW behavior varies by distribution.
  - **Recommendation:** Specify "Uniform Random" for the initial baseline to ensure deterministic and comparable results.

---

## Verdict

**REJECTED**

This artifact fails [1] critical quality gate and cannot proceed. The memory leak "strategy" directly contradicts the stated success criteria.

---

## Required Actions Before Resubmission

1. [ ] **Resolve Memory Contradiction:** Decide whether to fix the allocator now (recommended) or formally defer the memory budget requirement.
2. [ ] **Clarify Signatures:** Update W3D14 to reflect proper error handling.
3. [ ] **Update Plan:** Ensure Daily Plans and Weekly Validation Criteria are consistent.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-07*
*Verdict: REJECTED*

