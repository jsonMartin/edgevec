//! SIMD capability detection and runtime optimization.
//!
//! This module provides runtime detection of CPU SIMD features
//! to enable performance warnings and feature-appropriate code paths.
//!
//! # Example
//!
//! ```rust
//! use edgevec::simd::{capabilities, warn_if_suboptimal, SimdCapabilities};
//!
//! // Get detected capabilities
//! let caps = capabilities();
//! println!("AVX2 available: {}", caps.avx2);
//!
//! // Check if configuration is optimal
//! if !caps.is_optimal() {
//!     // Warn user about performance impact
//!     warn_if_suboptimal();
//! }
//! ```

pub mod detect;

pub use detect::{capabilities, warn_if_suboptimal, SimdCapabilities};
