# PROMPT: SIMD Test Specification (Test-First)

**Target Agent:** TEST_ENGINEER
**Command:** `/test-spec simd_hamming`
**Priority:** P0 (BLOCKING — Must complete before implementation)
**Estimated Time:** 1.5 hours (realistic: 4.5h with 3x rule)
**Dependencies:**
  - `docs/architecture/SIMD_DESIGN.md` exists
  - `docs/reviews/*SIMD_DESIGN*_APPROVED.md` exists
**Output:** `tests/simd_spec.rs` (test specifications, ALL FAILING initially)

---

## MISSION

Write comprehensive test specifications BEFORE any SIMD implementation code exists.

**Test-First Principle:**
> "If you can't write the test, you don't understand the requirement."

**CRITICAL REQUIREMENT:** These tests will FAIL initially because no implementation exists yet. **THIS IS CORRECT AND EXPECTED.**

The RUST_ENGINEER will later implement `src/quantization/simd.rs` to make ALL these tests pass.

---

## DEPENDENCY VERIFICATION

Before proceeding, verify architecture is approved:

```bash
# Check architecture file exists
test -f docs/architecture/SIMD_DESIGN.md || echo "BLOCK: No SIMD architecture"

# Check architecture has been reviewed and approved
ls docs/reviews/*SIMD_DESIGN*.md 2>/dev/null || echo "BLOCK: Architecture not reviewed"
grep -q "APPROVED" docs/reviews/*SIMD_DESIGN*.md || echo "BLOCK: Architecture not approved"
```

---

## CROSS-PLATFORM TEST MATRIX

### Required Test Coverage

All test specifications MUST be verified on multiple platforms to ensure correctness and portability.

| Platform | Instruction Set | Test Status | CI Job | Notes |
|:---------|:----------------|:------------|:-------|:------|
| x86_64 (Linux) | AVX2 | ✅ Required | `test-x86-avx2` | Primary target platform |
| x86_64 (Linux) | Portable (no AVX2) | ✅ Required | `test-x86-noavx2` | Fallback verification |
| x86_64 (macOS) | AVX2 | ✅ Required | `test-macos-x86` | Developer machines |
| ARM64 (Linux) | NEON | ✅ Required | `test-arm-neon` | AWS Graviton, embedded |
| ARM64 (macOS) | NEON | ✅ Required | `test-macos-arm` | Apple Silicon (M1/M2) |
| WASM | WASM SIMD | ⚠️ Optional | `test-wasm-simd` | Browser compatibility |

### Test Execution Protocol

**For Each Platform:**
1. Run full test suite: `cargo test --target <target-triple> simd_spec`
2. Run property tests: `cargo test --target <target-triple> prop_`
3. Verify SIMD dispatch: Check CPU feature detection logs
4. Compare results: All platforms must produce identical results for same inputs

**Platform-Specific Commands:**

```bash
# x86_64 Linux with AVX2 (default)
cargo test simd_spec

# x86_64 Linux portable (disable AVX2)
RUSTFLAGS="-C target-feature=-avx2" cargo test simd_spec

# x86_64 macOS with AVX2
cargo test simd_spec

# ARM64 Linux NEON (requires cross or ARM hardware)
cross test --target aarch64-unknown-linux-gnu simd_spec
# OR on ARM hardware:
cargo test simd_spec

# ARM64 macOS NEON (on Apple Silicon)
cargo test simd_spec

# WASM (requires wasm-pack)
wasm-pack test --node
```

### Acceptance Criteria for Cross-Platform Testing

- [ ] All "Required" platforms show passing tests
- [ ] Property tests pass on all platforms (10,000 cases each)
- [ ] SIMD vs portable correctness verified on all platforms
- [ ] Performance targets met on primary platforms (x86_64 AVX2, ARM64 NEON)
- [ ] Portable fallback works when SIMD unavailable

### Platform-Specific Notes

**x86_64 Portable (no AVX2):**
- Uses std::simd fallback or scalar implementation
- Performance target relaxed: <200 cycles (vs <50 for AVX2)
- MUST still pass all correctness tests

**ARM64 NEON:**
- Different instruction latencies than AVX2
- Target: <60 cycles (vs <50 for AVX2)
- Verify on actual ARM hardware, not QEMU (emulation skews timing)

**WASM SIMD:**
- May not support rdtsc cycle measurement
- Use time-based benchmarks only
- Test in actual browser environment (not just wasmtime)
- Performance targets relaxed due to browser overhead

**macOS (both x86_64 and ARM64):**
- Apple's CPU governor may affect benchmarks
- Run benchmarks multiple times to account for thermal throttling
- Verify both Debug and Release builds

### Test Matrix Verification

Before marking test spec complete, verify:

```bash
# Check all required platforms tested
# (This will be filled during actual testing phase)

# Placeholder for platform test results:
# - [ ] x86_64 Linux AVX2: cargo test simd_spec (result: PASS/FAIL)
# - [ ] x86_64 Linux portable: RUSTFLAGS="-C target-feature=-avx2" cargo test simd_spec (result: PASS/FAIL)
# - [ ] ARM64 Linux NEON: cross test --target aarch64-unknown-linux-gnu simd_spec (result: PASS/FAIL)
# - [ ] x86_64 macOS AVX2: cargo test simd_spec (result: PASS/FAIL)
# - [ ] ARM64 macOS NEON: cargo test simd_spec (result: PASS/FAIL)
```

---

## TEST CATEGORIES TO SPECIFY

### 1. Correctness Tests (MUST HAVE) — 12 tests minimum

Write test skeletons that prove SIMD matches portable implementation:

```rust
// tests/simd_spec.rs

#[cfg(test)]
mod simd_correctness {
    use edgevec::quantization::{BinaryQuantizer, QuantizedVector};

    /// CRITICAL: SIMD must return identical result to portable
    #[test]
    fn test_simd_matches_portable_zeros() {
        let a = [0x00u8; 96];
        let b = [0x00u8; 96];

        // These functions don't exist yet - test will FAIL (correct!)
        let portable_result = portable_hamming_distance(&a, &b);
        let simd_result = simd_hamming_distance(&a, &b);

        assert_eq!(portable_result, simd_result);
        assert_eq!(simd_result, 0); // Expected: 0 differing bits
    }

    #[test]
    fn test_simd_matches_portable_ones() {
        let a = [0xFFu8; 96];
        let b = [0x00u8; 96];

        let portable_result = portable_hamming_distance(&a, &b);
        let simd_result = simd_hamming_distance(&a, &b);

        assert_eq!(portable_result, simd_result);
        assert_eq!(simd_result, 768); // Expected: all 768 bits differ
    }

    #[test]
    fn test_simd_matches_portable_alternating() {
        let a = [0xAAu8; 96];  // 10101010...
        let b = [0x55u8; 96];  // 01010101...

        let portable_result = portable_hamming_distance(&a, &b);
        let simd_result = simd_hamming_distance(&a, &b);

        assert_eq!(portable_result, simd_result);
        assert_eq!(simd_result, 768); // All bits differ
    }

    #[test]
    fn test_simd_symmetry() {
        let a = [0xABu8; 96];
        let b = [0xCDu8; 96];

        // SPECIFICATION: distance(a, b) == distance(b, a)
        let ab = simd_hamming_distance(&a, &b);
        let ba = simd_hamming_distance(&b, &a);

        assert_eq!(ab, ba);
    }

    #[test]
    fn test_simd_self_distance() {
        let a = [0x42u8; 96];

        // SPECIFICATION: distance(a, a) == 0
        assert_eq!(simd_hamming_distance(&a, &a), 0);
    }

    #[test]
    fn test_simd_single_bit_difference() {
        let mut a = [0x00u8; 96];
        let b = [0x00u8; 96];
        a[0] = 0x01;  // Single bit set

        let portable_result = portable_hamming_distance(&a, &b);
        let simd_result = simd_hamming_distance(&a, &b);

        assert_eq!(portable_result, simd_result);
        assert_eq!(simd_result, 1);
    }

    #[test]
    fn test_simd_last_byte() {
        let mut a = [0x00u8; 96];
        let b = [0x00u8; 96];
        a[95] = 0xFF;  // Last byte all ones

        let portable_result = portable_hamming_distance(&a, &b);
        let simd_result = simd_hamming_distance(&a, &b);

        assert_eq!(portable_result, simd_result);
        assert_eq!(simd_result, 8);
    }

    #[test]
    fn test_simd_first_byte() {
        let mut a = [0x00u8; 96];
        let b = [0x00u8; 96];
        a[0] = 0xFF;  // First byte all ones

        let portable_result = portable_hamming_distance(&a, &b);
        let simd_result = simd_hamming_distance(&a, &b);

        assert_eq!(portable_result, simd_result);
        assert_eq!(simd_result, 8);
    }

    #[test]
    fn test_simd_avx2_boundary_32() {
        // Bytes 31-32 cross first AVX2 YMM register boundary
        let mut a = [0x00u8; 96];
        let b = [0x00u8; 96];
        a[31] = 0xFF;
        a[32] = 0xFF;

        let portable_result = portable_hamming_distance(&a, &b);
        let simd_result = simd_hamming_distance(&a, &b);

        assert_eq!(portable_result, simd_result);
        assert_eq!(simd_result, 16); // 2 bytes × 8 bits
    }

    #[test]
    fn test_simd_avx2_boundary_64() {
        // Bytes 63-64 cross second AVX2 YMM register boundary
        let mut a = [0x00u8; 96];
        let b = [0x00u8; 96];
        a[63] = 0xFF;
        a[64] = 0xFF;

        let portable_result = portable_hamming_distance(&a, &b);
        let simd_result = simd_hamming_distance(&a, &b);

        assert_eq!(portable_result, simd_result);
        assert_eq!(simd_result, 16);
    }

    #[test]
    fn test_simd_mixed_pattern() {
        let a = [0xF0u8; 96];  // 11110000 pattern
        let b = [0x0Fu8; 96];  // 00001111 pattern

        let portable_result = portable_hamming_distance(&a, &b);
        let simd_result = simd_hamming_distance(&a, &b);

        assert_eq!(portable_result, simd_result);
        assert_eq!(simd_result, 768); // All bits differ
    }

    #[test]
    fn test_simd_sparse_differences() {
        let mut a = [0x00u8; 96];
        let mut b = [0x00u8; 96];

        // Set 1 bit every 8 bytes
        for i in (0..96).step_by(8) {
            a[i] = 0x01;
        }

        let portable_result = portable_hamming_distance(&a, &b);
        let simd_result = simd_hamming_distance(&a, &b);

        assert_eq!(portable_result, simd_result);
        assert_eq!(simd_result, 12); // 96/8 = 12 bits
    }
}
```

---

### 2. Property Tests (MUST HAVE) — 10,000+ cases

```rust
#[cfg(test)]
mod simd_properties {
    use proptest::prelude::*;
    use edgevec::quantization::*;

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

            let portable = portable_hamming_distance(&a_arr, &b_arr);
            let simd = simd_hamming_distance(&a_arr, &b_arr);

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
                simd_hamming_distance(&a_arr, &b_arr),
                simd_hamming_distance(&b_arr, &a_arr)
            );
        }

        /// SPECIFICATION: Self-distance is always 0
        #[test]
        fn prop_simd_self_zero(
            a in proptest::collection::vec(any::<u8>(), 96)
        ) {
            let a_arr: [u8; 96] = a.try_into().unwrap();
            prop_assert_eq!(simd_hamming_distance(&a_arr, &a_arr), 0);
        }

        /// SPECIFICATION: Triangle inequality holds
        #[test]
        fn prop_simd_triangle_inequality(
            a in proptest::collection::vec(any::<u8>(), 96),
            b in proptest::collection::vec(any::<u8>(), 96),
            c in proptest::collection::vec(any::<u8>(), 96)
        ) {
            let a_arr: [u8; 96] = a.try_into().unwrap();
            let b_arr: [u8; 96] = b.try_into().unwrap();
            let c_arr: [u8; 96] = c.try_into().unwrap();

            let ab = simd_hamming_distance(&a_arr, &b_arr);
            let bc = simd_hamming_distance(&b_arr, &c_arr);
            let ac = simd_hamming_distance(&a_arr, &c_arr);

            // d(a,c) <= d(a,b) + d(b,c)
            prop_assert!(ac <= ab + bc);
        }

        /// SPECIFICATION: Distance is bounded by vector size
        #[test]
        fn prop_simd_bounded(
            a in proptest::collection::vec(any::<u8>(), 96),
            b in proptest::collection::vec(any::<u8>(), 96)
        ) {
            let a_arr: [u8; 96] = a.try_into().unwrap();
            let b_arr: [u8; 96] = b.try_into().unwrap();

            let distance = simd_hamming_distance(&a_arr, &b_arr);

            prop_assert!(distance <= 768); // Max: all 768 bits differ
        }
    }
}
```

---

### 3. Integration Tests (MUST HAVE)

```rust
#[cfg(test)]
mod simd_integration {
    use edgevec::quantization::{BinaryQuantizer, QuantizedVector};

    #[test]
    fn test_quantized_vector_uses_simd() {
        let quantizer = BinaryQuantizer::new();
        let v1: Vec<f32> = (0..768).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        let v2: Vec<f32> = (0..768).map(|i| if i % 2 == 0 { -1.0 } else { 1.0 }).collect();

        let q1 = quantizer.quantize(&v1);
        let q2 = quantizer.quantize(&v2);

        // This should use SIMD internally when available
        let distance = q1.hamming_distance(&q2);
        assert_eq!(distance, 768); // All bits different
    }

    #[test]
    fn test_day36_api_unchanged() {
        // SPECIFICATION: Day 36 API must remain unchanged
        let q1 = QuantizedVector::from_bytes([0xAA; 96]);
        let q2 = QuantizedVector::from_bytes([0x55; 96]);

        assert_eq!(q1.hamming_distance(&q2), 768);
        assert!((q1.similarity(&q2) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_portable_fallback_works() {
        // Force portable path (implementation will provide mechanism)
        let a = [0xAAu8; 96];
        let b = [0x55u8; 96];

        // Portable implementation should always work
        let result = portable_hamming_distance(&a, &b);
        assert_eq!(result, 768);
    }
}
```

---

## DELIVERABLES

| Artifact | Status | Content |
|:---------|:-------|:--------|
| `tests/simd_spec.rs` | [ ] | All test skeletons (will fail initially) |
| Test count | [ ] | Minimum 25 tests (12 unit + 5 property + 3 integration + 5 edge cases) |
| Property test config | [ ] | ProptestConfig::with_cases(10_000) |
| Stub functions | [ ] | `simd_hamming_distance`, `portable_hamming_distance` imports |

---

## ACCEPTANCE CRITERIA (BINARY)

**Critical (Must Pass):**
- [ ] File `tests/simd_spec.rs` created
  - Verify: `test -f tests/simd_spec.rs`
  - Expected: Exit code 0
- [ ] File compiles (may have missing function errors, but no syntax errors)
  - Verify: `cargo test --no-run simd_spec 2>&1`
  - Expected: Either success OR compilation errors mentioning "cannot find function" but NOT "syntax error" or "unexpected token"
  - Acceptance: Errors only about missing `simd_hamming_distance`/`portable_hamming_distance` functions
- [ ] All tests currently FAIL (because implementation doesn't exist yet)
  - Verify: `cargo test simd_spec 2>&1 | grep "test result"`
  - Expected: "test result: FAILED" OR compilation errors (both acceptable - tests not passing yet)
  - Forbidden: "test result: ok" (tests must NOT pass before implementation exists)
- [ ] Test count ≥25
  - Verify: `grep -c "#\[test\]" tests/simd_spec.rs`
  - Expected: Number ≥ 25
- [ ] Property test count ≥5
  - Verify: `grep -c "fn prop_" tests/simd_spec.rs`
  - Expected: Number ≥ 5
- [ ] Property test cases configured for ≥10,000
  - Verify: `grep -c "with_cases(10_000)" tests/simd_spec.rs`
  - Expected: Number ≥ 1
- [ ] No implementation code exists yet (test-first verified)
  - Verify: `test ! -f src/quantization/simd.rs; echo $?`
  - Expected: Exit code 0 (file does NOT exist)
- [ ] Git log shows test spec committed BEFORE implementation
  - Verify: `git log --oneline --all -- tests/simd_spec.rs src/quantization/simd.rs | head -5`
  - Expected: simd_spec.rs appears in git history BEFORE simd.rs (or simd.rs not yet committed)

**Verification Commands:**
```bash
# Check file created
test -f tests/simd_spec.rs && echo "✅ Test spec created"

# Check test count
echo "Test count:"
grep -c "#\[test\]" tests/simd_spec.rs

# Check property tests
echo "Property tests:"
grep -c "proptest!" tests/simd_spec.rs

# Verify no implementation exists yet
test ! -f src/quantization/simd.rs && echo "✅ No implementation yet (correct!)"

# Try to compile tests (will fail due to missing functions - expected)
cargo test --no-run simd 2>&1 | head -20
echo "^ Expected to see compilation errors for missing functions"
```

---

## HANDOFF

```
TEST_ENGINEER → RUST_ENGINEER

Deliverable: tests/simd_spec.rs with all failing tests
Status: TEST SPECIFICATIONS READY
Evidence: `cargo test --no-run simd` compiles test structure (may fail on missing imports)

Next: 04_SIMD_HAMMING_IMPL.md (RUST_ENGINEER makes these tests pass)

CRITICAL CONSTRAINTS for RUST_ENGINEER:
1. MUST make ALL tests in simd_spec.rs pass
2. MUST NOT modify any tests in simd_spec.rs
3. MUST verify: `git diff tests/simd_spec.rs` shows no changes after implementation
```

---

## FAILURE PROTOCOL

### Detection

If test specification cannot be completed:
- Symptom: Cannot define test for core requirement
- Evidence: Stuck on test design for >30 minutes

### Categorization

1. **Type A: Architecture unclear**
   - Action: Request clarification from META_ARCHITECT
   - Question: "What is the exact signature of simd_hamming_distance?"
   - Time limit: 30 minutes
   - Block: Cannot proceed without architecture clarity

2. **Type B: Test framework issues (proptest, cargo)**
   - Action: Debug test setup, verify proptest dependency in Cargo.toml
   - Commands:
     ```bash
     cargo add proptest --dev
     cargo test --no-run
     ```
   - Time limit: 1 hour
   - Fallback: Use manual test cases if proptest unavailable

3. **Type C: Fundamental uncertainty about SIMD behavior**
   - Action: Escalate to PLANNER immediately
   - Issue: "Cannot write test without understanding SIMD implementation details"
   - Options:
     - A. Request more detailed architecture spec
     - B. Write tests based on portable behavior only
     - C. Defer some tests until implementation exists (anti-pattern!)

### Escalation Triggers

Escalate to PLANNER if:
- [ ] >2 hours without 25 test specifications complete
- [ ] Unable to define property tests
- [ ] Architecture contradictions discovered (SIMD spec contradicts portable spec)
- [ ] Test framework fundamentally incompatible

### Alternative Paths

If test specification unfeasible:
- Option A: Defer SIMD to Week 9, ship with portable only
- Option B: Simplify test requirements (reduce from 25 to 10 tests)
- Option C: Write tests after implementation (violates test-first but unblocks)

Document decision in: `docs/planning/weeks/week8/W8D37_TEST_SPEC_BLOCKER.md`

---

**END OF TEST SPEC PROMPT**
