# PROMPT: SIMD Hamming Distance Implementation

**Target Agent:** RUST_ENGINEER
**Command:** `/rust-implement W8.37.1`
**Priority:** P0 (PRIMARY DELIVERABLE)
**Estimated Time:** 3 hours
**Dependencies:**
  - `01_SIMD_ARCHITECTURE.md` APPROVED
  - `02_SIMD_TEST_SPEC.md` complete (tests exist)
  - `03_SIMD_BENCHMARK_SPEC.md` complete (targets defined)
**Output:** `src/quantization/simd.rs`

---

## DEPENDENCY VERIFICATION

**Before proceeding, verify ALL dependencies complete:**

```bash
# Check architecture approved
test -f docs/architecture/SIMD_DESIGN.md || { echo "BLOCK: Architecture missing"; exit 1; }
grep -q "APPROVED" docs/reviews/*SIMD_DESIGN*.md || { echo "BLOCK: Architecture not approved"; exit 1; }

# Check test spec ready
test -f tests/simd_spec.rs || { echo "BLOCK: Test spec missing"; exit 1; }
TEST_COUNT=$(grep -c "#\[test\]" tests/simd_spec.rs)
[ "$TEST_COUNT" -ge 25 ] || { echo "BLOCK: Only $TEST_COUNT tests (need ≥25)"; exit 1; }

# Check benchmark spec ready
test -f benches/bench_simd.rs || { echo "BLOCK: Benchmark spec missing"; exit 1; }
test -f docs/benchmarks/SIMD_TARGETS.md || { echo "BLOCK: Targets missing"; exit 1; }

# Verify targets are documented
grep -q "<50" docs/benchmarks/SIMD_TARGETS.md || { echo "BLOCK: Cycle target not defined"; exit 1; }
grep -q ">5x" docs/benchmarks/SIMD_TARGETS.md || { echo "BLOCK: Speedup target not defined"; exit 1; }

echo "✅ All dependencies verified - ready to implement"
```

**Expected Output:**
```
✅ All dependencies verified - ready to implement
```

**If ANY check fails, STOP and escalate to PLANNER.**

---

## MISSION

Implement SIMD-accelerated Hamming distance for 96-byte (768-bit) binary vectors. This is the **core performance deliverable** for Day 37.

**Target:** <50 CPU cycles per comparison (vs ~300 cycles baseline)

---

## CONTEXT FILES TO LOAD

```bash
# REQUIRED
cat src/quantization/binary.rs           # Current portable implementation
cat docs/architecture/SIMD_DESIGN.md     # Architecture decisions (from 01)

# REFERENCE
cat benches/bench_quantization.rs        # Existing benchmarks
```

---

## TECHNICAL SPECIFICATION

### Input/Output Contract

```rust
/// Computes Hamming distance between two 96-byte binary vectors using SIMD.
///
/// # Arguments
/// * `a` - First 96-byte array (768 bits), 64-byte aligned
/// * `b` - Second 96-byte array (768 bits), 64-byte aligned
///
/// # Returns
/// Number of differing bits (0..=768)
///
/// # Performance
/// Target: <50 CPU cycles on AVX2-capable hardware
pub fn hamming_distance_simd(a: &[u8; 96], b: &[u8; 96]) -> u32
```

### Algorithm

```
SIMD Hamming Distance Algorithm:

1. Load vectors into SIMD registers
   AVX2: 3 × ymm (256-bit) registers = 96 bytes

2. XOR corresponding registers
   vpxor ymm0, ymm1  →  differing bits become 1

3. Population count (popcount)
   Option A: AVX-512 VPOPCNTDQ (native)
   Option B: AVX2 emulated via lookup table
   Option C: Horizontal sum of byte popcounts

4. Horizontal sum across registers
   Sum all partial popcounts → final distance
```

### AVX2 Implementation Sketch

```rust
#[cfg(target_arch = "x86_64")]
mod avx2 {
    use std::arch::x86_64::*;

    /// AVX2 Hamming distance for 96 bytes.
    ///
    /// # Safety
    /// - Caller must ensure AVX2 is available (is_x86_feature_detected!)
    /// - Input arrays must be 32-byte aligned (guaranteed by QuantizedVector)
    #[target_feature(enable = "avx2")]
    pub unsafe fn hamming_distance(a: &[u8; 96], b: &[u8; 96]) -> u32 {
        // Load 96 bytes in 3 × 256-bit registers
        let a0 = _mm256_loadu_si256(a.as_ptr() as *const __m256i);
        let a1 = _mm256_loadu_si256(a.as_ptr().add(32) as *const __m256i);
        let a2 = _mm256_loadu_si256(a.as_ptr().add(64) as *const __m256i);

        let b0 = _mm256_loadu_si256(b.as_ptr() as *const __m256i);
        let b1 = _mm256_loadu_si256(b.as_ptr().add(32) as *const __m256i);
        let b2 = _mm256_loadu_si256(b.as_ptr().add(64) as *const __m256i);

        // XOR to find differing bits
        let xor0 = _mm256_xor_si256(a0, b0);
        let xor1 = _mm256_xor_si256(a1, b1);
        let xor2 = _mm256_xor_si256(a2, b2);

        // Popcount (AVX2 doesn't have native popcount, use lookup table)
        let pop0 = popcnt_avx2(xor0);
        let pop1 = popcnt_avx2(xor1);
        let pop2 = popcnt_avx2(xor2);

        // Horizontal sum
        horizontal_sum_avx2(pop0) + horizontal_sum_avx2(pop1) + horizontal_sum_avx2(pop2)
    }

    /// AVX2 popcount using lookup table method.
    #[target_feature(enable = "avx2")]
    unsafe fn popcnt_avx2(v: __m256i) -> __m256i {
        // Nibble lookup table for popcount
        let lookup = _mm256_setr_epi8(
            0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4,
            0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4,
        );
        let mask = _mm256_set1_epi8(0x0F);

        let lo = _mm256_and_si256(v, mask);
        let hi = _mm256_and_si256(_mm256_srli_epi16(v, 4), mask);

        let pop_lo = _mm256_shuffle_epi8(lookup, lo);
        let pop_hi = _mm256_shuffle_epi8(lookup, hi);

        _mm256_add_epi8(pop_lo, pop_hi)
    }

    /// Horizontal sum of bytes in YMM register.
    #[target_feature(enable = "avx2")]
    unsafe fn horizontal_sum_avx2(v: __m256i) -> u32 {
        // SAD against zero gives sum of absolute values (which equals sum for unsigned)
        let zero = _mm256_setzero_si256();
        let sad = _mm256_sad_epu8(v, zero);

        // Extract and sum the 4 × 64-bit partial sums
        let lo = _mm256_castsi256_si128(sad);
        let hi = _mm256_extracti128_si256(sad, 1);
        let sum128 = _mm_add_epi64(lo, hi);

        let sum64 = _mm_add_epi64(sum128, _mm_srli_si128(sum128, 8));
        _mm_cvtsi128_si64(sum64) as u32
    }
}
```

### Portable Fallback (std::simd)

```rust
#[cfg(feature = "portable_simd")]
mod portable_simd {
    use std::simd::*;

    pub fn hamming_distance(a: &[u8; 96], b: &[u8; 96]) -> u32 {
        let mut total = 0u32;

        // Process 32 bytes at a time with portable SIMD
        for i in (0..96).step_by(32) {
            let chunk_a = u8x32::from_slice(&a[i..i+32]);
            let chunk_b = u8x32::from_slice(&b[i..i+32]);

            let xor = chunk_a ^ chunk_b;

            // Count bits in each byte
            for byte in xor.to_array() {
                total += byte.count_ones();
            }
        }

        total
    }
}
```

### Dispatch Function

```rust
/// SIMD-accelerated Hamming distance with runtime feature detection.
pub fn hamming_distance_simd(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { avx2::hamming_distance(a, b) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        // NEON is always available on AArch64
        return unsafe { neon::hamming_distance(a, b) };
    }

    // Fallback to portable
    portable::hamming_distance(a, b)
}

/// Portable Hamming distance (no SIMD).
pub fn hamming_distance_portable(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    let mut distance = 0u32;
    for i in 0..96 {
        distance += (a[i] ^ b[i]).count_ones();
    }
    distance
}
```

---

## IMPLEMENTATION TASKS

### Task 1: Create SIMD Module Structure (30 min)
- [ ] Create `src/quantization/simd.rs` (or `simd/mod.rs`)
- [ ] Add to `src/quantization/mod.rs` exports
- [ ] Define public interface

### Task 2: Implement AVX2 Hamming (1.5 hours)
- [ ] Implement `avx2::hamming_distance`
- [ ] Implement `avx2::popcnt_avx2` (lookup table method)
- [ ] Implement `avx2::horizontal_sum_avx2`
- [ ] Add `#[target_feature(enable = "avx2")]` annotations
- [ ] Document all unsafe blocks with safety proofs

### Task 3: Implement Portable Fallback (30 min)
- [ ] Implement `portable::hamming_distance`
- [ ] Ensure it matches Day 36 results exactly
- [ ] Consider std::simd for portable vectorization

### Task 4: Implement Dispatch Logic (30 min)
- [ ] Add `is_x86_feature_detected!` runtime check
- [ ] Wire up AVX2 → portable fallback chain
- [ ] Integrate with `QuantizedVector::hamming_distance`

### Task 5: Unit Tests (30 min)
- [ ] Test SIMD matches portable for all Day 36 test cases
- [ ] Test with edge cases (all zeros, all ones, alternating)
- [ ] Test dispatch logic (force portable path)

---

## SAFETY REQUIREMENTS

### Every `unsafe` Block Must Document:

```rust
/// # Safety
///
/// This function is safe because:
/// 1. AVX2 availability is checked by caller via `is_x86_feature_detected!("avx2")`
/// 2. Input arrays are exactly 96 bytes (enforced by type `&[u8; 96]`)
/// 3. Input arrays are 64-byte aligned (guaranteed by QuantizedVector)
/// 4. No aliasing (enforced by Rust borrow checker)
/// 5. All memory accesses are within bounds (96 bytes ≤ 3 × 32-byte loads)
#[target_feature(enable = "avx2")]
unsafe fn hamming_distance_avx2(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    // ...
}
```

### Miri Verification

```bash
# Run Miri to check for undefined behavior
cargo +nightly miri test --lib -- simd
```

---

## VERIFICATION CHECKLIST

### Correctness
- [ ] SIMD result == portable result for all inputs
- [ ] All Day 36 fuzz corpus passes
- [ ] Property test: `∀ a, b: simd(a,b) == portable(a,b)`

### Performance
- [ ] AVX2 path: <50 cycles (measure with rdtsc or criterion)
- [ ] No regression on portable path

### Safety
- [ ] All unsafe documented with safety proof
- [ ] Miri passes (no UB detected)
- [ ] Clippy clean

### Integration
- [ ] `QuantizedVector::hamming_distance` uses SIMD automatically
- [ ] Day 36 tests pass unchanged
- [ ] `cargo doc` clean

---

## DELIVERABLES

| Artifact | Path | Status |
|:---------|:-----|:-------|
| SIMD module | `src/quantization/simd.rs` | [ ] |
| Updated mod.rs | `src/quantization/mod.rs` | [ ] |
| Unit tests | In `simd.rs` or separate file | [ ] |
| Integration | Updated `binary.rs` | [ ] |

---

## ANTI-HALLUCINATION CLAMPS

### Forbidden Phrases

**NEVER claim performance without measurement:**
- ❌ "This should be fast"
- ❌ "Approximately 50 cycles"
- ❌ "About 5x faster"
- ❌ "Roughly 1 billion ops/sec"
- ❌ "Probably <50 cycles"
- ❌ "Should achieve target"

**REQUIRED Evidence Format:**
- ✅ "Measured: 46 cycles (rdtsc, 10,000 iterations)"
- ✅ "Benchmark: 5.2x speedup (criterion output: 14.2ns SIMD vs 73.8ns portable)"
- ✅ "Measured: 1.12 billion ops/sec (criterion throughput output pasted)"
- ✅ "Test result: ok. 25 passed; 0 failed (full cargo test output pasted)"

### Verification Protocol

Every performance claim MUST include:
1. **Measurement tool:** rdtsc, criterion, perf, cachegrind
2. **Exact numbers:** Not estimates or ranges
3. **Evidence artifact:** Paste benchmark output or link to report
4. **Methodology:** Iterations count, warmup, environment

**Example Evidence:**

```markdown
## Performance Validation

**Cycle Count (rdtsc):**
- Measured: 46 cycles
- Target: <50 cycles
- Status: ✅ PASS
- Methodology: rdtsc with 10,000 iterations, 1,000 warmup
- Evidence:
  ```
  $ cargo test test_simd_cycle_target --release -- --nocapture
  Measured cycles: 46
  test test_simd_cycle_target ... ok
  ```

**Speedup (criterion):**
- SIMD time: 14.2 ns
- Portable time: 73.8 ns
- Speedup: 5.2x
- Target: >5x
- Status: ✅ PASS
- Evidence:
  ```
  $ cargo bench --bench bench_simd -- simd_vs_portable
  simd_vs_portable/simd_avx2
                        time:   [14.0 ns 14.2 ns 14.4 ns]
  simd_vs_portable/portable
                        time:   [73.2 ns 73.8 ns 74.5 ns]
  ```

**Throughput (criterion):**
- Measured: 1.12 billion ops/sec
- Target: >1B ops/sec
- Status: ✅ PASS
- Evidence:
  ```
  $ cargo bench --bench bench_simd -- hamming_ops_per_sec
  hamming_ops_per_sec     time:   [14.2 ns 14.4 ns 14.6 ns]
                          thrpt:  [1.12 Gelem/s 1.13 Gelem/s 1.14 Gelem/s]
  ```
```

### Performance Claims Checklist

Before claiming implementation complete, verify:
- [ ] Cycle count measured with rdtsc (not estimated)
- [ ] Speedup calculated from actual criterion output (not approximated)
- [ ] Throughput measured with criterion (not guessed)
- [ ] All benchmark output pasted (not summarized)
- [ ] Test results show actual pass count (not "tests pass")
- [ ] No use of "should", "probably", "approximately", "about", "roughly"

### Rejection Criteria

Implementation REJECTED if:
- [ ] Any performance claim lacks measurement evidence
- [ ] Claims use "approximately", "about", "roughly", "should be", "probably"
- [ ] Benchmark output not pasted
- [ ] Cycle count not from rdtsc
- [ ] Says "tests pass" without pasting full output showing exact count
- [ ] Says "faster" without specific speedup calculation

**Acceptable:**
- "Measured: 46 cycles (rdtsc output pasted)"
- "Speedup: 5.2x (calculated from 14.2ns / 73.8ns, criterion output pasted)"
- "All 25 tests pass (cargo test output: test result: ok. 25 passed; 0 failed)"

**NOT Acceptable:**
- "About 50 cycles"
- "Much faster"
- "Tests are passing"

---

## HANDOFF

```
RUST_ENGINEER → TEST_ENGINEER + BENCHMARK_SCIENTIST

Deliverable: Working SIMD implementation
Status: READY FOR VALIDATION

Verification needed:
- 04_SIMD_BENCHMARKS.md → Measure <50 cycles
- 05_SIMD_TESTS.md → Verify correctness
```

---

## ANTI-PATTERNS TO AVOID

```rust
// BAD: Unsafe without safety documentation
unsafe fn bad_simd(a: &[u8], b: &[u8]) -> u32 {
    // No safety comment!
    _mm256_loadu_si256(a.as_ptr() as *const __m256i);
}

// BAD: Assuming alignment without proof
unsafe fn bad_aligned(a: &[u8; 96]) -> __m256i {
    _mm256_load_si256(a.as_ptr() as *const __m256i)  // Requires 32-byte alignment!
}

// BAD: Forgetting feature gate
fn bad_dispatch(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    unsafe { avx2::hamming_distance(a, b) }  // No feature check!
}

// GOOD: Proper pattern
fn good_dispatch(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        // SAFETY: Feature detected, alignment guaranteed by QuantizedVector
        return unsafe { avx2::hamming_distance(a, b) };
    }
    portable::hamming_distance(a, b)
}
```

---

**END OF PROMPT**
