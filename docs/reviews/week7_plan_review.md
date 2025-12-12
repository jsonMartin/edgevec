# HOSTILE_REVIEWER: Rejection — Week 7 Plan

**Date:** 2025-12-10
**Artifact:** Week 7 Plan (Reliability)
**Author:** PLANNER
**Status:** ❌ REJECTED

---

## Summary

The Week 7 Plan aims to implement the persistence layer (WAL, Snapshots, Atomic Save). While it correctly prioritizes safety and data integrity, it fundamentally violates the system architecture by coupling the core logic to `std::fs`, ignoring the required WASM/IndexedDB abstraction.

---

## Findings

### Critical Issues: 2

- [C1] **Architectural Violation: Persistence Coupling**
  - **Description:** W7D33 proposes implementing `atomic_save` using `std::fs::rename` and `path: &Path`. This directly violates `ARCHITECTURE.md` Section 2.4, which mandates a `StorageBackend` trait to support both Native (File) and WASM (IndexedDB) environments.
  - **Evidence:** `W7D33.md` section 2.3: `pub fn atomic_save<T: Serialize>(path: &Path...`.
  - **Impact:** The implemented logic will be unusable for the primary WASM target, requiring a complete rewrite for W7.B1.
  - **Required Action:** Redesign W7.1, W7.2, and W7.3 to operate exclusively against the `StorageBackend` trait (or a comparable abstraction) as defined in the Architecture.

- [C2] **Missing Backend Agnosticism**
  - **Description:** The Hostile Audit explicitly asked: "Does it cover both IndexedDB (WASM) and File (Native)?". The plan explicitly blocks IndexedDB (W7.B1) and implements only Native logic.
  - **Evidence:** `WEEKLY_TASK_PLAN.md` tasks W7.1-W7.3 implicitly assume file I/O; W7D31 refers to "Open WAL file"; W7D33 uses `std::fs`.
  - **Impact:** Violates "WASM-first design" (Requirement R2 in `ARCHITECTURE.md`).
  - **Required Action:** Ensure the "Core" tasks (W7.1-W7.3) produce logic that works for *both* backends by using the architectural abstractions.

### Major Issues: 1

- [M1] **Missing Atomic Primitive in Abstraction**
  - **Description:** The `StorageBackend` trait (Architecture 2.4) supports `write`, `read`, `sync`, `truncate`, but NOT `rename` or `atomic_save`. The plan relies on `std::fs::rename` for atomicity, which has no equivalent in the current trait definition.
  - **Evidence:** `W7D33.md` relies on `rename` for atomicity; `ARCHITECTURE.md` `StorageBackend` trait lists only basic I/O.
  - **Required Action:** The Plan must define how atomicity is achieved abstractly (e.g., adding `atomic_write` to the trait or defining a transaction mechanism) so that IndexedDB (which uses transactions) and File (which uses rename) can both be supported.

### Minor Issues: 0

---

## Verdict

**REJECTED**

This plan fails to adhere to the WASM-first architecture. It proposes building a Native-only persistence layer that effectively orphans the WASM requirement, creating significant technical debt.

---

## Required Actions Before Resubmission

1. [ ] Refactor Task W7.3 to design/implement atomicity via the `StorageBackend` abstraction, not `std::fs`.
2. [ ] Update W7.1 and W7.2 descriptions to ensure they use `StorageBackend` and not direct file I/O.
3. [ ] Explicitly demonstrate how the `atomic_save` logic maps to IndexedDB (conceptually) to prove the abstraction holds.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-10*
*Verdict: REJECTED*

