//! Distance metrics for vector comparison.
//!
//! This module defines the `Metric` trait and implements standard distance
//! metrics used in HNSW indexing (L2 Squared, Dot Product, Hamming).
//!
//! # SIMD Acceleration
//!
//! Implementations for `L2Squared` and `DotProduct` automatically use SIMD instructions
//! if the target architecture supports them (e.g., AVX2 on x86_64, SIMD128 on WASM).
//!
//! To enable WASM SIMD, compile with `RUSTFLAGS="-C target-feature=+simd128"`.

pub mod dot;
pub mod hamming;
pub mod l2;
pub mod scalar;
pub mod simd;

pub use dot::DotProduct;
pub use hamming::Hamming;
pub use l2::L2Squared;

/// A trait for calculating distance between two vectors.
pub trait Metric<T> {
    /// Calculates the distance between two vectors.
    ///
    /// # Arguments
    ///
    /// * `a` - The first vector.
    /// * `b` - The second vector.
    ///
    /// # Returns
    ///
    /// The distance as a floating-point value.
    ///
    /// # Panics
    ///
    /// Implementations MUST panic if:
    /// - The vectors have different lengths.
    /// - The inputs contain NaN (for float types).
    fn distance(a: &[T], b: &[T]) -> f32;
}
