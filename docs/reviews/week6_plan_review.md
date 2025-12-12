# HOSTILE_REVIEWER: Rejection — Week 6 Plan (Quantization)

**Date:** 2025-12-09
**Artifact:** Week 6 Plan (`WEEKLY_TASK_PLAN.md` + Daily Plans)
**Author:** PLANNER
**Status:** ❌ REJECTED

---

## Summary

The plan proposes a pivot to Scalar Quantization (SQ8) to address the scaling failure with 1M vectors. It includes architecture updates, `u8` storage implementation, HNSW integration, and benchmarking.

---

## Findings

### Critical Issues: 2
- [C1] **Explicit Exclusion of SIMD for `u8` Metrics**
  - Description: The plan (`WEEKLY_TASK_PLAN.md` Task 41) explicitly marks SIMD optimization as "NOT IN SCOPE".
  - Evidence: `W6D27` proposes a "naive `u8` implementation initially" using scalar operations.
  - Impact: Scalar `u8` distance calculation involves casting `u8` to `i32`/`u32` for every element to prevent overflow. This casting overhead often makes scalar `u8` slower than vectorized `f32`, potentially failing the <1ms latency target.
  - Required Action: Must include at least a basic `std::simd` or `wasm_bindgen` intrinsic path for `u8` metrics, or prove scalar `u8` is faster than `f32` (unlikely).

- [C2] **Missing Pre-Quantized Loading Strategy**
  - Description: The plan only accounts for `insert(vec: &[f32])` where quantization happens on the fly.
  - Evidence: `W6D28` Task 2 describes the insert pipeline as "Quantize `f32` -> `u8`". No mention of a direct `u8` insert path.
  - Impact: For 1M vectors, re-quantizing the entire dataset upon reload/bulk-insert is CPU intensive. A scalable system must allow loading already-quantized data.
  - Required Action: Add tasks to support direct insertion/loading of `u8` vectors (e.g., `insert_quantized`).

### Major Issues: 1
- [M1] **Inconsistent Memory Budget Calculations**
  - Description: `W6D26` calculates memory as "1M vectors * (1536 bytes / 4) = 384MB". This implies a dimensionality of 384 (if 1536 was bytes) or confusing units. `ARCHITECTURE.md` uses 768 dimensions. OpenAI uses 1536 dimensions.
  - Evidence: `W6D26` Line 25 vs `ARCHITECTURE.md` Line 476.
  - Required Action: Standardize on a reference dimensionality (e.g., 1536 for OpenAI or 768 for BERT) and recalculate the budget. If 1536 dims, 1M `u8` vectors = 1.5GB, not 384MB.

### Minor Issues: 1
- [m1] **Ambiguity on Re-ranking Mandatory Status**
  - Description: The plan treats the `f32` sidecar as "optional" (`W6.3`). While good for flexibility, high recall *requires* re-ranking. Ensure the default config or benchmarks prioritize the re-ranking path to validate recall targets.

---

## Verdict

**REJECTED**

The plan fails to address the performance overhead of scalar `u8` casting (a known bottleneck) and overlooks the necessity of bulk-loading quantized data for the 1M vector target. The math supporting the memory budget is also inconsistent with the architecture.

---

## Required Actions Before Resubmission

1. [ ] **Add SIMD Task:** Include a task for SIMD-accelerated `u8` metrics (even if experimental/basic) to ensure latency targets are reachable.
2. [ ] **Add Pre-Quantized Insert:** Add API/Tasks for inserting/loading raw `u8` data to avoid re-quantization overhead.
3. [ ] **Fix Memory Math:** Correct the memory budget calculations in `W6D26` to reflect consistent dimensionality (likely 1536 or 768) and verify it fits the budget.
4. [ ] **Resubmit:** Update `WEEKLY_TASK_PLAN.md` and Daily Plans.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-09*
*Verdict: REJECTED*
