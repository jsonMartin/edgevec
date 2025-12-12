# HOSTILE_REVIEWER: Rejection — W4D17 Artifacts (Iteration 5)

**Date:** 2025-12-08
**Artifact:** W4.2_internal_opt (Internal Reuse Optimization)
**Author:** RUST_ENGINEER / BENCHMARK_SCIENTIST
**Status:** ❌ REJECTED

---

## Summary

Review of the "Internal Reuse" optimization strategy intended to reduce WASM insertion latency by reusing `SearchContext` buffers (`scratch`, `neighbor_scratch`) inside the HNSW insert loop.

---

## Findings

### Critical Issues: 2

- [C1] **Catastrophic Performance Regression**
  - **Description:** The optimization made performance **worse**, not better.
  - **Evidence:**
    - Baseline: **1.47ms**
    - Optimization 1 (Ctx Reuse): **1.54ms**
    - Optimization 2 (Internal Reuse): **2.22ms** (+51% regression from baseline)
  - **Impact:** Violates the core objective of the task (< 1ms).
  - **Required Action:** **IMMEDIATE REVERT** of `W4.2_internal_opt`. The code complexity cost is not paid for by performance.

- [C2] **Optimization Hallucination**
  - **Description:** The strategy assumed `Vec::new()` was the bottleneck without profiling data to back it up.
  - **Evidence:** Removing allocations caused a slowdown, suggesting the bottleneck is elsewhere (likely `wasm-bindgen` boundary overhead or cache locality issues with large reused buffers).
  - **Impact:** Wasted engineering cycle.
  - **Required Action:** Pivot strategy. Do not attempt further "micro-optimizations" of the Rust code. The problem is the **WASM Boundary crossing frequency**.

### Major Issues: 1

- [M1] **Code Complexity Debt**
  - **Description:** The `insert` function signature now requires an optional context, and the internal logic has complex split-borrow scopes to manage `SearchContext` fields.
  - **Evidence:** `src/hnsw/insert.rs` requires explicit scoping `{ ... }` to satisfy the borrow checker.
  - **Impact:** Harder to maintain.
  - **Required Action:** Revert to the clean, allocation-heavy version until a real fix (Batching) is implemented.

### Minor Issues: 0

---

## Verdict

**REJECTED**

This artifact fails the primary performance gate. The optimization is a net negative.

---

## Required Actions Before Resubmission

1. [ ] **Revert** changes from `W4.2_internal_opt` (Go back to clean `insert` signature).
2. [ ] **Implement Batching** (`insert_batch`) to amortize the WASM boundary cost (1 call for N vectors).
3. [ ] **Update Plan** to reflect that single-vector insert < 1ms might be impossible with current `wasm-bindgen` overhead on this hardware/setup, and Batching is the approved path forward.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-08*
*Verdict: REJECTED*

