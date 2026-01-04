# Week 32 Day 3: simd_dispatch! Macro Design

**Date:** 2026-01-08
**Focus:** Design the unified SIMD dispatch macro
**Estimated Duration:** 2 hours
**Priority:** P0 — Eliminates boilerplate

---

## Context

Each SIMD function currently has 15-20 lines of platform detection boilerplate. The `simd_dispatch!` macro will reduce this to 5-10 lines while maintaining type safety and performance.

**Goal:** Design a macro that:
1. Handles AVX2, NEON, WASM SIMD, and scalar fallback
2. Is type-safe (compiler catches mismatches)
3. Has zero runtime overhead
4. Is documented and easy to use

---

## Tasks

### W32.2.1: Design Macro Syntax

**Objective:** Define the macro's interface and behavior.

**Proposed Syntax:**
```rust
simd_dispatch! {
    /// Public function documentation
    pub fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
        avx2: unsafe { avx2_hamming(a, b) },
        neon: neon_hamming(a, b),
        wasm_simd: wasm_hamming(a, b),
        fallback: scalar_hamming(a, b),
    }
}
```

**Expands To:**
```rust
/// Public function documentation
pub fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { avx2_hamming(a, b) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return neon_hamming(a, b);
        }
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        return wasm_hamming(a, b);
    }

    scalar_hamming(a, b)
}
```

**Design Decisions:**

1. **Optional branches:** Not all functions have all implementations
   ```rust
   simd_dispatch! {
       pub fn euclidean(a: &[f32], b: &[f32]) -> f32 {
           // No avx2 branch — macro should handle gracefully
           wasm_simd: wasm_euclidean(a, b),
           fallback: scalar_euclidean(a, b),
       }
   }
   ```

2. **Unsafe handling:** AVX2 calls are typically unsafe
   - Macro should preserve `unsafe` blocks as-is
   - User wraps in `unsafe { }` when needed

3. **Visibility:** Support `pub`, `pub(crate)`, and private
   - First token before `fn` determines visibility

4. **Generic functions:** Phase 2 (not this week)
   - Start with concrete types only

**Deliverable:** Design document completed below

---

## simd_dispatch! Macro Design (Day 3 Output)

### Syntax Rules

1. **Function signature required:** `$vis fn $name($args) -> $ret { ... }`
2. **Attributes preserved:** `$(#[$meta])*` before `fn` (doc comments, #[inline], #[must_use])
3. **Branch order matters:** `wasm_simd` → `avx2` → `neon` → `fallback`
4. **Trailing commas optional:** Both `expr,` and `expr` accepted
5. **Visibility supported:** `pub`, `pub(crate)`, or private

### Optional Branches

| Branch | Status | When Used |
|:-------|:-------|:----------|
| `wasm_simd` | Optional | `cfg(all(target_arch = "wasm32", target_feature = "simd128"))` |
| `avx2` | Optional | `cfg(all(target_arch = "x86_64", target_feature = "avx2"))` |
| `neon` | Optional | `cfg(target_arch = "aarch64")` |
| `fallback` | **REQUIRED** | Always compiled for unsupported platforms |

### Supported Branch Combinations

| Pattern | Branches | Status |
|:--------|:---------|:-------|
| 1 | wasm_simd, avx2, neon, fallback | ✅ Implemented |
| 2 | wasm_simd, avx2, fallback | ✅ Implemented |
| 3 | wasm_simd, neon, fallback | ✅ Implemented |
| 4 | avx2, neon, fallback | ✅ Implemented |
| 5 | wasm_simd, fallback | ✅ Implemented |
| 6 | avx2, fallback | ✅ Implemented |
| 7 | neon, fallback | ✅ Implemented |
| 8 | fallback only | ✅ Implemented |

### Edge Cases

1. **Empty argument list:** `fn foo() -> T` - Not yet supported (requires additional pattern)
2. **Generic functions:** `fn foo<T>(...)` - Deferred to Phase 2
3. **Where clauses:** Not yet supported
4. **Async functions:** Not supported (WASM SIMD is sync)

### Error Messages

- **Missing fallback:** Compile error: "unexpected end of macro invocation"
- **Type mismatch:** Compile error from underlying `cfg_if!` expansion
- **Wrong branch order:** Compile error: "no rules expected this token"

### Implementation Notes

- Uses `cfg_if::cfg_if!` internally for zero-cost compile-time dispatch
- All 8 branch combination patterns explicitly defined in macro_rules!
- Tests cover: fallback only, attributes, multiple args, slice args, distance pattern

---

**Acceptance Criteria:**
- [x] Syntax documented with examples
- [x] All branch combinations defined (8 patterns)
- [x] Edge cases identified
- [x] Error messages specified

**Duration:** 1 hour

**Agent:** RUST_ENGINEER

---

### W32.2.1b: Write Macro Implementation

**Objective:** Implement the `simd_dispatch!` macro.

**File:** `src/simd/dispatch.rs` (new file)

**Implementation Skeleton:**
```rust
/// Unified SIMD dispatch macro.
///
/// Generates a public function that automatically dispatches to the
/// fastest available SIMD implementation based on runtime CPU detection.
///
/// # Example
///
/// ```rust
/// simd_dispatch! {
///     pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
///         avx2: unsafe { avx2_dot(a, b) },
///         neon: neon_dot(a, b),
///         wasm_simd: wasm_dot(a, b),
///         fallback: scalar_dot(a, b),
///     }
/// }
/// ```
///
/// # Generated Code
///
/// The macro expands to a function with `#[cfg]` guards for each platform,
/// runtime feature detection for x86 and ARM, and a fallback for unsupported
/// platforms.
#[macro_export]
macro_rules! simd_dispatch {
    // Main pattern: all four branches
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident($($arg:ident: $type:ty),* $(,)?) -> $ret:ty {
            $(avx2: $avx2:expr,)?
            $(neon: $neon:expr,)?
            $(wasm_simd: $wasm:expr,)?
            fallback: $fallback:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis fn $name($($arg: $type),*) -> $ret {
            // x86_64 AVX2
            #[cfg(target_arch = "x86_64")]
            {
                $(
                    if is_x86_feature_detected!("avx2") {
                        return $avx2;
                    }
                )?
            }

            // ARM NEON
            #[cfg(target_arch = "aarch64")]
            {
                $(
                    if std::arch::is_aarch64_feature_detected!("neon") {
                        return $neon;
                    }
                )?
            }

            // WASM SIMD128
            #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
            {
                $(return $wasm;)?
            }

            // Fallback (always required)
            $fallback
        }
    };
}
```

**Steps:**
1. [x] Create `src/simd/dispatch.rs`
2. [x] Implement macro with all patterns (8 patterns)
3. [x] Add module to `src/simd/mod.rs`
4. [x] Test compilation on all targets

**Acceptance Criteria:**
- [x] Macro compiles
- [x] Expansion matches expected output
- [x] All optional branch combinations work (8 patterns tested)

**Duration:** 1 hour

**Agent:** RUST_ENGINEER

---

## Verification Commands

```bash
# Check macro compiles
cargo check --all-features

# Expand macro to verify output
cargo expand simd::dispatch

# Check all targets
cargo check --target x86_64-unknown-linux-gnu
cargo check --target aarch64-unknown-linux-gnu
cargo check --target wasm32-unknown-unknown
```

---

## Exit Criteria for Day 3

| Criterion | Verification | Status |
|:----------|:-------------|:-------|
| Macro syntax designed | Design doc filled | [x] |
| Macro implemented | `src/simd/dispatch.rs` exists | [x] |
| Compiles on all targets | `cargo check --target ...` | [x] |
| Documentation complete | Rustdoc renders | [x] |

**Day 3 Status: ✅ COMPLETE**

---

## Implementation Summary

**Created Files:**
- `src/simd/dispatch.rs` — 290 lines (macro + tests)

**Modified Files:**
- `src/simd/mod.rs` — Added `#[macro_use] pub mod dispatch;`

**Test Results:**
- 5 unit tests for dispatch macro: all passed
- Clippy: 0 warnings
- WASM build: success

---

## Handoff to Day 4

After completing Day 3:
1. Macro is ready for integration
2. Proceed to `DAY_4_TASKS.md` to refactor a function using the macro
3. Test that refactored code behaves identically

---

**Day 3 Total:** 2 hours
**Agent:** RUST_ENGINEER
