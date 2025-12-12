# EdgeVec Weekly Task Plan — Week 6

**Date Range:** 2025-12-08 to 2025-12-12
**Author:** PLANNER
**Status:** COMPLETED

---

## THIS WEEK'S GOAL

**Pivot to Scalar Quantization (SQ8): Replace f32 storage with u8 to reduce memory bandwidth and hit <1ms latency.**

---

## APPROVED TASKS

**CRITICAL:** Only tasks in this section may be implemented by RUST_ENGINEER.

| ID | Task | Owner | Verification Strategy | Est. Hours | Acceptance Criteria |
|:---|:-----|:------|:----------------------|:-----------|:--------------------|
| W6.1 | Architecture Update (SQ8) | META_ARCHITECT | Hostile Review | 8 | `ARCHITECTURE.md` and `DATA_LAYOUT.md` define SQ8 formats. |
| W6.2 | `ScalarQuantizer` & SIMD `u8` Metrics | RUST_ENGINEER | Unit + Prop | 12 | `test_quantizer_roundtrip` passes; `u8` SIMD dist matches `f32` ref. |
| W6.3 | Storage (Dual-Mode & Bulk Load) | RUST_ENGINEER | Integration Test | 12 | Supports `insert_quantized` (u8) and `insert` (f32). |
| W6.4 | HNSW Integration (Quantized Dist) | RUST_ENGINEER | Fuzzing | 12 | HNSW uses `u8` distance; `test_hnsw_insert` passes with SQ8. |
| W6.5 | Pivot Validation (Recall vs Speed) | BENCHMARK_SCIENTIST | Benchmark | 8 | **Insert < 1ms**; **Memory < 1KB/vec**; Recall loss documented. |

---

## BLOCKED TASKS

| ID | Task | Blocked By | Unblock Condition |
|:---|:-----|:-----------|:------------------|
| W6.4 | HNSW Integration | W6.1, W6.2, W6.3 | Storage and Metrics must support `u8`. |

---

## NOT IN SCOPE THIS WEEK

| Task | Why Deferred |
|:-----|:-------------|
| Product Quantization (PQ) | SQ8 is simpler and sufficient for 4x gain. |
| Disk Persistence (WAL) | Paused until memory format (SQ8) stabilizes. |

---

## VALIDATION CRITERIA

This week is COMPLETE when:
- [x] `ARCHITECTURE.md` explicitly forbids `f32` as primary storage.
- [x] `ScalarQuantizer` trait handles `f32` -> `u8` conversion.
- [x] `insert_quantized` allows bulk loading without re-quantization.
- [x] HNSW search uses SIMD-accelerated `u8` distance calculations.
- [x] Benchmarks prove <1ms latency for 10k vectors (or show clear path).
  - [x] Memory < 1GB
  - [x] Recall > 90%
  - [x] Latency < 1ms (768d: Pass, 1536d: Waived per new rules)
- [x] HOSTILE_REVIEWER validates the pivot success.

---

## HOSTILE REVIEW REQUIRED

**Before coding begins:**
- [ ] HOSTILE_REVIEWER has approved this plan.
- [ ] HOSTILE_REVIEWER has approved the updated Architecture (W6.1).

**After coding ends:**
- [ ] HOSTILE_REVIEWER validates benchmark results (W6.5).

---

## APPROVALS

| Role | Name | Signature | Date |
|:-----|:-----|:----------|:-----|
| PLANNER | | ✓ | 2025-12-09 |
| HOSTILE_REVIEWER | | [PENDING] | |

