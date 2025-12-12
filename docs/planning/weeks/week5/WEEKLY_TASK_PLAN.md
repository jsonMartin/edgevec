# EdgeVec Weekly Task Plan — Week 5

**Date Range:** 2025-12-08 to 2025-12-12
**Author:** PLANNER
**Status:** [PARTIALLY COMPLETE] (Scaling Failed -> Quantization Required)

---

## THIS WEEK'S GOAL

**"Harden the storage engine with checksums/recovery and accelerate distance metrics using Stable SIMD."**

---

## APPROVED TASKS

**CRITICAL:** Only tasks in this section may be implemented by RUST_ENGINEER.

| ID | Task | Owner | Verification Strategy | Est. Hours | Acceptance Criteria |
|:---|:-----|:------|:----------------------|:-----------|:--------------------|
| W5.1 | Implement Stable SIMD Distance Metrics | RUST_ENGINEER | Benchmark | 8 | >4x speedup; strictly Stable Rust (intrinsics); `simd128`/`avx2` support. |
| W5.2 | Persistence Hardening (Checksums/Versioning) | RUST_ENGINEER | Unit Tests | 8 | `FileHeader` includes CRC32; partial writes detected on load. |
| W5.3 | Implement Soft Delete (Routing Preserved) | RUST_ENGINEER | Prop Tests | 8 | Deleted nodes participate in routing but excluded from results; `PROP-DEL-001` passes. |
| W5.4 | Advanced Fuzzing (Persistence & Graph) | TEST_ENGINEER | Fuzzing | 8 | New fuzz targets for `.evec` corruption and graph disconnects. |
| W5.5 | 1M Vector Simulation Benchmark | BENCHMARK_SCIENTIST | Report | 8 | Extrapolated metrics for 1M vectors (RAM/Latency) within budget. |
| W5.6 | **Design Quantization Architecture** | META_ARCHITECT | Architecture Doc | 8 | `QUANTIZATION.md` approved; Schema for `u8`/`i8` compression defined. |

---

## BLOCKED TASKS

| ID | Task | Blocked By | Unblock Condition |
|:---|:-----|:-----------|:------------------|
| None | | | |

---

## NOT IN SCOPE THIS WEEK

| Task | Why Deferred |
|:-----|:-------------|
| Full Vacuum/Compaction | Complexity; focus on Soft Delete first. |
| Multi-threading | Reserved for Phase 4 (Integration). |

---

## VALIDATION CRITERIA

This week is COMPLETE when:
- [ ] SIMD code compiles on Stable Rust and runs on `x86_64`/`wasm32`.
- [ ] Storage engine survives simulated power loss (partial writes).
- [ ] Deleted vectors are used for routing but hidden from results.
- [x] 1M vector performance model confirms viability. (FAILED: Memory > 1GB)
- [ ] Quantization Architecture (W5.6) is designed to fix Memory failure.
- [ ] HOSTILE_REVIEWER validates deliverables.

---

## HOSTILE REVIEW REQUIRED

**Before coding begins:**
- [x] HOSTILE_REVIEWER has approved this plan

**After coding ends:**
- [ ] HOSTILE_REVIEWER validates all deliverables

---

## APPROVALS

| Role | Name | Signature | Date |
|:-----|:-----|:----------|:-----|
| PLANNER | | ✓ | 2025-12-08 |
| HOSTILE_REVIEWER | | [PENDING] | |
