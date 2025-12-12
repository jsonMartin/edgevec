# PROMPT: SIMD Quantization Implementation (Optional)

**Target Agent:** RUST_ENGINEER
**Command:** `/rust-implement W8.37.2`
**Priority:** P2 (SECONDARY — Only if time permits after 02)
**Estimated Time:** 1 hour
**Dependencies:**
  - `04_SIMD_HAMMING_IMPL.md` COMPLETE (Hamming SIMD implemented and verified)
  - `02_SIMD_TEST_SPEC.md` complete (tests exist)
  - `03_SIMD_BENCHMARK_SPEC.md` complete (targets defined)
**Output:** Enhanced `quantize()` in `src/quantization/simd.rs`

---

## DEPENDENCY VERIFICATION

**Before proceeding, verify ALL dependencies complete:**

```bash
# Check Hamming SIMD implemented
test -f src/quantization/simd.rs || { echo "BLOCK: SIMD module missing"; exit 1; }
grep -q "hamming" src/quantization/simd.rs || { echo "BLOCK: Hamming function not found"; exit 1; }

# Check Hamming tests pass
cargo test simd_spec 2>&1 | grep -q "test result: ok" || { echo "BLOCK: Hamming tests not passing"; exit 1; }

# Check Hamming benchmarks meet targets
test -f docs/benchmarks/W8D37_VALIDATION_REPORT.md || { echo "BLOCK: Hamming not validated yet"; exit 1; }
grep -q "APPROVED\|CONDITIONAL" docs/benchmarks/W8D37_VALIDATION_REPORT.md || { echo "BLOCK: Hamming validation failed"; exit 1; }

# Verify test spec exists
test -f tests/simd_spec.rs || { echo "BLOCK: Test spec missing"; exit 1; }

# Verify benchmark targets documented
test -f docs/benchmarks/SIMD_TARGETS.md || { echo "BLOCK: Targets missing"; exit 1; }

echo "✅ All dependencies verified - Hamming SIMD complete, ready for quantize optimization"
```

**Expected Output:**
```
✅ All dependencies verified - Hamming SIMD complete, ready for quantize optimization
```

**If ANY check fails:**
- **DO NOT** proceed with quantize SIMD
- **REASON:** Hamming is P0, quantize is P2 (optional)
- **ACTION:** Mark quantize as DEFERRED and document in validation report

---

## MISSION

Optionally implement SIMD-accelerated quantization (f32 → binary). This is **lower priority** than Hamming distance because:

1. Quantization is O(n) amortized (done once per vector)
2. Hamming distance is O(n²) in search (done for every comparison)
3. Day 36 quantization is already fast enough (~1ms for 768 floats)

**Only implement if Day 37 Hamming SIMD is complete and time remains.**

---

## CONTEXT

### Current Portable Implementation (Day 36)

```rust
// src/quantization/binary.rs:298-318
pub fn quantize(&self, vector: &[f32]) -> QuantizedVector {
    assert_eq!(vector.len(), BINARY_QUANTIZATION_DIM);  // 768

    let mut data = [0u8; QUANTIZED_VECTOR_SIZE];  // 96 bytes

    for (i, &value) in vector.iter().enumerate() {
        if value > 0.0 {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            data[byte_idx] |= 1 << bit_idx;
        }
    }

    QuantizedVector { data }
}
```

**Current Performance:** ~500ns for 768 floats (acceptable)
**Target:** <200ns (nice-to-have, not critical)

---

## SIMD OPPORTUNITY

### Algorithm

```
SIMD Quantization Algorithm:

1. Load 8 f32 values into YMM register (256 bits = 8 × 32-bit floats)

2. Compare against zero
   vcmpps ymm0, ymm1, 0x0E  →  0xFFFFFFFF if > 0, else 0x00000000

3. Extract sign bits
   vmovmskps eax, ymm0  →  8 bits in integer register

4. Store byte to output

5. Repeat for all 96 iterations (768 / 8 = 96 bytes)
```

### AVX2 Implementation Sketch

```rust
#[cfg(target_arch = "x86_64")]
mod avx2 {
    use std::arch::x86_64::*;

    /// SIMD quantization: f32[768] → u8[96]
    ///
    /// # Safety
    /// - Caller must ensure AVX2 is available
    /// - Input must be exactly 768 f32 values
    #[target_feature(enable = "avx2")]
    pub unsafe fn quantize(vector: &[f32; 768]) -> [u8; 96] {
        let mut result = [0u8; 96];
        let zero = _mm256_setzero_ps();

        for i in 0..96 {
            // Load 8 floats
            let v = _mm256_loadu_ps(vector.as_ptr().add(i * 8));

            // Compare: v > 0.0
            // _CMP_GT_OQ = 0x1E (greater than, ordered, quiet)
            let cmp = _mm256_cmp_ps(v, zero, _CMP_GT_OQ);

            // Extract comparison results as 8-bit mask
            let mask = _mm256_movemask_ps(cmp) as u8;

            result[i] = mask;
        }

        result
    }
}
```

### Potential Issues

1. **Byte order:** `vmovmskps` extracts bits in a specific order — verify it matches Day 36 bit layout
2. **NaN handling:** `vcmpps` with NaN returns false (matches Day 36 behavior)
3. **Alignment:** f32 input may not be aligned — use `loadu` not `load`

---

## IMPLEMENTATION TASKS

### Task 1: Verify Bit Order (15 min)
- [ ] Write test comparing SIMD output to portable output
- [ ] Document bit order if different
- [ ] Adjust extraction logic if needed

### Task 2: Implement AVX2 Quantize (30 min)
- [ ] Implement `avx2::quantize`
- [ ] Handle edge cases (NaN, Inf, -0.0)
- [ ] Document safety invariants

### Task 3: Integrate with BinaryQuantizer (15 min)
- [ ] Add dispatch logic to `BinaryQuantizer::quantize`
- [ ] Maintain portable fallback
- [ ] Ensure Day 36 tests pass

---

## VERIFICATION

```rust
#[test]
fn test_simd_quantize_matches_portable() {
    use proptest::prelude::*;

    proptest!(|(v in proptest::collection::vec(-1.0f32..1.0f32, 768))| {
        let v_arr: [f32; 768] = v.try_into().unwrap();

        let portable = portable::quantize(&v_arr);

        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") {
            let simd = unsafe { avx2::quantize(&v_arr) };
            prop_assert_eq!(portable, simd);
        }
    });
}
```

---

## DECISION GATE

**Before implementing, ask:**

1. Is Day 37 Hamming SIMD complete and passing? → If NO, skip this
2. Is there ≥1 hour remaining in Day 37? → If NO, defer to future
3. Is quantization a bottleneck? → If NO (likely), this is optimization theater

**Recommendation:** Implement only if all three conditions are YES.

---

## DELIVERABLES (If Implemented)

| Artifact | Path | Status |
|:---------|:-----|:-------|
| SIMD quantize | `src/quantization/simd.rs` | [ ] |
| Integration | `BinaryQuantizer::quantize` dispatch | [ ] |
| Tests | Property test SIMD == portable | [ ] |

---

## ANTI-HALLUCINATION CLAMPS

### Forbidden Phrases

**NEVER claim performance without measurement:**
- ❌ "This should be faster"
- ❌ "Approximately 2x speedup"
- ❌ "About the same speed"
- ❌ "Probably faster"
- ❌ "Should improve quantization"
- ❌ "Likely no performance impact"

**REQUIRED Evidence Format:**
- ✅ "Measured: 2.1x speedup (criterion output: 850ns SIMD vs 1.8μs portable)"
- ✅ "Benchmark: no regression on Hamming (still 46 cycles, rdtsc output pasted)"
- ✅ "Property test: SIMD == portable for 10,000 cases (proptest output pasted)"
- ✅ "Test result: ok. 15 passed; 0 failed (full cargo test output pasted)"

### Verification Protocol

Every performance or correctness claim MUST include:
1. **Measurement tool:** criterion, rdtsc, proptest
2. **Exact numbers:** Not estimates or ranges
3. **Evidence artifact:** Paste benchmark/test output
4. **Methodology:** Iterations count, warmup, test cases

**Example Evidence:**

```markdown
## Quantize SIMD Validation

**Correctness (proptest):**
- Test: SIMD quantize == portable quantize
- Cases: 10,000
- Status: ✅ PASS
- Evidence:
  ```
  $ cargo test prop_quantize_simd_matches_portable
  test prop_quantize_simd_matches_portable ... ok (10000 cases)
  ```

**Performance (criterion):**
- SIMD quantize time: 850 ns
- Portable quantize time: 1.8 μs
- Speedup: 2.1x
- Target: ≥2x
- Status: ✅ PASS
- Evidence:
  ```
  $ cargo bench --bench bench_simd -- quantize
  quantize_simd           time:   [840 ns 850 ns 860 ns]
  quantize_portable       time:   [1.78 μs 1.80 μs 1.82 μs]
  ```

**No Regression (rdtsc):**
- Hamming cycles after adding quantize SIMD: 46 cycles
- Hamming cycles before (baseline): 46 cycles
- Status: ✅ NO REGRESSION
- Evidence:
  ```
  $ cargo test test_simd_cycle_target --release -- --nocapture
  Measured cycles: 46
  test test_simd_cycle_target ... ok
  ```
```

### Performance Claims Checklist

Before claiming quantize SIMD complete, verify:
- [ ] Correctness verified with property test (SIMD == portable, 10k+ cases)
- [ ] Speedup measured with criterion (not estimated)
- [ ] No regression on Hamming distance (cycles still <50)
- [ ] All benchmark output pasted (not summarized)
- [ ] Test results show actual pass count
- [ ] No use of "should", "probably", "approximately", "about", "likely"

### Rejection Criteria

Implementation REJECTED if:
- [ ] Claims "faster" without criterion evidence
- [ ] Claims "correct" without property test evidence showing exact case count
- [ ] Uses "approximately", "about", "probably", "should be", "likely"
- [ ] Benchmark output not pasted
- [ ] Says "no regression" without showing Hamming still meets <50 cycle target
- [ ] Says "tests pass" without pasting full output

**Acceptable:**
- "Measured: 2.1x speedup (850ns / 1.8μs, criterion output pasted)"
- "Property test: SIMD == portable for 10,000 cases (proptest output pasted)"
- "No regression: Hamming still 46 cycles (rdtsc output pasted)"

**NOT Acceptable:**
- "About 2x faster"
- "Tests are passing"
- "No performance impact"

---

## HANDOFF

```
If implemented:
    RUST_ENGINEER → BENCHMARK_SCIENTIST
    Include quantize benchmarks in 04_SIMD_BENCHMARKS.md

If deferred:
    Document in Day 37 summary: "Quantize SIMD deferred — Hamming was priority"
```

---

**END OF PROMPT**
