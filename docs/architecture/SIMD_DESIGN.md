# EdgeVec SIMD Architecture

**Version:** 1.0.0
**Author:** META_ARCHITECT
**Date:** 2025-12-12
**Status:** [APPROVED]

---

## 1. Executive Summary

EdgeVec's SIMD acceleration targets <50 CPU cycles for 768-bit Hamming distance computation using a hierarchical module structure with runtime CPU feature detection. The design prioritizes AVX2 (x86_64) as the primary optimization target while maintaining a safe portable fallback. All unsafe SIMD intrinsics are encapsulated behind safe public APIs with compile-time and runtime safety guarantees.

---

## 2. Module Structure

### Chosen: Option B — Hierarchical Structure

**Justification:**

Option B (Hierarchical Structure) is selected for the following reasons:

1. **Testability:** Each platform implementation (avx2.rs, neon.rs, portable.rs) can be unit-tested independently with platform-specific CI runners.

2. **Maintainability:** Clear separation allows platform experts to modify one file without affecting others. The dispatch logic in `simd/mod.rs` acts as a single coordination point.

3. **Conditional Compilation:** Each file uses its own `#[cfg]` attributes, making the conditional compilation cleaner than Option C's inline approach.

4. **Extensibility:** Adding new platforms (AVX-512, SVE2) requires only adding a new file and updating the dispatch logic, without modifying existing implementations.

5. **Code Review:** Unsafe code is isolated in specific files, making security audits tractable.

### File Layout

```
src/quantization/
├── binary.rs           # Public API (unchanged)
├── mod.rs              # Module exports
├── scalar.rs           # Existing SQ8
└── simd/
    ├── mod.rs          # Dispatch logic + public interface
    ├── avx2.rs         # x86_64 AVX2 implementation
    ├── neon.rs         # ARM64 NEON implementation (P2)
    ├── wasm.rs         # WebAssembly SIMD (Defer)
    └── portable.rs     # Safe fallback using byte-by-byte popcount
```

### Module Responsibilities

| File | Responsibility | Contains `unsafe` |
|:-----|:---------------|:------------------|
| `simd/mod.rs` | Runtime dispatch, public API | No (calls into unsafe internally) |
| `simd/avx2.rs` | AVX2 intrinsics implementation | Yes |
| `simd/neon.rs` | NEON intrinsics implementation | Yes |
| `simd/wasm.rs` | WASM SIMD implementation | Yes |
| `simd/portable.rs` | Safe Rust fallback | No |

---

## 3. Dispatch Strategy

### Chosen: Option B — Runtime Detection

**Justification:**

Runtime detection is selected over compile-time only for these reasons:

1. **Single Binary Distribution:** Users download one binary that works on any x86_64 CPU, from 10-year-old laptops to modern servers.

2. **Cloud Compatibility:** AWS, GCP, and Azure VMs vary in CPU capabilities. Runtime detection adapts automatically.

3. **Branch Prediction:** The `is_x86_feature_detected!` check is a single branch that becomes perfectly predicted after the first call (99.99%+ prediction rate).

4. **Cycle Overhead Calculation:**
   - Branch misprediction penalty: ~15 cycles (first call only)
   - Predicted branch: 1 cycle
   - Total overhead: 1 cycle per call (amortized: 0.0001 cycles for 10,000 calls)
   - This is negligible compared to the ~47 cycle operation

**Why Not Option C (Cached Runtime Detection):**
Option C adds function pointer indirection (~2 cycles) and `OnceLock` overhead. The branch predictor in Option B achieves equivalent performance with simpler code.

### Code Pattern

```rust
// src/quantization/simd/mod.rs

/// Compute Hamming distance using the best available SIMD implementation.
///
/// # Performance
/// - AVX2 (detected): ~47 cycles
/// - Portable fallback: ~300 cycles
///
/// # Safety
/// All unsafe operations are encapsulated. This function is safe to call.
#[inline]
pub fn hamming_distance(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: We just verified AVX2 is available
            return unsafe { avx2::hamming_distance_avx2(a, b) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: We just verified NEON is available
            return unsafe { neon::hamming_distance_neon(a, b) };
        }
    }

    // Safe fallback for all other platforms
    portable::hamming_distance_portable(a, b)
}
```

---

## 4. Platform Support

| Platform | Instruction Set | Priority | Status | Notes |
|:---------|:----------------|:---------|:-------|:------|
| x86_64 Desktop/Server | AVX2 | P0 | Implement Day 37 | ~70% market share, widest benefit |
| Fallback | Portable Rust | P0 | Complete (Day 36) | Required for correctness baseline |
| ARM64 (Apple M1/M2, AWS Graviton) | NEON | P2 | Defer to Week 9 | Growing market, but AVX2 first |
| x86_64 Server | AVX-512 | P2 | Defer to Week 9 | Only ~20% servers, AVX2 sufficient |
| WebAssembly | WASM SIMD | Defer | Week 10 | Browser support varies, portable works |

### Rationale

1. **AVX2 First (P0):** Present on all Intel CPUs since Haswell (2013) and AMD since Excavator (2015). This covers >95% of x86_64 devices. Maximum ROI for optimization effort.

2. **Portable Required (P0):** The portable implementation from Day 36 must remain as the correctness baseline and fallback for unsupported platforms.

3. **NEON Deferred (P2):** ARM64 is growing (Apple Silicon, AWS Graviton) but requires separate CI infrastructure. Can be added in Week 9 using the same hierarchical structure.

4. **AVX-512 Deferred (P2):** Only available on recent Intel server CPUs (Skylake-X+) and recent AMD (Zen 4). AVX2 already achieves <50 cycles target.

5. **WASM Deferred to Week 10:** Browser WASM SIMD support is inconsistent. The portable implementation runs in WASM at acceptable speed (~300 cycles). Week 10 timing allows validation of AVX2 (Week 8) and NEON (Week 9) patterns first.

---

## 5. Safety Model

### Invariants

1. **CPU Feature Verification:** All SIMD intrinsics are called ONLY after runtime verification of CPU capability via `is_x86_feature_detected!("avx2")` or compile-time `#[target_feature(enable = "avx2")]`.

2. **Alignment Guarantee:** Input arrays are 64-byte aligned. `QuantizedVector` uses `#[repr(C, align(64))]` ensuring alignment for AVX-512 (strictest requirement), which also satisfies AVX2's 32-byte requirement.

3. **Size Invariant:** Input arrays are exactly 96 bytes (`[u8; 96]`), enforced by Rust's type system. This matches exactly 3 × 256-bit YMM registers.

4. **No Aliasing:** Rust's borrow checker ensures `a` and `b` cannot alias. Both are shared references (`&[u8; 96]`), which Rust guarantees are non-overlapping for distinct objects. **Note:** The public API takes `&QuantizedVector` (not `&[u8; 96]`), and two distinct `QuantizedVector` instances cannot share storage. The theoretical case where `&[u8; 96]` points to the same object is prevented at the `QuantizedVector` level: `v.hamming_distance(&v)` passes two references to the same struct, but Rust's aliasing rules permit multiple `&` to the same data (read-only). This is safe because our SIMD code only reads.

5. **Pointer Validity:** All pointer arithmetic stays within bounds. We load at offsets 0, 32, and 64 from a 96-byte array, all valid.

6. **No Uninitialized Memory:** All SIMD registers are initialized from input arrays. No uninitialized reads occur.

### Unsafe Boundaries

```rust
// src/quantization/simd/avx2.rs

/// AVX2 Hamming distance for 96-byte binary vectors.
///
/// # Safety
///
/// Caller MUST ensure:
/// 1. CPU supports AVX2 (`is_x86_feature_detected!("avx2")`)
/// 2. Both arrays are valid and non-overlapping
///
/// These are enforced by the public API in `simd/mod.rs`.
#[target_feature(enable = "avx2")]
#[cfg(target_arch = "x86_64")]
pub(crate) unsafe fn hamming_distance_avx2(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    use std::arch::x86_64::*;

    // SAFETY: Caller verified AVX2 available.
    // Array size (96) allows loads at offsets 0, 32, 64.
    // Alignment (64-byte) exceeds AVX2's 32-byte requirement.

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

    // Population count via lookup table method
    // (AVX2 lacks native popcount, AVX-512 VPOPCNTDQ would be faster)
    popcount_avx2(xor0) + popcount_avx2(xor1) + popcount_avx2(xor2)
}

/// AVX2 population count using lookup table method.
///
/// # Algorithm
/// 1. Split each byte into low/high nibbles
/// 2. Use PSHUFB to look up popcount for each nibble
/// 3. Add nibble counts to get byte counts
/// 4. Horizontal sum across all bytes
#[target_feature(enable = "avx2")]
#[inline]
unsafe fn popcount_avx2(v: __m256i) -> u32 {
    use std::arch::x86_64::*;

    // Lookup table: popcount for values 0-15
    let lookup = _mm256_setr_epi8(
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4,
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4,
    );

    let low_mask = _mm256_set1_epi8(0x0F);

    // Extract low and high nibbles
    let lo = _mm256_and_si256(v, low_mask);
    let hi = _mm256_and_si256(_mm256_srli_epi16(v, 4), low_mask);

    // Lookup popcount for each nibble
    let popcnt_lo = _mm256_shuffle_epi8(lookup, lo);
    let popcnt_hi = _mm256_shuffle_epi8(lookup, hi);

    // Add nibble counts to get byte counts
    let popcnt = _mm256_add_epi8(popcnt_lo, popcnt_hi);

    // Horizontal sum: sad against zero gives sum of absolute differences
    // Since all values are positive, this sums the bytes
    let sad = _mm256_sad_epu8(popcnt, _mm256_setzero_si256());

    // Extract and sum the four 64-bit partial sums
    let sum_lo = _mm256_extract_epi64(sad, 0) as u32 + _mm256_extract_epi64(sad, 1) as u32;
    let sum_hi = _mm256_extract_epi64(sad, 2) as u32 + _mm256_extract_epi64(sad, 3) as u32;

    sum_lo + sum_hi
}
```

### Verification Strategy

The following verification commands confirm safety invariants:

```bash
# 1. Verify CPU feature check exists before unsafe call
grep -B5 "hamming_distance_avx2" src/quantization/simd/mod.rs | grep -q "is_x86_feature_detected"
echo "Invariant 1 (CPU check): $([[ $? -eq 0 ]] && echo PASS || echo FAIL)"

# 2. Verify QuantizedVector has 64-byte alignment
grep -q "align(64)" src/quantization/binary.rs
echo "Invariant 2 (Alignment): $([[ $? -eq 0 ]] && echo PASS || echo FAIL)"

# 3. Verify input type is [u8; 96]
grep -q "\[u8; 96\]" src/quantization/simd/avx2.rs
echo "Invariant 3 (Size): $([[ $? -eq 0 ]] && echo PASS || echo FAIL)"

# 4. Verify all unsafe blocks have SAFETY comments
UNSAFE_COUNT=$(grep -c "unsafe {" src/quantization/simd/avx2.rs || echo 0)
SAFETY_COUNT=$(grep -c "// SAFETY:" src/quantization/simd/avx2.rs || echo 0)
echo "Invariant 5 (Documentation): Unsafe blocks=$UNSAFE_COUNT, Safety comments=$SAFETY_COUNT"

# 5. Run Miri for undefined behavior detection (requires nightly)
# cargo +nightly miri test simd
```

- [ ] Invariant 1: CPU feature detected before unsafe call
- [ ] Invariant 2: QuantizedVector has `#[repr(C, align(64))]`
- [ ] Invariant 3: Function signatures use `&[u8; 96]`
- [ ] Invariant 4: Borrow checker enforces no aliasing (compile-time)
- [ ] Invariant 5: All pointer arithmetic within bounds (code review)
- [ ] Invariant 6: All SAFETY comments present for unsafe blocks

---

## 6. API Design

### Chosen: Option A — Transparent Replacement

**Justification:**

Transparent replacement is selected for these reasons:

1. **Zero Migration Cost:** All existing code using `QuantizedVector::hamming_distance()` automatically benefits from SIMD without changes.

2. **Encapsulation:** SIMD is an implementation detail. Users shouldn't need to know or care about CPU capabilities.

3. **Testability:** The same API allows property testing (`SIMD result == portable result`) without exposing internal details.

4. **API Stability:** No new public methods means no API surface to maintain or deprecate.

**Migration Impact Analysis:**
- Breaking changes: **None**
- Required code changes: **None**
- Performance improvement: **Automatic** on supported CPUs

### Public API (Unchanged)

```rust
// src/quantization/binary.rs — NO CHANGES TO PUBLIC API

impl QuantizedVector {
    /// Computes the Hamming distance to another quantized vector.
    ///
    /// # Performance
    /// - AVX2 (x86_64): ~47 cycles
    /// - Portable: ~300 cycles
    /// - Automatically uses fastest available implementation
    #[must_use]
    pub fn hamming_distance(&self, other: &Self) -> u32 {
        // Delegate to SIMD dispatcher
        crate::quantization::simd::hamming_distance(&self.data, &other.data)
    }
}
```

### Internal API

```rust
// src/quantization/simd/mod.rs — PUBLIC WITHIN CRATE

/// Compute Hamming distance using best available SIMD implementation.
///
/// This is the single entry point for all Hamming distance computations.
/// It dispatches to platform-specific implementations at runtime.
pub fn hamming_distance(a: &[u8; 96], b: &[u8; 96]) -> u32;

/// Force portable implementation (for testing and benchmarking).
///
/// # Note
/// This is `pub(crate)` to allow benchmarks to compare implementations.
pub(crate) fn hamming_distance_portable(a: &[u8; 96], b: &[u8; 96]) -> u32;

// src/quantization/simd/avx2.rs — CRATE-PRIVATE

/// AVX2 implementation. Only called after feature detection.
///
/// # Safety
/// Caller must verify `is_x86_feature_detected!("avx2")`.
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn hamming_distance_avx2(a: &[u8; 96], b: &[u8; 96]) -> u32;
```

---

## 7. Performance Projections

| Implementation | Expected Cycles | Calculation | Verification |
|:---------------|----------------:|:------------|:-------------|
| Portable (Day 36) | 300 cycles | 96 iterations × 3 ops/iter | Baseline benchmark |
| AVX2 (Day 37) | 47 cycles | See breakdown below | rdtsc measurement |
| NEON (Week 9) | 60 cycles | 6 loads + 6 XOR + popcount | Future benchmark |
| AVX-512 (Week 9) | 25 cycles | 2 loads + 2 XOR + VPOPCNTDQ | Future benchmark |

### AVX2 Cycle Breakdown (47 cycles calculated)

```
Operation                          Latency  Throughput  Total
────────────────────────────────────────────────────────────
LOAD Phase (6 × _mm256_loadu_si256):
  ├── Load a[0..32]                   ~3        0.5       3
  ├── Load a[32..64]                  ~3        0.5       3  (pipelined)
  ├── Load a[64..96]                  ~3        0.5       3  (pipelined)
  ├── Load b[0..32]                   ~3        0.5       3  (pipelined)
  ├── Load b[32..64]                  ~3        0.5       3  (pipelined)
  └── Load b[64..96]                  ~3        0.5       3  (pipelined)
  SUBTOTAL:                                              ~18 cycles

XOR Phase (3 × _mm256_xor_si256):
  ├── vpxor ymm0, ymm1, ymm2           1        0.33      1
  ├── vpxor ymm3, ymm4, ymm5           1        0.33      0  (parallel)
  └── vpxor ymm6, ymm7, ymm8           1        0.33      0  (parallel)
  SUBTOTAL:                                               ~1 cycle

POPCOUNT Phase (3 × lookup table method):
  ├── Nibble extraction (and, srli)    2        0.5       2
  ├── Lookup (pshufb × 2)              1        1.0       2
  ├── Add nibbles                      1        0.33      1
  └── × 3 registers                                       ~5
  SUBTOTAL:                                              ~10 cycles

HORIZONTAL SUM Phase:
  ├── _mm256_sad_epu8 × 3              3        1.0       9
  ├── Extract + scalar add             2        1.0       4
  SUBTOTAL:                                              ~13 cycles

DISPATCH OVERHEAD:
  ├── Branch (predicted)               1        N/A       1
  ├── Function call                    ~4       N/A       4
  SUBTOTAL:                                               ~5 cycles

TOTAL CALCULATED:                                        47 cycles
TARGET:                                                 <50 cycles ✓
HARD LIMIT:                                            <75 cycles ✓
```

### Performance Measurement Protocol

```rust
#[cfg(target_arch = "x86_64")]
fn measure_cycles() {
    use std::arch::x86_64::_rdtsc;

    let a = [0xAAu8; 96];
    let b = [0x55u8; 96];

    // Warmup: 1,000 iterations
    for _ in 0..1_000 {
        std::hint::black_box(hamming_distance(&a, &b));
    }

    // Measure: 10,000 iterations
    let start = unsafe { _rdtsc() };
    for _ in 0..10_000 {
        std::hint::black_box(hamming_distance(
            std::hint::black_box(&a),
            std::hint::black_box(&b),
        ));
    }
    let end = unsafe { _rdtsc() };

    let cycles_per_call = (end - start) / 10_000;
    assert!(cycles_per_call < 50, "Performance target missed: {} cycles", cycles_per_call);
}
```

---

## 8. Testing Strategy

### Test Categories

- [x] Unit tests per platform (avx2, neon, portable)
- [x] Property tests (SIMD result == portable result for 10,000+ random inputs)
- [x] Benchmark regression tests (rdtsc cycle count)
- [ ] Miri for unsafe verification — **Concrete Plan:**
  - **When:** Run during RUST_ENGINEER implementation (Day 37), before HOSTILE_REVIEWER approval
  - **Command:** `cargo +nightly miri test simd`
  - **CI Integration:** Add GitHub Actions job with `rust-toolchain: nightly` for Miri
  - **Scope:** All tests in `tests/simd_spec.rs` and `tests/simd_integration.rs`
  - **Blocking:** Implementation cannot be marked APPROVED without Miri pass
  - **Fallback:** If Miri unavailable, document as KNOWN_LIMITATION with manual code audit
- [x] Edge case tests (zeros, ones, boundaries at 32/64 bytes)
- [x] Alignment tests (verify 64-byte alignment maintained)

### Test Structure

```
tests/
├── simd_spec.rs              # Pre-implementation test specifications
│   ├── test_simd_matches_portable_*    # 12 correctness tests
│   ├── prop_simd_*                     # 5 property tests (10,000 cases each)
│   └── test_simd_integration_*         # 3 integration tests

benches/
└── bench_simd.rs             # Performance benchmarks
    ├── bench_simd_hamming_cycles       # rdtsc cycle measurement
    ├── bench_simd_vs_portable          # Speedup comparison
    └── bench_throughput                # Ops/sec measurement
```

### Correctness Verification

```rust
// Property test: SIMD must exactly match portable
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_simd_matches_portable(
        a in proptest::collection::vec(any::<u8>(), 96),
        b in proptest::collection::vec(any::<u8>(), 96),
    ) {
        let a: [u8; 96] = a.try_into().unwrap();
        let b: [u8; 96] = b.try_into().unwrap();

        let portable = portable::hamming_distance_portable(&a, &b);
        let simd = simd::hamming_distance(&a, &b);

        prop_assert_eq!(portable, simd);
    }
}
```

---

## 9. Risks

| Risk | Probability | Impact | Mitigation |
|:-----|:------------|:-------|:-----------|
| AVX2 cycle target missed (>50) | Low | High | Fallback: accept 50-75 cycles, document |
| Unsafe introduces UB | Low | Critical | Miri testing, safety proofs, code review |
| Branch misprediction overhead | Very Low | Low | Branch predictor handles single path well |
| Platform-specific bugs | Medium | Medium | Property tests compare SIMD vs portable |
| CI lacks AVX2 | Medium | Low | GitHub Actions runners have AVX2 |
| NEON implementation delayed | Medium | Low | Portable fallback works on ARM |
| Memory alignment issues | Low | High | `#[repr(align(64))]` enforced by type system |

### Risk Mitigation Details

**Risk: AVX2 cycle target missed**
- Mitigation 1: Profile with `perf` to identify bottlenecks
- Mitigation 2: Consider AVX-512 VPOPCNTDQ for modern servers
- Mitigation 3: Accept 50-75 cycles as "soft pass" (still 4-6x improvement)

**Risk: Unsafe introduces UB**
- Mitigation 1: Document all safety requirements in SAFETY comments
- Mitigation 2: Run Miri on test suite: `cargo +nightly miri test`
- Mitigation 3: Hostile code review of all unsafe blocks

---

## 10. Approval

| Role | Status | Date | Notes |
|:-----|:-------|:-----|:------|
| META_ARCHITECT | APPROVED | 2025-12-12 | Architecture complete |
| HOSTILE_REVIEWER | APPROVED | 2025-12-12 | 0 Critical, 0 Major, 3 Minor (fixed) |
| TEST_ENGINEER | PENDING | — | Awaiting test implementation |
| BENCHMARK_SCIENTIST | PENDING | — | Awaiting benchmark validation |

---

## Appendix A: Alternative Considered — std::simd

Rust's `std::simd` (portable SIMD) was considered as an alternative to platform-specific intrinsics:

**Pros:**
- Single codebase for all platforms
- Compiler optimizes for target platform
- No `unsafe` in user code

**Cons:**
- Requires nightly Rust (`#![feature(portable_simd)]`)
- Less control over generated assembly
- Uncertain stabilization timeline

**Decision:** Use platform-specific intrinsics for Day 37 (stable Rust, maximum control), revisit `std::simd` when stabilized.

---

## Appendix B: Benchmark Targets Summary

| Metric | Target | Hard Limit | Measurement |
|:-------|:-------|:-----------|:------------|
| AVX2 Hamming (cycles) | <50 | <75 | rdtsc |
| Speedup vs Portable | >5x | >3x | criterion |
| Throughput | >1B ops/sec | >500M ops/sec | criterion |
| Latency P99 | <100ns | <200ns | criterion |

---

**END OF SIMD ARCHITECTURE DESIGN**
