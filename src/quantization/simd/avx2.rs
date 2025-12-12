//! AVX2-accelerated Hamming distance for x86_64.
//!
//! Provides SIMD implementation using Intel AVX2 instructions (256-bit registers).
//!
//! # CPU Requirements
//!
//! - AVX2 support (Intel Haswell 2013+, AMD Excavator 2015+)
//! - Runtime detection via `is_x86_feature_detected!("avx2")`
//!
//! # Performance Target
//!
//! <50 CPU cycles per comparison (vs ~300 cycles portable)
//!
//! # Algorithm
//!
//! 1. Load 96 bytes in 3 × 256-bit YMM registers
//! 2. XOR corresponding registers to find differing bits
//! 3. Population count using lookup table method (AVX2 lacks native popcount)
//! 4. Horizontal sum across all registers
//!
//! # Safety
//!
//! All functions in this module are marked `unsafe` and require:
//! 1. AVX2 CPU feature verified by caller
//! 2. Input arrays are exactly 96 bytes
//! 3. No undefined behavior in pointer arithmetic

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m256i, _mm256_add_epi8, _mm256_and_si256, _mm256_extract_epi64, _mm256_loadu_si256,
    _mm256_sad_epu8, _mm256_set1_epi8, _mm256_setr_epi8, _mm256_setzero_si256, _mm256_shuffle_epi8,
    _mm256_srli_epi16, _mm256_xor_si256,
};

/// AVX2-accelerated Hamming distance for 96-byte binary vectors.
///
/// # Safety
///
/// Caller MUST ensure:
/// 1. AVX2 is available (`is_x86_feature_detected!("avx2")` returned true)
/// 2. Both arrays are valid `[u8; 96]` arrays
/// 3. No aliasing violations (enforced by Rust's borrow checker)
///
/// These invariants are enforced by the public API in `simd/mod.rs`.
///
/// # Algorithm
///
/// 1. Load 96 bytes in 3 × 256-bit registers:
///    - Register 0: bytes [0..32)
///    - Register 1: bytes [32..64)
///    - Register 2: bytes [64..96)
/// 2. XOR: `vpxor ymm_a, ymm_b` → differing bits become 1
/// 3. Popcount: Lookup table method (AVX2 has no native popcount)
/// 4. Horizontal sum: Sum all partial popcounts
///
/// # Performance
///
/// Target: <50 CPU cycles
///
/// # Arguments
///
/// * `a` - First 96-byte array (768 bits), 64-byte aligned
/// * `b` - Second 96-byte array (768 bits), 64-byte aligned
///
/// # Returns
///
/// Number of differing bits (0..=768)
#[target_feature(enable = "avx2")]
#[cfg(target_arch = "x86_64")]
#[allow(clippy::cast_ptr_alignment)] // _mm256_loadu_si256 is designed for unaligned access
pub(crate) unsafe fn hamming_distance_avx2(a: &[u8; 96], b: &[u8; 96]) -> u32 {
    // SAFETY: Caller verified AVX2 is available.
    // Array size (96 bytes) allows loads at offsets 0, 32, 64.
    // QuantizedVector guarantees 64-byte alignment, which exceeds AVX2's 32-byte requirement.

    // Load 96 bytes in 3 × 256-bit registers
    // Using _mm256_loadu_si256 (unaligned load) for safety,
    // though QuantizedVector is 64-byte aligned
    let a0 = _mm256_loadu_si256(a.as_ptr().cast::<__m256i>());
    let a1 = _mm256_loadu_si256(a.as_ptr().add(32).cast::<__m256i>());
    let a2 = _mm256_loadu_si256(a.as_ptr().add(64).cast::<__m256i>());

    let b0 = _mm256_loadu_si256(b.as_ptr().cast::<__m256i>());
    let b1 = _mm256_loadu_si256(b.as_ptr().add(32).cast::<__m256i>());
    let b2 = _mm256_loadu_si256(b.as_ptr().add(64).cast::<__m256i>());

    // XOR to find differing bits
    let xor0 = _mm256_xor_si256(a0, b0);
    let xor1 = _mm256_xor_si256(a1, b1);
    let xor2 = _mm256_xor_si256(a2, b2);

    // Population count for each register
    // AVX2 doesn't have native popcount, so we use lookup table method
    let pop0 = popcount_avx2(xor0);
    let pop1 = popcount_avx2(xor1);
    let pop2 = popcount_avx2(xor2);

    // Sum all popcounts
    pop0 + pop1 + pop2
}

/// AVX2 population count using lookup table method.
///
/// # Algorithm
///
/// AVX2 doesn't provide native popcount (AVX-512 has VPOPCNTDQ).
/// We use the SSSE3 PSHUFB-based lookup table technique:
///
/// 1. Split each byte into low/high nibbles (4 bits each)
/// 2. Use PSHUFB to look up popcount for each nibble in a 16-entry table
/// 3. Add low + high nibble counts to get full byte popcount
/// 4. Horizontal sum all byte counts
///
/// # Safety
///
/// This function is safe to call after `is_x86_feature_detected!("avx2")`.
///
/// # Arguments
///
/// * `v` - 256-bit register containing XOR result
///
/// # Returns
///
/// Sum of all set bits in the register (0..=256)
#[target_feature(enable = "avx2")]
#[inline]
#[cfg(target_arch = "x86_64")]
unsafe fn popcount_avx2(v: __m256i) -> u32 {
    // SAFETY: AVX2 feature is enabled via #[target_feature]

    // Lookup table: popcount for nibbles 0-15
    // Index: nibble value, Value: number of 1 bits
    // [0]=0, [1]=1, [2]=1, [3]=2, ..., [15]=4
    let lookup = _mm256_setr_epi8(
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, // Low 128 bits
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, // High 128 bits
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

    // Horizontal sum: reduce 32 bytes to single u32
    horizontal_sum_avx2(popcnt)
}

/// Horizontal sum of bytes in a 256-bit AVX2 register.
///
/// # Algorithm
///
/// 1. Use _mm256_sad_epu8 (Sum of Absolute Differences) against zero
///    This gives us 4 × 64-bit partial sums (one per 64-bit lane)
/// 2. Extract all four 64-bit values
/// 3. Add them together to get final sum
///
/// # Safety
///
/// This function is safe to call after `is_x86_feature_detected!("avx2")`.
///
/// # Arguments
///
/// * `v` - 256-bit register containing byte values to sum
///
/// # Returns
///
/// Sum of all bytes in the register (0..=8160 for max input 0xFF×32)
#[target_feature(enable = "avx2")]
#[inline]
#[cfg(target_arch = "x86_64")]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // _mm256_sad_epu8 returns unsigned values ≤8160
unsafe fn horizontal_sum_avx2(v: __m256i) -> u32 {
    // SAFETY: AVX2 feature is enabled via #[target_feature]

    // SAD against zero gives sum of absolute values
    // Since all values are unsigned, this is just a sum
    // Result: 4 × 64-bit partial sums in the register
    let zero = _mm256_setzero_si256();
    let sad = _mm256_sad_epu8(v, zero);

    // Extract the 4 × 64-bit partial sums
    // _mm256_sad_epu8 places sums at specific positions in the register
    // The values are guaranteed to fit in u32 (max 8 × 255 = 2040 per lane)
    let sum0 = _mm256_extract_epi64(sad, 0) as u32;
    let sum1 = _mm256_extract_epi64(sad, 1) as u32;
    let sum2 = _mm256_extract_epi64(sad, 2) as u32;
    let sum3 = _mm256_extract_epi64(sad, 3) as u32;

    // Add all partial sums
    sum0 + sum1 + sum2 + sum3
}

#[cfg(test)]
#[cfg(target_arch = "x86_64")]
mod tests {
    use super::*;

    // Helper to check if AVX2 is available
    fn avx2_available() -> bool {
        is_x86_feature_detected!("avx2")
    }

    #[test]
    fn test_avx2_identical() {
        if !avx2_available() {
            eprintln!("Skipping AVX2 test: CPU does not support AVX2");
            return;
        }

        let a = [0xAA; 96];
        let b = [0xAA; 96];

        let distance = unsafe { hamming_distance_avx2(&a, &b) };
        assert_eq!(distance, 0);
    }

    #[test]
    fn test_avx2_opposite() {
        if !avx2_available() {
            eprintln!("Skipping AVX2 test: CPU does not support AVX2");
            return;
        }

        let a = [0x00; 96];
        let b = [0xFF; 96];

        let distance = unsafe { hamming_distance_avx2(&a, &b) };
        assert_eq!(distance, 768);
    }

    #[test]
    fn test_avx2_alternating() {
        if !avx2_available() {
            eprintln!("Skipping AVX2 test: CPU does not support AVX2");
            return;
        }

        let a = [0xAA; 96]; // 10101010...
        let b = [0x55; 96]; // 01010101...

        let distance = unsafe { hamming_distance_avx2(&a, &b) };
        assert_eq!(distance, 768);
    }

    #[test]
    fn test_avx2_single_bit() {
        if !avx2_available() {
            eprintln!("Skipping AVX2 test: CPU does not support AVX2");
            return;
        }

        let mut a = [0x00; 96];
        let b = [0x00; 96];
        a[0] = 0x01; // Only bit 0 differs

        let distance = unsafe { hamming_distance_avx2(&a, &b) };
        assert_eq!(distance, 1);
    }

    #[test]
    fn test_avx2_boundary_32() {
        if !avx2_available() {
            eprintln!("Skipping AVX2 test: CPU does not support AVX2");
            return;
        }

        let mut a = [0x00; 96];
        let b = [0x00; 96];
        a[31] = 0xFF; // Last byte of first register
        a[32] = 0xFF; // First byte of second register

        let distance = unsafe { hamming_distance_avx2(&a, &b) };
        assert_eq!(distance, 16); // 8 bits × 2 bytes
    }

    #[test]
    fn test_avx2_boundary_64() {
        if !avx2_available() {
            eprintln!("Skipping AVX2 test: CPU does not support AVX2");
            return;
        }

        let mut a = [0x00; 96];
        let b = [0x00; 96];
        a[63] = 0xFF; // Last byte of second register
        a[64] = 0xFF; // First byte of third register

        let distance = unsafe { hamming_distance_avx2(&a, &b) };
        assert_eq!(distance, 16);
    }
}
