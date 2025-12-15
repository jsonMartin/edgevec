# Week 14 Task Plan: WASM Completion & Performance Validation

**Sprint:** Dec 23-27, 2025 (Week 14)
**Previous:** Week 13 (Safety Hardening) APPROVED
**Theme:** Complete Phase 4 Integration + Performance CI
**Status:** [REVISED]

---

## Revision History

| Version | Date | Changes |
|:--------|:-----|:--------|
| 1.0 | 2025-12-14 | Initial Week 14 plan |
| 1.1 | 2025-12-14 | **[REVISED]** Fixed HOSTILE_REVIEWER rejection issues |

### Changes Made (v1.1)

Addressing rejection document `docs/reviews/2025-12-14_WEEK14_PLAN_REJECTED.md`:

| Issue | Fix Applied |
|:------|:------------|
| **[C1]** W14.1 scope false premise | Reduced from 12h to 4h; acknowledged existing implementation |
| **[M1]** Vague dependencies | Added specific file references |
| **[M2]** 5% buffer only | Reduced total to 28h (30% buffer achieved) |
| **[M3]** 3x rule not stated | Added explicit estimation methodology |
| **[M4]** baselines.json exists | Updated AC14.2.2 to "verify and update" |
| **[m1]** Vague synthetic regression | Added specific test procedure |
| **[m2]** Pre-flight inaccuracy | Corrected checklist |

---

## Executive Summary

Week 14 completes Phase 4 (Integration) by adding progress callback to existing WASM batch bindings, establishing P99 latency tracking in CI, and executing competitive benchmarks. With v0.2.1 released and safety hardening complete, we focus on API completeness and performance regression prevention.

**Goals:**
1. Add progress callback to WASM batch insert (existing API enhancement)
2. P99 latency tracking in GitHub Actions
3. Execute competitive benchmarks with actual numbers
4. Documentation for new features

**Estimation Methodology:** All estimates include 3x contingency multiplier applied to optimistic base estimates. Base estimates are shown in parentheses.

---

## Existing Implementation Acknowledgment

**IMPORTANT:** The following WASM batch insert functionality ALREADY EXISTS:

| Function | Location | Status |
|:---------|:---------|:-------|
| `insert_batch_flat()` | `src/wasm/mod.rs:261-311` | COMPLETE |
| `insertBatch()` (v2) | `src/wasm/mod.rs:341-348` | COMPLETE |
| `BatchInsertConfig` | `src/wasm/batch.rs:25-56` | COMPLETE |
| `BatchInsertResult` | `src/wasm/batch.rs:62-104` | COMPLETE |
| Unit tests (15) | `src/wasm/batch.rs:203-484` | COMPLETE |

**Baselines file also exists:** `benches/baselines.json` (created 2025-12-13)

---

## Task Breakdown

### W14.1: WASM Batch Insert Enhancement

**Agent:** WASM_SPECIALIST
**Estimated Hours:** 4h (base: 1.3h × 3x)
**Priority:** HIGH (Day 1)

**Description:**
Add progress callback functionality to existing WASM batch insert implementation. Create browser demo showcasing the complete batch API.

**Existing Implementation (VERIFY ONLY):**
- `src/wasm/mod.rs:261-311` — `insert_batch_flat()`
- `src/wasm/mod.rs:341-348` — `insertBatch()` (v2)
- `src/wasm/batch.rs` — Config, Result, 15 unit tests

**NEW Deliverables:**
1. `batch_insert_with_progress()` — New function with JS callback
2. `examples/wasm_batch_insert.html` — Browser demo
3. Additional unit tests for progress callback

**Acceptance Criteria:**

| ID | Criterion | Verification |
|:---|:----------|:-------------|
| AC14.1.1 | Existing `insertBatch` works | `grep "insertBatch" pkg/edgevec.d.ts` (VERIFY) |
| AC14.1.2 | Existing API accepts Float32Array | TypeScript types verify (VERIFY) |
| AC14.1.3 | Existing API returns IDs | Browser console test (VERIFY) |
| AC14.1.4 | **NEW:** Progress callback supported | `batch_insert_with_progress` function exists |
| AC14.1.5 | Browser demo functional | Manual test in Chrome/Firefox |

**Dependencies:** None (enhances existing: `src/wasm/batch.rs`, `src/wasm/mod.rs:261-348`)
**Risk Level:** LOW (incremental addition to working code)

---

### W14.2: P99 Latency Tracking in CI

**Agent:** BENCHMARK_SCIENTIST
**Estimated Hours:** 8h (base: 2.7h × 3x)
**Priority:** HIGH (Day 1-2)

**Description:**
Add GitHub Actions workflow to run benchmarks on each PR and track P99 latency regressions. Alert if P99 exceeds baseline by >10%.

**Existing Implementation:**
- `benches/baselines.json` — Baseline thresholds ALREADY EXIST

**NEW Deliverables:**
1. `.github/workflows/benchmark.yml` — Benchmark CI workflow
2. `scripts/check_regression.py` — Regression detection script
3. Badge for README showing P99 status

**Acceptance Criteria:**

| ID | Criterion | Verification |
|:---|:----------|:-------------|
| AC14.2.1 | CI runs `cargo bench` on PRs | Workflow triggers on PR |
| AC14.2.2 | Baseline thresholds verified/updated | `benches/baselines.json` reviewed and current |
| AC14.2.3 | P99 latency extracted from results | Script parses Criterion output |
| AC14.2.4 | Regression >10% fails CI | **Test procedure:** Temporarily set `baselines.json` threshold to 0.001ms, verify CI fails, restore |
| AC14.2.5 | Results uploaded as artifact | Check Actions artifacts |
| AC14.2.6 | README badge reflects status | Badge URL works |

**Dependencies:** None
**Risk Level:** MEDIUM (CI environment variability)

---

### W14.3: Execute Competitive Benchmarks

**Agent:** BENCHMARK_SCIENTIST
**Estimated Hours:** 6h (base: 2h × 3x)
**Priority:** MEDIUM (Day 2-3)

**Description:**
Run the benchmark harness created in W13.3a against actual competitor libraries. Collect real performance numbers to fill in competitive_analysis.md.

**Deliverables:**
1. `benches/competitive/results/` — JSON benchmark results
2. `docs/benchmarks/competitive_analysis.md` — Updated with real data
3. Competitor adapters validated
4. README performance table with actual numbers

**Acceptance Criteria:**

| ID | Criterion | Verification |
|:---|:----------|:-------------|
| AC14.3.1 | EdgeVec benchmark runs successfully | `node harness.js --library=edgevec` |
| AC14.3.2 | At least 2 competitors benchmarked | Results include hnswlib-wasm, voy |
| AC14.3.3 | P50/P99 latency recorded | JSON contains percentile fields |
| AC14.3.4 | competitive_analysis.md has real numbers | `grep -c "X.XX" docs/benchmarks/competitive_analysis.md` returns 0 |
| AC14.3.5 | README table updated | Performance section accurate |

**Dependencies:** W14.1 (WASM build must be current)
**Risk Level:** LOW (infrastructure exists)

---

### W14.4: Documentation Polish

**Agent:** DOCWRITER
**Estimated Hours:** 6h (base: 2h × 3x)
**Priority:** MEDIUM (Day 3-4)

**Description:**
Update documentation to reflect v0.2.1 features and prepare for stable release. Ensure examples work, API docs are complete.

**Deliverables:**
1. `README.md` — Updated with v0.2.1 features
2. `docs/API_REFERENCE.md` — Complete API documentation
3. `examples/` — All examples tested and working
4. Rustdoc comments updated

**Acceptance Criteria:**

| ID | Criterion | Verification |
|:---|:----------|:-------------|
| AC14.4.1 | README reflects v0.2.1 release | Version number correct |
| AC14.4.2 | All code examples compile | `cargo test --doc` |
| AC14.4.3 | API reference covers batch insert | Section exists |
| AC14.4.4 | No broken links | `markdown-link-check` |
| AC14.4.5 | Rustdoc builds without warnings | `cargo doc --no-deps` |

**Dependencies:** W14.1, W14.3
**Risk Level:** LOW

---

### W14.5: Week 14 Integration Testing

**Agent:** TEST_ENGINEER
**Estimated Hours:** 4h (base: 1.3h × 3x)
**Priority:** HIGH (Day 4)

**Description:**
End-to-end integration testing for all Week 14 deliverables. Verify WASM batch works in browser, CI catches regressions, documentation is accurate.

**Deliverables:**
1. Integration test results
2. Bug fixes for any discovered issues
3. Week 14 status report

**Acceptance Criteria:**

| ID | Criterion | Verification |
|:---|:----------|:-------------|
| AC14.5.1 | WASM batch works in browser | Manual browser test |
| AC14.5.2 | All unit tests pass | `cargo test --lib` |
| AC14.5.3 | CI benchmark workflow functional | Test PR triggers workflow |
| AC14.5.4 | No clippy warnings | `cargo clippy -- -D warnings` |
| AC14.5.5 | Week 14 deliverables documented | Status report complete |

**Dependencies:** W14.1, W14.2, W14.3, W14.4
**Risk Level:** LOW

---

## Schedule

| Day | Tasks | Hours | Agent |
|:----|:------|:------|:------|
| Day 1 (Mon) | W14.1: Progress callback + W14.2 (Part 1) | 6h | WASM_SPECIALIST, BENCHMARK_SCIENTIST |
| Day 2 (Tue) | W14.2 (Part 2) + W14.3 (Part 1) | 7h | BENCHMARK_SCIENTIST |
| Day 3 (Wed) | W14.3 (Part 2) + W14.4 (Part 1) | 7h | BENCHMARK_SCIENTIST, DOCWRITER |
| Day 4 (Thu) | W14.4 (Part 2) + W14.5 | 8h | DOCWRITER, TEST_ENGINEER |
| Day 5 (Fri) | Buffer / Overflow | 0h | (Reserved) |

**Total Scheduled:** 28h
**Total Budget:** 40h
**Contingency Buffer:** 12h (30%)

---

## Week 14 Acceptance Criteria Checklist

### W14.1: WASM Batch Insert Enhancement
- [ ] AC14.1.1: Existing `insertBatch` works (VERIFY)
- [ ] AC14.1.2: Existing API accepts Float32Array (VERIFY)
- [ ] AC14.1.3: Existing API returns IDs (VERIFY)
- [ ] AC14.1.4: **NEW:** Progress callback supported
- [ ] AC14.1.5: Browser demo functional

### W14.2: P99 CI Tracking
- [ ] AC14.2.1: CI runs benchmarks on PRs
- [ ] AC14.2.2: Baseline thresholds verified/updated
- [ ] AC14.2.3: P99 latency extracted
- [ ] AC14.2.4: Regression >10% fails CI (with test procedure)
- [ ] AC14.2.5: Results uploaded as artifact
- [ ] AC14.2.6: README badge works

### W14.3: Competitive Benchmarks
- [ ] AC14.3.1: EdgeVec benchmark runs
- [ ] AC14.3.2: 2+ competitors benchmarked
- [ ] AC14.3.3: Percentiles recorded
- [ ] AC14.3.4: Analysis has real numbers (no X.XX)
- [ ] AC14.3.5: README table updated

### W14.4: Documentation
- [ ] AC14.4.1: README reflects v0.2.1
- [ ] AC14.4.2: Examples compile
- [ ] AC14.4.3: API reference complete
- [ ] AC14.4.4: No broken links
- [ ] AC14.4.5: Rustdoc clean

### W14.5: Integration
- [ ] AC14.5.1: Browser test passes
- [ ] AC14.5.2: Unit tests pass
- [ ] AC14.5.3: CI workflow functional
- [ ] AC14.5.4: Clippy clean
- [ ] AC14.5.5: Status report complete

---

## Dependencies Graph

```
W14.1 (WASM Enhancement — 4h)
    │
    ├──> W14.3 (Benchmarks — 6h) ──> W14.4 (Docs — 6h) ──> W14.5 (Integration — 4h)
    │
    └──> W14.2 (P99 CI — 8h) ──────────────────────────> W14.5

Total Critical Path: 4h + 6h + 6h + 4h = 20h (within 28h budget)
```

---

## Risk Register

| ID | Risk | Impact | Likelihood | Mitigation |
|:---|:-----|:-------|:-----------|:-----------|
| R14.1 | CI environment variability affects benchmarks | MEDIUM | HIGH | Use relative comparisons, not absolute |
| R14.2 | Competitor libraries have breaking changes | LOW | LOW | Pin versions in package.json |
| R14.3 | Holiday week reduced availability | HIGH | HIGH | **MITIGATED:** 30% buffer, front-load critical work |

---

## HOSTILE_REVIEWER Pre-Flight Checklist

Before submitting for review:

- [x] All tasks < 16 hours (max: 8h)
- [x] All tasks have acceptance criteria (26 total ACs)
- [x] All dependencies documented with specific file references
- [x] Schedule totals < 40 hours (28h scheduled, 30% buffer)
- [x] Estimation methodology stated (3x rule explicit)
- [x] Existing implementations acknowledged
- [x] Builds on v0.2.1 foundation
- [x] Aligns with Phase 4 (Integration) objectives
- [x] No new unsafe code planned
- [x] Multi-agent days have non-conflicting tasks

---

## Week 14 Success Metrics

| Metric | Target | Measurement |
|:-------|:-------|:------------|
| **WASM API Completeness** | Progress callback added | TypeScript types |
| **CI Coverage** | P99 tracking active | Workflow runs |
| **Benchmark Data** | 3+ libraries compared | Results JSON |
| **Documentation** | 100% public API | Rustdoc coverage |
| **Test Pass Rate** | 100% | `cargo test` |

---

**Status:** [REVISED]
**Next:** Submit for HOSTILE_REVIEWER approval via `/review WEEKLY_TASK_PLAN.md`
