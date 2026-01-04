//! Unified SIMD dispatch macro for compile-time platform selection.
//!
//! This module provides the `simd_dispatch!` macro which generates functions
//! that automatically select the best SIMD implementation at compile time
//! based on target architecture and feature flags.
//!
//! # Design
//!
//! The macro uses `cfg_if::cfg_if!` internally for zero-cost compile-time
//! dispatch. This is more efficient than runtime feature detection because:
//! - No runtime branches
//! - Full compiler inlining and optimization
//! - Dead code elimination for unused paths
//!
//! # Priority Order
//!
//! 1. WASM SIMD128 (wasm32 + simd128 target feature)
//! 2. x86_64 AVX2 (x86_64 + avx2 target feature)
//! 3. ARM NEON (aarch64)
//! 4. Scalar fallback (all other platforms)
//!
//! # Example
//!
//! ```rust,ignore
//! use edgevec::simd_dispatch;
//!
//! simd_dispatch! {
//!     /// Compute dot product with SIMD acceleration.
//!     #[inline]
//!     #[must_use]
//!     pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
//!         wasm_simd: wasm::dot_product(a, b),
//!         avx2: unsafe { x86::dot_product(a, b) },
//!         neon: neon::dot_product(a, b),
//!         fallback: scalar::dot_product(a, b),
//!     }
//! }
//! ```
//!
//! # Optional Branches
//!
//! All branches except `fallback` are optional. If a platform-specific
//! branch is missing, that platform falls through to the next available
//! implementation or the fallback.
//!
//! ```rust,ignore
//! simd_dispatch! {
//!     // Only WASM and fallback - no AVX2 or NEON
//!     pub fn simple_op(x: u32) -> u32 {
//!         wasm_simd: wasm_impl(x),
//!         fallback: scalar_impl(x),
//!     }
//! }
//! ```

/// Unified SIMD dispatch macro for compile-time platform selection.
///
/// Generates a public function that automatically dispatches to the
/// best available SIMD implementation based on compile-time target
/// architecture and feature flags.
///
/// # Syntax
///
/// ```rust,ignore
/// simd_dispatch! {
///     $(#[$attr:meta])*           // Optional attributes (doc comments, #[inline], etc.)
///     $vis fn $name($args) -> $ret {
///         wasm_simd: $wasm_expr,  // Optional: WASM SIMD128 implementation
///         avx2: $avx2_expr,       // Optional: x86_64 AVX2 implementation
///         neon: $neon_expr,       // Optional: ARM NEON implementation
///         fallback: $fallback_expr, // REQUIRED: Scalar fallback
///     }
/// }
/// ```
///
/// # Branch Order
///
/// Branches are matched in this priority order:
/// 1. `wasm_simd` - Compiled for wasm32 with simd128 feature
/// 2. `avx2` - Compiled for x86_64 with avx2 feature
/// 3. `neon` - Compiled for aarch64
/// 4. `fallback` - All other platforms (always required)
///
/// # Generated Code
///
/// The macro expands to a `cfg_if::cfg_if!` block with compile-time
/// platform detection. Each branch is only compiled for its target.
///
/// # Example: Full Dispatch
///
/// ```rust,ignore
/// simd_dispatch! {
///     /// Compute Hamming distance with SIMD acceleration.
///     #[inline]
///     #[must_use]
///     pub fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
///         wasm_simd: wasm::hamming_distance(a, b),
///         avx2: unsafe { x86::hamming_distance(a, b) },
///         neon: neon::hamming_distance(a, b),
///         fallback: scalar::hamming_distance(a, b),
///     }
/// }
/// ```
///
/// # Example: Partial Dispatch
///
/// ```rust,ignore
/// simd_dispatch! {
///     // Only WASM and AVX2, no NEON
///     pub fn l2_squared(a: &[f32], b: &[f32]) -> f32 {
///         wasm_simd: wasm::l2_squared(a, b),
///         avx2: x86::l2_squared(a, b),
///         fallback: scalar::l2_squared(a, b),
///     }
/// }
/// ```
///
/// # Notes
///
/// - The `fallback` branch is always required
/// - All branches must return the same type
/// - Use `unsafe { }` when calling unsafe SIMD intrinsics
/// - Attributes like `#[inline]`, `#[must_use]`, and doc comments are preserved
#[macro_export]
macro_rules! simd_dispatch {
    // Pattern 1: All four branches (wasm_simd, avx2, neon, fallback)
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($($arg:ident: $type:ty),* $(,)?) -> $ret:ty {
            wasm_simd: $wasm:expr,
            avx2: $avx2:expr,
            neon: $neon:expr,
            fallback: $fallback:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis fn $name($($arg: $type),*) -> $ret {
            cfg_if::cfg_if! {
                if #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))] {
                    $wasm
                } else if #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))] {
                    $avx2
                } else if #[cfg(target_arch = "aarch64")] {
                    $neon
                } else {
                    $fallback
                }
            }
        }
    };

    // Pattern 2: wasm_simd, avx2, fallback (no neon)
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($($arg:ident: $type:ty),* $(,)?) -> $ret:ty {
            wasm_simd: $wasm:expr,
            avx2: $avx2:expr,
            fallback: $fallback:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis fn $name($($arg: $type),*) -> $ret {
            cfg_if::cfg_if! {
                if #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))] {
                    $wasm
                } else if #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))] {
                    $avx2
                } else {
                    $fallback
                }
            }
        }
    };

    // Pattern 3: wasm_simd, neon, fallback (no avx2)
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($($arg:ident: $type:ty),* $(,)?) -> $ret:ty {
            wasm_simd: $wasm:expr,
            neon: $neon:expr,
            fallback: $fallback:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis fn $name($($arg: $type),*) -> $ret {
            cfg_if::cfg_if! {
                if #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))] {
                    $wasm
                } else if #[cfg(target_arch = "aarch64")] {
                    $neon
                } else {
                    $fallback
                }
            }
        }
    };

    // Pattern 4: avx2, neon, fallback (no wasm)
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($($arg:ident: $type:ty),* $(,)?) -> $ret:ty {
            avx2: $avx2:expr,
            neon: $neon:expr,
            fallback: $fallback:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis fn $name($($arg: $type),*) -> $ret {
            cfg_if::cfg_if! {
                if #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))] {
                    $avx2
                } else if #[cfg(target_arch = "aarch64")] {
                    $neon
                } else {
                    $fallback
                }
            }
        }
    };

    // Pattern 5: wasm_simd, fallback only
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($($arg:ident: $type:ty),* $(,)?) -> $ret:ty {
            wasm_simd: $wasm:expr,
            fallback: $fallback:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis fn $name($($arg: $type),*) -> $ret {
            cfg_if::cfg_if! {
                if #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))] {
                    $wasm
                } else {
                    $fallback
                }
            }
        }
    };

    // Pattern 6: avx2, fallback only
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($($arg:ident: $type:ty),* $(,)?) -> $ret:ty {
            avx2: $avx2:expr,
            fallback: $fallback:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis fn $name($($arg: $type),*) -> $ret {
            cfg_if::cfg_if! {
                if #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))] {
                    $avx2
                } else {
                    $fallback
                }
            }
        }
    };

    // Pattern 7: neon, fallback only
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($($arg:ident: $type:ty),* $(,)?) -> $ret:ty {
            neon: $neon:expr,
            fallback: $fallback:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis fn $name($($arg: $type),*) -> $ret {
            cfg_if::cfg_if! {
                if #[cfg(target_arch = "aarch64")] {
                    $neon
                } else {
                    $fallback
                }
            }
        }
    };

    // Pattern 8: fallback only (no SIMD)
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($($arg:ident: $type:ty),* $(,)?) -> $ret:ty {
            fallback: $fallback:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis fn $name($($arg: $type),*) -> $ret {
            $fallback
        }
    };
}

#[cfg(test)]
mod tests {
    // Test helper functions
    fn scalar_add(a: u32, b: u32) -> u32 {
        a + b
    }

    fn scalar_mul(a: f32, b: f32) -> f32 {
        a * b
    }

    // Test: Pattern 8 - fallback only
    simd_dispatch! {
        /// Test function with fallback only.
        #[inline]
        fn test_fallback_only(a: u32, b: u32) -> u32 {
            fallback: scalar_add(a, b),
        }
    }

    #[test]
    fn test_fallback_only_works() {
        assert_eq!(test_fallback_only(2, 3), 5);
    }

    // Test: Pattern with attributes preserved
    simd_dispatch! {
        /// Documented test function.
        #[inline]
        #[must_use]
        pub fn test_with_attrs(x: f32, y: f32) -> f32 {
            fallback: scalar_mul(x, y),
        }
    }

    #[test]
    fn test_attributes_preserved() {
        let result = test_with_attrs(2.0, 3.0);
        assert!((result - 6.0).abs() < 1e-6);
    }

    // Test: Multiple arguments
    simd_dispatch! {
        fn test_multi_arg(a: u32, b: u32, c: u32) -> u32 {
            fallback: a + b + c,
        }
    }

    #[test]
    fn test_multiple_arguments() {
        assert_eq!(test_multi_arg(1, 2, 3), 6);
    }

    // Test: Slice arguments (common pattern)
    fn scalar_sum(slice: &[f32]) -> f32 {
        slice.iter().sum()
    }

    simd_dispatch! {
        fn test_slice_arg(data: &[f32]) -> f32 {
            fallback: scalar_sum(data),
        }
    }

    #[test]
    fn test_slice_argument() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = test_slice_arg(&data);
        assert!((result - 10.0).abs() < 1e-6);
    }

    // Test: Two slice arguments (distance pattern)
    fn scalar_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum::<f32>()
            .sqrt()
    }

    simd_dispatch! {
        /// Euclidean distance test.
        #[inline]
        fn test_distance(a: &[f32], b: &[f32]) -> f32 {
            fallback: scalar_distance(a, b),
        }
    }

    #[test]
    fn test_distance_pattern() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        let dist = test_distance(&a, &b);
        assert!((dist - 5.0).abs() < 1e-6);
    }
}
