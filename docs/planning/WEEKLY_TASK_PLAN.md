# EdgeVec Weekly Task Plan — Week 6

**Date Range:** 2025-12-09 to 2025-12-16
**Author:** PLANNER
**Status:** PROPOSED

---

## THIS WEEK'S GOAL

**Implement Scalar Quantization (SQ8) and SIMD-accelerated integer metrics to solve the 1536d latency bottleneck.**

---

## APPROVED TASKS

**CRITICAL:** Only tasks in this section may be implemented by RUST_ENGINEER.

| ID | Task | Owner | Verification Strategy | Est. Hours | Acceptance Criteria |
|:---|:-----|:------|:----------------------|:-----------|:--------------------|
| W6.1 | Implement `ScalarQuantizer` (SQ8) | RUST_ENGINEER | Unit + Prop | 8 | `quantize` + `dequantize` roundtrip error < 1% (normalized) |
| W6.2 | Implement SIMD `l2_squared_u8` (AVX2/WASM) | RUST_ENGINEER | Unit (vs Scalar) | 8 | Matches scalar `u8` calculation; no panics on unaligned |
| W6.3 | Implement SIMD `dot_product_u8` (AVX2/WASM) | RUST_ENGINEER | Unit (vs Scalar) | 8 | Matches scalar; `assert_eq!` guards present |
| W6.4 | Benchmarking SQ8 vs F32 | BENCHMARK_SCIENTIST | Criterion | 4 | SQ8 is >3x faster than F32 |
| W6.5 | Verify Quantization Error Bounds | TEST_ENGINEER | Proptest | 4 | Error remains within statistical bounds for random vectors |

---

## BLOCKED TASKS

| ID | Task | Blocked By | Unblock Condition |
|:---|:-----|:-----------|:------------------|
| W6.B1 | Integrate SQ8 into `HnswIndex` | W6.1, W6.2, W6.3 | Quantizer and Metrics proven correct |

---

## NOT IN SCOPE THIS WEEK

| Task | Why Deferred |
|:-----|:-------------|
| Product Quantization (PQ) | Complexity; trying SQ8 first (simpler, faster) |
| Disk Persistence of Quantizers | Focus is on in-memory performance first |

---

## VALIDATION CRITERIA

This week is COMPLETE when:
- [ ] `ScalarQuantizer` struct exists and passes property tests.
- [ ] `l2_squared_u8` and `dot_product_u8` are implemented with SIMD.
- [ ] Benchmarks show >3x speedup over F32 metrics.
- [ ] HOSTILE_REVIEWER validates correctness and safety.

---

## HOSTILE REVIEW REQUIRED

**Before coding begins:**
- [ ] HOSTILE_REVIEWER has approved this plan

**After coding ends:**
- [ ] HOSTILE_REVIEWER validates all deliverables

---

## APPROVALS

| Role | Name | Signature | Date |
|:-----|:-----|:----------|:-----|
| PLANNER | PLANNER | ✓ | 2025-12-09 |
| HOSTILE_REVIEWER | | [PENDING] | |
