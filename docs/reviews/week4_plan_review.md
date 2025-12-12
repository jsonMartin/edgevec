# HOSTILE_REVIEWER: Rejection — Week 4 Implementation Plan

**Date:** 2025-12-07
**Artifact:** edgevec/docs/planning/weeks/week4/WEEKLY_TASK_PLAN.md
**Author:** PLANNER
**Status:** ❌ REJECTED

---

## Summary

The Week 4 plan focuses on WASM bindings and browser persistence via IndexedDB. While the approach correctly handles the "Async Gap" and thread-safety constraints, it fails to account for browser storage limits relative to the project's 1M vector scale target.

---

## Findings

### Critical Issues: 1

- [C1] **IndexedDB "Snapshot" Strategy Violates Scalability Goals**
  - **Description:** Task W4.3 and W4.4 implement a "Snapshot" persistence strategy where the entire database is serialized into a single `Uint8Array` and written to IndexedDB as one blob.
  - **Evidence:** `W4D18.md` explicitly states "save() triggers serialization and async IDB write" and `WASM_BOUNDARY.md` defines `export_bytes` returning a single `Uint8Array`.
  - **Impact:** The project targets 1M vectors (~100MB data). Many browsers (especially on mobile) have strict limits on single IndexedDB value sizes (often significantly less than 100MB) or will crash when attempting to allocate a contiguous 100MB buffer in JS/WASM glue code. This approach effectively caps the DB capacity well below the architectural target.
  - **Required Action:**
    1. Update W4.3/W4.4 to implement **chunked persistence** (splitting the serialized data into smaller chunks, e.g., 10MB) OR explicitly reduce the v1 persistence target to <100k vectors.
    2. Add a specific validation step for "Maximum Save Size".

### Major Issues: 1

- [M1] **Missing Memory Budget Verification for Serialization**
  - **Description:** Serializing the database creates a copy of the data in memory. For a 100MB index, this requires an additional ~100MB of contiguous memory for the buffer during the `save()` operation.
  - **Required Action:** Add a task or constraint to verify that `save()` checks for available WASM memory before attempting allocation to prevent OOM crashes.

### Minor Issues: 1

- [m1] **Main Thread Blocking during Serialization**
  - **Description:** While the IDB write is async, the serialization (`export_bytes`) appears to be synchronous. This will freeze the UI thread for large datasets.
  - **Recommendation:** Acceptable for v1, but should be documented as a known limitation in `WASM_BOUNDARY.md`.

---

## Verdict

**REJECTED**

This artifact fails 1 critical quality gate (Scalability/Storage Limits) and cannot proceed.

---

## Required Actions Before Resubmission

1. [ ] **Refactor W4.3/W4.4:** Modify the persistence plan to handle chunking or set explicit capacity limits for v1.
2. [ ] **Update Acceptance Criteria:** Add "Persistence works with X MB data" (where X is the target capacity size).
3. [ ] **Resubmit Plan:** Update `WEEKLY_TASK_PLAN.md` and `W4D18.md`.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-07*
*Verdict: REJECTED*

