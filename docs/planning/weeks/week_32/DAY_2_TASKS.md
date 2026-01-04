# Week 32 Day 2: SIMD Euclidean Implementation

**Date:** 2026-01-07
**Focus:** Implement SIMD Euclidean distance for WASM and x86_64
**Estimated Duration:** 2 hours
**Priority:** P0 — Core deliverable

---

## Context

Day 1 provided the design. Today we implement:
1. WASM SIMD128 `euclidean_distance_f32()`
2. x86_64 AVX2 `euclidean_distance_f32()`
3. Dispatcher update in `l2.rs`

**Prerequisites:**
- Day 1 design document completed
- Understanding of existing L2 squared implementations

---

## Tasks

### W32.1.2: Implement WASM SIMD128 Euclidean Distance

**Objective:** Add WASM SIMD euclidean distance function.

**File:** `src/metric/simd.rs`

**Implementation Pattern:**
```rust
/// WASM SIMD128 Euclidean distance.
///
/// Computes sqrt(sum((a[i] - b[i])^2)) using SIMD for the squared sum,
/// then a single scalar sqrt at the end.
///
/// # Safety
/// Caller must ensure a.len() == b.len().
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
pub fn euclidean_distance_f32_wasm(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    // Compute L2 squared using existing SIMD
    let l2_sq = l2_squared_f32_wasm(a, b);

    // Single scalar sqrt at end
    l2_sq.sqrt()
}
```

**Steps:**
1. [x] Add function to `src/metric/simd.rs` in WASM section
2. [x] Use existing `l2_squared` internally
3. [x] Add unit test

**Test:**
```rust
#[test]
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
fn test_euclidean_distance_wasm() {
    let a = vec![1.0f32, 2.0, 3.0, 4.0];
    let b = vec![5.0f32, 6.0, 7.0, 8.0];
    let expected = ((4.0f32.powi(2) * 4.0) as f32).sqrt(); // sqrt(64) = 8
    let result = euclidean_distance_f32_wasm(&a, &b);
    assert!((result - expected).abs() < 1e-5);
}
```

**Acceptance Criteria:**
- [x] Function compiles for wasm32 target
- [x] Unit test passes
- [x] No clippy warnings

**Duration:** 45 minutes

**Agent:** RUST_ENGINEER

---

### W32.1.3: Implement x86_64 AVX2 Euclidean Distance

**Objective:** Add x86_64 AVX2 euclidean distance function.

**File:** `src/metric/simd.rs`

**Implementation Pattern:**
```rust
/// x86_64 AVX2 Euclidean distance.
///
/// Computes sqrt(sum((a[i] - b[i])^2)) using AVX2 for the squared sum,
/// then a single scalar sqrt at the end.
///
/// # Safety
/// Requires AVX2 support. Caller must verify with `is_x86_feature_detected!`.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn euclidean_distance_f32_avx2(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());

    // Compute L2 squared using existing AVX2 SIMD
    let l2_sq = l2_squared_f32_avx2(a, b);

    // Single scalar sqrt at end
    l2_sq.sqrt()
}
```

**Steps:**
1. [x] Add function to `src/metric/simd.rs` in x86 section
2. [x] Use existing `l2_squared` internally
3. [x] Add unit test with feature detection

**Test:**
```rust
#[test]
#[cfg(target_arch = "x86_64")]
fn test_euclidean_distance_avx2() {
    if !is_x86_feature_detected!("avx2") {
        return; // Skip on non-AVX2 hardware
    }

    let a = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let b = vec![9.0f32, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0];
    let expected: f32 = a.iter().zip(&b).map(|(x, y)| (x - y).powi(2)).sum::<f32>().sqrt();

    let result = unsafe { euclidean_distance_f32_avx2(&a, &b) };
    assert!((result - expected).abs() < 1e-5);
}
```

**Acceptance Criteria:**
- [x] Function compiles for x86_64 target
- [x] Unit test passes on AVX2 hardware
- [x] SAFETY comments documented

**Duration:** 30 minutes

**Agent:** RUST_ENGINEER

---

### W32.1.4: Update Dispatcher in l2.rs

**Objective:** Route euclidean distance calls to SIMD when available.

**File:** `src/metric/l2.rs`

**Current State:** Likely uses scalar or L2 squared + sqrt

**Target State:**
```rust
/// Compute Euclidean distance between two vectors.
///
/// Automatically dispatches to the fastest available SIMD implementation.
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { simd::euclidean_distance_f32_avx2(a, b) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return simd::neon::euclidean_distance(a, b);
        }
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        return simd::euclidean_distance_f32_wasm(a, b);
    }

    // Scalar fallback
    scalar_euclidean_distance(a, b)
}
```

**Steps:**
1. [x] Create `euclidean_distance()` dispatcher in `simd.rs`
2. [x] Ensure proper feature gates via `cfg_if!`
3. [x] Add comprehensive unit tests

**Integration Test:**
```rust
#[test]
fn test_euclidean_dispatches_correctly() {
    let a = vec![1.0f32; 768];
    let b = vec![2.0f32; 768];

    // Should use fastest available (SIMD if available)
    let result = euclidean_distance(&a, &b);

    // Verify against known calculation
    let expected = (768.0f32).sqrt(); // sqrt(768 * 1^2)
    assert!((result - expected).abs() < 1e-3);
}
```

**Acceptance Criteria:**
- [x] Dispatcher routes to correct implementation
- [x] Integration test passes
- [x] All platforms compile

**Duration:** 45 minutes

**Agent:** RUST_ENGINEER

---

## Verification Commands

```bash
# Run unit tests
cargo test metric::simd::euclidean --all-features

# Run integration tests
cargo test metric::l2::euclidean --all-features

# Check WASM compilation
cargo check --target wasm32-unknown-unknown --all-features

# Clippy
cargo clippy --all-features -- -D warnings
```

---

## Exit Criteria for Day 2

| Criterion | Verification | Status |
|:----------|:-------------|:-------|
| WASM euclidean implemented | Function exists | [x] |
| x86 euclidean implemented | Function exists | [x] |
| Dispatcher updated | Routes correctly | [x] |
| Unit tests pass | `cargo test` | [x] |
| WASM build succeeds | `cargo check --target wasm32...` | [x] |
| No clippy warnings | `cargo clippy` | [x] |

**Day 2 Status: ✅ COMPLETE**

---

## Implementation Notes

**Actual Implementation:**
- WASM `euclidean_distance`: `src/metric/simd.rs:620-658` (wasm module)
- x86 AVX2 `euclidean_distance`: `src/metric/simd.rs:1200-1240` (x86 module)
- Scalar fallback: `src/metric/scalar.rs:22-30`
- Dispatcher: `src/metric/simd.rs:1345-1399` (compile-time via `cfg_if!`)

**Design Decision:**
Used Option B from Day 1 design — SIMD for L2 squared, scalar `.sqrt()` at end.
This matches ARM NEON pattern for consistency.

**Test Results:**
- 19 euclidean tests passed
- 0 clippy warnings
- WASM build succeeded

---

## Handoff to Day 3

After completing Day 2:
1. Verify all tests pass
2. Document any issues encountered
3. Proceed to `DAY_3_TASKS.md` for macro design

---

**Day 2 Total:** 2 hours
**Agent:** RUST_ENGINEER
