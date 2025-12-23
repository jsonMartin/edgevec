# Week 27 Day 1: Variable-Dimension BQ + SIMD Popcount

**Date:** 2025-12-22
**Focus:** Generalize binary quantization and add SIMD popcount
**Estimated Duration:** 12 hours
**Phase:** RFC-002 Implementation Phase 2 (Binary Quantization)

---

## Tasks

### W27.1.1: Generalize BinaryQuantizer for Variable Dimensions

**Objective:** Extend binary quantization to support any dimension divisible by 8.

**Current State:**
- `QuantizedVector` is fixed at 768 dimensions (96 bytes)
- `BinaryQuantizer::quantize()` panics if dimension != 768

**Target State:**
- `BinaryVector` supports any dimension ≥ 8 where dimension % 8 == 0
- Common dimensions: 128, 384, 768, 1024, 1536

**Acceptance Criteria:**
- [ ] `BinaryVector::quantize(vector: &[f32]) -> Result<Self, QuantizationError>`
- [ ] Returns error if dimension is not divisible by 8
- [ ] Dimension stored as field for runtime validation
- [ ] `BinaryVector::dimension() -> usize` accessor
- [ ] `BinaryVector::bytes() -> usize` returns dimension / 8
- [ ] Existing 768D tests still pass
- [ ] New tests for 128, 384, 1024, 1536 dimensions

**Files:**
- `src/quantization/variable.rs` (new file)
- `src/quantization/mod.rs` (add module)
- `tests/binary_vector.rs` (new file)

**Estimated Duration:** 3 hours

**Agent:** RUST_ENGINEER

**API Design:**

```rust
/// Error type for quantization operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuantizationError {
    /// Dimension must be divisible by 8.
    InvalidDimension { dimension: usize },
    /// Dimension mismatch between vectors.
    DimensionMismatch { expected: usize, actual: usize },
}

/// Variable-dimension binary quantized vector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryVector {
    data: Vec<u8>,
    dimension: usize,
}

impl BinaryVector {
    /// Quantizes a vector to binary representation.
    ///
    /// # Errors
    /// Returns `QuantizationError::InvalidDimension` if dimension % 8 != 0.
    pub fn quantize(vector: &[f32]) -> Result<Self, QuantizationError>;

    /// Creates from raw bytes with known dimension.
    ///
    /// # Errors
    /// Returns `QuantizationError::InvalidDimension` if dimension % 8 != 0
    /// or if data.len() != dimension / 8.
    pub fn from_bytes(data: Vec<u8>, dimension: usize) -> Result<Self, QuantizationError>;

    /// Returns the dimension of this vector.
    pub fn dimension(&self) -> usize;

    /// Returns the size in bytes.
    pub fn bytes(&self) -> usize;

    /// Returns the underlying binary data.
    pub fn data(&self) -> &[u8];

    /// Computes Hamming distance to another vector.
    ///
    /// # Errors
    /// Returns `QuantizationError::DimensionMismatch` if dimensions differ.
    pub fn hamming_distance(&self, other: &Self) -> Result<u32, QuantizationError>;

    /// Returns similarity score [0, 1] based on Hamming distance.
    ///
    /// # Errors
    /// Returns `QuantizationError::DimensionMismatch` if dimensions differ.
    pub fn similarity(&self, other: &Self) -> Result<f32, QuantizationError>;
}
```

**Dependencies:** None

---

### W27.1.2: SIMD Popcount for x86 (SSE4.1/AVX2)

**Objective:** Implement hardware-accelerated popcount using x86 SIMD.

**Current State:**
- `hamming_distance()` uses `crate::quantization::simd::hamming_distance()`
- Not clear if current implementation uses hardware popcount

**Target State:**
- x86_64 with `popcnt` uses native instruction
- AVX2 available: parallel popcount on 32 bytes at a time
- SSE4.1 fallback: 16-byte chunks
- Scalar fallback for older CPUs

**Acceptance Criteria:**
- [ ] `simd_popcount_xor(a: &[u8], b: &[u8]) -> u32` function
- [ ] Runtime detection: AVX2 > popcnt > scalar
- [ ] Unit test: SIMD result equals scalar result
- [ ] Benchmark: >2x speedup vs scalar on AVX2

**Files:**
- `src/simd/popcount.rs` (new file)
- `src/simd/mod.rs` (add module)

**Estimated Duration:** 3 hours

**Agent:** RUST_ENGINEER

**Implementation Notes:**

```rust
// src/simd/popcount.rs

/// Computes popcount of XOR between two byte slices.
///
/// Uses the fastest available implementation:
/// - AVX2: 32-byte parallel processing
/// - popcnt: Native instruction per u64
/// - Scalar: Lookup table fallback
pub fn simd_popcount_xor(a: &[u8], b: &[u8]) -> u32 {
    debug_assert_eq!(a.len(), b.len());

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { avx2_popcount_xor(a, b) };
        }
        if is_x86_feature_detected!("popcnt") {
            return native_popcount_xor(a, b);
        }
    }

    scalar_popcount_xor(a, b)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn avx2_popcount_xor(a: &[u8], b: &[u8]) -> u32 {
    // Use SIMD XOR + parallel popcount
    // Process 32 bytes at a time with _mm256 intrinsics
    ...
}

#[cfg(target_arch = "x86_64")]
fn native_popcount_xor(a: &[u8], b: &[u8]) -> u32 {
    // Use native popcnt instruction on u64 chunks
    let mut count = 0u32;
    for (chunk_a, chunk_b) in a.chunks_exact(8).zip(b.chunks_exact(8)) {
        let va = u64::from_le_bytes(chunk_a.try_into().unwrap());
        let vb = u64::from_le_bytes(chunk_b.try_into().unwrap());
        count += (va ^ vb).count_ones();
    }
    // Handle remainder...
    count
}

fn scalar_popcount_xor(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}
```

**Dependencies:** W27.1.1 (for BinaryVector integration)

---

### W27.1.3: SIMD Popcount for ARM (NEON)

**Objective:** Implement hardware-accelerated popcount using ARM NEON.

**Current State:**
- NEON detection exists in `src/simd/mod.rs`
- No NEON popcount implementation

**Target State:**
- ARM64 with NEON uses `vcntq_u8` for parallel popcount
- 16-byte chunks processed in parallel
- Fallback to scalar on non-NEON ARM

**Acceptance Criteria:**
- [ ] `neon_popcount_xor(a: &[u8], b: &[u8]) -> u32` function
- [ ] Uses `vcntq_u8` intrinsic
- [ ] Unit test: NEON result equals scalar result
- [ ] Compiles on ARM64 (CI matrix)

**Files:**
- `src/simd/popcount.rs` (extend with NEON)
- `src/simd/neon.rs` (add popcount support)

**Estimated Duration:** 3 hours

**Agent:** RUST_ENGINEER

**Implementation Notes:**

```rust
#[cfg(target_arch = "aarch64")]
pub fn simd_popcount_xor(a: &[u8], b: &[u8]) -> u32 {
    if std::arch::is_aarch64_feature_detected!("neon") {
        return unsafe { neon_popcount_xor(a, b) };
    }
    scalar_popcount_xor(a, b)
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn neon_popcount_xor(a: &[u8], b: &[u8]) -> u32 {
    use std::arch::aarch64::*;

    let mut total = 0u32;

    // Process 16 bytes at a time
    for (chunk_a, chunk_b) in a.chunks_exact(16).zip(b.chunks_exact(16)) {
        let va = vld1q_u8(chunk_a.as_ptr());
        let vb = vld1q_u8(chunk_b.as_ptr());
        let xor = veorq_u8(va, vb);
        let cnt = vcntq_u8(xor); // Popcount per byte
        // Sum horizontally
        let sum8 = vpaddlq_u8(cnt);    // u8x16 -> u16x8
        let sum16 = vpaddlq_u16(sum8); // u16x8 -> u32x4
        let sum32 = vpaddlq_u32(sum16); // u32x4 -> u64x2
        // Extract final sum
        total += (vgetq_lane_u64(sum32, 0) + vgetq_lane_u64(sum32, 1)) as u32;
    }

    // Handle remainder
    ...

    total
}
```

**Dependencies:** W27.1.1 (for BinaryVector integration)

---

### W27.1.4: Integration and Unit Tests

**Objective:** Wire up SIMD popcount to BinaryVector and write tests.

**Acceptance Criteria:**
- [ ] `BinaryVector::hamming_distance()` uses `simd_popcount_xor()`
- [ ] Unit tests verify SIMD equals scalar for all dimensions
- [ ] Property test: Hamming distance is symmetric
- [ ] Property test: Hamming distance to self is 0
- [ ] Benchmark: SIMD vs scalar speedup

**Files:**
- `tests/binary_vector.rs` (tests)
- `benches/bq_popcount.rs` (new benchmark)

**Estimated Duration:** 3 hours

**Agent:** RUST_ENGINEER + TEST_ENGINEER

**Test Cases:**

```rust
// tests/binary_vector.rs

mod quantization {
    use edgevec::quantization::variable::BinaryVector;

    #[test]
    fn test_quantize_768d() {
        let v = vec![1.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();
        assert_eq!(bv.dimension(), 768);
        assert_eq!(bv.bytes(), 96);
        // All positive -> all bits set
        assert!(bv.data().iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_quantize_128d() {
        let v = vec![-1.0f32; 128];
        let bv = BinaryVector::quantize(&v).unwrap();
        assert_eq!(bv.dimension(), 128);
        assert_eq!(bv.bytes(), 16);
        // All negative -> all bits zero
        assert!(bv.data().iter().all(|&b| b == 0x00));
    }

    #[test]
    fn test_invalid_dimension() {
        let v = vec![1.0f32; 100]; // Not divisible by 8
        let result = BinaryVector::quantize(&v);
        assert!(result.is_err());
    }
}

mod hamming {
    use edgevec::quantization::variable::BinaryVector;

    #[test]
    fn test_hamming_identical() {
        let v = vec![1.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();
        assert_eq!(bv.hamming_distance(&bv).unwrap(), 0);
    }

    #[test]
    fn test_hamming_opposite() {
        let v1 = vec![1.0f32; 768];
        let v2 = vec![-1.0f32; 768];
        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();
        assert_eq!(bv1.hamming_distance(&bv2).unwrap(), 768);
    }

    #[test]
    fn test_hamming_symmetric() {
        let v1 = vec![1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0]; // 8D
        let v2 = vec![-1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0]; // 8D
        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();
        assert_eq!(
            bv1.hamming_distance(&bv2).unwrap(),
            bv2.hamming_distance(&bv1).unwrap()
        );
    }
}
```

**Dependencies:** W27.1.1, W27.1.2, W27.1.3

---

## Day 1 Checklist

- [ ] W27.1.1: Variable-dimension BinaryVector implemented
- [ ] W27.1.2: x86 SIMD popcount (SSE4.1/AVX2/popcnt)
- [ ] W27.1.3: ARM NEON popcount
- [ ] W27.1.4: Integration + unit tests
- [ ] All existing tests pass (`cargo test`)
- [ ] New tests pass (`cargo test binary_vector`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Formatted (`cargo fmt --check`)

## Day 1 Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | CI green |
| Variable dimension works | Tests with 128, 384, 768, 1024 |
| SIMD popcount works | Benchmark shows speedup |
| No new clippy warnings | `cargo clippy` clean |

## Day 1 Handoff

After completing Day 1:

**Artifacts Generated:**
- `src/quantization/variable.rs`
- `src/simd/popcount.rs`
- `tests/binary_vector.rs`
- `benches/bq_popcount.rs`

**Status:** PENDING_DAY_2

**Next:** Day 2 — BinaryVectorStorage implementation

---

*Agent: RUST_ENGINEER*
*Status: [APPROVED] (2025-12-21)*

## Revision Notes (2025-12-21)

### Changes Made

**[C1] Fixed: Removed `unwrap()` from library code**
- File: `src/simd/popcount.rs:192-193`
- Change: Replaced `try_into().unwrap()` with explicit array construction
- Rationale: Violated "No `unwrap()` in library code" rule

**[M1] Fixed: Created benchmark file**
- File: `benches/bq_popcount.rs`
- Content: SIMD vs scalar popcount benchmarks for all dimensions
- Added to `Cargo.toml` [[bench]] section
