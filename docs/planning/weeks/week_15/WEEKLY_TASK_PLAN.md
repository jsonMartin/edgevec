# Week 15 Task Plan — v0.3.0 Foundation & Quality Hardening

**Sprint:** Dec 30, 2025 - Jan 3, 2026
**Phase:** 4.5 (Transition: WASM Complete → v0.3.0 Planning)
**Status:** [PROPOSED]
**PLANNER:** Week 15 Planning
**Date Created:** 2025-12-14

---

## Executive Summary

Week 15 bridges the completion of Phase 4 (WASM Integration) and the preparation for v0.3.0 feature development. This sprint focuses on:

1. **Quality Hardening:** Runtime SIMD detection, formal recall benchmarks
2. **v0.3.0 Preparation:** Soft delete architecture RFC
3. **Ecosystem Validation:** Browser compatibility matrix

**Strategic Goal:** Build confidence in v0.2.x stability while laying groundwork for v0.3.0 features.

---

## Week 15 Context

### Previous Week (Week 14) Accomplishments

| Task | Status | Key Deliverable |
|:-----|:-------|:----------------|
| W14.1 | ✅ Complete | `insertBatchWithProgress` WASM API |
| W14.2 | ✅ Complete | CI benchmark workflow + regression detection |
| W14.3 | ✅ Complete | Competitive benchmarks (24x faster than voy) |
| W14.4 | ✅ Complete | `docs/API_REFERENCE.md` |
| W14.5 | ✅ Complete | 142 tests passing, 0 warnings |

**Gate Status:** `.claude/GATE_14_COMPLETE.md` ✅

### Current State

- **Version:** v0.2.1 (Safety Hardening Release)
- **Tests:** 125 unit + 17 doc = 142 total
- **Warnings:** 0 clippy, 0 rustdoc
- **WASM Bundle:** 182 KB (70% under target)
- **Performance:** 0.20ms search P50 at 10k vectors

### Known Limitations to Address

| ID | Limitation | Severity | Week 15 Action |
|:---|:-----------|:---------|:---------------|
| #4 | SIMD requires compiler flags | HIGH | W15.1: Detection + warnings |
| #5 | SQ8 recall not formally benchmarked | MEDIUM | W15.2: SIFT-1M evaluation |
| #2 | No delete/update operations | HIGH | W15.3: RFC for v0.3.0 |
| - | Browser compat not documented | MEDIUM | W15.4: Compatibility matrix |

---

## Task Overview

| Day | Task | Focus | Agent | Hours |
|:----|:-----|:------|:------|:------|
| **Day 1** | W15.1 | SIMD Detection System | RUST_ENGINEER | 8h |
| **Day 2** | W15.2 | Recall Benchmarks (SIFT-1M) | BENCHMARK_SCIENTIST | 8h |
| **Day 3** | W15.3 | Soft Delete RFC | META_ARCHITECT | 8h |
| **Day 4** | W15.4 | Browser Compatibility | WASM_SPECIALIST | 8h |
| **Day 5** | Buffer | Overflow + Final Review | As needed | 12h |

**Total Planned:** 32h + 12h buffer = 44h
**Buffer Allocation:** 27% (within 30% target)

---

## Day 1: SIMD Detection (Monday, Dec 30)

### W15.1: Runtime SIMD Detection System

**Agent:** RUST_ENGINEER
**Estimate:** 8h (2h base × 3x + 2h validation)
**Priority:** P0

#### Objective

Implement runtime CPU feature detection to warn users when SIMD optimizations are unavailable. Addresses 60-78% performance gap when `-C target-cpu=native` is missing.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC15.1.1 | Create `src/simd/detect.rs` module | `test -f src/simd/detect.rs` |
| AC15.1.2 | `SimdCapabilities` struct with AVX2/FMA/SSE4.2/NEON | Unit test |
| AC15.1.3 | `is_x86_feature_detected!` for x86_64 | Unit test on CI |
| AC15.1.4 | `warn_if_suboptimal()` logs actionable warning | Log output inspection |
| AC15.1.5 | Unit tests for all detection paths | `cargo test simd` |

#### Deliverables

1. `src/simd/detect.rs` — Detection module
2. `examples/simd_check.rs` — User-facing capability check
3. Tests in `src/simd/detect.rs`

#### Command

```
/rust-implement W15.1
```

**Details:** [DAY_1_TASKS.md](./DAY_1_TASKS.md)

---

## Day 2: Recall Benchmarks (Tuesday, Dec 31)

### W15.2: Standard Dataset Recall Benchmarks

**Agent:** BENCHMARK_SCIENTIST
**Estimate:** 8h (2.7h base × 3x)
**Priority:** P0

#### Objective

Establish formal recall benchmarks using SIFT-1M dataset. Quantify Float32 vs SQ8 recall trade-off.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC15.2.1 | Create `benches/recall/` module | Directory exists |
| AC15.2.2 | SIFT-1M benchmark harness | Compiles and runs |
| AC15.2.3 | GloVe-100 benchmark harness | Compiles and runs |
| AC15.2.4 | Measure recall@1, recall@10, recall@100 | Numbers documented |
| AC15.2.5 | Compare Float32 vs SQ8 recall | Comparison table |
| AC15.2.6 | Results in benchmark report | Markdown document |

#### Deliverables

1. `benches/recall/mod.rs` — Recall calculation utilities
2. `benches/recall/sift.rs` — SIFT-1M harness
3. `benches/recall_bench.rs` — Criterion wrapper
4. `docs/benchmarks/week15_recall.md` — Results report

#### Command

```
/bench-baseline recall
```

**Details:** [DAY_2_TASKS.md](./DAY_2_TASKS.md)

---

## Day 3: Soft Delete RFC (Wednesday, Jan 1)

### W15.3: Soft Delete Architecture Design

**Agent:** META_ARCHITECT
**Estimate:** 8h (2h base × 3x + 2h review)
**Priority:** P0 (v0.3.0 Critical Path)

#### Objective

Design tombstone-based soft delete system for v0.3.0. This RFC must be approved before implementation begins in Week 16.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC15.3.1 | RFC document in `docs/rfcs/` | File exists |
| AC15.3.2 | Tombstone data structure designed | Size calculated |
| AC15.3.3 | Compaction strategy defined | Algorithm documented |
| AC15.3.4 | WAL extension for DELETE | Format specified |
| AC15.3.5 | Memory overhead calculated | Numbers in RFC |
| AC15.3.6 | API changes defined | Method signatures |

#### Deliverables

1. `docs/rfcs/RFC-001-soft-delete.md` — Complete RFC
2. `examples/size_check.rs` — Struct size validation

#### Command

```
/architect-design soft-delete
```

**Details:** [DAY_3_TASKS.md](./DAY_3_TASKS.md)

---

## Day 4: Browser Compatibility (Thursday, Jan 2)

### W15.4: Browser Compatibility Testing

**Agent:** WASM_SPECIALIST
**Estimate:** 8h (2h base × 3x + 2h testing)
**Priority:** P0

#### Objective

Create comprehensive browser compatibility matrix. Test Chrome, Firefox, Safari, Edge with WASM and IndexedDB features.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC15.4.1 | Browser matrix document | File exists |
| AC15.4.2 | Chrome tested (latest, latest-1) | Results documented |
| AC15.4.3 | Firefox tested | Results documented |
| AC15.4.4 | Safari tested | Results documented |
| AC15.4.5 | Edge tested | Results documented |
| AC15.4.6 | IndexedDB differences documented | Section in matrix |
| AC15.4.7 | Playwright config (stretch) | Config file exists |

#### Deliverables

1. `docs/BROWSER_COMPATIBILITY.md` — Full matrix
2. `examples/browser-demo/stress-test.html` — IndexedDB stress test
3. `tests/browser/playwright.config.ts` — Automated test config (stretch)

#### Command

```
/wasm-bind browser-test
```

**Details:** [DAY_4_TASKS.md](./DAY_4_TASKS.md)

---

## Day 5: Buffer (Friday, Jan 3)

### Buffer Day + Final Review

**Agent:** As needed
**Buffer:** 12h

#### Priority Order

1. **Critical fixes** from Days 1-4
2. **Overflow work** if tasks incomplete
3. **Enhancement** if buffer remains

#### Success Criteria

- Week 15 status report complete
- All 24 ACs verified
- Ready for HOSTILE_REVIEWER submission

**Details:** [DAY_5_TASKS.md](./DAY_5_TASKS.md)

---

## Risk Register

| ID | Risk | Probability | Impact | Mitigation |
|:---|:-----|:------------|:-------|:-----------|
| R15.1 | SIFT-1M download slow | MEDIUM | LOW | Use cached data or synthetic fallback |
| R15.2 | Safari testing needs macOS | HIGH | MEDIUM | Use BrowserStack or document as gap |
| R15.3 | SIMD detection fails on exotic CPUs | LOW | LOW | Graceful fallback to no detection |
| R15.4 | RFC scope creep | MEDIUM | MEDIUM | Time-box Day 3, defer extras to Week 16 |

---

## Dependencies

### External Dependencies

| Dependency | Required For | Status |
|:-----------|:-------------|:-------|
| SIFT-1M dataset (~130MB) | W15.2 | Download required |
| Safari browser | W15.4 | macOS required |
| BrowserStack (optional) | W15.4 stretch | Account needed |

### Internal Dependencies

| Task | Depends On | Notes |
|:-----|:-----------|:------|
| W15.2 | SIFT-1M data | Day 2 blocked without data |
| W15.3 | None | Independent design work |
| W15.4 | WASM build | Use existing pkg/ |

---

## Success Metrics

### Quality Gates

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| Unit Tests | +10 new tests | `cargo test --lib \| grep "passed"` |
| Doc Tests | Maintain 17 | `cargo test --doc` |
| Clippy | 0 warnings | `cargo clippy -- -D warnings` |
| Coverage | Maintain 80%+ | `cargo tarpaulin` |

### Performance Targets

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| SIMD warning accuracy | 100% | Manual on varied hardware |
| SIFT-1M recall@10 (Float32) | >0.95 | Benchmark output |
| SIFT-1M recall@10 (SQ8) | >0.92 | Benchmark output |

---

## HOSTILE_REVIEWER Checkpoints

| Day | Artifact | Review Command |
|:----|:---------|:---------------|
| Day 1 | SIMD detection module | `/review src/simd/detect.rs` |
| Day 2 | Recall benchmark results | `/review docs/benchmarks/week15_recall.md` |
| Day 3 | Soft delete RFC | `/review docs/rfcs/RFC-001-soft-delete.md` |
| Day 4 | Browser compat matrix | `/review docs/BROWSER_COMPATIBILITY.md` |
| Day 5 | Full week | `/review WEEKLY_TASK_PLAN.md` |

---

## Week 16 Preview

**Theme:** Soft Delete Implementation (v0.3.0 Feature)

**Planned Tasks:**

| Task | Description |
|:-----|:------------|
| W16.1 | Add `deleted` field to HnswNode |
| W16.2 | Implement `delete()` and search filtering |
| W16.3 | Extend WAL for DELETE entries |
| W16.4 | Implement compaction |
| W16.5 | WASM bindings + TypeScript types |

**Prerequisite:** W15.3 RFC approved by HOSTILE_REVIEWER

---

## Appendix: Detailed Task Files

- [DAY_1_TASKS.md](./DAY_1_TASKS.md) — SIMD Detection
- [DAY_2_TASKS.md](./DAY_2_TASKS.md) — Recall Benchmarks
- [DAY_3_TASKS.md](./DAY_3_TASKS.md) — Soft Delete RFC
- [DAY_4_TASKS.md](./DAY_4_TASKS.md) — Browser Compatibility
- [DAY_5_TASKS.md](./DAY_5_TASKS.md) — Buffer + Review

---

## Revision History

| Version | Date | Author | Changes |
|:--------|:-----|:-------|:--------|
| 1.0 | 2025-12-14 | PLANNER | Initial Week 15 plan |

---

**Status:** [PROPOSED]
**Next:** HOSTILE_REVIEWER approval → Week 15 execution
