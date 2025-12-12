//! Hamming distance metric.

use super::Metric;

/// Hamming distance metric.
///
/// Calculates the number of differing bits between two binary vectors.
///
/// # Attribution
///
/// Adapted from `binary_semantic_cache` v1.0 (MIT License)
/// Copyright (c) 2024 Matteo Panzeri
/// Original: <https://github.com/mp-monitor/binary_semantic_cache>
#[derive(Debug, Clone, Copy, Default)]
pub struct Hamming;

impl Metric<u8> for Hamming {
    #[inline]
    fn distance(a: &[u8], b: &[u8]) -> f32 {
        assert_eq!(
            a.len(),
            b.len(),
            "dimension mismatch: {} != {}",
            a.len(),
            b.len()
        );

        let mut distance: u32 = 0;
        for (x, y) in a.iter().zip(b.iter()) {
            // SALVAGE: Adapted from binary_semantic_cache similarity.rs
            distance += (x ^ y).count_ones();
        }

        // Precision loss is acceptable because max distance for expected vector sizes
        // (< 1MB) fits within f32 mantissa (2^24).
        #[allow(clippy::cast_precision_loss)]
        {
            distance as f32
        }
    }
}
