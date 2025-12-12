//! Quantization logic for vector compression.
//!
//! This module provides vector quantization implementations for memory-efficient
//! storage and fast similarity computation.
//!
//! # Available Quantizers
//!
//! - [`ScalarQuantizer`]: SQ8 quantization (f32 -> u8), 4x compression
//! - [`BinaryQuantizer`]: Binary quantization (f32 -> bit), 32x compression
//!
//! # Example
//!
//! ```
//! use edgevec::quantization::binary::BinaryQuantizer;
//!
//! let quantizer = BinaryQuantizer::new();
//! let vector = vec![0.5f32; 768];
//! let quantized = quantizer.quantize(&vector);
//!
//! // 768 f32 values (3072 bytes) -> 96 bytes
//! assert_eq!(quantized.data().len(), 96);
//! ```

/// Binary quantization (sign-based) implementation.
pub mod binary;

/// Scalar quantization (SQ8) implementation.
pub mod scalar;

/// SIMD-accelerated operations for quantized vectors.
///
/// This module provides high-performance SIMD implementations for operations
/// on quantized vectors. It automatically selects the best implementation
/// based on CPU capabilities at runtime.
///
/// # Public API
///
/// While this module is public, most users will access SIMD functionality
/// indirectly through the [`BinaryQuantizer`] and [`ScalarQuantizer`] APIs.
///
/// Advanced users can use this module directly for:
/// - Benchmarking SIMD vs portable implementations
/// - Custom quantized vector operations
/// - Performance analysis
pub mod simd;

pub use binary::{
    BinaryQuantizer, QuantizedVector, BINARY_QUANTIZATION_DIM, QUANTIZED_VECTOR_SIZE,
};
pub use scalar::{QuantizerConfig, ScalarQuantizer};
