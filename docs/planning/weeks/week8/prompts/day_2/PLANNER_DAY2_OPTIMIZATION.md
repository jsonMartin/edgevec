# PLANNER: W8D37 (Day 2) SIMD Workflow Optimization

**Agent:** PLANNER
**Command:** `/planner-optimize day2`
**Priority:** P0 (CRITICAL — Workflow Correctness)
**Date:** 2025-12-12
**Input:** Existing Day 2 prompts + 07_NVIDIA_GRADE_HOSTILE_REVIEW.md
**Output:** Optimized execution plan with test-first enforcement

---

## OBJECTIVE

Reorganize Week 8 Day 2 (SIMD Implementation) workflow to enforce:
1. **Test-First Development** — Tests BEFORE code
2. **Benchmark-First Targets** — Performance targets BEFORE optimization
3. **Architecture-First Design** — Design BEFORE implementation
4. **Hostile Review Gates** — Quality gates at every phase
5. **Anti-Hallucination Protocol** — Evidence required for all claims

---

## CURRENT STATE ANALYSIS

### Existing Prompts (As Created)

```
Current folder: docs/planning/weeks/week8/prompts/day_2/

Files:
00_MASTER_DISPATCH.md
01_SIMD_ARCHITECTURE.md          (META_ARCHITECT)
02_SIMD_HAMMING_IMPL.md          (RUST_ENGINEER)
03_SIMD_QUANTIZE_IMPL.md         (RUST_ENGINEER - optional)
04_SIMD_BENCHMARKS.md            (BENCHMARK_SCIENTIST)
05_SIMD_TESTS.md                 (TEST_ENGINEER)
06_HOSTILE_REVIEW.md             (HOSTILE_REVIEWER)
07_NVIDIA_GRADE_HOSTILE_REVIEW.md (HOSTILE_REVIEWER - enhanced)
META_CORRECTION_TEST_FIRST.md    (PROMPT_MAKER meta-analysis)
```

### Problem Identified

**CRITICAL WORKFLOW ERROR:** Tests come AFTER implementation (prompts 02-03 before 05)

```
WRONG ORDER:
01_ARCHITECTURE → 02_IMPL → 03_IMPL → 04_BENCHMARKS → 05_TESTS → 06_REVIEW
                   ↑ CODE FIRST                         ↑ TESTS AFTER
```

This violates Test-First mandate and enables hallucinations.

---

## OPTIMIZED WORKFLOW DESIGN

### Reorganized Execution Sequence

```
CORRECT ORDER (TEST-FIRST):

PHASE A: SPECIFICATION (NO CODE ALLOWED)
├── A.1: Architecture Design
├── A.2: Test Specification (BEFORE implementation)
├── A.3: Benchmark Specification (BEFORE implementation)
└── GATE A: Architecture + Tests + Benchmarks reviewed

PHASE B: IMPLEMENTATION (CODE ALLOWED)
├── B.1: SIMD Hamming Implementation (make tests pass)
├── B.2: SIMD Quantize Implementation (optional)
└── GATE B: All tests pass

PHASE C: VALIDATION (PROVE CORRECTNESS)
├── C.1: Benchmark Execution (verify targets met)
├── C.2: Integration Testing
└── GATE C: Performance validated

PHASE D: QUALITY ASSURANCE
├── D.1: Standard Hostile Review
├── D.2: NVIDIA-Grade Hostile Review
└── GATE D: Final approval

PHASE E: DOCUMENTATION
├── E.1: SIMD Architecture Documentation
└── GATE E: Day 2 complete
```

---

## OPTIMIZED PROMPT SUITE

### New File Structure

```
docs/planning/weeks/week8/prompts/day_2/

EXECUTION ORDER (renamed for clarity):
├── 00_MASTER_PLAN.md                 (This file - execution guide)
│
├── PHASE_A_SPECIFICATION/
│   ├── A1_ARCHITECTURE.md            (META_ARCHITECT - design SIMD)
│   ├── A2_TEST_SPEC.md               (TEST_ENGINEER - write failing tests)
│   ├── A3_BENCHMARK_SPEC.md          (BENCHMARK_SCIENTIST - define targets)
│   └── A4_ARCHITECTURE_REVIEW.md     (HOSTILE_REVIEWER - approve design)
│
├── PHASE_B_IMPLEMENTATION/
│   ├── B1_HAMMING_IMPL.md            (RUST_ENGINEER - make tests pass)
│   ├── B2_QUANTIZE_IMPL.md           (RUST_ENGINEER - optional)
│   └── B3_IMPL_REVIEW.md             (HOSTILE_REVIEWER - code review)
│
├── PHASE_C_VALIDATION/
│   ├── C1_BENCHMARK_EXECUTION.md     (BENCHMARK_SCIENTIST - measure)
│   ├── C2_INTEGRATION_TESTS.md       (TEST_ENGINEER - full validation)
│   └── C3_VALIDATION_REVIEW.md       (HOSTILE_REVIEWER - verify claims)
│
├── PHASE_D_QUALITY_GATE/
│   ├── D1_HOSTILE_REVIEW.md          (HOSTILE_REVIEWER - standard)
│   └── D2_NVIDIA_GRADE_REVIEW.md     (HOSTILE_REVIEWER - final gate)
│
└── SUPPORTING/
    ├── META_CORRECTION_TEST_FIRST.md (PROMPT_MAKER - methodology)
    └── ANTI_HALLUCINATION_PROTOCOL.md (Verification checklist)
```

---

## DETAILED PHASE BREAKDOWN

### PHASE A: SPECIFICATION (3-4 hours, NO CODE)

#### A.1: Architecture Design (1 hour)

**Prompt:** `A1_ARCHITECTURE.md`
**Agent:** META_ARCHITECT
**Command:** `/architect-design simd_hamming`

**Deliverables:**
- `docs/architecture/SIMD_DESIGN.md`
- Algorithm pseudocode
- Dispatch strategy (compile-time vs runtime)
- Platform support matrix (AVX2, NEON, WASM)
- Memory layout and alignment requirements

**Acceptance Criteria:**
- [ ] All SIMD operations documented
- [ ] Safety invariants documented
- [ ] Performance budget calculated (target: <50 cycles)
- [ ] Fallback strategy defined

**Gate:** Proceed only if architecture is internally consistent

---

#### A.2: Test Specification (1.5 hours)

**Prompt:** `A2_TEST_SPEC.md`
**Agent:** TEST_ENGINEER
**Command:** `/test-spec simd_hamming`

**Deliverables:**
- `tests/simd_correctness.rs` (tests WILL FAIL initially)
- Minimum 25 unit tests
- Property-based tests (10,000+ cases)
- Edge case tests (boundaries, zeros, ones)
- Fuzz corpus replay tests

**Critical Requirement:**
```rust
// ALL TESTS MUST BE WRITTEN BEFORE IMPLEMENTATION EXISTS

#[test]
fn test_simd_matches_portable_zeros() {
    let a = [0x00u8; 96];
    let b = [0x00u8; 96];

    // This will FAIL until implementation exists (correct!)
    let portable = portable::hamming_distance(&a, &b);
    let simd = simd::hamming_distance(&a, &b);

    assert_eq!(portable, simd);
}
```

**Acceptance Criteria:**
- [ ] Tests compile (may need stub imports)
- [ ] All tests currently FAIL (no implementation yet)
- [ ] Test coverage plan: correctness, boundaries, properties
- [ ] No implementation code exists in `src/quantization/simd.rs`

**Gate:** Proceed only if tests are comprehensive and failing

---

#### A.3: Benchmark Specification (30 minutes)

**Prompt:** `A3_BENCHMARK_SPEC.md`
**Agent:** BENCHMARK_SCIENTIST
**Command:** `/bench-spec simd_hamming`

**Deliverables:**
- `benches/bench_simd.rs` (skeleton with targets)
- Performance targets documented
- Cycle count measurement protocol
- Speedup calculation methodology

**Performance Targets:**
| Metric | Target | Hard Limit |
|:-------|:-------|:-----------|
| AVX2 Hamming (cycles) | <50 | <75 |
| Speedup vs Portable | >5x | >3x |
| Throughput | >1B ops/sec | >500M ops/sec |

**Acceptance Criteria:**
- [ ] All targets quantified
- [ ] Measurement methodology defined
- [ ] Benchmark skeleton compiles

**Gate:** Proceed only if targets are measurable and realistic

---

#### A.4: Architecture Review (30 minutes)

**Prompt:** `A4_ARCHITECTURE_REVIEW.md`
**Agent:** HOSTILE_REVIEWER
**Command:** `/review SIMD_DESIGN.md`

**Review Dimensions:**
- Internal consistency
- Safety analysis
- Performance budget
- Testability

**Gate:** BLOCKING — No implementation until architecture approved

---

### PHASE B: IMPLEMENTATION (3-4 hours, CODE ALLOWED)

#### B.1: SIMD Hamming Implementation (2.5 hours)

**Prompt:** `B1_HAMMING_IMPL.md`
**Agent:** RUST_ENGINEER
**Command:** `/rust-implement W8.37.1`

**Mission:** Make ALL tests from A.2 pass

**Deliverables:**
- `src/quantization/simd.rs`
- AVX2 implementation
- Portable fallback
- Runtime dispatch

**Constraints:**
- [ ] ALL tests from A.2 must pass
- [ ] No new tests added (use spec tests only)
- [ ] Every `unsafe` documented with safety proof
- [ ] No performance regression on portable path

**TDD Protocol:**
```bash
# 1. Run tests (should fail)
cargo test simd

# 2. Implement until one test passes
# 3. Repeat until ALL tests pass
# 4. Verify no tests were modified
git diff tests/simd_correctness.rs  # Should be empty
```

**Gate:** Proceed only if `cargo test` shows 0 failures

---

#### B.2: SIMD Quantize Implementation (1 hour, OPTIONAL)

**Prompt:** `B2_QUANTIZE_IMPL.md`
**Agent:** RUST_ENGINEER
**Command:** `/rust-implement W8.37.2`

**Decision Gate:**
```
Implement ONLY if:
1. B.1 complete with ALL tests passing
2. Time remaining ≥1 hour
3. Quantization is a bottleneck (measure first!)

If any condition is NO → SKIP (defer to Week 9)
```

**Gate:** Proceed only if optional work doesn't jeopardize Day 2

---

#### B.3: Implementation Review (30 minutes)

**Prompt:** `B3_IMPL_REVIEW.md`
**Agent:** HOSTILE_REVIEWER
**Command:** `/review-impl simd`

**Review Focus:**
- Test-First compliance (git log verification)
- All tests passing
- No test modifications
- Safety documentation

**Gate:** BLOCKING — No validation until code review passes

---

### PHASE C: VALIDATION (2 hours, PROVE CORRECTNESS)

#### C.1: Benchmark Execution (1 hour)

**Prompt:** `C1_BENCHMARK_EXECUTION.md`
**Agent:** BENCHMARK_SCIENTIST
**Command:** `/bench-validate simd_hamming`

**Mission:** Prove implementation hits ALL targets from A.3

**Deliverables:**
- `docs/benchmarks/W8D37_simd_report.md`
- Cycle count measurement (<50 cycles)
- Speedup measurement (>5x)
- Throughput measurement (>1B ops/sec)

**Evidence Required:**
```bash
# Cycle count (rdtsc)
Cycles per Hamming: 46 ✅ (target: <50)

# Speedup (criterion)
SIMD:     14.2 ns
Portable: 78.5 ns
Speedup:  5.5x ✅ (target: >5x)

# Throughput
1.12B ops/sec ✅ (target: >1B)
```

**Gate:** BLOCKING — Any target missed = REJECT Day 2

---

#### C.2: Integration Testing (30 minutes)

**Prompt:** `C2_INTEGRATION_TESTS.md`
**Agent:** TEST_ENGINEER
**Command:** `/test-integration simd`

**Mission:** Verify SIMD integrates correctly with existing system

**Deliverables:**
- Day 36 regression tests pass
- `QuantizedVector::hamming_distance` uses SIMD
- No API breaking changes

**Gate:** Proceed only if integration clean

---

#### C.3: Validation Review (30 minutes)

**Prompt:** `C3_VALIDATION_REVIEW.md`
**Agent:** HOSTILE_REVIEWER
**Command:** `/review-validation W8D37`

**Review Focus:**
- All benchmark targets met
- Evidence provided for all claims
- No hallucinations detected

**Gate:** BLOCKING — Validation must be proven

---

### PHASE D: QUALITY GATE (2 hours, FINAL APPROVAL)

#### D.1: Standard Hostile Review (1 hour)

**Prompt:** `D1_HOSTILE_REVIEW.md`
**Agent:** HOSTILE_REVIEWER
**Command:** `/review W8D37_simd`

**Dimensions:**
- Correctness (35%)
- Performance (30%)
- Safety (20%)
- API Compatibility (10%)
- Code Quality (5%)

**Threshold:** ≥8.5/10 to proceed

---

#### D.2: NVIDIA-Grade Hostile Review (1 hour)

**Prompt:** `D2_NVIDIA_GRADE_REVIEW.md`
**Agent:** HOSTILE_REVIEWER
**Command:** `/review-nvidia W8D37_simd`

**Dimensions (11 Total):**
1. Test-First Compliance (15%)
2. Correctness (20%)
3. Performance (15%)
4. Safety (15%)
5. API Compatibility (10%)
6. Code Quality (5%)
7. Anti-Hallucination (10%)
8. Documentation (3%)
9. Benchmark Methodology (4%)
10. Architecture Compliance (2%)
11. Regression Testing (1%)

**Threshold:** ≥9.0/10 for APPROVED

**Auto-Reject Conditions:**
- Test-First violation
- Any test failure
- Undocumented unsafe
- Performance >75 cycles
- Any hallucinated claim
- Breaking API change

**Gate:** FINAL — This is the ultimate quality gate

---

### PHASE E: DOCUMENTATION (30 minutes)

**Deliverables:**
- Update `docs/architecture/SIMD_DESIGN.md` with final details
- Create `docs/reviews/2025-12-12_W8D37_simd_NVIDIA_AUDIT.md`
- Lock all artifacts

---

## EXECUTION TIMELINE

```
┌─────────────────────────────────────────────────────────────────────┐
│                    W8D37 (Day 2) EXECUTION PLAN                     │
│                     Total: 10-12 hours (1.5 days)                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  00:00-01:00  PHASE A.1: Architecture Design                        │
│               Agent: META_ARCHITECT                                 │
│               Output: SIMD_DESIGN.md                                │
│               Gate: Internal consistency ✅                         │
│                                                                     │
│  01:00-02:30  PHASE A.2: Test Specification                         │
│               Agent: TEST_ENGINEER                                  │
│               Output: tests/simd_correctness.rs (FAILING)           │
│               Gate: Comprehensive tests ✅                          │
│                                                                     │
│  02:30-03:00  PHASE A.3: Benchmark Specification                    │
│               Agent: BENCHMARK_SCIENTIST                            │
│               Output: benches/bench_simd.rs (targets)               │
│               Gate: Measurable targets ✅                           │
│                                                                     │
│  03:00-03:30  PHASE A.4: Architecture Review                        │
│               Agent: HOSTILE_REVIEWER                               │
│               Output: Architecture approval                         │
│               Gate: BLOCKING ⛔                                      │
│                                                                     │
│  ├───────────────── CHECKPOINT A ──────────────────┤               │
│  │  Deliverables: Architecture + Tests + Benchmarks  │               │
│  │  Code Status: NO IMPLEMENTATION YET (correct!)    │               │
│  └───────────────────────────────────────────────────┘               │
│                                                                     │
│  03:30-06:00  PHASE B.1: SIMD Hamming Implementation                │
│               Agent: RUST_ENGINEER                                  │
│               Output: src/quantization/simd.rs                      │
│               Constraint: Make ALL tests pass                       │
│               Gate: cargo test = 0 failures ✅                      │
│                                                                     │
│  06:00-07:00  PHASE B.2: SIMD Quantize (OPTIONAL)                   │
│               Agent: RUST_ENGINEER                                  │
│               Decision: Skip if time constrained                    │
│                                                                     │
│  07:00-07:30  PHASE B.3: Implementation Review                      │
│               Agent: HOSTILE_REVIEWER                               │
│               Output: Code review approval                          │
│               Gate: BLOCKING ⛔                                      │
│                                                                     │
│  ├───────────────── CHECKPOINT B ──────────────────┤               │
│  │  Deliverables: Working implementation              │               │
│  │  Code Status: ALL TESTS PASSING                    │               │
│  └───────────────────────────────────────────────────┘               │
│                                                                     │
│  07:30-08:30  PHASE C.1: Benchmark Execution                        │
│               Agent: BENCHMARK_SCIENTIST                            │
│               Output: W8D37_simd_report.md                          │
│               Constraint: ALL targets met                           │
│               Gate: <50 cycles, >5x speedup ✅                      │
│                                                                     │
│  08:30-09:00  PHASE C.2: Integration Testing                        │
│               Agent: TEST_ENGINEER                                  │
│               Output: Integration test results                      │
│               Gate: No regressions ✅                               │
│                                                                     │
│  09:00-09:30  PHASE C.3: Validation Review                          │
│               Agent: HOSTILE_REVIEWER                               │
│               Output: Validation approval                           │
│               Gate: BLOCKING ⛔                                      │
│                                                                     │
│  ├───────────────── CHECKPOINT C ──────────────────┤               │
│  │  Deliverables: Performance validated              │               │
│  │  Evidence: All benchmarks ≥ targets               │               │
│  └───────────────────────────────────────────────────┘               │
│                                                                     │
│  09:30-10:30  PHASE D.1: Standard Hostile Review                    │
│               Agent: HOSTILE_REVIEWER                               │
│               Output: Standard review report                        │
│               Threshold: ≥8.5/10 ✅                                 │
│                                                                     │
│  10:30-11:30  PHASE D.2: NVIDIA-Grade Hostile Review                │
│               Agent: HOSTILE_REVIEWER                               │
│               Output: NVIDIA audit report                           │
│               Threshold: ≥9.0/10 for APPROVED ✅                    │
│               Gate: FINAL GATE ⛔                                    │
│                                                                     │
│  ├───────────────── CHECKPOINT D ──────────────────┤               │
│  │  Verdict: APPROVED / CONDITIONAL / REJECTED      │               │
│  │  Authority: HOSTILE_REVIEWER (kill authority)    │               │
│  └───────────────────────────────────────────────────┘               │
│                                                                     │
│  11:30-12:00  PHASE E: Documentation & Lock                         │
│               Update docs, create final reports                     │
│                                                                     │
│  ═══════════════════ DAY 2 COMPLETE ═══════════════                │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## QUALITY GATES SUMMARY

| Gate | Phase | Blocker | Criteria | Authority |
|:-----|:------|:--------|:---------|:----------|
| A | Architecture | YES | Design approved | HOSTILE_REVIEWER |
| B | Implementation | YES | All tests pass | HOSTILE_REVIEWER |
| C | Validation | YES | Targets met | HOSTILE_REVIEWER |
| D | Final Review | YES | ≥9.0/10 score | HOSTILE_REVIEWER |

**Any gate failure → STOP → Fix → Resubmit**

---

## ANTI-HALLUCINATION PROTOCOL

### Verification Checklist (Run Before Each Gate)

```bash
# 1. Test-First Compliance
git log --diff-filter=A --oneline -- tests/ src/quantization/simd.rs
# Tests MUST appear before implementation

# 2. All Tests Pass
cargo test 2>&1 | grep "test result"
# REQUIRED: "0 failed"

# 3. All Claims Have Evidence
grep -rn "cycle\|faster\|speedup" src/ docs/
# Each claim must reference benchmark

# 4. No Undocumented Unsafe
diff <(grep -c "unsafe" src/quantization/simd.rs) \
     <(grep -c "// SAFETY:" src/quantization/simd.rs)
# Counts must match

# 5. Performance Targets Met
cargo bench --bench bench_simd 2>&1 | grep "time:"
# Verify <50 cycles

# 6. No Test Modifications
git diff tests/simd_correctness.rs
# Should be empty (tests unchanged since spec phase)
```

---

## SUCCESS CRITERIA

Day 2 is **COMPLETE** when ALL of the following are TRUE:

### Code Quality
- [ ] All tests pass (0 failures)
- [ ] All property tests pass (≥10,000 cases)
- [ ] Clippy clean (0 warnings)
- [ ] cargo doc clean (0 warnings)
- [ ] All unsafe documented with safety proofs

### Performance
- [ ] AVX2 Hamming: <50 cycles (measured via rdtsc)
- [ ] Speedup: >5x vs portable (measured via criterion)
- [ ] Throughput: >1B ops/sec (measured via criterion)
- [ ] No portable regression (<5% change)

### Test-First Compliance
- [ ] Tests created before implementation (git log verified)
- [ ] Tests unchanged after implementation (git diff verified)
- [ ] All test cases from spec implemented

### Quality Gates
- [ ] Architecture approved (Gate A)
- [ ] Implementation approved (Gate B)
- [ ] Validation approved (Gate C)
- [ ] NVIDIA-Grade review: ≥9.0/10 (Gate D)

### Documentation
- [ ] `SIMD_DESIGN.md` complete
- [ ] `W8D37_simd_report.md` complete
- [ ] NVIDIA audit report complete
- [ ] All safety proofs documented

---

## FAILURE RESPONSE PROTOCOL

### If Gate A Fails (Architecture)
```
Action: Revise architecture
Time: +2 hours
Decision: META_ARCHITECT reworks design
Resubmit: A.4 Architecture Review
```

### If Gate B Fails (Implementation)
```
Action: Fix code until tests pass
Time: +1-3 hours
Decision: RUST_ENGINEER debugs failures
Resubmit: B.3 Implementation Review
```

### If Gate C Fails (Validation)
```
Action: Optimize or revise targets
Time: +2-4 hours
Decision:
  Option A: Optimize implementation
  Option B: Revise targets (requires architecture review)
  Option C: Defer SIMD to Week 9
Escalate: PLANNER if >4 hours needed
```

### If Gate D Fails (Final Review)
```
Score <9.0 → Review report for issues
Score 8.0-8.9 → Fix within 2 hours
Score <8.0 → Major rework or defer

Decision: HOSTILE_REVIEWER has kill authority
Escalate: PLANNER for schedule impact
```

---

## RESOURCE ALLOCATION

| Phase | Agent | Hours | Buffer |
|:------|:------|:------|:-------|
| A: Specification | META_ARCHITECT, TEST_ENGINEER, BENCHMARK_SCIENTIST | 3.5 | 0.5 |
| B: Implementation | RUST_ENGINEER | 3.5 | 1.0 |
| C: Validation | BENCHMARK_SCIENTIST, TEST_ENGINEER | 2.0 | 0.5 |
| D: Quality Gate | HOSTILE_REVIEWER | 2.0 | 0.5 |
| E: Documentation | DOCWRITER | 0.5 | 0.0 |
| **Total** | — | **11.5** | **2.5** |

**Buffered Total:** 14 hours (1.75 days)

---

## DELIVERABLES CHECKLIST

### Code Artifacts
- [ ] `src/quantization/simd.rs` (AVX2 + portable + dispatch)
- [ ] `tests/simd_correctness.rs` (≥25 tests, all passing)
- [ ] `benches/bench_simd.rs` (cycle count + criterion)

### Documentation Artifacts
- [ ] `docs/architecture/SIMD_DESIGN.md`
- [ ] `docs/benchmarks/W8D37_simd_report.md`
- [ ] `docs/reviews/2025-12-12_W8D37_simd_NVIDIA_AUDIT.md`

### Review Artifacts
- [ ] Architecture review (Gate A)
- [ ] Implementation review (Gate B)
- [ ] Validation review (Gate C)
- [ ] Standard hostile review (Gate D.1)
- [ ] NVIDIA-grade hostile review (Gate D.2)

---

## PLANNER CERTIFICATION

I, PLANNER, certify that this optimized workflow:

1. **Enforces Test-First Development** — Tests written before code
2. **Enforces Benchmark-First Targets** — Targets defined before optimization
3. **Includes 4 Quality Gates** — Each phase has approval checkpoint
4. **Provides Anti-Hallucination Protocol** — All claims require evidence
5. **Defines Success Criteria** — Binary pass/fail for each deliverable
6. **Allocates Buffer Time** — 2.5 hours buffer for unexpected issues
7. **Establishes Failure Protocol** — Clear escalation paths

**This plan is APPROVED for execution.**

**Planner:** PLANNER v2.0
**Date:** 2025-12-12
**Status:** READY FOR EXECUTION

---

**END OF PLANNER DAY 2 OPTIMIZATION**
