//! Property-based tests for Binary Quantization (W27.5.2).
//!
//! These tests verify fundamental invariants of BQ operations:
//! 1. Hamming distance is symmetric
//! 2. Hamming distance to self is 0
//! 3. Triangle inequality holds
//! 4. Quantization is deterministic
//! 5. Dimension is preserved
//! 6. Similarity is bounded [0, 1]
//!
//! Run with: `cargo test --test bq_proptests`

use edgevec::quantization::variable::{BinaryVector, QuantizationError};
use edgevec::simd::popcount::{scalar_popcount_xor, simd_popcount_xor};
use proptest::prelude::*;

// ============================================================================
// STRATEGY GENERATORS
// ============================================================================

/// Generate a random f32 vector of specified dimension.
/// Uses proper [-1, 1] range for BQ quantization.
fn random_f32_vector(dim: usize) -> impl Strategy<Value = Vec<f32>> {
    proptest::collection::vec(-1.0f32..1.0f32, dim)
}

/// Generate a dimension divisible by 8 (required by BQ).
fn valid_dimension() -> impl Strategy<Value = usize> {
    (1usize..=192).prop_map(|x| x * 8) // 8 to 1536 in steps of 8
}

/// Generate a random byte vector of specified length.
fn random_bytes(len: usize) -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), len)
}

// ============================================================================
// HAMMING DISTANCE PROPERTY TESTS
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Hamming distance is symmetric.
    /// d(a, b) == d(b, a)
    #[test]
    fn prop_hamming_symmetric(
        a in random_f32_vector(768),
        b in random_f32_vector(768),
    ) {
        let bva = BinaryVector::quantize(&a).unwrap();
        let bvb = BinaryVector::quantize(&b).unwrap();

        let d_ab = bva.hamming_distance(&bvb).unwrap();
        let d_ba = bvb.hamming_distance(&bva).unwrap();

        prop_assert_eq!(d_ab, d_ba, "Hamming distance should be symmetric");
    }

    /// Property: Hamming distance to self is zero.
    /// d(a, a) == 0
    #[test]
    fn prop_hamming_identity(v in random_f32_vector(768)) {
        let bv = BinaryVector::quantize(&v).unwrap();
        let d = bv.hamming_distance(&bv).unwrap();

        prop_assert_eq!(d, 0, "Hamming distance to self should be 0");
    }

    /// Property: Triangle inequality holds.
    /// d(a, c) <= d(a, b) + d(b, c)
    #[test]
    fn prop_triangle_inequality(
        a in random_f32_vector(768),
        b in random_f32_vector(768),
        c in random_f32_vector(768),
    ) {
        let bva = BinaryVector::quantize(&a).unwrap();
        let bvb = BinaryVector::quantize(&b).unwrap();
        let bvc = BinaryVector::quantize(&c).unwrap();

        let d_ab = bva.hamming_distance(&bvb).unwrap();
        let d_bc = bvb.hamming_distance(&bvc).unwrap();
        let d_ac = bva.hamming_distance(&bvc).unwrap();

        prop_assert!(
            d_ac <= d_ab + d_bc,
            "Triangle inequality violated: d(a,c)={} > d(a,b)={} + d(b,c)={}",
            d_ac, d_ab, d_bc
        );
    }

    /// Property: Hamming distance is bounded by dimension.
    /// 0 <= d(a, b) <= dimension
    #[test]
    fn prop_hamming_bounded(
        a in random_f32_vector(768),
        b in random_f32_vector(768),
    ) {
        let bva = BinaryVector::quantize(&a).unwrap();
        let bvb = BinaryVector::quantize(&b).unwrap();

        let distance = bva.hamming_distance(&bvb).unwrap();

        prop_assert!(distance <= 768, "Distance {} exceeds dimension 768", distance);
    }
}

// ============================================================================
// QUANTIZATION PROPERTY TESTS
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Quantization is deterministic.
    /// quantize(v) == quantize(v)
    #[test]
    fn prop_quantize_deterministic(v in random_f32_vector(768)) {
        let bv1 = BinaryVector::quantize(&v).unwrap();
        let bv2 = BinaryVector::quantize(&v).unwrap();

        prop_assert_eq!(bv1, bv2, "Quantization should be deterministic");
    }

    /// Property: Dimension is preserved.
    /// bv.dimension() == v.len()
    #[test]
    fn prop_dimension_preserved(dim in valid_dimension()) {
        let v: Vec<f32> = (0..dim).map(|i| (i as f32).sin()).collect();
        let bv = BinaryVector::quantize(&v).unwrap();

        prop_assert_eq!(bv.dimension(), dim, "Dimension should be preserved");
        prop_assert_eq!(bv.bytes(), dim / 8, "Byte count should be dimension / 8");
    }

    /// Property: Invalid dimensions are rejected.
    #[test]
    fn prop_invalid_dimension_rejected(dim in 1usize..1000) {
        prop_assume!(dim % 8 != 0); // Only test non-divisible-by-8

        let v: Vec<f32> = (0..dim).map(|i| i as f32).collect();
        let result = BinaryVector::quantize(&v);

        prop_assert!(
            matches!(result, Err(QuantizationError::InvalidDimension { .. })),
            "Should reject dimension {} not divisible by 8",
            dim
        );
    }
}

// ============================================================================
// SIMILARITY PROPERTY TESTS
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Similarity is bounded [0, 1].
    #[test]
    fn prop_similarity_bounded(
        a in random_f32_vector(768),
        b in random_f32_vector(768),
    ) {
        let bva = BinaryVector::quantize(&a).unwrap();
        let bvb = BinaryVector::quantize(&b).unwrap();

        let sim = bva.similarity(&bvb).unwrap();

        prop_assert!(sim >= 0.0, "Similarity {} should be >= 0", sim);
        prop_assert!(sim <= 1.0, "Similarity {} should be <= 1", sim);
    }

    /// Property: Identical vectors have similarity 1.0.
    #[test]
    fn prop_similarity_identity(v in random_f32_vector(768)) {
        let bv = BinaryVector::quantize(&v).unwrap();
        let sim = bv.similarity(&bv).unwrap();

        prop_assert!(
            (sim - 1.0).abs() < f32::EPSILON,
            "Similarity to self should be 1.0, got {}",
            sim
        );
    }

    /// Property: Similarity is symmetric.
    /// sim(a, b) == sim(b, a)
    #[test]
    fn prop_similarity_symmetric(
        a in random_f32_vector(768),
        b in random_f32_vector(768),
    ) {
        let bva = BinaryVector::quantize(&a).unwrap();
        let bvb = BinaryVector::quantize(&b).unwrap();

        let sim_ab = bva.similarity(&bvb).unwrap();
        let sim_ba = bvb.similarity(&bva).unwrap();

        prop_assert!(
            (sim_ab - sim_ba).abs() < f32::EPSILON,
            "Similarity should be symmetric: {} != {}",
            sim_ab, sim_ba
        );
    }
}

// ============================================================================
// SIMD POPCOUNT PROPERTY TESTS
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property: SIMD popcount matches scalar popcount.
    #[test]
    fn prop_simd_matches_scalar(
        a in random_bytes(96),
        b in random_bytes(96),
    ) {
        let simd_result = simd_popcount_xor(&a, &b);
        let scalar_result = scalar_popcount_xor(&a, &b);

        prop_assert_eq!(
            simd_result, scalar_result,
            "SIMD and scalar popcount should match"
        );
    }

    /// Property: Popcount is symmetric.
    /// popcount(a XOR b) == popcount(b XOR a)
    #[test]
    fn prop_popcount_symmetric(
        a in random_bytes(96),
        b in random_bytes(96),
    ) {
        let ab = simd_popcount_xor(&a, &b);
        let ba = simd_popcount_xor(&b, &a);

        prop_assert_eq!(ab, ba, "XOR popcount should be symmetric");
    }

    /// Property: Popcount of identical slices is zero.
    #[test]
    fn prop_popcount_identity(a in random_bytes(96)) {
        let result = simd_popcount_xor(&a, &a);

        prop_assert_eq!(result, 0, "XOR of identical slices should be 0");
    }

    /// Property: Popcount is bounded by bit count.
    #[test]
    fn prop_popcount_bounded(
        a in random_bytes(96),
        b in random_bytes(96),
    ) {
        let result = simd_popcount_xor(&a, &b);
        let max_bits = (a.len() * 8) as u32;

        prop_assert!(
            result <= max_bits,
            "Popcount {} exceeds max bits {}",
            result, max_bits
        );
    }

    /// Property: SIMD popcount works for various lengths.
    #[test]
    fn prop_simd_various_lengths(len in 1usize..512) {
        let a: Vec<u8> = (0..len).map(|i| (i % 256) as u8).collect();
        let b: Vec<u8> = (0..len).map(|i| ((i * 2) % 256) as u8).collect();

        let simd_result = simd_popcount_xor(&a, &b);
        let scalar_result = scalar_popcount_xor(&a, &b);

        prop_assert_eq!(
            simd_result, scalar_result,
            "SIMD and scalar should match for length {}",
            len
        );
    }
}

// ============================================================================
// EDGE CASE TESTS (Non-property-based)
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_zero_vector_all_bits_zero() {
        let v = vec![0.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();
        // 0.0 > 0.0 is false, so all bits should be 0
        assert!(bv.data().iter().all(|&b| b == 0x00));
    }

    #[test]
    fn test_all_positive_all_bits_set() {
        let v = vec![1.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();
        // All positive -> all bits should be 1
        assert!(bv.data().iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_all_negative_all_bits_zero() {
        let v = vec![-1.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();
        // All negative -> all bits should be 0
        assert!(bv.data().iter().all(|&b| b == 0x00));
    }

    #[test]
    fn test_alternating_pattern() {
        let v: Vec<f32> = (0..768)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let bv = BinaryVector::quantize(&v).unwrap();
        // Alternating pattern: bit 0=1, bit 1=0, ... -> 0b01010101 = 0x55
        assert!(bv.data().iter().all(|&b| b == 0x55));
    }

    #[test]
    fn test_nan_treated_as_non_positive() {
        let mut v = vec![1.0f32; 768];
        v[0] = f32::NAN;
        let bv = BinaryVector::quantize(&v).unwrap();
        // NAN > 0.0 is false, so bit 0 should be 0
        // Bit 0 is the least significant bit of byte 0
        assert_eq!(bv.data()[0] & 0x01, 0);
    }

    #[test]
    fn test_positive_infinity() {
        let mut v = vec![0.0f32; 768];
        v[0] = f32::INFINITY;
        let bv = BinaryVector::quantize(&v).unwrap();
        // +Inf > 0.0 is true, so bit 0 should be 1
        assert_eq!(bv.data()[0] & 0x01, 1);
    }

    #[test]
    fn test_negative_infinity() {
        let mut v = vec![0.0f32; 768];
        v[0] = f32::NEG_INFINITY;
        let bv = BinaryVector::quantize(&v).unwrap();
        // -Inf > 0.0 is false, so bit 0 should be 0
        assert_eq!(bv.data()[0] & 0x01, 0);
    }

    #[test]
    fn test_subnormal_positive() {
        let mut v = vec![0.0f32; 768];
        v[0] = f32::MIN_POSITIVE / 2.0; // Subnormal positive
        let bv = BinaryVector::quantize(&v).unwrap();
        // Subnormal positive > 0.0 is true
        assert_eq!(bv.data()[0] & 0x01, 1);
    }

    #[test]
    fn test_dimension_mismatch_error() {
        let v1 = vec![1.0f32; 128];
        let v2 = vec![1.0f32; 256];
        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();

        let result = bv1.hamming_distance(&bv2);
        assert!(matches!(
            result,
            Err(QuantizationError::DimensionMismatch {
                expected: 128,
                actual: 256
            })
        ));
    }

    #[test]
    fn test_from_bytes_valid() {
        let data = vec![0xFF; 96];
        let bv = BinaryVector::from_bytes(data, 768).unwrap();
        assert_eq!(bv.dimension(), 768);
        assert_eq!(bv.bytes(), 96);
    }

    #[test]
    fn test_from_bytes_length_mismatch() {
        let data = vec![0xFF; 96];
        let result = BinaryVector::from_bytes(data, 1024);
        assert!(matches!(
            result,
            Err(QuantizationError::ByteLengthMismatch {
                expected: 128,
                actual: 96
            })
        ));
    }
}

// ============================================================================
// MINIMUM DIMENSION TESTS
// ============================================================================

mod minimum_dimensions {
    use super::*;

    #[test]
    fn test_8d_minimum() {
        let v = vec![1.0f32; 8];
        let bv = BinaryVector::quantize(&v).unwrap();
        assert_eq!(bv.dimension(), 8);
        assert_eq!(bv.bytes(), 1);
    }

    #[test]
    fn test_16d() {
        let v = vec![1.0f32; 16];
        let bv = BinaryVector::quantize(&v).unwrap();
        assert_eq!(bv.dimension(), 16);
        assert_eq!(bv.bytes(), 2);
    }

    #[test]
    fn test_128d() {
        let v = vec![1.0f32; 128];
        let bv = BinaryVector::quantize(&v).unwrap();
        assert_eq!(bv.dimension(), 128);
        assert_eq!(bv.bytes(), 16);
    }

    #[test]
    fn test_384d() {
        let v = vec![1.0f32; 384];
        let bv = BinaryVector::quantize(&v).unwrap();
        assert_eq!(bv.dimension(), 384);
        assert_eq!(bv.bytes(), 48);
    }

    #[test]
    fn test_768d() {
        let v = vec![1.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();
        assert_eq!(bv.dimension(), 768);
        assert_eq!(bv.bytes(), 96);
    }

    #[test]
    fn test_1024d() {
        let v = vec![1.0f32; 1024];
        let bv = BinaryVector::quantize(&v).unwrap();
        assert_eq!(bv.dimension(), 1024);
        assert_eq!(bv.bytes(), 128);
    }

    #[test]
    fn test_1536d() {
        let v = vec![1.0f32; 1536];
        let bv = BinaryVector::quantize(&v).unwrap();
        assert_eq!(bv.dimension(), 1536);
        assert_eq!(bv.bytes(), 192);
    }
}
