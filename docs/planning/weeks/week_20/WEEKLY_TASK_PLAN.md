# Week 20: ARM Infrastructure & NEON SIMD Sprint

**Version:** 0.5.0-alpha
**Date Range:** 2025-12-23 to 2025-12-27
**Theme:** ARM Cross-Compilation & NEON SIMD Foundation
**Status:** APPROVED_WITH_AMENDMENTS
**Revision:** 3.0 (Post-Hostile-Review Amendments Applied)

---

## Executive Summary

Week 20 initiates the v0.5.0 development cycle following the successful v0.4.0 release. This sprint focuses **exclusively** on ARM infrastructure and NEON SIMD implementation — the foundation for mobile platform support.

**SCOPE AMENDMENTS (Per Hostile Review HR-2025-12-16-W20-PLAN-FINAL):**
- ACKNOWLEDGED: ARM CI workflow ALREADY EXISTS (.github/workflows/arm-ci.yml)
- ACKNOWLEDGED: NEON detection ALREADY EXISTS (src/simd/detect.rs)
- AMENDED: Day 1 → "VERIFY & EXTEND ARM CI" (not "CREATE")
- AMENDED: Day 2 → Focus on neon.rs module creation (detection exists)
- DEFERRED: Metadata API → Week 21 **P0 CRITICAL** (per external review analysis)
- DEFERRED: Mobile Browser Testing → Week 21
- FOCUS: NEON SIMD implementations + correctness verification

**v0.4.0 Accomplishments (Week 19):**
- Complete documentation suite
- Benchmark dashboard with visualization
- 15 chaos tests + load testing
- P99 latency tracking in CI
- Published to crates.io, npm

**Week 20 Target Scope (Reduced):**
- ARM64 cross-compilation CI pipeline
- NEON feature detection
- NEON SIMD implementations (hamming, dot product, euclidean)
- Basic bundle size analysis
- Correctness verification tests

**Deferred to Week 21 (PRIORITY-ORDERED per External Review Analysis):**
- **P0 CRITICAL:** Metadata storage API (requires design document) — USER #1 COMPLAINT
- P1: Mobile browser testing (requires ARM build working)
- P2: BrowserStack integration (external dependency)
- P3: iOS/Android device testing

---

## Week Overview

| Day | Task ID | Title | Hours | Priority |
|:----|:--------|:------|:------|:---------|
| 1 | W20.1 | **VERIFY** ARM CI & Extend Documentation | 8 | CRITICAL |
| 2 | W20.2 | Create neon.rs Module & Dispatcher Integration | 8 | CRITICAL |
| 3 | W20.3 | NEON Hamming Distance Implementation | 8 | HIGH |
| 4 | W20.4 | NEON Dot Product & Euclidean Distance | 8 | HIGH |
| 5 | W20.5 | Correctness Testing & Bundle Analysis | 8 | HIGH |

**Total Estimated Hours:** 40 hours (full budget, focused scope)

---

## Strategic Objectives

### Primary Goal: ARM Cross-Platform Foundation

**Success Criteria (ALL BINARY):**
- [ ] ARM64 CI workflow passes with exit code 0
- [ ] `cargo test` runs successfully under QEMU emulation
- [ ] NEON feature detection returns correct boolean on ARM64
- [ ] NEON hamming_distance output matches portable output exactly
- [ ] NEON dot_product output matches portable output within 1e-6 epsilon
- [ ] No regressions in x86/AVX2 test suite (all 159 tests pass)

### Non-Goals (Explicitly Deferred)

- Metadata storage API (Week 21 - needs design doc first)
- Mobile browser testing (Week 21 - needs working ARM build)
- iOS Safari testing (Week 22 - needs Xcode, device access)
- Android Chrome testing (Week 22 - needs SDK, emulator)
- Bundle lazy loading (Week 22 - optimization phase)

---

## Estimation Methodology

**3x Rule Applied Correctly:**

| Day | Task | Optimistic | 3x Applied | Buffer | Final |
|:----|:-----|:-----------|:-----------|:-------|:------|
| 1 | ARM CI Setup | 2h | 6h | 2h | 8h |
| 2 | NEON Detection | 2h | 6h | 2h | 8h |
| 3 | NEON Hamming | 2h | 6h | 2h | 8h |
| 4 | NEON Dot/Euclidean | 2h | 6h | 2h | 8h |
| 5 | Testing & Analysis | 2h | 6h | 2h | 8h |

**Total:** 40h with buffer included
**Contingency:** If Day 1 ARM CI fails, Days 2-5 shift to x86-focused work

---

## Risk Register (Concrete Mitigations)

| Risk | Probability | Impact | Mitigation | Fallback |
|:-----|:------------|:-------|:-----------|:---------|
| R1: QEMU CI fails | Medium | CRITICAL | Pre-verify QEMU in isolated test | Use GitHub Actions ARM runners ($$$) |
| R2: Cross-compilation errors | Medium | HIGH | Use Docker `cross` tool | Build in QEMU instead |
| R3: NEON intrinsics bugs | Medium | MEDIUM | Test every function against portable | Ship portable-only for v0.5.0 |
| R4: Performance regression x86 | Low | HIGH | Run full test suite before merge | Revert NEON changes |
| R5: Day 1 overrun | Medium | HIGH | Time-box to 8h, defer remainder | Continue Day 2 on ARM CI |

**Risk Response Protocol:**
- If R1 triggers: Allocate Day 2 to ARM CI completion, shift all tasks +1 day
- If R3 triggers: Feature-flag NEON behind `cfg(feature = "neon")`, ship disabled
- If R4 triggers: Immediate bisect, revert to last known good

---

## Dependency Graph (Corrected)

```
W20.1 (ARM CI) ─────────────────────────┐
    │                                    │
    │ [BLOCKS]                           │
    ▼                                    │
W20.2 (NEON Detection) ─────────────────┤
    │                                    │
    │ [BLOCKS]                           │
    ▼                                    │
W20.3 (NEON Hamming) ──────────────────┤
    │                                    │
    │ [BLOCKS]                           │
    ▼                                    │
W20.4 (NEON Dot/Euclidean) ────────────┤
    │                                    │
    │ [BLOCKS]                           │
    ▼                                    │
W20.5 (Testing & Analysis) ◄────────────┘
```

**Critical Path:** W20.1 → W20.2 → W20.3 → W20.4 → W20.5
**No Parallel Tasks:** Each day depends on previous day completion

---

## Day Summaries

### Day 1 (W20.1): VERIFY ARM CI & Extend Documentation

**AMENDMENT:** ARM CI already exists (.github/workflows/arm-ci.yml, 131 lines).
Day 1 is now **VERIFICATION + EXTENSION**, not creation.

**Objective:** Verify existing ARM CI passes, extend with documentation.

**Deliverables:**
1. **VERIFY** `.github/workflows/arm-ci.yml` passes (already exists)
2. **VERIFY** QEMU test execution works (run locally or trigger CI)
3. **CREATE** Cross-compilation documentation (does not exist)
4. **FIX** Any issues found during verification

**Acceptance Criteria (Binary):**
- [ ] `aarch64-unknown-linux-gnu` target builds with exit code 0
- [ ] `cargo test` runs under QEMU with exit code 0
- [ ] Workflow triggers on push to main and PRs
- [ ] All existing x86 tests still pass (159/159)

**Hostile Review Checkpoint:** End of Day 1

---

### Day 2 (W20.2): Create neon.rs Module & Dispatcher Integration

**AMENDMENT:** NEON detection already exists (src/simd/detect.rs, 330 lines).
SimdCapabilities.neon field and is_aarch64_feature_detected!("neon") already implemented.
Day 2 focuses on **neon.rs MODULE CREATION**, not detection.

**Depends On:** W20.1 complete (ARM CI verified)

**Objective:** Create neon.rs module with stubs, integrate into dispatcher.

**Deliverables:**
1. `src/simd/neon.rs` - NEON module with stubs
2. `src/simd/mod.rs` - Updated dispatcher with NEON path
3. Detection tests passing on ARM CI

**Acceptance Criteria (Binary):**
- [ ] `src/simd/neon.rs` file exists with function stubs
- [ ] `SimdBackend::Neon` variant added to enum (if not exists)
- [ ] Dispatcher routes to NEON path on ARM64
- [ ] Module compiles on all targets with exit code 0
- [ ] `tests/simd_detection.rs` created and passes

**Hostile Review Checkpoint:** End of Day 2

---

### Day 3 (W20.3): NEON Hamming Distance Implementation

**Depends On:** W20.2 complete (NEON detection working)

**Objective:** Implement NEON-optimized hamming distance with correctness proof.

**Deliverables:**
1. `hamming_distance_neon()` implementation
2. Property tests verifying NEON == Portable
3. Benchmark comparison

**Acceptance Criteria (Binary):**
- [ ] `hamming_distance_neon(a, b) == hamming_distance_portable(a, b)` for all inputs
- [ ] Property test with 1000+ random inputs passes
- [ ] Performance measured (target: 2x faster than portable)
- [ ] No unsafe code without safety comment

**Hostile Review Checkpoint:** End of Day 3

---

### Day 4 (W20.4): NEON Dot Product & Euclidean Distance

**Depends On:** W20.3 complete (Hamming working)

**Objective:** Implement NEON-optimized similarity functions.

**Deliverables:**
1. `dot_product_neon()` implementation
2. `euclidean_distance_neon()` implementation
3. Property tests for both
4. Benchmark comparison

**Acceptance Criteria (Binary):**
- [ ] `|dot_product_neon(a, b) - dot_product_portable(a, b)| < 1e-6`
- [ ] `|euclidean_distance_neon(a, b) - euclidean_distance_portable(a, b)| < 1e-6`
- [ ] Property tests with 1000+ random inputs pass
- [ ] Performance measured for both functions

**Hostile Review Checkpoint:** End of Day 4

---

### Day 5 (W20.5): Correctness Testing & Bundle Analysis

**Depends On:** W20.4 complete (All NEON functions implemented)

**Objective:** Comprehensive validation and bundle size documentation.

**Deliverables:**
1. `tests/simd_neon_correctness.rs` - Full test suite
2. `docs/benchmarks/NEON_PERFORMANCE.md` - Performance report
3. `docs/benchmarks/BUNDLE_SIZE_BASELINE.md` - Size analysis
4. Week 20 completion summary

**Acceptance Criteria (Binary):**
- [ ] All NEON correctness tests pass (100%)
- [ ] Performance report shows measurements for all functions
- [ ] Bundle size measured and documented (current: ~227KB gzipped)
- [ ] No regressions in x86 test suite (159/159 pass)

**Hostile Review Checkpoint:** End of Day 5 (Week Final)

---

## Quality Gates

### Gate 20.1: ARM CI Complete
- [ ] Cross-compilation succeeds
- [ ] QEMU tests pass
- [ ] `/review` approved

### Gate 20.2: NEON Detection Complete
- [ ] Detection function works
- [ ] Module scaffold in place
- [ ] `/review` approved

### Gate 20.3: NEON Hamming Complete
- [ ] Implementation correct
- [ ] Property tests pass
- [ ] `/review` approved

### Gate 20.4: NEON Similarity Complete
- [ ] Both functions correct
- [ ] Property tests pass
- [ ] `/review` approved

### Gate 20.5: Week 20 Complete
- [ ] All tests pass
- [ ] Documentation complete
- [ ] HOSTILE_REVIEWER final approval
- [ ] GATE_20_COMPLETE.md created

---

## Technical Clarifications (Per Hostile Review)

### WASM vs Native SIMD (CORRECTED)

**Native ARM builds:** Use NEON intrinsics (`std::arch::aarch64::*`)
**WASM builds:** Use WASM SIMD128 (`std::arch::wasm32::*`)
**These are SEPARATE codepaths:**

```rust
#[cfg(target_arch = "aarch64")]
mod neon;  // Uses NEON intrinsics

#[cfg(target_arch = "wasm32")]
mod wasm_simd;  // Uses WASM SIMD128

#[cfg(not(any(target_arch = "aarch64", target_arch = "wasm32", target_arch = "x86_64")))]
mod portable;  // Fallback for all other architectures
```

Week 20 focuses on **Native ARM NEON only**. WASM SIMD128 is a separate effort.

---

## Success Criteria

Week 20 is **COMPLETE** when:

1. [ ] ARM64 CI pipeline operational (green builds)
2. [ ] NEON feature detection working
3. [ ] NEON hamming_distance correct (matches portable)
4. [ ] NEON dot_product correct (within epsilon)
5. [ ] NEON euclidean_distance correct (within epsilon)
6. [ ] All property tests pass (1000+ inputs each)
7. [ ] Performance benchmarks documented
8. [ ] Bundle size baseline documented
9. [ ] No regressions in x86 tests (159/159)
10. [ ] HOSTILE_REVIEWER grants final GO verdict
11. [ ] GATE_20_COMPLETE.md created

---

## Constraints

- **Time:** Maximum 8 hours per day, 40 hours total
- **Technical:** MSRV 1.70, no new dependencies without approval
- **Quality:** All deliverables must pass `/review`
- **Process:** Each day BLOCKS the next (sequential execution)
- **Safety:** All `unsafe` code requires safety comment

---

## v0.5.0 Strategic Roadmap (BINDING)

**REFERENCE:** `docs/planning/V0.5.0_STRATEGIC_ROADMAP.md`

**STRATEGIC CONTEXT:** External v0.4.0 review identified METADATA and FILTERING as production blockers.
Both features will ship in v0.5.0 via a phased approach approved by HOSTILE_REVIEWER.

```
Week 20: ARM/NEON SIMD ◄── YOU ARE HERE
    │
    ▼
Week 21: METADATA_API + Mobile Testing (40h)
    │    └── Deliverable: GATE_W21_COMPLETE.md + schema FROZEN
    ▼
Week 22: FILTERING_ARCHITECTURE (16h design sprint)
    │    └── Deliverable: docs/architecture/FILTERING_API.md (HOSTILE approved)
    ▼
Week 23: FILTERING_IMPLEMENTATION (60h)
    │    └── Deliverable: Full filtering with tests, docs, WASM bindings
    ▼
Week 24: v0.5.0 RELEASE
```

### Week 21 Scope (After Week 20)

| Item | Hours | Priority | Notes |
|:-----|:------|:---------|:------|
| **Metadata API** | 24h | **P0 CRITICAL** | USER #1 COMPLAINT |
| Mobile Browser Testing | 8h | P1 HIGH | iOS Safari, Android Chrome |
| BrowserStack Setup | 8h | P2 MEDIUM | CI integration |

**Week 21 Day 1 MANDATORY:** Begin METADATA_API implementation.
**Week 21 Exit Criteria:** Metadata schema FROZEN (no changes without major version bump).

### Week 22 Scope (Filtering Design)

| Item | Hours | Priority | Notes |
|:-----|:------|:---------|:------|
| **Filtering Architecture** | 16h | **P0 CRITICAL** | Design only, NO implementation |
| Other work | 24h | As needed | — |

**Prerequisite:** GATE_W21_COMPLETE.md must exist.
**Deliverable:** `docs/architecture/FILTERING_API.md` with HOSTILE_REVIEWER approval.

### Week 23 Scope (Filtering Implementation)

| Item | Hours | Priority | Notes |
|:-----|:------|:---------|:------|
| Query Parser | 12h | P0 | — |
| Filter Evaluator | 12h | P0 | — |
| Search Integration | 9h | P0 | — |
| WASM Bindings | 6h | P0 | — |
| Tests | 15h | P0 | Unit + Property |
| Documentation | 6h | P0 | — |

**Prerequisite:** FILTERING_API.md approved by HOSTILE_REVIEWER.
**Total:** 60h (may extend into Week 24).

**PLANNER INSTRUCTION:** After Week 20, consult `docs/planning/V0.5.0_STRATEGIC_ROADMAP.md` for binding timeline.

---

## Handoff Protocol

After each day:
1. Complete all deliverables listed in DAY_X_TASKS.md
2. Run `/review [artifact]` for each deliverable
3. Update this document with completion status
4. **ONLY proceed to next day after review approval**
5. If blocked, document blocker and apply risk mitigation

---

**PLANNER:** Week 20 Planning Complete (Revision 3.0)
**Status:** APPROVED_WITH_AMENDMENTS
**Changes:**
- Acknowledged existing ARM CI (C1 fix)
- Acknowledged existing NEON detection (M2 fix)
- Updated Day 1 to VERIFY mode
- Updated Day 2 to focus on neon.rs creation
- Elevated Metadata API to Week 21 P0 per external review
**Review:** docs/reviews/2025-12-16_W20_PLAN_HOSTILE_REVIEW.md
**Next:** Execute Week 20, then create GATE_W20_PLANNING_COMPLETE.md
