# PROMPT_MAKER: PLANNER — Fix Day 2 Prompt Suite Critical Issues

**Target Agent:** PLANNER
**Command:** `/planner-fix day2_prompts`
**Priority:** P0 (CRITICAL — Blocks Day 2 execution)
**Estimated Time:** 2.5 hours
**Dependencies:** `docs/reviews/2025-12-12_DAY2_PROMPTS_META_REVIEW.md` (COMPLETED)
**Output:** Fixed Day 2 prompt suite with test-first ordering
**Date:** 2025-12-12

---

## MISSION

The HOSTILE_REVIEWER meta-review has identified **critical blocking issues** in the Day 2 prompt suite that prevent execution. Your mission is to systematically fix all critical and major issues within 2.5 hours.

**Meta-Review Verdict:** MAJOR REVISIONS REQUIRED (Score: 7.95/10.0)
**Auto-Reject Condition:** Test-First Violation Detected

**Your Authority:** Make necessary structural changes to prompts to achieve test-first compliance

---

## CONTEXT

**Meta-Review Location:**
```
docs/reviews/2025-12-12_DAY2_PROMPTS_META_REVIEW.md
```

**Current Prompt Suite:**
```
docs/planning/weeks/week8/prompts/day_2/
├── 00_MASTER_DISPATCH.md
├── 01_SIMD_ARCHITECTURE.md
├── 02_SIMD_HAMMING_IMPL.md         ← CODE BEFORE TESTS ❌
├── 03_SIMD_QUANTIZE_IMPL.md        ← CODE BEFORE TESTS ❌
├── 04_SIMD_BENCHMARKS.md
├── 05_SIMD_TESTS.md                ← TESTS AFTER CODE ❌
├── 06_HOSTILE_REVIEW.md
├── 07_NVIDIA_GRADE_HOSTILE_REVIEW.md
├── META_CORRECTION_TEST_FIRST.md
├── PLANNER_DAY2_OPTIMIZATION.md
└── PROMPT_MAKER_META_REVIEW.md
```

**Problem:** Execution order violates test-first methodology (implementation before tests)

---

## CRITICAL ISSUES TO FIX

### CRIT-001: Test-First Workflow Violation (30 min)

**Current Order (WRONG):**
```
01_ARCHITECTURE → 02_IMPL → 03_IMPL → 04_BENCHMARKS → 05_TESTS → 06_REVIEW
```

**Required Order (CORRECT):**
```
01_ARCHITECTURE → 02_TEST_SPEC → 03_BENCHMARK_SPEC → 04_IMPL → 05_VALIDATION → 06_REVIEW
```

**Your Tasks:**

#### Task 1.1: Renumber Existing Prompts (15 min)

Rename files to enforce test-first order:

```bash
# OLD → NEW
00_MASTER_DISPATCH.md → 00_MASTER_DISPATCH.md (update references)
01_SIMD_ARCHITECTURE.md → 01_SIMD_ARCHITECTURE.md (unchanged)
02_SIMD_HAMMING_IMPL.md → 04_SIMD_HAMMING_IMPL.md
03_SIMD_QUANTIZE_IMPL.md → 05_SIMD_QUANTIZE_IMPL.md
04_SIMD_BENCHMARKS.md → (merge into 06_SIMD_VALIDATION.md)
05_SIMD_TESTS.md → (split into 02_TEST_SPEC + 06_VALIDATION)
06_HOSTILE_REVIEW.md → 07_HOSTILE_REVIEW.md
07_NVIDIA_GRADE_HOSTILE_REVIEW.md → 08_NVIDIA_GRADE_HOSTILE_REVIEW.md
```

**Deliverable:**
- [ ] All prompt files renamed
- [ ] Cross-references updated in all files

---

#### Task 1.2: Create 02_SIMD_TEST_SPEC.md (NEW) (30 min)

**Purpose:** Specify tests BEFORE any implementation code is written

**Extract Content From:**
- `05_SIMD_TESTS.md` (test specifications sections)
- `META_CORRECTION_TEST_FIRST.md` (test-first methodology)

**Template:**

```markdown
# PROMPT: SIMD Test Specification (Test-First)

**Target Agent:** TEST_ENGINEER
**Command:** `/test-spec simd_hamming`
**Priority:** P0 (BLOCKING — Must complete before implementation)
**Estimated Time:** 1.5 hours
**Dependencies:**
  - docs/architecture/SIMD_DESIGN.md exists
  - docs/reviews/*SIMD_DESIGN*_APPROVED.md exists
**Output:** `tests/simd_spec.rs` (test specifications, ALL FAILING initially)

---

## MISSION

Write comprehensive test specifications BEFORE any SIMD implementation code exists.

**Test-First Principle:**
> "If you can't write the test, you don't understand the requirement."

**Critical Requirement:** These tests will FAIL initially because no implementation exists yet. THIS IS CORRECT AND EXPECTED.

---

## TEST CATEGORIES TO SPECIFY

### 1. Correctness Tests (MUST HAVE)

Write test skeletons for:
- [ ] SIMD matches portable (all zeros)
- [ ] SIMD matches portable (all ones)
- [ ] SIMD matches portable (alternating pattern)
- [ ] Symmetry property: distance(a, b) == distance(b, a)
- [ ] Self-distance property: distance(a, a) == 0
- [ ] Triangle inequality property

Each test MUST:
```rust
#[test]
fn test_simd_matches_portable_zeros() {
    let a = [0x00u8; 96];
    let b = [0x00u8; 96];

    // This will FAIL until implementation exists (correct!)
    let portable_result = portable::hamming_distance(&a, &b);
    let simd_result = simd::hamming_distance(&a, &b);

    assert_eq!(portable_result, simd_result);
}
```

### 2. Boundary Tests (MUST HAVE)

Specify tests for:
- [ ] AVX2 register boundary (bytes 31-32)
- [ ] AVX2 register boundary (bytes 63-64)
- [ ] Last byte (byte 95)
- [ ] First byte (byte 0)

### 3. Property Tests (MUST HAVE)

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_simd_matches_portable(
        a in proptest::collection::vec(any::<u8>(), 96),
        b in proptest::collection::vec(any::<u8>(), 96)
    ) {
        let a_arr: [u8; 96] = a.try_into().unwrap();
        let b_arr: [u8; 96] = b.try_into().unwrap();

        let portable = portable::hamming_distance(&a_arr, &b_arr);
        let simd = simd::hamming_distance(&a_arr, &b_arr);

        prop_assert_eq!(portable, simd);
    }
}
```

### 4. Integration Tests (MUST HAVE)

- [ ] `QuantizedVector::hamming_distance` uses SIMD internally
- [ ] Day 36 API unchanged
- [ ] Portable fallback works when SIMD unavailable

---

## DELIVERABLES

| Artifact | Status | Content |
|:---------|:-------|:--------|
| `tests/simd_spec.rs` | [ ] | All test skeletons |
| Test count | [ ] | Minimum 25 tests |
| Property tests | [ ] | ≥10,000 cases configured |

---

## ACCEPTANCE CRITERIA (BINARY)

- [ ] All tests compile (may need stub imports like `use crate::quantization::simd;`)
- [ ] All tests currently FAIL (no implementation exists yet)
- [ ] Test file has ≥25 unit tests + ≥3 property tests
- [ ] No implementation code exists in `src/quantization/simd.rs`
- [ ] Git log shows this file created BEFORE implementation files

**Verification:**
```bash
# Tests compile but fail (correct!)
cargo test simd 2>&1 | grep -E "test result|FAILED"
# Should see: "FAILED" (because no implementation yet)

# No implementation exists yet
test ! -f src/quantization/simd.rs && echo "CORRECT: No impl yet"

# Test count
grep -c "#\[test\]" tests/simd_spec.rs
# Should be ≥ 25
```

---

## HANDOFF

```
TEST_ENGINEER → RUST_ENGINEER

Deliverable: tests/simd_spec.rs with all failing tests
Status: TESTS READY

Next: 04_SIMD_HAMMING_IMPL.md (RUST_ENGINEER makes these tests pass)

CRITICAL: Implementation MUST NOT modify any tests in simd_spec.rs
```

---

## FAILURE PROTOCOL

If test specification cannot be completed:

1. **Categorize blocker:**
   - Type A: Architecture unclear → Request clarification from META_ARCHITECT (30 min)
   - Type B: Test framework issues → Debug proptest/cargo (1 hour max)
   - Type C: Fundamental uncertainty → Escalate to PLANNER

2. **Time limit:** 2 hours total

3. **Escalation trigger:**
   - >2 hours without completion
   - Unable to define test for core requirement
   - Architecture contradictions discovered

4. **Escalation path:** PLANNER reviews, may defer SIMD or extend timeline

---

**END OF TEST SPEC PROMPT**
```

**Deliverable:**
- [ ] `02_SIMD_TEST_SPEC.md` created with above structure
- [ ] References 04_SIMD_HAMMING_IMPL.md (renumbered implementation)
- [ ] Includes test-first guards: "will FAIL initially", "no implementation yet"

---

#### Task 1.3: Create 03_SIMD_BENCHMARK_SPEC.md (NEW) (30 min)

**Purpose:** Define performance targets BEFORE optimization begins

**Extract Content From:**
- `04_SIMD_BENCHMARKS.md` (target specification sections)
- `META_CORRECTION_TEST_FIRST.md` (benchmark-first methodology)

**Template:**

```markdown
# PROMPT: SIMD Benchmark Specification (Benchmark-First)

**Target Agent:** BENCHMARK_SCIENTIST
**Command:** `/bench-spec simd_hamming`
**Priority:** P0 (BLOCKING — Targets must be defined before optimization)
**Estimated Time:** 30 minutes
**Dependencies:**
  - docs/architecture/SIMD_DESIGN.md exists
  - docs/reviews/*SIMD_DESIGN*_APPROVED.md exists
**Output:** `benches/bench_simd.rs` (skeleton with targets), `docs/benchmarks/SIMD_TARGETS.md`

---

## MISSION

Define benchmark targets BEFORE any implementation or optimization begins.

**Benchmark-First Principle:**
> "If you can't measure it, you can't optimize it. Define the target before coding."

**Critical Requirement:** The engineer must hit ALL these targets or the implementation is REJECTED.

---

## PERFORMANCE TARGETS (NON-NEGOTIABLE)

| Metric | Target | Hard Limit | Measurement Method |
|:-------|:-------|:-----------|:-------------------|
| AVX2 Hamming (cycles) | <50 | <75 | rdtsc |
| Speedup vs Portable | >5x | >3x | criterion relative |
| Throughput | >1B ops/sec | >500M ops/sec | criterion |
| Latency P99 | <100ns | <200ns | criterion |

**Rationale for Targets:**

```
<50 Cycle Target Calculation:
├── Load 3 × 32-byte YMM registers: ~9 cycles (3 × 3)
├── XOR 3 registers: ~3 cycles (3 × 1)
├── Popcount (lookup table): ~24 cycles
└── Horizontal sum: ~10 cycles
TOTAL: ~46 cycles (target: <50 with margin)
```

---

## BENCHMARK SKELETON

**File:** `benches/bench_simd.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use edgevec::quantization::QuantizedVector;

/// TARGET: <50 cycles per call
fn bench_simd_hamming_cycles(c: &mut Criterion) {
    let q1 = QuantizedVector::from_bytes([0xAA; 96]);
    let q2 = QuantizedVector::from_bytes([0x55; 96]);

    c.bench_function("simd_hamming_96bytes", |b| {
        b.iter(|| black_box(&q1).hamming_distance(black_box(&q2)))
    });
}

/// TARGET: >5x speedup over portable
fn bench_simd_vs_portable(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_comparison");

    let q1 = QuantizedVector::from_bytes([0xAA; 96]);
    let q2 = QuantizedVector::from_bytes([0x55; 96]);

    group.bench_function("simd", |b| {
        b.iter(|| black_box(&q1).hamming_distance(black_box(&q2)))
    });

    group.bench_function("portable", |b| {
        // Force portable path (implementation will provide this)
        b.iter(|| /* portable call */)
    });

    group.finish();
}

criterion_group!(benches, bench_simd_hamming_cycles, bench_simd_vs_portable);
criterion_main!(benches);
```

---

## CYCLE MEASUREMENT PROTOCOL

**Mandatory Protocol:**

```rust
#[cfg(target_arch = "x86_64")]
fn measure_cycles<F: Fn() -> u32>(f: F, iterations: u64) -> u64 {
    use std::arch::x86_64::_rdtsc;

    // Warmup: 1,000 iterations
    for _ in 0..1000 {
        std::hint::black_box(f());
    }

    // Measure: 10,000 iterations minimum
    let start = unsafe { _rdtsc() };
    for _ in 0..iterations {
        std::hint::black_box(f());
    }
    let end = unsafe { _rdtsc() };

    (end - start) / iterations
}
```

**Anti-Hallucination Guard:**
- REQUIRED: Exact cycle count (not "~50", must be "46" or specific number)
- REQUIRED: Evidence from rdtsc measurement
- FORBIDDEN: Approximations without measurement
- FORBIDDEN: Claims without benchmark proof

---

## DELIVERABLES

| Artifact | Status | Content |
|:---------|:-------|:--------|
| `benches/bench_simd.rs` | [ ] | Skeleton with all targets |
| `docs/benchmarks/SIMD_TARGETS.md` | [ ] | Target specification document |
| Cycle measurement function | [ ] | rdtsc protocol implemented |

---

## ACCEPTANCE CRITERIA (BINARY)

- [ ] All 4 targets documented in SIMD_TARGETS.md
- [ ] Benchmark skeleton compiles
- [ ] Cycle measurement protocol implemented with rdtsc
- [ ] Warmup and iteration counts specified (1k warmup, 10k measurement)
- [ ] black_box usage enforced for all inputs/outputs

**Verification:**
```bash
# Benchmark compiles
cargo bench --bench bench_simd --no-run 2>&1

# All targets documented
grep -E "<50|>5x|>1B|<100ns" docs/benchmarks/SIMD_TARGETS.md | wc -l
# Should be ≥ 4

# Cycle measurement exists
grep -c "rdtsc" benches/bench_simd.rs
# Should be > 0
```

---

## HANDOFF

```
BENCHMARK_SCIENTIST → RUST_ENGINEER

Deliverable: Benchmark skeleton with defined targets
Status: TARGETS READY

Next: 04_SIMD_HAMMING_IMPL.md

RUST_ENGINEER constraint: Implementation MUST hit ALL targets
If ANY target missed → Implementation REJECTED
```

---

## FAILURE PROTOCOL

If benchmark targets cannot be defined:

1. **Categorize blocker:**
   - Type A: Unrealistic targets → Recalculate based on operation count (1 hour)
   - Type B: Measurement methodology unclear → Research rdtsc (30 min)
   - Type C: Architecture doesn't support measurement → Escalate to META_ARCHITECT

2. **Time limit:** 1 hour total

3. **Escalation:** PLANNER if targets fundamentally incompatible with architecture

---

**END OF BENCHMARK SPEC PROMPT**
```

**Deliverable:**
- [ ] `03_SIMD_BENCHMARK_SPEC.md` created
- [ ] References 04_SIMD_HAMMING_IMPL.md (implementation must hit targets)
- [ ] Includes anti-hallucination guards for measurements

---

### CRIT-002: Time Estimate Conflict (5 min)

**Current State:**
- `00_MASTER_DISPATCH.md` says: "Total Estimated Time: 8 hours"
- `PLANNER_DAY2_OPTIMIZATION.md` says: "Total: 11.5 hours + 2.5h buffer = 14 hours"

**Conflict:** 6-hour discrepancy (75% underestimate)

**Your Task:**

Update `00_MASTER_DISPATCH.md` line 117:

```markdown
OLD:
**Total Estimated Time:** 8 hours

NEW:
**Total Estimated Time:** 14 hours (11.5h base + 2.5h buffer)
**Source:** PLANNER_DAY2_OPTIMIZATION.md (realistic 3x rule estimates)
**Note:** Previous 8h estimate did not apply 3x rule to all subtasks
```

**Also Update:** Prompt file table (lines 108-116)

```markdown
| File | Agent | Priority | Est. Time | Dependencies |
|:-----|:------|:---------|:----------|:-------------|
| `01_SIMD_ARCHITECTURE.md` | META_ARCHITECT | P0 | 1h (+3x=3h realistic) | None |
| `02_SIMD_TEST_SPEC.md` | TEST_ENGINEER | P0 | 1.5h (+3x=4.5h realistic) | 01 |
| `03_SIMD_BENCHMARK_SPEC.md` | BENCHMARK_SCIENTIST | P0 | 0.5h (+3x=1.5h realistic) | 01 |
| `04_SIMD_HAMMING_IMPL.md` | RUST_ENGINEER | P0 | 3h (+3x=9h realistic) | 02, 03 |
| `05_SIMD_QUANTIZE_IMPL.md` | RUST_ENGINEER | P2 | 1h (+3x=3h realistic) | 02, 03 |
| `06_SIMD_VALIDATION.md` | TEST_ENGINEER + BENCHMARK_SCIENTIST | P0 | 2h (+3x=6h realistic) | 04, 05 |
| `07_HOSTILE_REVIEW.md` | HOSTILE_REVIEWER | P0 | 1h (+3x=3h realistic) | All |
| `08_NVIDIA_GRADE_HOSTILE_REVIEW.md` | HOSTILE_REVIEWER | P0 | 1h (+3x=3h realistic) | All |

**Total Base:** 11.5 hours
**Total with 3x Rule:** ~33 hours (realistic for zero issues)
**Total Allocated:** 14 hours (assumes some efficiency, includes buffer)
```

**Deliverable:**
- [ ] Time estimates updated in 00_MASTER_DISPATCH.md
- [ ] Reference to PLANNER_DAY2_OPTIMIZATION.md added
- [ ] Table updated with realistic estimates

---

## MAJOR ISSUES TO FIX

### MAJOR-001: Add Binary Acceptance Criteria (1 hour)

**Prompts Needing Fixes:** 01, 03 (and new 02, 03)

#### Fix 01_SIMD_ARCHITECTURE.md

**Current (Subjective):**
```markdown
- [ ] Module structure decided
- [ ] Dispatch strategy chosen with perf analysis
- [ ] Safety model fully documented
```

**Fixed (Binary & Measurable):**
```markdown
## ACCEPTANCE CRITERIA (BINARY)

- [ ] Module structure chosen from Options A/B/C (documented in Section 2 with rationale)
- [ ] Dispatch strategy chosen from Options A/B/C (documented in Section 3 with cycle overhead calculation)
- [ ] Safety model includes ≥4 invariants (listed in Section 5)
- [ ] Safety model includes verification bash commands for each invariant (Section 5)
- [ ] Performance projections calculated from operation counts (not estimated)
- [ ] SIMD_DESIGN.md file created with all 10 sections complete

**Verification:**
```bash
# Check file exists
test -f docs/architecture/SIMD_DESIGN.md || echo "FAIL: File missing"

# Check all sections present
grep -c "^## " docs/architecture/SIMD_DESIGN.md
# Should be ≥ 10

# Check safety invariants count
grep -c "^[0-9]\+\." docs/architecture/SIMD_DESIGN.md | awk '$1 >= 4'
# Should be ≥ 4
```
```

**Deliverable:**
- [ ] `01_SIMD_ARCHITECTURE.md` updated with binary acceptance criteria

---

#### Fix 05_SIMD_QUANTIZE_IMPL.md (renumbered from 03)

**Current:** Missing acceptance criteria entirely

**Add Section:**
```markdown
## ACCEPTANCE CRITERIA (BINARY)

### If Implemented (Decision Gate Passed):

- [ ] `src/quantization/simd.rs` includes `quantize_simd()` function
- [ ] Property test: SIMD quantize == portable quantize for 10,000 cases
- [ ] Benchmark shows quantize speedup ≥2x (measured via criterion)
- [ ] No regression on Hamming SIMD performance (<50 cycles still met)
- [ ] All Day 36 tests still pass (0 failures)

**Verification:**
```bash
# Function exists
grep -q "fn quantize" src/quantization/simd.rs || echo "FAIL"

# Property test exists
grep -q "prop_quantize_simd_matches_portable" tests/ || echo "FAIL"

# Benchmark measured
cargo bench --bench bench_simd 2>&1 | grep "quantize"
```

### If Skipped (Decision Gate Failed):

- [ ] Document reason in `docs/planning/weeks/week8/W8D37_SIMD_DEFERRED.md`
- [ ] Time remaining <1 hour documented
- [ ] Hamming SIMD complete and validated

**Verification:**
```bash
# If skipped, deferred doc exists
test -f docs/planning/weeks/week8/W8D37_SIMD_DEFERRED.md || echo "FAIL"
```
```

**Deliverable:**
- [ ] `05_SIMD_QUANTIZE_IMPL.md` updated with binary acceptance criteria

---

### MAJOR-002: Add Failure Protocols (1 hour)

**Prompts Needing Protocols:** 01, 04 (renumbered), 02 (new), 03 (new)

**Standard Template to Add:**

```markdown
## FAILURE PROTOCOL

### Detection

If [acceptance criterion X] fails:
- Symptom: [How to detect failure]
- Evidence: [Command output or log]

### Categorization

1. **Type A: [Category]** (Example: Logic error)
   - Action: [Immediate response]
   - Time limit: [X hours]
   - Tools: [Debugging commands]

2. **Type B: [Category]** (Example: Architecture issue)
   - Action: [Immediate response]
   - Time limit: [X hours]
   - Escalation: [Agent name]

3. **Type C: [Category]** (Example: Fundamental impossibility)
   - Action: Escalate immediately
   - Escalation: PLANNER
   - Options: [A. Defer, B. Simplify, C. Cancel]

### Escalation Triggers

Escalate to PLANNER if:
- [ ] >X hours without resolution
- [ ] >3 failed fix attempts
- [ ] Fundamental architectural issue detected
- [ ] Conflicting requirements discovered

### Alternative Paths

If unfixable:
- Option A: [Defer to Week 9]
- Option B: [Simplify requirement]
- Option C: [Cancel feature]

Document decision in: `docs/planning/weeks/week8/W8D37_ESCALATION.md`
```

**Apply to Each Prompt:**

#### 01_SIMD_ARCHITECTURE.md:
```markdown
## FAILURE PROTOCOL

If module structure/dispatch strategy cannot be decided:

1. **Type A: All options seem equally bad**
   - Action: Create comparison matrix with pros/cons
   - Time limit: 30 min
   - Decision: Choose option with fewest cons

2. **Type B: Performance projections show <50 cycles impossible**
   - Action: Recalculate with actual operation counts
   - Time limit: 1 hour
   - Escalation: PLANNER if still impossible

3. **Type C: Safety model has unresolvable unsafe**
   - Action: Escalate to PLANNER immediately
   - Options:
     - A. Use only safe portable implementation
     - B. Defer SIMD to Week 9
     - C. Accept documented unsafe with extra review

Escalate if: >2 hours without complete SIMD_DESIGN.md
```

#### 02_SIMD_TEST_SPEC.md (new):
```markdown
## FAILURE PROTOCOL

If test specification cannot be completed:

1. **Type A: Architecture unclear**
   - Action: Request clarification from META_ARCHITECT
   - Time limit: 30 min
   - Block: Cannot proceed without architecture

2. **Type B: Test framework issues (proptest, cargo)**
   - Action: Debug test setup
   - Time limit: 1 hour
   - Fallback: Use manual test cases if proptest unfixable

3. **Type C: Fundamental uncertainty about requirements**
   - Action: Escalate to PLANNER
   - Issue: Cannot write test without understanding requirement

Escalate if: >2 hours without 25 test specifications
```

#### 03_SIMD_BENCHMARK_SPEC.md (new):
```markdown
## FAILURE PROTOCOL

If benchmark targets cannot be defined:

1. **Type A: Targets seem unrealistic**
   - Action: Recalculate from operation count
   - Time limit: 1 hour
   - Formula: (loads + ops + stores) × cycles_per_op

2. **Type B: Measurement methodology unclear**
   - Action: Research rdtsc best practices
   - Time limit: 30 min
   - Reference: Intel optimization manual

3. **Type C: Architecture doesn't support cycle measurement**
   - Action: Escalate to META_ARCHITECT
   - Options:
     - A. Use criterion time estimates only
     - B. Defer cycle count requirement

Escalate if: >1 hour without all 4 targets defined
```

#### 04_SIMD_HAMMING_IMPL.md (renumbered from 02):
```markdown
## FAILURE PROTOCOL

If implementation fails to meet targets:

1. **Type A: Tests fail (correctness issue)**
   - Action: Debug SIMD logic
   - Time limit: 2 hours
   - Tools: `cargo test`, `lldb`, print debugging

2. **Type B: Performance <50 cycles not achieved**
   - Action: Profile with `perf`, optimize hot path
   - Time limit: 2 hours
   - Fallback: Document achieved cycle count if >50 but <75

3. **Type C: Miri detects UB**
   - Action: Review all unsafe blocks
   - Time limit: 2 hours
   - Escalation: PLANNER if UB unfixable

Escalate if:
- >4 hours total debugging
- Fundamental SIMD approach flawed
- Safety issues unresolvable
```

**Deliverables:**
- [ ] Failure protocol added to 01_SIMD_ARCHITECTURE.md
- [ ] Failure protocol added to 02_SIMD_TEST_SPEC.md (new)
- [ ] Failure protocol added to 03_SIMD_BENCHMARK_SPEC.md (new)
- [ ] Failure protocol added to 04_SIMD_HAMMING_IMPL.md

---

### MAJOR-003: Make Dependencies Specific (30 min)

**Fix Pattern:**

```markdown
BAD (Vague):
Dependencies: 01

GOOD (Specific):
Dependencies:
  - docs/architecture/SIMD_DESIGN.md exists
  - docs/reviews/*SIMD_DESIGN*_APPROVED.md exists

Verification:
```bash
# Check architecture exists
test -f docs/architecture/SIMD_DESIGN.md || echo "BLOCK: No architecture"

# Check architecture approved
grep -q "APPROVED" docs/reviews/*SIMD_DESIGN*.md || echo "BLOCK: Not approved"
```
```

**Apply to:**
- 00_MASTER_DISPATCH.md table
- 04_SIMD_HAMMING_IMPL.md (renumbered from 02)
- 05_SIMD_QUANTIZE_IMPL.md (renumbered from 03)
- All new prompts (02, 03)

**Deliverables:**
- [ ] All dependencies converted to file paths with verification commands

---

## FINAL FILE STRUCTURE

After all fixes, the directory should look like:

```
docs/planning/weeks/week8/prompts/day_2/
├── 00_MASTER_DISPATCH.md (UPDATED - time estimates fixed)
├── 01_SIMD_ARCHITECTURE.md (UPDATED - binary criteria + failure protocol)
├── 02_SIMD_TEST_SPEC.md (NEW - test specifications)
├── 03_SIMD_BENCHMARK_SPEC.md (NEW - performance targets)
├── 04_SIMD_HAMMING_IMPL.md (RENAMED from 02 + updated deps)
├── 05_SIMD_QUANTIZE_IMPL.md (RENAMED from 03 + binary criteria)
├── 06_SIMD_VALIDATION.md (NEW - execute tests + benchmarks)
├── 07_HOSTILE_REVIEW.md (RENAMED from 06)
├── 08_NVIDIA_GRADE_HOSTILE_REVIEW.md (RENAMED from 07)
├── META_CORRECTION_TEST_FIRST.md (unchanged)
├── PLANNER_DAY2_OPTIMIZATION.md (unchanged)
└── PROMPT_MAKER_META_REVIEW.md (unchanged)
```

---

## DELIVERABLES CHECKLIST

### Critical Fixes (Must Complete):
- [ ] Files renumbered for test-first order (02→04, 03→05, etc.)
- [ ] `02_SIMD_TEST_SPEC.md` created (test specifications)
- [ ] `03_SIMD_BENCHMARK_SPEC.md` created (performance targets)
- [ ] `00_MASTER_DISPATCH.md` time estimate updated (8h → 14h)

### Major Fixes (Must Complete):
- [ ] Binary acceptance criteria added to 01_SIMD_ARCHITECTURE.md
- [ ] Binary acceptance criteria added to 05_SIMD_QUANTIZE_IMPL.md
- [ ] Failure protocols added to 01, 02, 03, 04
- [ ] Dependencies made specific with file paths (all prompts)

### Optional Enhancements:
- [ ] Create `06_SIMD_VALIDATION.md` (combined test execution + benchmark validation)
- [ ] Update cross-references in all files
- [ ] Add verification script `scripts/verify_day2_prompts.sh`

---

## ACCEPTANCE CRITERIA (FOR THIS FIX TASK)

**Critical (Must Pass for Approval):**
- [ ] All files renumbered: `ls day_2/ | grep -E "^0[2-8]"` shows correct sequence
- [ ] Test-first order verified: 02=TEST_SPEC, 03=BENCHMARK_SPEC, 04=IMPL
- [ ] New prompts exist: `test -f 02_SIMD_TEST_SPEC.md && test -f 03_SIMD_BENCHMARK_SPEC.md`
- [ ] Time conflict resolved: `grep "14 hours" 00_MASTER_DISPATCH.md`

**Major (Must Pass for Approval):**
- [ ] Binary criteria count: `grep -r "^\- \[ \]" 01_SIMD_ARCHITECTURE.md | wc -l` ≥ 6
- [ ] Failure protocol count: `grep -r "## FAILURE PROTOCOL" day_2/*.md | wc -l` ≥ 4
- [ ] Specific dependencies: `grep -r "docs/architecture/" day_2/*.md | wc -l` > 0

**Verification Commands:**
```bash
cd docs/planning/weeks/week8/prompts/day_2/

# Check test-first order
echo "Checking file order..."
ls -1 | grep -E "^0[0-9]" | head -6
# Expected: 00, 01, 02_TEST, 03_BENCHMARK, 04_IMPL, 05_IMPL

# Check new files created
echo "Checking new files..."
test -f 02_SIMD_TEST_SPEC.md && echo "✅ TEST_SPEC created"
test -f 03_SIMD_BENCHMARK_SPEC.md && echo "✅ BENCHMARK_SPEC created"

# Check time estimate fixed
echo "Checking time estimate..."
grep -q "14 hours" 00_MASTER_DISPATCH.md && echo "✅ Time fixed"

# Check acceptance criteria added
echo "Checking binary criteria..."
grep -c "## ACCEPTANCE CRITERIA" 01_SIMD_ARCHITECTURE.md
# Should be 1

# Check failure protocols added
echo "Checking failure protocols..."
grep -c "## FAILURE PROTOCOL" *.md
# Should be ≥ 4

echo "✅ All critical fixes verified"
```

---

## TIME ALLOCATION

| Task | Time | Running Total |
|:-----|:-----|:--------------|
| 1.1 Renumber files | 15 min | 0:15 |
| 1.2 Create 02_TEST_SPEC | 30 min | 0:45 |
| 1.3 Create 03_BENCHMARK_SPEC | 30 min | 1:15 |
| CRIT-002 Fix time estimates | 5 min | 1:20 |
| MAJOR-001 Binary criteria (01) | 15 min | 1:35 |
| MAJOR-001 Binary criteria (05) | 15 min | 1:50 |
| MAJOR-002 Failure protocols (×4) | 40 min | 2:30 |
| MAJOR-003 Specific dependencies | 15 min | 2:45 |
| Verification | 10 min | 2:55 |
| Buffer | 5 min | 3:00 |

**Total:** 3 hours (includes buffer)

---

## HANDOFF

```
PLANNER → HOSTILE_REVIEWER

Deliverables:
  - Fixed Day 2 prompt suite (8 execution prompts)
  - Test-first ordering enforced
  - Binary acceptance criteria added
  - Failure protocols added
  - Time estimates realistic

Status: FIXES COMPLETE

Next: Re-run meta-review
Command: /prompt-review day2_suite_v2

Expected Result: Score ≥9.0/10.0, APPROVED for execution
```

---

## FAILURE PROTOCOL (FOR THIS FIX TASK)

If fixes cannot be completed within 3 hours:

1. **Blocker Type A: Technical issues (file renaming, git conflicts)**
   - Action: Resolve conflicts, use git mv for renames
   - Time limit: 30 min

2. **Blocker Type B: Content creation takes longer than estimated**
   - Action: Prioritize critical fixes (CRIT-001, CRIT-002)
   - Defer: MAJOR issues to subsequent fix iteration

3. **Blocker Type C: Fundamental prompt design issues discovered**
   - Action: Escalate to PROMPT_MAKER
   - Document: Issues requiring architectural rework

**Escalation:** If >4 hours total, escalate to senior PLANNER for schedule impact assessment

---

**END OF PLANNER FIX PROMPT**
