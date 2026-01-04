# Week 32 Day 1: Planning & Research

**Date:** 2026-01-06
**Focus:** Analyze existing SIMD implementations, design Euclidean distance approach
**Estimated Duration:** 2 hours
**Priority:** P0 — Foundation for week's work

---

## Context

Before implementing SIMD Euclidean distance, we need to:
1. Understand existing SIMD patterns in the codebase
2. Study how L2 squared is currently implemented
3. Design the Euclidean (with sqrt) approach for WASM and x86

**Reference Files:**
- `src/metric/simd.rs` — Main SIMD implementations
- `src/metric/simd/neon.rs` — ARM NEON (has euclidean_distance already)
- `src/metric/l2.rs` — L2 distance dispatcher
- `docs/planning/V0.8.0_CONSOLIDATION_PLAN.md` — Design spec

---

## Tasks

### W32.1.1: Analyze Existing SIMD Implementations

**Objective:** Document current SIMD patterns and identify gaps.

**Steps:**

1. **Read WASM SIMD implementations** (`src/metric/simd.rs`)
   ```bash
   # Find all WASM SIMD functions
   grep -n "wasm32" src/metric/simd.rs
   ```

   Document:
   - [ ] L2 squared implementation pattern
   - [ ] How horizontal sum is done
   - [ ] Tail handling approach

2. **Read x86 AVX2 implementations**
   ```bash
   # Find AVX2 functions
   grep -n "avx2\|x86_64" src/metric/simd.rs
   ```

   Document:
   - [ ] How `_mm256_sqrt_ps` could be used
   - [ ] FMA patterns
   - [ ] Register usage

3. **Study ARM NEON euclidean** (`src/metric/simd/neon.rs:91-105`)
   - This already exists — can we port the pattern?

**Deliverable:** Analysis notes in this file (update with findings)

**Acceptance Criteria:**
- [x] All SIMD patterns documented
- [x] Gap analysis complete (what's missing)
- [x] Clear understanding of sqrt options

**Duration:** 1 hour

**Agent:** RUST_ENGINEER

---

## Analysis Findings (W32.1.1 Output)

### WASM SIMD128 Pattern (`src/metric/simd.rs:28-97`)

**L2 Squared Implementation:**
```rust
// 4 accumulators for instruction-level parallelism (ILP)
let mut sum0 = f32x4_splat(0.0);
let mut sum1 = f32x4_splat(0.0);
let mut sum2 = f32x4_splat(0.0);
let mut sum3 = f32x4_splat(0.0);

// Process 16 floats (4 vectors) per main loop iteration
while i + 16 <= n {
    let diff = f32x4_sub(va, vb);
    sum0 = f32x4_add(sum0, f32x4_mul(diff, diff));
    // ... repeat for sum1, sum2, sum3
}

// Horizontal sum via lane extraction
let sum = f32x4_extract_lane::<0>(sum_v)
    + f32x4_extract_lane::<1>(sum_v)
    + f32x4_extract_lane::<2>(sum_v)
    + f32x4_extract_lane::<3>(sum_v);
```

**Key Observations:**
- ✅ 4-accumulator pattern breaks dependency chains
- ✅ Processes 16 floats per iteration (4x unrolling)
- ✅ Scalar tail handling for non-aligned lengths
- ❌ No sqrt - returns L2 **squared** only

---

### x86 AVX2 Pattern (`src/metric/simd.rs:645-720`)

**L2 Squared Implementation:**
```rust
let mut sum256 = _mm256_setzero_ps();

while i + 16 <= n {
    let diff = _mm256_sub_ps(va, vb);
    // FMA: sum += diff * diff
    #[cfg(target_feature = "fma")]
    { sum256 = _mm256_fmadd_ps(diff, diff, sum256); }
    #[cfg(not(target_feature = "fma"))]
    { sum256 = _mm256_add_ps(sum256, _mm256_mul_ps(diff, diff)); }
}

let sum = hsum256_ps_avx(sum256);  // Horizontal sum helper
```

**Key Observations:**
- ✅ Uses FMA when available (more accurate, faster)
- ✅ 2x unrolling (16 floats per iteration)
- ✅ `hsum256_ps_avx` helper for horizontal sum
- ❌ No sqrt - returns L2 **squared** only
- Note: Has `_mm256_sqrt_ps` available but not used

---

### ARM NEON Pattern (`src/simd/neon.rs:393-457`)

**Euclidean Distance Implementation (COMPLETE!):**
```rust
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    let mut sum_sq = vdupq_n_f32(0.0);

    for i in 0..chunks {
        let va = vld1q_f32(a.as_ptr().add(offset));
        let vb = vld1q_f32(b.as_ptr().add(offset));
        let diff = vsubq_f32(va, vb);
        sum_sq = vfmaq_f32(sum_sq, diff, diff);  // FMA
    }

    let result = vaddvq_f32(sum_sq);  // Horizontal sum
    result.sqrt()  // Scalar sqrt at end
}
```

**Key Observations:**
- ✅ NEON already has euclidean_distance with sqrt
- ✅ Uses scalar `result.sqrt()` at the end
- ✅ FMA via `vfmaq_f32`
- ✅ Horizontal sum via `vaddvq_f32`
- ✅ Good pattern to follow

---

### Gap Analysis

| Platform | L2 Squared | Euclidean (with sqrt) | Status |
|:---------|:-----------|:---------------------|:-------|
| WASM SIMD128 | ✅ `wasm::l2_squared` | ❌ Missing | **NEED TO ADD** |
| x86 AVX2 | ✅ `x86::l2_squared` | ❌ Missing | **NEED TO ADD** |
| ARM NEON | ✅ Implicit | ✅ `neon::euclidean_distance` | Complete |
| Scalar | ✅ In l2.rs | ❌ Missing dispatcher | **NEED TO ADD** |

---

### Sqrt Strategy Options

| Option | Description | Pros | Cons |
|:-------|:------------|:-----|:-----|
| A | SIMD sqrt intrinsic | Full SIMD pipeline | Complex, WASM f32x4_sqrt may be slow |
| **B** | SIMD L2² + scalar sqrt | Simple, accurate | One scalar op at end |
| C | Newton-Raphson approx | Fast approx | Accuracy loss, complex |

**Decision: Option B** — Match NEON pattern. Compute L2 squared with SIMD, call `.sqrt()` once at the end on the scalar result.

**Rationale:**
1. sqrt is called only ONCE on final scalar result (minimal overhead)
2. Matches existing NEON implementation pattern
3. Full accuracy (no approximation errors)
4. Simplest implementation - reuses existing `l2_squared` functions

---

### W32.1.1b: Design Euclidean Distance Approach

**Objective:** Create implementation plan for WASM + x86 euclidean.

**Design Decisions:**

1. **WASM SIMD128 sqrt options:**
   - Option A: Use `f32x4_sqrt` intrinsic (if available)
   - Option B: Compute L2 squared with SIMD, scalar sqrt at end
   - Option C: Newton-Raphson approximation in SIMD

   **Recommendation:** Option B (simplest, sqrt only once at end)

2. **x86 AVX2 approach:**
   - Use `_mm256_sqrt_ps` after computing sum of squared differences
   - Or compute squared sum with SIMD, scalar sqrt at end

   **Recommendation:** Scalar sqrt at end (matches NEON pattern)

3. **API design:**
   ```rust
   /// Euclidean distance using SIMD acceleration.
   /// Falls back to scalar for tail elements.
   pub fn euclidean_distance_f32(a: &[f32], b: &[f32]) -> f32 {
       l2_squared_f32(a, b).sqrt()
   }
   ```

**Deliverable:** Design document section below (fill in after research)

---

## Euclidean Distance Design (Day 1 Output)

### WASM SIMD128 Approach

**Pattern:** Wrapper function calling existing `l2_squared` + scalar sqrt

```rust
/// Euclidean distance using WASM SIMD128.
#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    l2_squared(a, b).sqrt()
}
```

**Sqrt Strategy:** Scalar `.sqrt()` on final result (Option B)

**Expected Speedup:** ~2x vs scalar (same as l2_squared, sqrt overhead negligible)

**File Location:** `src/metric/simd.rs` in the `wasm` module (lines ~618-622)

---

### x86 AVX2 Approach

**Pattern:** Wrapper function calling existing `l2_squared` + scalar sqrt

```rust
/// Euclidean distance using AVX2.
#[inline]
#[must_use]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    l2_squared(a, b).sqrt()
}
```

**Sqrt Strategy:** Scalar `.sqrt()` on final result (Option B)

**Expected Speedup:** ~2-4x vs scalar (same as l2_squared)

**File Location:** `src/metric/simd.rs` in the `x86` module (lines ~966-970)

**Note:** `_mm256_sqrt_ps` exists but not worth using since we only need ONE sqrt at the end.

---

### Dispatcher Design

**New File:** `src/metric/euclidean.rs` (or update `src/metric/l2.rs`)

```rust
/// Euclidean distance dispatcher.
///
/// Automatically selects the best implementation:
/// - WASM SIMD128 for wasm32 targets
/// - AVX2 for x86_64 targets
/// - NEON for aarch64 targets
/// - Scalar fallback for others
#[inline]
#[must_use]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    cfg_if::cfg_if! {
        if #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))] {
            super::simd::wasm::euclidean_distance(a, b)
        } else if #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))] {
            super::simd::x86::euclidean_distance(a, b)
        } else if #[cfg(target_arch = "aarch64")] {
            crate::simd::neon::euclidean_distance(a, b)
        } else {
            // Scalar fallback
            super::scalar::euclidean_distance(a, b)
        }
    }
}
```

---

### Implementation Order

1. **Add WASM `euclidean_distance` to `src/metric/simd.rs::wasm`**
   - 5 lines: wrapper calling `l2_squared().sqrt()`
   - Add rustdoc with example

2. **Add x86 `euclidean_distance` to `src/metric/simd.rs::x86`**
   - 5 lines: wrapper calling `l2_squared().sqrt()`
   - Add rustdoc with example

3. **Add scalar `euclidean_distance` to `src/metric/scalar.rs`**
   - 5 lines: simple loop with sqrt at end
   - Reference implementation for testing

4. **Create dispatcher in `src/metric/simd.rs` or new file**
   - Routes to correct implementation
   - Exports public `euclidean_distance` function

5. **Add unit tests**
   - Empty vectors, single element, known values
   - Compare SIMD vs scalar results
   - Test various dimensions (1, 15, 16, 17, 63, 64, 128, 768)

6. **Add benchmark**
   - Compare euclidean vs l2_squared
   - Verify sqrt overhead is negligible (<5%)

---

### Test Cases to Add

| Test | Input | Expected | Purpose |
|:-----|:------|:---------|:--------|
| Empty vectors | `[], []` | `0.0` | Edge case |
| Single element | `[5.0], [3.0]` | `2.0` | sqrt(4) |
| 3-4-5 triangle | `[0,0,0], [3,4,0]` | `5.0` | Known value |
| Large vectors | `768 dims` | Matches scalar | SIMD correctness |
| All zeros | `[0;128], [0;128]` | `0.0` | Zero handling |

---

**Acceptance Criteria:**
- [x] Both platform approaches documented
- [x] Sqrt strategy decided
- [x] Implementation order clear

**Duration:** 1 hour

**Agent:** RUST_ENGINEER

---

## Exit Criteria for Day 1

| Criterion | Verification | Status |
|:----------|:-------------|:-------|
| Existing SIMD patterns analyzed | Notes in this file | [x] |
| WASM approach documented | Design section filled | [x] |
| x86 approach documented | Design section filled | [x] |
| Implementation order clear | Numbered list | [x] |

**Day 1 Status: ✅ COMPLETE**

---

## Handoff to Day 2

After completing Day 1:
1. Update this file with findings
2. Proceed to `DAY_2_TASKS.md` for implementation
3. Carry forward any open questions

---

**Day 1 Total:** 2 hours
**Agent:** RUST_ENGINEER
