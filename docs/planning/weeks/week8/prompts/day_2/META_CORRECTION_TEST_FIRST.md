# PROMPT_MAKER: Day 2 Meta-Correction — Test-First Enforcement

**Agent:** PROMPT_MAKER (Quality Control Mode)
**Priority:** P0 (CRITICAL — Workflow Correctness)
**Date:** 2025-12-12
**Purpose:** Correct Day 2 workflow to enforce TEST-FIRST methodology

---

## CRITICAL WORKFLOW VIOLATION DETECTED

The current Day 2 prompt sequence has a **fundamental ordering error**:

```
CURRENT (WRONG):
01_ARCHITECTURE → 02_IMPL → 03_IMPL → 04_BENCHMARKS → 05_TESTS → 06_REVIEW
                   ↑                                      ↑
                   CODE FIRST                             TESTS AFTER
```

This violates the EdgeVec Test-First Mandate. **Tests must be written BEFORE implementation.**

---

## CORRECTED WORKFLOW SEQUENCE

```
CORRECT (TEST-FIRST):
01_ARCHITECTURE → 02_TESTS_SPEC → 03_BENCHMARKS_SPEC → 04_IMPL → 05_VALIDATION → 06_REVIEW
                   ↑               ↑                    ↑         ↑
                   TESTS FIRST     BENCHMARKS FIRST     CODE      VERIFY ALL PASS
```

---

## CORRECTED PROMPT EXECUTION ORDER

### Phase A: Specification (No Code)

| Step | Prompt | Agent | Output | Blocking |
|:-----|:-------|:------|:-------|:---------|
| A.1 | `01_SIMD_ARCHITECTURE.md` | META_ARCHITECT | `SIMD_DESIGN.md` | YES |
| A.2 | **NEW: `02_SIMD_TEST_SPEC.md`** | TEST_ENGINEER | Test specifications | YES |
| A.3 | **NEW: `03_SIMD_BENCH_SPEC.md`** | BENCHMARK_SCIENTIST | Benchmark targets | YES |

### Phase B: Implementation (Code Allowed)

| Step | Prompt | Agent | Output | Blocking |
|:-----|:-------|:------|:-------|:---------|
| B.1 | `04_SIMD_HAMMING_IMPL.md` | RUST_ENGINEER | `simd.rs` | YES |
| B.2 | `05_SIMD_QUANTIZE_IMPL.md` | RUST_ENGINEER | Enhanced quantize | NO |

### Phase C: Validation (Prove Correctness)

| Step | Prompt | Agent | Output | Blocking |
|:-----|:-------|:------|:-------|:---------|
| C.1 | `06_SIMD_VALIDATION.md` | TEST_ENGINEER | All tests passing | YES |
| C.2 | `07_SIMD_BENCHMARKS.md` | BENCHMARK_SCIENTIST | `W8D37_simd_report.md` | YES |
| C.3 | `08_HOSTILE_REVIEW.md` | HOSTILE_REVIEWER | Final verdict | YES |

---

## NEW PROMPT: 02_SIMD_TEST_SPEC.md

**This prompt MUST be executed BEFORE any implementation.**

```markdown
# PROMPT: SIMD Test Specification (Test-First)

**Target Agent:** TEST_ENGINEER
**Command:** `/test-spec simd_hamming`
**Priority:** P0 (BLOCKING — Must complete before implementation)
**Estimated Time:** 1 hour
**Dependencies:** `01_SIMD_ARCHITECTURE.md` COMPLETE
**Output:** `tests/simd_spec.rs` (skeleton with assertions)

---

## MISSION

Write comprehensive test specifications BEFORE any SIMD code is written.
The tests will initially FAIL (no implementation exists). This is correct.

**Test-First Principle:**
> "If you can't write the test, you don't understand the requirement."

---

## TEST CATEGORIES TO SPECIFY

### 1. Correctness Tests (MUST HAVE)

```rust
/// Test skeleton - implementation will make this pass
#[test]
fn test_simd_matches_portable_zeros() {
    let a = [0x00u8; 96];
    let b = [0x00u8; 96];

    // SPECIFICATION: SIMD must return identical result to portable
    let portable_result = portable::hamming_distance(&a, &b);
    let simd_result = simd::hamming_distance(&a, &b);

    assert_eq!(portable_result, simd_result);
    assert_eq!(simd_result, 0); // Expected: 0 differing bits
}

#[test]
fn test_simd_matches_portable_ones() {
    let a = [0xFFu8; 96];
    let b = [0x00u8; 96];

    let portable_result = portable::hamming_distance(&a, &b);
    let simd_result = simd::hamming_distance(&a, &b);

    assert_eq!(portable_result, simd_result);
    assert_eq!(simd_result, 768); // Expected: all 768 bits differ
}

#[test]
fn test_simd_symmetry() {
    let a = [0xABu8; 96];
    let b = [0xCDu8; 96];

    // SPECIFICATION: distance(a, b) == distance(b, a)
    assert_eq!(
        simd::hamming_distance(&a, &b),
        simd::hamming_distance(&b, &a)
    );
}

#[test]
fn test_simd_self_distance() {
    let a = [0x42u8; 96];

    // SPECIFICATION: distance(a, a) == 0
    assert_eq!(simd::hamming_distance(&a, &a), 0);
}
```

### 2. Boundary Tests (MUST HAVE)

```rust
#[test]
fn test_simd_avx2_register_boundary_32() {
    let mut a = [0x00u8; 96];
    let b = [0x00u8; 96];

    // Bytes 31-32 cross first YMM register boundary
    a[31] = 0xFF;
    a[32] = 0xFF;

    // SPECIFICATION: Must correctly handle AVX2 register boundaries
    assert_eq!(simd::hamming_distance(&a, &b), 16);
}

#[test]
fn test_simd_avx2_register_boundary_64() {
    let mut a = [0x00u8; 96];
    let b = [0x00u8; 96];

    // Bytes 63-64 cross second YMM register boundary
    a[63] = 0xFF;
    a[64] = 0xFF;

    assert_eq!(simd::hamming_distance(&a, &b), 16);
}

#[test]
fn test_simd_last_byte() {
    let mut a = [0x00u8; 96];
    let b = [0x00u8; 96];

    // Last byte of 96-byte array
    a[95] = 0xFF;

    // SPECIFICATION: Must correctly process final partial register
    assert_eq!(simd::hamming_distance(&a, &b), 8);
}
```

### 3. Property Tests (MUST HAVE)

```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// SPECIFICATION: SIMD must EXACTLY match portable for ALL inputs
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

    /// SPECIFICATION: Hamming distance is symmetric
    #[test]
    fn prop_simd_symmetric(
        a in proptest::collection::vec(any::<u8>(), 96),
        b in proptest::collection::vec(any::<u8>(), 96)
    ) {
        let a_arr: [u8; 96] = a.try_into().unwrap();
        let b_arr: [u8; 96] = b.try_into().unwrap();

        prop_assert_eq!(
            simd::hamming_distance(&a_arr, &b_arr),
            simd::hamming_distance(&b_arr, &a_arr)
        );
    }

    /// SPECIFICATION: Hamming distance satisfies triangle inequality
    #[test]
    fn prop_simd_triangle_inequality(
        a in proptest::collection::vec(any::<u8>(), 96),
        b in proptest::collection::vec(any::<u8>(), 96),
        c in proptest::collection::vec(any::<u8>(), 96)
    ) {
        let a_arr: [u8; 96] = a.try_into().unwrap();
        let b_arr: [u8; 96] = b.try_into().unwrap();
        let c_arr: [u8; 96] = c.try_into().unwrap();

        let ab = simd::hamming_distance(&a_arr, &b_arr);
        let bc = simd::hamming_distance(&b_arr, &c_arr);
        let ac = simd::hamming_distance(&a_arr, &c_arr);

        // d(a,c) <= d(a,b) + d(b,c)
        prop_assert!(ac <= ab + bc);
    }
}
```

### 4. Integration Tests (MUST HAVE)

```rust
#[test]
fn test_quantized_vector_uses_simd() {
    let quantizer = BinaryQuantizer::new();
    let v1: Vec<f32> = (0..768).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
    let v2: Vec<f32> = (0..768).map(|i| if i % 2 == 0 { -1.0 } else { 1.0 }).collect();

    let q1 = quantizer.quantize(&v1);
    let q2 = quantizer.quantize(&v2);

    // SPECIFICATION: Public API must use SIMD internally when available
    let distance = q1.hamming_distance(&q2);
    assert_eq!(distance, 768);
}

#[test]
fn test_day36_api_unchanged() {
    // SPECIFICATION: Day 36 API must remain unchanged
    let q1 = QuantizedVector::from_bytes([0xAA; 96]);
    let q2 = QuantizedVector::from_bytes([0x55; 96]);

    assert_eq!(q1.hamming_distance(&q2), 768);
    assert!((q1.similarity(&q2) - 0.0).abs() < f32::EPSILON);
}
```

---

## DELIVERABLES

| Artifact | Status | Content |
|:---------|:-------|:--------|
| `tests/simd_spec.rs` | [ ] | All test skeletons |
| Test count | [ ] | Minimum 25 tests |
| Property tests | [ ] | Minimum 10,000 cases |

---

## ACCEPTANCE CRITERIA

| Criterion | Requirement |
|:----------|:------------|
| All tests compile | YES (with stub imports) |
| All tests FAIL | YES (no implementation yet) |
| No implementation code | YES |
| Coverage of all edge cases | YES |

---

## HANDOFF

```
TEST_ENGINEER → RUST_ENGINEER

Deliverable: tests/simd_spec.rs with all failing tests
Status: READY FOR IMPLEMENTATION

Next: RUST_ENGINEER implements until ALL tests pass
```
```

---

## NEW PROMPT: 03_SIMD_BENCH_SPEC.md

**This prompt MUST be executed BEFORE any implementation.**

```markdown
# PROMPT: SIMD Benchmark Specification (Benchmark-First)

**Target Agent:** BENCHMARK_SCIENTIST
**Command:** `/bench-spec simd_hamming`
**Priority:** P0 (BLOCKING — Must complete before implementation)
**Estimated Time:** 30 minutes
**Dependencies:** `01_SIMD_ARCHITECTURE.md` COMPLETE
**Output:** `benches/bench_simd.rs` (skeleton with targets)

---

## MISSION

Define benchmark targets BEFORE implementation. The engineer must hit these targets
or the implementation is rejected.

**Benchmark-First Principle:**
> "If you can't measure it, you can't optimize it. Define the target before coding."

---

## PERFORMANCE TARGETS

| Metric | Target | Hard Limit | Measurement |
|:-------|:-------|:-----------|:------------|
| AVX2 Hamming (cycles) | <50 | <75 | rdtsc |
| AVX2 Hamming (ns) | <15ns | <25ns | criterion |
| Speedup vs Portable | >5x | >3x | relative |
| Throughput | >1B ops/sec | >500M ops/sec | criterion |

---

## BENCHMARK SKELETON

```rust
// benches/bench_simd.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use edgevec::quantization::{QuantizedVector, BinaryQuantizer};

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
        b.iter(|| {
            // Force portable path
            portable_hamming_distance(black_box(q1.data()), black_box(q2.data()))
        })
    });

    group.finish();
}

/// TARGET: >1B operations/second
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_throughput");
    group.throughput(Throughput::Elements(1));

    let q1 = QuantizedVector::from_bytes([0xAA; 96]);
    let q2 = QuantizedVector::from_bytes([0x55; 96]);

    group.bench_function("hamming_ops_per_sec", |b| {
        b.iter(|| black_box(&q1).hamming_distance(black_box(&q2)))
    });

    group.finish();
}

criterion_group!(benches,
    bench_simd_hamming_cycles,
    bench_simd_vs_portable,
    bench_throughput
);
criterion_main!(benches);
```

---

## CYCLE MEASUREMENT PROTOCOL

```rust
/// Measure exact CPU cycles using rdtsc
#[cfg(target_arch = "x86_64")]
fn measure_cycles<F: Fn() -> u32>(f: F, iterations: u64) -> u64 {
    use std::arch::x86_64::_rdtsc;

    // Warmup
    for _ in 0..1000 {
        std::hint::black_box(f());
    }

    // Measure
    let start = unsafe { _rdtsc() };
    for _ in 0..iterations {
        std::hint::black_box(f());
    }
    let end = unsafe { _rdtsc() };

    (end - start) / iterations
}
```

---

## DELIVERABLES

| Artifact | Status |
|:---------|:-------|
| `benches/bench_simd.rs` | [ ] |
| Performance targets documented | [ ] |
| Cycle measurement function | [ ] |

---

## HANDOFF

```
BENCHMARK_SCIENTIST → RUST_ENGINEER

Deliverable: Benchmark skeleton with defined targets
Status: READY FOR IMPLEMENTATION

RUST_ENGINEER must make implementation hit ALL targets.
If targets not hit → Implementation REJECTED.
```
```

---

## CORRECTED HOSTILE REVIEW PROMPT

The existing `06_HOSTILE_REVIEW.md` must be enhanced to NVIDIA-grade standards:

### Additional Review Dimensions

```markdown
### 7. TEST-FIRST COMPLIANCE (Weight: 15%) — MANDATORY

**Verification:**
- [ ] Tests were written BEFORE implementation (check git log)
- [ ] Test file creation timestamp < implementation file timestamp
- [ ] All test cases from spec are implemented
- [ ] No tests were modified after implementation to pass

**Evidence Required:**
```bash
# Git log must show tests committed before implementation
git log --oneline --name-only -- tests/simd_spec.rs src/quantization/simd.rs
```

**Scoring:**
- 10/10: Tests clearly preceded implementation
- 5/10: Tests and implementation in same commit (suspicious)
- 0/10: Implementation before tests → **AUTOMATIC REJECTION**

---

### 8. BENCHMARK TARGET COMPLIANCE (Weight: 10%) — MANDATORY

**Verification:**
- [ ] All targets from 03_SIMD_BENCH_SPEC.md met
- [ ] Cycle count <50 (measured, not estimated)
- [ ] Speedup >5x (measured)
- [ ] No benchmark manipulation (synthetic favorable inputs)

**Scoring:**
- 10/10: All targets met
- 0/10: Any target missed → **AUTOMATIC REJECTION**

---

### 9. ANTI-HALLUCINATION AUDIT (Weight: 10%) — MANDATORY

**Check for hallucinated claims:**
- [ ] All performance numbers have measurement evidence
- [ ] All "optimizations" have benchmark proof
- [ ] No claims without code to back them up
- [ ] No placeholder implementations marked as "complete"

**Red Flags:**
- "Should be faster" without benchmark
- "Approximately X cycles" without rdtsc measurement
- "Works on all platforms" without platform tests
- "SIMD automatically used" without dispatch verification

**Scoring:**
- 10/10: All claims verified with evidence
- 0/10: Any hallucinated claim → **AUTOMATIC REJECTION**
```

---

## EXECUTION SEQUENCE ENFORCEMENT

```
┌──────────────────────────────────────────────────────────────────────┐
│                     DAY 2 CORRECT EXECUTION ORDER                    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  PHASE A: SPECIFICATION (NO CODE)                                   │
│  ├── A.1: /architect-design simd_hamming                            │
│  │        Output: docs/architecture/SIMD_DESIGN.md                  │
│  │        Gate: HOSTILE_REVIEWER approval                           │
│  │                                                                  │
│  ├── A.2: /test-spec simd_hamming                                   │
│  │        Output: tests/simd_spec.rs (ALL TESTS FAIL)               │
│  │        Gate: Tests compile, all fail (correct!)                  │
│  │                                                                  │
│  └── A.3: /bench-spec simd_hamming                                  │
│           Output: benches/bench_simd.rs (targets defined)           │
│           Gate: Targets documented                                  │
│                                                                      │
│  ─────────────────── PHASE A GATE ───────────────────               │
│  Checkpoint: Architecture + Tests + Benchmarks ALL ready            │
│  NO IMPLEMENTATION CODE EXISTS YET                                  │
│  ─────────────────────────────────────────────────────               │
│                                                                      │
│  PHASE B: IMPLEMENTATION                                            │
│  ├── B.1: /rust-implement W8.37.1 (Hamming SIMD)                    │
│  │        Output: src/quantization/simd.rs                          │
│  │        Constraint: ALL tests from A.2 must pass                  │
│  │                                                                  │
│  └── B.2: /rust-implement W8.37.2 (Quantize SIMD) [OPTIONAL]        │
│           Output: Enhanced quantize in simd.rs                      │
│           Constraint: No regression on existing tests               │
│                                                                      │
│  ─────────────────── PHASE B GATE ───────────────────               │
│  Checkpoint: ALL tests pass (cargo test)                            │
│  ─────────────────────────────────────────────────────               │
│                                                                      │
│  PHASE C: VALIDATION                                                │
│  ├── C.1: /bench-validate simd_hamming                              │
│  │        Output: docs/benchmarks/W8D37_simd_report.md              │
│  │        Constraint: ALL targets from A.3 must be met              │
│  │                                                                  │
│  └── C.2: /review W8D37_simd                                        │
│           Output: docs/reviews/2025-12-12_W8D37_simd_*.md           │
│           Gate: HOSTILE_REVIEWER final verdict                      │
│                                                                      │
│  ─────────────────── DAY 2 COMPLETE ───────────────────             │
│  All gates passed → Proceed to Day 3                                │
│  Any gate failed → STOP, fix issues, re-review                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## VIOLATION DETECTION QUERIES

### Check Test-First Compliance

```bash
# List files by creation order
git log --diff-filter=A --name-only --pretty=format: -- tests/ src/quantization/simd.rs | tac

# Expected output:
# tests/simd_spec.rs      <- MUST appear BEFORE
# src/quantization/simd.rs <- implementation
```

### Check for Hallucinations

```bash
# Find performance claims without benchmarks
grep -r "cycles\|faster\|speedup\|optimized" src/ --include="*.rs" | \
  grep -v "// Benchmark:" # All claims must reference benchmarks
```

### Check All Tests Pass

```bash
cargo test 2>&1 | tail -20
# Expected: "test result: ok. XX passed; 0 failed"
```

---

## SUMMARY

This meta-correction prompt establishes the **correct Test-First workflow** for Day 2:

1. **Tests BEFORE Code** — Non-negotiable
2. **Benchmarks BEFORE Code** — Targets must be defined upfront
3. **Implementation GUIDED BY Tests** — Engineer makes tests pass
4. **Validation PROVES Targets** — Benchmarks verify performance
5. **Hostile Review AUDITS Everything** — Including Test-First compliance

**The engineer's job is to make pre-written tests pass, not to write code and hope it works.**

---

**END OF META-CORRECTION PROMPT**
