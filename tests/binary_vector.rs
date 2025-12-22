//! Integration tests for variable-dimension BinaryVector.
//!
//! Tests cover:
//! - Multiple dimensions (128, 384, 768, 1024, 1536)
//! - Edge cases (NaN, Infinity, zero)
//! - Hamming distance invariants
//! - SIMD correctness

use edgevec::quantization::variable::{BinaryVector, QuantizationError};

mod quantization {
    use super::*;

    #[test]
    fn test_quantize_128d() {
        let v = vec![1.0f32; 128];
        let bv = BinaryVector::quantize(&v).unwrap();

        assert_eq!(bv.dimension(), 128);
        assert_eq!(bv.bytes(), 16);
        assert!(bv.data().iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_quantize_384d() {
        let v = vec![-1.0f32; 384];
        let bv = BinaryVector::quantize(&v).unwrap();

        assert_eq!(bv.dimension(), 384);
        assert_eq!(bv.bytes(), 48);
        assert!(bv.data().iter().all(|&b| b == 0x00));
    }

    #[test]
    fn test_quantize_768d() {
        let v: Vec<f32> = (0..768)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let bv = BinaryVector::quantize(&v).unwrap();

        assert_eq!(bv.dimension(), 768);
        assert_eq!(bv.bytes(), 96);
        // 0b01010101 = 0x55
        assert!(bv.data().iter().all(|&b| b == 0x55));
    }

    #[test]
    fn test_quantize_1024d() {
        let v = vec![1.0f32; 1024];
        let bv = BinaryVector::quantize(&v).unwrap();

        assert_eq!(bv.dimension(), 1024);
        assert_eq!(bv.bytes(), 128);
    }

    #[test]
    fn test_quantize_1536d() {
        let v = vec![1.0f32; 1536];
        let bv = BinaryVector::quantize(&v).unwrap();

        assert_eq!(bv.dimension(), 1536);
        assert_eq!(bv.bytes(), 192);
    }

    #[test]
    fn test_invalid_dimension() {
        let v = vec![1.0f32; 100]; // Not divisible by 8
        let result = BinaryVector::quantize(&v);

        assert!(matches!(
            result,
            Err(QuantizationError::InvalidDimension { dimension: 100 })
        ));
    }
}

mod hamming {
    use super::*;

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
        let v1: Vec<f32> = (0..768).map(|i| (i as f32).sin()).collect();
        let v2: Vec<f32> = (0..768).map(|i| (i as f32).cos()).collect();
        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();

        let d1 = bv1.hamming_distance(&bv2).unwrap();
        let d2 = bv2.hamming_distance(&bv1).unwrap();

        assert_eq!(d1, d2, "Hamming distance must be symmetric");
    }

    #[test]
    fn test_hamming_dimension_mismatch() {
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
    fn test_hamming_8d_detailed() {
        // Test with minimal dimension to verify bit ordering
        let v1 = vec![1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0]; // 0b01010101 = 0x55
        let v2 = vec![-1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0]; // 0b10101010 = 0xAA

        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();

        assert_eq!(bv1.data()[0], 0x55);
        assert_eq!(bv2.data()[0], 0xAA);
        assert_eq!(bv1.hamming_distance(&bv2).unwrap(), 8); // All bits differ
    }
}

mod similarity {
    use super::*;

    #[test]
    fn test_similarity_identical() {
        let v = vec![1.0f32; 128];
        let bv = BinaryVector::quantize(&v).unwrap();

        let sim = bv.similarity(&bv).unwrap();
        assert!((sim - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_similarity_opposite() {
        let v1 = vec![1.0f32; 128];
        let v2 = vec![-1.0f32; 128];
        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();

        let sim = bv1.similarity(&bv2).unwrap();
        assert!(sim.abs() < f32::EPSILON);
    }

    #[test]
    fn test_similarity_half() {
        // First half positive, second half negative
        let v1 = vec![1.0f32; 128];
        let v2: Vec<f32> = (0..128).map(|i| if i < 64 { 1.0 } else { -1.0 }).collect();

        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();

        let sim = bv1.similarity(&bv2).unwrap();
        assert!((sim - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_similarity_bounded() {
        let v1: Vec<f32> = (0..768).map(|i| (i as f32).sin()).collect();
        let v2: Vec<f32> = (0..768).map(|i| (i as f32).cos()).collect();

        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();

        let sim = bv1.similarity(&bv2).unwrap();

        assert!(sim >= 0.0, "Similarity must be >= 0");
        assert!(sim <= 1.0, "Similarity must be <= 1");
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn test_zero_vector() {
        let v = vec![0.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();

        // 0.0 > 0.0 is false, so all bits should be 0
        assert!(bv.data().iter().all(|&b| b == 0x00));
    }

    #[test]
    fn test_all_positive() {
        let v = vec![1.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();

        // All positive -> all bits should be 1
        assert!(bv.data().iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn test_all_negative() {
        let v = vec![-1.0f32; 768];
        let bv = BinaryVector::quantize(&v).unwrap();

        // All negative -> all bits should be 0
        assert!(bv.data().iter().all(|&b| b == 0x00));
    }

    #[test]
    fn test_alternating() {
        let v: Vec<f32> = (0..768)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let bv = BinaryVector::quantize(&v).unwrap();

        // Alternating -> 0b01010101 = 0x55
        assert!(bv.data().iter().all(|&b| b == 0x55));
    }

    #[test]
    fn test_nan_treated_as_non_positive() {
        let mut v = vec![1.0f32; 128];
        v[0] = f32::NAN;
        let bv = BinaryVector::quantize(&v).unwrap();

        // NaN > 0.0 is false, so bit 0 should be 0
        assert_eq!(bv.data()[0] & 0x01, 0);
    }

    #[test]
    fn test_infinity() {
        let mut v = vec![0.0f32; 128];
        v[0] = f32::INFINITY;
        v[1] = f32::NEG_INFINITY;

        let bv = BinaryVector::quantize(&v).unwrap();

        // +Inf > 0.0 is true (bit 0 = 1)
        // -Inf > 0.0 is false (bit 1 = 0)
        assert_eq!(bv.data()[0] & 0x01, 1);
        assert_eq!(bv.data()[0] & 0x02, 0);
    }

    #[test]
    fn test_negative_zero() {
        let mut v = vec![1.0f32; 128];
        v[0] = -0.0f32;
        let bv = BinaryVector::quantize(&v).unwrap();

        // -0.0 > 0.0 is false, so bit 0 should be 0
        assert_eq!(bv.data()[0] & 0x01, 0);
    }

    #[test]
    fn test_subnormal() {
        let mut v = vec![-1.0f32; 128];
        v[0] = f32::MIN_POSITIVE / 2.0; // Subnormal positive

        let bv = BinaryVector::quantize(&v).unwrap();

        // Subnormal positive > 0.0 is true
        assert_eq!(bv.data()[0] & 0x01, 1);
    }
}

mod determinism {
    use super::*;

    #[test]
    fn test_quantize_deterministic() {
        let v: Vec<f32> = (0..768).map(|i| (i as f32).sin()).collect();

        let bv1 = BinaryVector::quantize(&v).unwrap();
        let bv2 = BinaryVector::quantize(&v).unwrap();

        assert_eq!(bv1, bv2, "Quantization must be deterministic");
    }

    #[test]
    fn test_from_bytes_roundtrip() {
        let v = vec![1.0f32; 128];
        let bv1 = BinaryVector::quantize(&v).unwrap();

        let bv2 = BinaryVector::from_bytes(bv1.data().to_vec(), bv1.dimension()).unwrap();

        assert_eq!(bv1, bv2);
    }
}

mod simd_correctness {
    use super::*;
    use edgevec::simd::popcount::{scalar_popcount_xor, simd_popcount_xor};

    #[test]
    fn test_simd_matches_scalar_768d() {
        let v1: Vec<f32> = (0..768).map(|i| (i as f32).sin()).collect();
        let v2: Vec<f32> = (0..768).map(|i| (i as f32).cos()).collect();

        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();

        let simd_dist = simd_popcount_xor(bv1.data(), bv2.data());
        let scalar_dist = scalar_popcount_xor(bv1.data(), bv2.data());

        assert_eq!(
            simd_dist, scalar_dist,
            "SIMD and scalar must produce identical results"
        );
    }

    #[test]
    fn test_simd_matches_scalar_1536d() {
        let v1: Vec<f32> = (0..1536).map(|i| (i as f32).sin()).collect();
        let v2: Vec<f32> = (0..1536).map(|i| (i as f32).cos()).collect();

        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();

        let simd_dist = simd_popcount_xor(bv1.data(), bv2.data());
        let scalar_dist = scalar_popcount_xor(bv1.data(), bv2.data());

        assert_eq!(simd_dist, scalar_dist);
    }

    #[test]
    fn test_simd_matches_binaryvector_hamming() {
        let v1: Vec<f32> = (0..768).map(|i| (i as f32 * 0.1).sin()).collect();
        let v2: Vec<f32> = (0..768).map(|i| (i as f32 * 0.2).cos()).collect();

        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();

        // BinaryVector::hamming_distance uses simd_popcount_xor internally
        let bv_distance = bv1.hamming_distance(&bv2).unwrap();
        let direct_simd = simd_popcount_xor(bv1.data(), bv2.data());

        assert_eq!(bv_distance, direct_simd);
    }
}

mod triangle_inequality {
    use super::*;

    #[test]
    fn test_triangle_inequality() {
        let v1: Vec<f32> = (0..128).map(|i| (i as f32).sin()).collect();
        let v2: Vec<f32> = (0..128).map(|i| (i as f32).cos()).collect();
        let v3: Vec<f32> = (0..128).map(|i| (i as f32 * 0.5).sin()).collect();

        let bv1 = BinaryVector::quantize(&v1).unwrap();
        let bv2 = BinaryVector::quantize(&v2).unwrap();
        let bv3 = BinaryVector::quantize(&v3).unwrap();

        let d12 = bv1.hamming_distance(&bv2).unwrap();
        let d23 = bv2.hamming_distance(&bv3).unwrap();
        let d13 = bv1.hamming_distance(&bv3).unwrap();

        assert!(
            d13 <= d12 + d23,
            "Triangle inequality violated: {} > {} + {}",
            d13,
            d12,
            d23
        );
    }
}
