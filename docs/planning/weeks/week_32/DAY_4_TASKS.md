# Week 32 Day 4: simd_dispatch! Macro Integration

**Date:** 2026-01-09
**Focus:** Refactor existing code to use the macro, verify behavior
**Estimated Duration:** 2 hours
**Priority:** P0 — Validate macro works in practice

---

## Context

Day 3 created the `simd_dispatch!` macro. Today we:
1. Refactor at least one existing function to use it
2. Verify the refactored code produces identical results
3. Document the macro with examples

---

## Tasks

### W32.2.3: Refactor popcount to Use Macro

**Objective:** Demonstrate macro works by refactoring `popcount_dispatch`.

**Current Code:** (approximate, in `src/simd/popcount.rs` or similar)
```rust
pub fn popcount(data: &[u8]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { popcount_avx2(data) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return popcount_neon(data);
        }
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        return popcount_wasm(data);
    }

    popcount_scalar(data)
}
```

**Target Code:**
```rust
use crate::simd_dispatch;

simd_dispatch! {
    /// Count the number of set bits (1s) in the byte slice.
    ///
    /// Automatically dispatches to the fastest available SIMD implementation.
    pub fn popcount(data: &[u8]) -> u32 {
        avx2: unsafe { popcount_avx2(data) },
        neon: popcount_neon(data),
        wasm_simd: popcount_wasm(data),
        fallback: popcount_scalar(data),
    }
}
```

**Verification Steps:**
1. [ ] Find current popcount dispatcher
2. [ ] Refactor to use `simd_dispatch!`
3. [ ] Run existing popcount tests
4. [ ] Verify same results via benchmark comparison

**Test Command:**
```bash
# Run popcount tests
cargo test popcount --all-features

# Compare before/after (if possible)
# Should produce identical results
```

**Acceptance Criteria:**
- [ ] Refactored code compiles
- [ ] All existing popcount tests pass
- [ ] Code is shorter and cleaner

**Duration:** 45 minutes

**Agent:** RUST_ENGINEER

---

### W32.2.3b: Verify Macro Expansion Matches Manual

**Objective:** Ensure macro-generated code is identical to manual code.

**Steps:**

1. **Expand the macro:**
   ```bash
   cargo expand --lib simd::popcount > expanded.rs
   ```

2. **Compare with original:**
   - Platform detection logic should be identical
   - Return statements should match
   - Feature gates should be correct

3. **Performance validation:**
   ```bash
   # If benchmarks exist
   cargo bench popcount
   ```

**Acceptance Criteria:**
- [ ] Expanded code matches expected pattern
- [ ] No performance regression
- [ ] Feature detection works correctly

**Duration:** 30 minutes

**Agent:** RUST_ENGINEER

---

### W32.2.4: Add Macro Documentation

**Objective:** Complete rustdoc for the macro with usage examples.

**File:** `src/simd/dispatch.rs`

**Documentation to Add:**
```rust
/// # simd_dispatch!
///
/// A macro for creating functions that automatically dispatch to the
/// fastest available SIMD implementation.
///
/// ## Supported Platforms
///
/// | Platform | Feature | Detection |
/// |:---------|:--------|:----------|
/// | x86_64 | AVX2 | `is_x86_feature_detected!` |
/// | aarch64 | NEON | `is_aarch64_feature_detected!` |
/// | wasm32 | SIMD128 | Compile-time `target_feature` |
///
/// ## Examples
///
/// ### Full Dispatch (All Platforms)
///
/// ```rust
/// simd_dispatch! {
///     pub fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
///         avx2: unsafe { avx2_hamming(a, b) },
///         neon: neon_hamming(a, b),
///         wasm_simd: wasm_hamming(a, b),
///         fallback: scalar_hamming(a, b),
///     }
/// }
/// ```
///
/// ### Partial Dispatch (Some Platforms)
///
/// ```rust
/// simd_dispatch! {
///     pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
///         avx2: unsafe { avx2_dot(a, b) },
///         wasm_simd: wasm_dot(a, b),
///         fallback: scalar_dot(a, b),
///     }
/// }
/// ```
///
/// ## Generated Code
///
/// The macro generates a function with:
/// - `#[cfg]` guards for each platform
/// - Runtime feature detection for x86_64 and aarch64
/// - Compile-time feature detection for WASM
/// - Fallback path always available
///
/// ## Notes
///
/// - `fallback` is always required
/// - `avx2`, `neon`, and `wasm_simd` are optional
/// - Unsafe blocks should wrap the expression when needed
/// - Visibility (`pub`, `pub(crate)`) is preserved
```

**Steps:**
1. [ ] Add documentation header to macro
2. [ ] Include examples that compile
3. [ ] Generate docs and verify rendering

**Command:**
```bash
cargo doc --open
# Navigate to simd_dispatch! macro
```

**Acceptance Criteria:**
- [ ] Documentation is complete
- [ ] Examples compile
- [ ] Rendered docs are clear

**Duration:** 45 minutes

**Agent:** RUST_ENGINEER

---

## Verification Commands

```bash
# All tests pass
cargo test --all-features

# Documentation renders
cargo doc --no-deps

# Macro expansion works
cargo expand simd::popcount

# WASM still builds
cargo check --target wasm32-unknown-unknown
```

---

## Exit Criteria for Day 4

| Criterion | Verification | Status |
|:----------|:-------------|:-------|
| One function refactored | euclidean_distance uses macro | [x] |
| Tests still pass | `cargo test` | [x] |
| Macro expansion verified | Compile-time dispatch works | [x] |
| Documentation complete | `cargo doc` renders | [x] |

**Day 4 Status: ✅ COMPLETE**

---

## Implementation Summary

### What Was Done

**Refactored:** `euclidean_distance` in `src/metric/simd.rs` (not popcount)

**Reason for Change:** The original plan suggested refactoring `popcount`, but:
- `popcount` uses **runtime** feature detection (`is_x86_feature_detected!`)
- The `simd_dispatch!` macro uses **compile-time** detection (`cfg_if!`)
- These are fundamentally different dispatch strategies

`euclidean_distance` already used compile-time dispatch via `cfg_if!`, making it the correct candidate.

### Code Changes

**Before (manual cfg_if!):**
```rust
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    cfg_if::cfg_if! {
        if #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))] {
            wasm::euclidean_distance(a, b)
        } else if #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))] {
            x86::euclidean_distance(a, b)
        } else if #[cfg(target_arch = "aarch64")] {
            crate::simd::neon::euclidean_distance(a, b)
        } else {
            crate::metric::scalar::euclidean_distance(a, b)
        }
    }
}
```

**After (simd_dispatch! macro):**
```rust
crate::simd_dispatch! {
    /// Dispatcher for Euclidean distance (f32 vectors).
    #[inline]
    #[must_use]
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        wasm_simd: wasm::euclidean_distance(a, b),
        avx2: x86::euclidean_distance(a, b),
        neon: crate::simd::neon::euclidean_distance(a, b),
        fallback: crate::metric::scalar::euclidean_distance(a, b),
    }
}
```

### Test Results

- **19 euclidean tests:** All passed
- **Clippy:** 0 warnings
- **WASM build:** Success
- **cargo doc:** Renders correctly

### Technical Notes

The macro provides:
1. **Reduced boilerplate:** ~15 lines → ~10 lines (with docs)
2. **Preserved attributes:** `#[inline]`, `#[must_use]` work correctly
3. **Doc preservation:** All documentation rendered in rustdoc
4. **Compile-time dispatch:** Zero runtime overhead

---

## Handoff to Day 5

After completing Day 4:
1. Macro is proven via euclidean_distance refactor
2. Proceed to `DAY_5_TASKS.md` for SIMD architecture documentation
3. Note: popcount uses runtime dispatch and cannot use this macro

---

**Day 4 Total:** 1 hour (faster than estimated)
**Agent:** RUST_ENGINEER
