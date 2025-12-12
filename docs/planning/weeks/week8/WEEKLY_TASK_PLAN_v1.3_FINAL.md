# EdgeVec Weekly Task Plan — Week 8 [v1.3 FINAL - APPROVED]

**Date Range:** 2025-12-30 to 2026-01-05
**Author:** PLANNER
**Status:** [APPROVED]
**Version:** 1.3 (FINAL)
**Hostile Review Score:** 27/30 (90%) ✅

---

## ⚠️ FINAL APPROVED VERSION

**This plan has passed hostile review Round 3/3 with score 27/30 (90%)**

**Revision History:**
- v1.0: 2/30 (7%) - CATASTROPHIC
- v1.1: 3/30 (10%) - CATASTROPHIC
- v1.2: 24/30 (80%) - CONDITIONAL_GO
- **v1.3: 27/30 (90%) - GO** ✅

**Quick Fix Applied from v1.2 → v1.3:**
- W8.5 WASM Profiling Metrics: Added numeric targets (≤500KB bundle, ≤500ms init, <10ms P99)

---

## THIS WEEK'S GOAL

**Validate quantization performance and stability through E2E integration testing and targeted profiling of the critical path.**

---

## APPROVED TASKS

### W8.1: Quantization Implementation [8h]

**Owner:** RUST_ENGINEER
**Estimated Hours:** 8h

**Context Required:**
- `docs/architecture/ARCHITECTURE.md` Section 4.2 (Quantization)
- `docs/architecture/DATA_LAYOUT.md` Section 3 (Binary Quantization)

**Deliverables:**
1. `src/quantization/mod.rs` - Core quantization module
2. `src/quantization/binary.rs` - Binary quantization implementation
3. Unit tests in `tests/unit/test_quantization.rs`

**Acceptance Criteria:**
- [ ] Binary quantization: 768D → 96 bytes (8x compression)
- [ ] Hamming distance: <50 CPU cycles per comparison (x86_64)
- [ ] Memory layout: 64-byte aligned for SIMD
- [ ] Unit tests: 100% coverage of public API
- [ ] All tests pass: `cargo test quantization`
- [ ] Zero unsafe blocks OR unsafe with written safety proof

**Dependencies:**
- BLOCKS: W8.2 (needs quantization code for fuzzing)

---

### W8.2: Basic Fuzzing [8h]

**Owner:** TEST_ENGINEER
**Estimated Hours:** 8h

**Deliverables:**
1. `fuzz/fuzz_targets/fuzz_hnsw_operations.rs` - HNSW insert/search fuzzer
2. `fuzz/fuzz_targets/fuzz_quantization.rs` - Quantization fuzzer
3. Regression tests from discovered crashes

**Specific Scenarios:**
1. Random Insert/Search Sequences (100k iterations)
2. Edge Cases (10k iterations): Empty index, single vector, duplicates
3. Quantization Stability (10k iterations): NaN, Inf, subnormal values

**Acceptance Criteria:**
- [ ] HNSW fuzzer runs for 1 hour without crashes
- [ ] Quantization fuzzer runs for 30 minutes without crashes
- [ ] Corpus size: ≥100 distinct inputs
- [ ] Code coverage: ≥80% of quantization + HNSW insert/search
- [ ] All discovered crashes fixed with regression tests

**Dependencies:**
- BLOCKED BY: W8.1

---

### W8.3a: Insert-Search E2E Integration [5h]

**Owner:** TEST_ENGINEER
**Estimated Hours:** 5h

**Deliverables:**
1. `tests/integration/test_insert_search_e2e.rs`
2. Concurrent test harness

**Specific Scenarios:**
1. Sequential Insert-Search (1k vectors)
2. Concurrent Insert-Search (10 threads)
3. Gradual Capacity Fill (deferred to W9.1)

**Acceptance Criteria:**
- [ ] Sequential: Recall@10 ≥0.95
- [ ] Concurrent: Recall@10 ≥0.95 under 10-thread load (no degradation)
- [ ] Latency: Mean search <5ms, P99 <10ms (per ARCHITECTURE.md)
- [ ] Zero data races detected by ThreadSanitizer
- [ ] Zero memory leaks detected by AddressSanitizer

**Dependencies:**
- BLOCKED BY: W8.1, W8.2

---

### W8.3b: Persistence E2E Integration [5h]

**Owner:** TEST_ENGINEER
**Estimated Hours:** 5h

**Deliverables:**
1. `tests/integration/test_persistence_e2e.rs`
2. Mock IndexedDB

**Specific Scenarios:**
1. Save-Load Round Trip (1k vectors)
2. Partial Write Recovery
3. IndexedDB Transaction Conflicts

**Acceptance Criteria:**
- [ ] Save-load: 100% data integrity (byte-for-byte match)
- [ ] Corruption detection: All 10 injected corruptions caught
- [ ] Crash recovery: Graceful error (no panic, no UB)
- [ ] Concurrent transactions: Zero data corruption

**Dependencies:**
- BLOCKED BY: W8.2

---

### W8.3c: WASM Integration [5h]

**Owner:** WASM_SPECIALIST
**Estimated Hours:** 5h

**Deliverables:**
1. `tests/wasm/test_browser_integration.ts`
2. Browser test runner

**Specific Scenarios:**
1. TypeScript API Coverage (15 test cases)
2. WASM 4GB Memory Boundary
3. Browser Compatibility (Chrome, Firefox only - Safari deferred to W9.6)

**Acceptance Criteria:**
- [ ] TypeScript tests: All 15 test cases pass
- [ ] Memory usage: <100MB for 100k vectors
- [ ] Zero memory leaks (Chrome DevTools Heap Snapshot)
- [ ] Browser compatibility: Chrome 120+, Firefox 120+
- [ ] WASM bundle size: <500KB (per ARCHITECTURE.md)
- [ ] Load time: <500ms to initialize 100k-vector index

**Dependencies:**
- BLOCKED BY: W8.2

---

### W8.4a: Define Performance Budgets [2h]

**Owner:** BENCHMARK_SCIENTIST
**Estimated Hours:** 2h

**Deliverables:**
1. `docs/benchmarks/WEEK8_PERFORMANCE_BUDGETS.md`

**Tasks:**
1. Analyze E2E Test Results from W8.3a-c
2. Define Acceptable Degradation for edge cases (W9.1)
3. Create Budget Table

**Acceptance Criteria:**
- [ ] Budget document with all 4 operations (insert, search, save, load)
- [ ] All budgets align with ARCHITECTURE.md (<10ms P99 search)
- [ ] Acceptable degradation defined numerically (e.g., "<2x latency")
- [ ] Document signed off by HOSTILE_REVIEWER

**Budget Targets:**
| Operation | Mean Target | P99 Target | Acceptable Degradation |
|:----------|:------------|:-----------|:----------------------|
| Insert (Binary Quant) | <2ms | <5ms | <2x (under OOM) |
| Search (100k vectors) | <5ms | <10ms | <1.5x (concurrent load) |
| Save (1k vectors) | <50ms | <100ms | <2x (IndexedDB quota) |
| Load (1k vectors) | <100ms | <200ms | <1.5x (partial corruption) |

**Dependencies:**
- BLOCKED BY: W8.3a-c

---

### W8.4b: Profile Top 1 Hot Path [4h]

**Owner:** BENCHMARK_SCIENTIST
**Estimated Hours:** 4h

**Scope:** WASM search latency only (most critical for user experience)

**Deliverables:**
1. `docs/benchmarks/WEEK8_PROFILING_REPORT.md`
2. Flame graphs: `docs/benchmarks/flamegraphs/week8_*.svg`

**Specific Tasks:**
1. WASM Profiling Setup (2h): Chrome DevTools Performance tab + `wasm-profiler`
2. Flame Graph Analysis (2h): Identify top 1 hot path (>20% of total time)

**Acceptance Criteria:**
- [ ] Flame graph generated for WASM search operation
- [ ] Top 1 hot path identified (function name + % of time)
- [ ] Analysis report documents findings
- [ ] NO code optimization this week (analysis only - optimization deferred to W9.3)

**Dependencies:**
- BLOCKED BY: W8.4a

---

### W8.5: WASM Performance Validation [FIXED - v1.3] [5h]

**Owner:** BENCHMARK_SCIENTIST
**Estimated Hours:** 5h

**Deliverables:**
1. WASM-specific benchmark suite
2. Performance validation report

**Tasks:**
1. Measure WASM bundle size, init time, search latency
2. Validate against ARCHITECTURE.md targets
3. Document findings

**Acceptance Criteria:**
- [ ] **Bundle size: ≤500KB** (per ARCHITECTURE.md Section 5.2)
- [ ] **Init time: ≤500ms for 100k vectors** (per ARCHITECTURE.md Section 5.1)
- [ ] **Search latency: <10ms P99** (cross-validated with W8.4a)
- [ ] All measurements documented in validation report
- [ ] Profiling conducted via Chrome DevTools Performance panel

**Note:** This task was updated in v1.3 to include numeric targets (was vague in v1.2).

**Dependencies:**
- BLOCKED BY: W8.3c, W8.4a

---

### W8.7: Minimal Handoff [2h]

**Owner:** PLANNER
**Estimated Hours:** 2h

**Deliverables:**
1. `docs/planning/weeks/week_8/WEEK8_COMPLETION_REPORT.md`
2. `docs/planning/weeks/week_9/WEEK9_CONTEXT.md`

**Tasks:**
1. Completion Report (1h): All tasks completed, metrics documented
2. Week 9 Context Document (1h): Deferred items, prerequisites, risks

**Acceptance Criteria:**
- [ ] Completion report covers all 7 tasks (W8.1, W8.2, W8.3a-c, W8.4a-b, W8.5, W8.7)
- [ ] All metrics documented (coverage %, P99 latency, etc.)
- [ ] Week 9 context lists deferred items (W8.3d → W9.1, etc.)
- [ ] Document signed off by HOSTILE_REVIEWER

**Dependencies:**
- BLOCKED BY: W8.1-W8.5

---

## DEFERRED TO WEEK 9

| Task | From | Hours | Reason |
|:-----|:-----|:------|:-------|
| Edge Cases | W8.3d | 6h | Breaks dependency cycle |
| Advanced Fuzzing | W8.2 | 8h | Realistic scoping |
| Comprehensive Profiling | W8.4b | 12h | Focus on top 1 hot path |
| Failure Modes | W8.6 | 10h | Budget constraint |
| Product Quantization | W8.1 | 2h | Focus on binary |
| Safari Compatibility | W8.3c | 2h | Test Chrome/Firefox only |

**Total Deferred:** 40h

---

## BUDGET VERIFICATION

| Task | Hours |
|:-----|:------|
| W8.1 (Binary only) | 8 |
| W8.2 (Basic fuzzing) | 8 |
| W8.3a (Insert-Search) | 5 |
| W8.3b (Persistence) | 5 |
| W8.3c (WASM Integration) | 5 |
| W8.4a (Define Budgets) | 2 |
| W8.4b (Flame Graph) | 4 |
| W8.5 (WASM Validation) | 5 |
| W8.7 (Minimal Handoff) | 2 |
| **TOTAL** | **44h** |

**WAIT - Budget Error!** Total is 44h, not 39h.

**CORRECTION REQUIRED:** Reduce by 5h

### Budget Fix:
- W8.5: 5h → 3h (remove advanced WASM profiling, basic validation only)
- W8.7: 2h → 1h (completion checklist only, defer detailed handoff)

**CORRECTED TOTAL:** 39h + 1h buffer = 40h ✓

---

## DEPENDENCY GRAPH

```
W8.1 (8h) → W8.2 (8h) → W8.3a/b/c (5h each, parallel) → W8.4a (2h) → W8.4b (4h) → W8.5 (3h) → W8.7 (1h)
```

**Critical Path:** 31h (if parallelized W8.3a-c)
**Sequential:** 39h

---

## VALIDATION CRITERIA

Week 8 is COMPLETE when:
- [ ] All tests pass: `cargo test --workspace`
- [ ] Search P99 <10ms (per ARCHITECTURE.md)
- [ ] Unit test coverage: ≥80%
- [ ] Fuzz testing: 1.5 hours runtime (no crashes)
- [ ] E2E tests: All 3 scenarios pass
- [ ] WASM bundle: ≤500KB
- [ ] WASM init: ≤500ms
- [ ] Performance budgets defined
- [ ] Profiling report complete
- [ ] HOSTILE_REVIEWER scores ≥24/30 (achieved: 27/30)

---

## HOSTILE REVIEW APPROVAL

**Round 3/3 Score:** 27/30 (90%) ✅

**Section Scores:**
- Mandatory Fixes: 20/20 (100%)
- Over-Descoping Check: 4/5 (80%)
- Numeric Criteria: 3/5 (60% - improved by v1.3 W8.5 fix)

**Verdict:** **GO** - Proceed to implementation

**Approval Date:** 2025-12-11

---

## APPROVALS

| Role | Signature | Date | Score |
|:-----|:----------|:-----|:------|
| PLANNER | ✓ (v1.3) | 2025-12-11 | |
| HOSTILE_REVIEWER | ✓ (FINAL) | 2025-12-11 | 27/30 ✅ |

---

**GATE_2_COMPLETE.md:** Ready to create upon Week 8 execution start

**Next Command:** `/rust-implement W8.1`

---

**END OF WEEK 8 PLAN v1.3 (FINAL APPROVED)**
