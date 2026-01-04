# EdgeVec SIMD Architecture

**Version:** 1.0.0
**Last Updated:** 2026-01-04
**Status:** [APPROVED] — HOSTILE_REVIEWER 2026-01-04

---

## Overview

EdgeVec uses SIMD (Single Instruction, Multiple Data) acceleration to achieve
2-10x speedups for vector distance calculations. The SIMD subsystem supports:

- **x86_64:** AVX2 (256-bit vectors) with runtime feature detection
- **aarch64:** NEON (128-bit vectors) with runtime feature detection
- **wasm32:** SIMD128 (128-bit vectors) with compile-time feature detection
- **Fallback:** Scalar implementation for all platforms

### Performance Summary

| Metric | Scalar | WASM SIMD | AVX2 | Speedup Range |
|:-------|:-------|:----------|:-----|:--------------|
| Hamming Distance | ~300 cycles | ~35 cycles | ~35 cycles | **8.75x** |
| Dot Product | ~200 cycles | ~80 cycles | ~80 cycles | **2.5x** |
| L2 Squared | ~250 cycles | ~100 cycles | ~100 cycles | **2.4x** |
| Euclidean | ~260 cycles | ~110 cycles | ~110 cycles | **2.4x** |

*Measured on 768-dimensional vectors (typical embedding size)*

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Public API Layer                               │
│  hamming_distance()  dot_product()  l2_squared()  euclidean_distance()  │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    Dispatch Layer                                        │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ simd_dispatch! Macro (compile-time)                                │ │
│  │ - Uses cfg_if! for zero-overhead platform selection                │ │
│  │ - Supports: wasm_simd, avx2, neon, fallback branches              │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ Runtime Detection (for popcount/hamming)                           │ │
│  │ - is_x86_feature_detected!("avx2")                                 │ │
│  │ - is_aarch64_feature_detected!("neon")                             │ │
│  └────────────────────────────────────────────────────────────────────┘ │
└────────┬───────────────┬───────────────┬───────────────┬────────────────┘
         │               │               │               │
         ▼               ▼               ▼               ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   AVX2 Impl  │  │   NEON Impl  │  │   WASM Impl  │  │ Scalar Impl  │
│              │  │              │  │              │  │              │
│ 256-bit regs │  │ 128-bit regs │  │ 128-bit ops  │  │  Loop-based  │
│  8x f32/op   │  │  4x f32/op   │  │  4x f32/op   │  │  1x f32/op   │
│  32x u8/op   │  │  16x u8/op   │  │  16x u8/op   │  │  1x u8/op    │
└──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘
```

---

## Module Structure

### SIMD Core Modules

| Module | Path | Responsibility |
|:-------|:-----|:---------------|
| `simd` | `src/simd/mod.rs` | Main SIMD exports, backend selection, `detect_neon()` |
| `simd::dispatch` | `src/simd/dispatch.rs` | `simd_dispatch!` macro for compile-time dispatch |
| `simd::detect` | `src/simd/detect.rs` | CPU capability detection, `warn_if_suboptimal()` |
| `simd::popcount` | `src/simd/popcount.rs` | XOR + popcount for Hamming distance |
| `simd::neon` | `src/simd/neon.rs` | ARM NEON implementations (aarch64 only) |

### Distance Metric Modules

| Module | Path | Responsibility |
|:-------|:-----|:---------------|
| `metric::simd` | `src/metric/simd.rs` | SIMD implementations for all metrics |
| `metric::scalar` | `src/metric/scalar.rs` | Scalar fallback implementations |
| `metric::l2` | `src/metric/l2.rs` | L2 Squared distance dispatcher |
| `metric::dot` | `src/metric/dot.rs` | Dot product dispatcher |
| `metric::hamming` | `src/metric/hamming.rs` | Hamming distance dispatcher |

### Dispatch Strategies

EdgeVec uses two dispatch strategies:

#### 1. Compile-Time Dispatch (simd_dispatch! macro)

Used when the implementation doesn't require runtime feature detection.

```rust
// Example: euclidean_distance uses compile-time dispatch
crate::simd_dispatch! {
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        wasm_simd: wasm::euclidean_distance(a, b),
        avx2: x86::euclidean_distance(a, b),
        neon: neon::euclidean_distance(a, b),
        fallback: scalar::euclidean_distance(a, b),
    }
}
```

**Advantages:**
- Zero runtime overhead
- Full compiler inlining
- Dead code elimination

#### 2. Runtime Detection

Used for operations that need to detect CPU features at runtime (mainly popcount/hamming).

```rust
// Example: popcount uses runtime detection
pub fn simd_popcount_xor(a: &[u8], b: &[u8]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { avx2_popcount_xor(a, b) };
        }
    }
    // ... fallback
}
```

**Advantages:**
- Works on CPUs with varying feature support
- Safe fallback if feature unavailable

---

## Adding a New SIMD Operation

Follow these steps to add a new SIMD-accelerated operation.

### Step 1: Implement Scalar Fallback

First, create a scalar implementation in `src/metric/scalar.rs`:

```rust
/// My operation description.
///
/// # Panics
///
/// Panics if `a` and `b` have different lengths.
#[inline]
#[must_use]
pub fn my_operation(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| /* your math */)
        .sum()
}
```

### Step 2: Add WASM SIMD Implementation

Add to `src/metric/simd.rs` in the `wasm` module:

```rust
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
pub mod wasm {
    use core::arch::wasm32::*;

    pub fn my_operation(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());

        let mut sum = f32x4_splat(0.0);
        let chunks = a.len() / 4;

        for i in 0..chunks {
            let offset = i * 4;
            let va = v128_load(a[offset..].as_ptr() as *const v128);
            let vb = v128_load(b[offset..].as_ptr() as *const v128);
            // Your SIMD math here
            sum = f32x4_add(sum, /* result */);
        }

        // Handle tail elements
        let tail_start = chunks * 4;
        let mut tail_sum = 0.0f32;
        for i in tail_start..a.len() {
            tail_sum += /* scalar math for remaining elements */;
        }

        // Horizontal sum of SIMD register
        f32x4_extract_lane::<0>(sum)
            + f32x4_extract_lane::<1>(sum)
            + f32x4_extract_lane::<2>(sum)
            + f32x4_extract_lane::<3>(sum)
            + tail_sum
    }
}
```

### Step 3: Add x86_64 AVX2 Implementation

Add to `src/metric/simd.rs` in the `x86` module:

```rust
#[cfg(target_arch = "x86_64")]
pub mod x86 {
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    #[target_feature(enable = "avx2")]
    pub unsafe fn my_operation(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());

        let mut sum = _mm256_setzero_ps();
        let chunks = a.len() / 8;

        for i in 0..chunks {
            let offset = i * 8;
            let va = _mm256_loadu_ps(a[offset..].as_ptr());
            let vb = _mm256_loadu_ps(b[offset..].as_ptr());
            // Your AVX2 math here
            sum = _mm256_add_ps(sum, /* result */);
        }

        // Horizontal sum (256-bit -> scalar)
        let low = _mm256_castps256_ps128(sum);
        let high = _mm256_extractf128_ps(sum, 1);
        let sum128 = _mm_add_ps(low, high);
        // ... continue reducing to scalar
    }
}
```

### Step 4: Add NEON Implementation (Optional)

Add to `src/simd/neon.rs`:

```rust
#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

pub fn my_operation(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    unsafe {
        let mut sum = vdupq_n_f32(0.0);
        let chunks = a.len() / 4;

        for i in 0..chunks {
            let offset = i * 4;
            let va = vld1q_f32(a[offset..].as_ptr());
            let vb = vld1q_f32(b[offset..].as_ptr());
            // Your NEON math here
            sum = vaddq_f32(sum, /* result */);
        }

        // Horizontal sum
        vaddvq_f32(sum) + /* tail handling */
    }
}
```

### Step 5: Create Dispatcher

Use `simd_dispatch!` macro for compile-time dispatch:

```rust
// In src/metric/simd.rs or appropriate location
crate::simd_dispatch! {
    /// My operation with SIMD acceleration.
    ///
    /// # Panics
    ///
    /// Panics if `a.len() != b.len()`.
    #[inline]
    #[must_use]
    pub fn my_operation(a: &[f32], b: &[f32]) -> f32 {
        wasm_simd: wasm::my_operation(a, b),
        avx2: unsafe { x86::my_operation(a, b) },
        neon: crate::simd::neon::my_operation(a, b),
        fallback: crate::metric::scalar::my_operation(a, b),
    }
}
```

### Step 6: Add Tests

```rust
#[cfg(test)]
mod my_operation_tests {
    use super::*;

    #[test]
    fn test_my_operation_basic() {
        let a = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = vec![5.0f32, 6.0, 7.0, 8.0];

        let expected = crate::metric::scalar::my_operation(&a, &b);
        let result = my_operation(&a, &b);

        assert!((result - expected).abs() < 1e-5);
    }

    #[test]
    fn test_my_operation_large_vectors() {
        let a = vec![1.0f32; 768];
        let b = vec![2.0f32; 768];

        let expected = crate::metric::scalar::my_operation(&a, &b);
        let result = my_operation(&a, &b);

        // Slightly larger epsilon for accumulated floating point errors
        assert!((result - expected).abs() < 1e-3);
    }

    #[test]
    fn test_my_operation_empty() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        let result = my_operation(&a, &b);
        assert_eq!(result, 0.0);
    }

    #[test]
    #[should_panic]
    fn test_my_operation_mismatched_lengths() {
        let a = vec![1.0f32, 2.0];
        let b = vec![1.0f32];
        my_operation(&a, &b);
    }
}
```

---

## Platform Matrix

### Supported Platforms

| Platform | Target | SIMD | Detection | Speedup |
|:---------|:-------|:-----|:----------|:--------|
| x86_64 Linux | `x86_64-unknown-linux-gnu` | AVX2 | Runtime | 4-8x |
| x86_64 macOS | `x86_64-apple-darwin` | AVX2 | Runtime | 4-8x |
| x86_64 Windows | `x86_64-pc-windows-msvc` | AVX2 | Runtime | 4-8x |
| Apple Silicon | `aarch64-apple-darwin` | NEON | Runtime | 2-4x |
| ARM64 Linux | `aarch64-unknown-linux-gnu` | NEON | Runtime | 2-4x |
| WASM | `wasm32-unknown-unknown` | SIMD128 | Compile | 2-4x |
| Other | Various | Scalar | N/A | 1x |

### Browser SIMD Support

| Browser | Version | SIMD128 | Notes |
|:--------|:--------|:--------|:------|
| Chrome | 91+ | Yes | Full support |
| Firefox | 89+ | Yes | Full support |
| Safari | 16.4+ | Yes | macOS/iPadOS only |
| Edge | 91+ | Yes | Chromium-based |
| iOS Safari | All | **No** | Falls back to scalar |

---

## Testing Guide

### Running Tests Locally

```bash
# All tests
cargo test --all-features

# SIMD-specific tests
cargo test simd --all-features
cargo test euclidean --all-features
cargo test hamming --all-features

# With verbose output
cargo test simd --all-features -- --nocapture
```

### Testing for Specific Platforms

```bash
# Native x86_64 (default on most dev machines)
cargo test --all-features

# WASM (check compilation)
cargo check --target wasm32-unknown-unknown

# WASM tests (requires wasm-pack)
wasm-pack test --headless --chrome

# Cross-compile check for ARM64
cargo check --target aarch64-unknown-linux-gnu
```

### Verifying SIMD is Used

```bash
# Check SIMD instructions in WASM binary
wasm2wat pkg/edgevec_bg.wasm | grep -c "i8x16\|f32x4\|i32x4"
# Expected: 100+ SIMD instructions

# Check AVX2 in native binary (Linux)
objdump -d target/release/libedgevec.rlib | grep -c "vpxor\|vpand\|vpaddd"
# Expected: AVX2 instructions present
```

### Performance Validation

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench hamming
cargo bench l2
cargo bench dot
```

---

## Troubleshooting

### SIMD Not Working in WASM

1. **Check `.cargo/config.toml`:**
   ```toml
   [target.wasm32-unknown-unknown]
   rustflags = ["-C", "target-feature=+simd128"]
   ```

2. **Verify browser supports SIMD:**
   - Open DevTools console
   - Check if `WebAssembly.validate` accepts SIMD opcodes

3. **iOS Safari doesn't support SIMD:**
   - EdgeVec automatically falls back to scalar
   - Performance will be 2-4x slower

### AVX2 Not Detected

1. **Check CPU supports AVX2:**
   ```bash
   # Linux
   cat /proc/cpuinfo | grep avx2

   # Windows (PowerShell)
   (Get-WmiObject Win32_Processor).Caption
   # Then check online if CPU supports AVX2
   ```

2. **Verify Rust target:**
   ```bash
   rustc --print cfg | grep target_arch
   ```

3. **Check runtime detection works:**
   ```rust
   use edgevec::simd::{select_backend, SimdBackend};
   let backend = select_backend();
   println!("Backend: {:?}", backend);
   ```

### Tests Fail with Floating Point Differences

SIMD operations may have slightly different rounding than scalar:
- Use `1e-5` epsilon for small vectors
- Use `1e-3` epsilon for 768+ dimension vectors
- This is expected due to different operation ordering

---

## Design Decisions

### Why Compile-Time Dispatch for Most Operations?

1. **Zero runtime overhead** - No feature detection on each call
2. **Full optimization** - Compiler can inline everything
3. **Dead code elimination** - Unused platforms not compiled

### Why Runtime Dispatch for Popcount?

1. **AVX2 popcount requires specific instructions** - Not all x86_64 CPUs have it
2. **Native popcnt fallback** - Can use `popcnt` instruction without full AVX2
3. **More granular control** - Different CPUs have different optimal paths

### Why simd_dispatch! Macro?

1. **Reduces boilerplate** - From ~15 lines to ~5 lines per function
2. **Consistent pattern** - All dispatchers follow same structure
3. **Type safety** - Compiler catches mismatches
4. **Documentation** - Single place to document dispatch behavior

---

## Performance Tuning Tips

1. **Vector Alignment:** SIMD works best with aligned data (16/32 byte boundaries)
2. **Batch Operations:** Process multiple vectors together when possible
3. **Avoid Small Vectors:** SIMD overhead not worth it for <16 elements
4. **Use L2 Squared:** Avoid sqrt when possible (use squared distances)

---

## References

- [Rust SIMD Intrinsics](https://doc.rust-lang.org/core/arch/)
- [WebAssembly SIMD Proposal](https://github.com/WebAssembly/simd)
- [Intel AVX2 Programming Guide](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/)
- [ARM NEON Intrinsics Reference](https://developer.arm.com/architectures/instruction-sets/intrinsics/)

---

**Document Status:** [APPROVED]
**Author:** DOCWRITER
**Reviewed By:** HOSTILE_REVIEWER (2026-01-04)
