# Week 32: SIMD Consolidation Phase 1

**Date Range:** 2026-01-06 to 2026-01-12
**Version Target:** v0.8.0 (Milestone 8.1)
**Author:** PLANNER
**Status:** [APPROVED]

---

## Executive Summary

Week 32 focuses on **SIMD Consolidation** — the first milestone of v0.8.0. This week delivers:

1. SIMD Euclidean Distance (x86_64 + WASM)
2. Unified `simd_dispatch!` macro
3. SIMD Architecture documentation

**Total Hours:** 12 hours (per V0.8.0_CONSOLIDATION_PLAN.md)
**Buffer:** 3 hours (25%)
**Working Hours:** 15 hours across 7 days (~2h/day)

---

## Week 32 Objectives

| ID | Objective | Hours | Deliverable |
|:---|:----------|:------|:------------|
| W32.1 | SIMD Euclidean Distance | 4h | `src/metric/simd.rs` updated |
| W32.2 | `simd_dispatch!` Macro | 4h | `src/simd/dispatch.rs` created |
| W32.3 | SIMD Architecture Docs | 4h | `docs/architecture/SIMD_ARCHITECTURE.md` |

---

## Daily Breakdown

| Day | Date | Focus | Hours | Tasks |
|:----|:-----|:------|:------|:------|
| 1 | 2026-01-06 | Planning & Research | 2h | W32.1.1: Analyze existing SIMD, plan euclidean |
| 2 | 2026-01-07 | SIMD Euclidean Impl | 2h | W32.1.2: Implement WASM + x86 euclidean |
| 3 | 2026-01-08 | Macro Design | 2h | W32.2.1: Design `simd_dispatch!` macro |
| 4 | 2026-01-09 | Macro Integration | 2h | W32.2.2: Integrate macro, refactor one function |
| 5 | 2026-01-10 | Documentation | 2h | W32.3.1: Create SIMD_ARCHITECTURE.md |
| 6 | 2026-01-11 | Testing & Benchmarks | 2.5h | W32.T: Full test suite, benchmarks |
| 7 | 2026-01-12 | Review & Gate | 2.5h | W32.R: Hostile review, gate creation |

---

## Task Details

### W32.1: SIMD Euclidean Distance (4 hours)

**Objective:** Add SIMD implementations for Euclidean distance on x86_64 and WASM.

**Current State:**
- `src/metric/simd.rs` has L2 squared (no sqrt)
- ARM NEON has euclidean_distance() (lines 91-105 in neon.rs)
- x86/WASM use scalar sqrt(l2_squared)

**Target State:**
- WASM SIMD128: `euclidean_distance_f32()` with SIMD sqrt approximation
- x86_64 AVX2: `euclidean_distance_f32()` with `_mm256_sqrt_ps`
- Dispatcher routes to SIMD when available

**Subtasks:**

| ID | Task | Hours | Verification |
|:---|:-----|:------|:-------------|
| W32.1.1 | Analyze existing SIMD, design euclidean | 1h | Design doc in DAY_1 |
| W32.1.2 | Implement WASM SIMD128 euclidean | 1.5h | Unit tests pass |
| W32.1.3 | Implement x86_64 AVX2 euclidean | 1h | Unit tests pass |
| W32.1.4 | Update dispatcher in l2.rs | 0.5h | Integration test |

**Acceptance Criteria:**
- [ ] `cargo test metric::simd::euclidean` passes
- [ ] Benchmark shows 2x+ speedup vs scalar
- [ ] WASM build succeeds with SIMD enabled
- [ ] No new clippy warnings

**Verification Strategy:** Unit tests + Benchmarks

---

### W32.2: Unified SIMD Dispatch Macro (4 hours)

**Objective:** Create `simd_dispatch!` macro to eliminate platform detection boilerplate.

**Current State (per function):**
```rust
#[cfg(target_arch = "x86_64")]
if is_x86_feature_detected!("avx2") {
    return unsafe { avx2_impl(...) };
}
#[cfg(target_arch = "aarch64")]
if std::arch::is_aarch64_feature_detected!("neon") {
    return neon_impl(...);
}
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
{
    return wasm_impl(...);
}
fallback(...)
```

**Target State:**
```rust
simd_dispatch! {
    fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
        avx2: unsafe { avx2_hamming(a, b) },
        neon: neon_hamming(a, b),
        wasm_simd: wasm_hamming(a, b),
        fallback: scalar_hamming(a, b),
    }
}
```

**Subtasks:**

| ID | Task | Hours | Verification |
|:---|:-----|:------|:-------------|
| W32.2.1 | Design macro syntax and expansion | 1h | RFC-style doc |
| W32.2.2 | Implement `simd_dispatch!` macro | 2h | Compiles on all targets |
| W32.2.3 | Refactor popcount to use macro | 0.5h | Tests pass |
| W32.2.4 | Add macro documentation | 0.5h | Rustdoc renders |

**Acceptance Criteria:**
- [ ] Macro compiles on x86_64, aarch64, wasm32
- [ ] At least one function refactored to use macro
- [ ] Generated code matches manual dispatch (verify via `cargo expand`)
- [ ] Documentation with examples in rustdoc

**Verification Strategy:** Compile tests + cargo expand comparison

---

### W32.3: SIMD Architecture Documentation (4 hours)

**Objective:** Create comprehensive SIMD architecture guide for contributors.

**Deliverable:** `docs/architecture/SIMD_ARCHITECTURE.md`

**Content Outline:**

1. **Overview** — Why SIMD matters for EdgeVec
2. **Architecture Diagram** — Dispatch flow (ASCII)
3. **Module Responsibilities** — What each file does
4. **Performance Expectations** — Expected speedups per platform
5. **Adding New Operations** — Step-by-step guide
6. **Testing Strategy** — How to test SIMD code
7. **Platform Matrix** — What works where

**Subtasks:**

| ID | Task | Hours | Verification |
|:---|:-----|:------|:-------------|
| W32.3.1 | Write overview and architecture diagram | 1h | Renders correctly |
| W32.3.2 | Document module responsibilities | 1h | All modules covered |
| W32.3.3 | Write "Adding New Operations" guide | 1h | Can follow to add op |
| W32.3.4 | Create platform matrix and testing guide | 1h | Complete coverage |

**Acceptance Criteria:**
- [ ] All 7 sections complete
- [ ] ASCII diagram renders in markdown
- [ ] "Adding New Operations" is step-by-step actionable
- [ ] Platform matrix matches actual code

**Verification Strategy:** Documentation review + link check

---

## Testing & Benchmarks (W32.T)

| Test Type | Target | Command |
|:----------|:-------|:--------|
| Unit Tests | All SIMD functions | `cargo test --all-features` |
| WASM Build | SIMD enabled | `wasm-pack build --target web` |
| Clippy | 0 warnings | `cargo clippy -- -D warnings` |
| Benchmarks | Euclidean 2x+ | `cargo bench euclidean` |

---

## Success Metrics

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| Euclidean SIMD speedup | 2x+ vs scalar | Benchmark report |
| Macro adoption | 1+ function refactored | Code inspection |
| Documentation completeness | 7/7 sections | Review |
| Test coverage | All new code tested | `cargo test` |
| Clippy warnings | 0 new warnings | `cargo clippy` |

---

## Dependencies

| Dependency | Status | Notes |
|:-----------|:-------|:------|
| v0.7.0 Release | ✅ COMPLETE | SIMD enabled in builds |
| V0.8.0_CONSOLIDATION_PLAN.md | ✅ APPROVED | Design source |
| ROADMAP.md v6.0 | ✅ APPROVED | Strategic context |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|:-----|:-----------|:-------|:-----------|
| WASM SIMD sqrt complexity | LOW | MEDIUM | Use existing scalar for final sqrt |
| Macro doesn't cover all cases | MEDIUM | LOW | Start simple, expand later |
| Documentation takes longer | LOW | LOW | Focus on essentials first |

---

## Daily Task Files

Each day has a dedicated task file with detailed instructions:

| Day | File | Focus |
|:----|:-----|:------|
| 1 | `DAY_1_TASKS.md` | Planning & Research |
| 2 | `DAY_2_TASKS.md` | SIMD Euclidean Implementation |
| 3 | `DAY_3_TASKS.md` | Macro Design |
| 4 | `DAY_4_TASKS.md` | Macro Integration |
| 5 | `DAY_5_TASKS.md` | Documentation |
| 6 | `DAY_6_TASKS.md` | Testing & Benchmarks |
| 7 | `DAY_7_TASKS.md` | Review & Gate |

---

## Exit Criteria

Week 32 is complete when:

- [ ] All unit tests pass
- [ ] Benchmarks show 2x+ euclidean speedup
- [ ] `simd_dispatch!` macro is functional
- [ ] SIMD_ARCHITECTURE.md is complete
- [ ] Clippy reports 0 warnings
- [ ] HOSTILE_REVIEWER approves all deliverables
- [ ] `.claude/GATE_W32_COMPLETE.md` created

---

## Approval Status

| Reviewer | Verdict | Date |
|:---------|:--------|:-----|
| HOSTILE_REVIEWER | ✅ APPROVED | 2026-01-04 |

**Review Document:** `docs/reviews/2026-01-04_W32_WEEKLY_TASK_PLAN_APPROVED.md`

---

**Author:** PLANNER
**Date:** 2026-01-04
**Version:** 1.0
