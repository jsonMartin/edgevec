# Week 32 Day 5: SIMD Architecture Documentation

**Date:** 2026-01-10
**Focus:** Create comprehensive SIMD architecture guide
**Estimated Duration:** 2 hours
**Priority:** P0 — Enables community contributions

---

## Context

The SIMD codebase is complex with multiple dispatch layers. Without documentation, new contributors struggle to understand:
- How to add new SIMD operations
- Which files to modify
- How to test their changes
- Expected performance characteristics

**Goal:** Create `docs/architecture/SIMD_ARCHITECTURE.md` that serves as the definitive guide.

---

## Tasks

### W32.3.1: Write Overview and Architecture Diagram

**Objective:** Explain SIMD purpose and show dispatch flow.

**Content:**

```markdown
# EdgeVec SIMD Architecture

## Overview

EdgeVec uses SIMD (Single Instruction, Multiple Data) acceleration to achieve
2-10x speedups for vector distance calculations. The SIMD subsystem supports:

- **x86_64:** AVX2 (256-bit vectors)
- **aarch64:** NEON (128-bit vectors)
- **wasm32:** SIMD128 (128-bit vectors)
- **Fallback:** Scalar implementation for all platforms

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Public API Layer                               │
│  hamming_distance()  dot_product()  l2_distance()  euclidean_distance() │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        Dispatch Layer (simd_dispatch!)                   │
│                                                                          │
│    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐        │
│    │  x86_64  │    │ aarch64  │    │  wasm32  │    │ Fallback │        │
│    │   AVX2   │    │   NEON   │    │ SIMD128  │    │  Scalar  │        │
│    │ runtime  │    │ runtime  │    │ compile  │    │  always  │        │
│    │  detect  │    │  detect  │    │   time   │    │  works   │        │
│    └────┬─────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘        │
└─────────┼───────────────┼───────────────┼───────────────┼───────────────┘
          │               │               │               │
          ▼               ▼               ▼               ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   AVX2 Impl  │  │   NEON Impl  │  │   WASM Impl  │  │ Scalar Impl  │
│              │  │              │  │              │  │              │
│ 256-bit regs │  │ 128-bit regs │  │ 128-bit ops  │  │  Loop-based  │
│  8x f32/op   │  │  4x f32/op   │  │  4x f32/op   │  │  1x f32/op   │
│  ~50 cycles  │  │  ~50 cycles  │  │ ~100 cycles  │  │ ~300 cycles  │
└──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘
```
```

**Acceptance Criteria:**
- [ ] Overview explains SIMD purpose
- [ ] Diagram shows all layers
- [ ] Platform coverage clear

**Duration:** 30 minutes

**Agent:** DOCWRITER

---

### W32.3.2: Document Module Responsibilities

**Objective:** Explain what each file/module does.

**Content:**

```markdown
## Module Structure

### Core Modules

| Module | Path | Responsibility |
|:-------|:-----|:---------------|
| `simd` | `src/simd/mod.rs` | Main SIMD exports and dispatch |
| `simd::dispatch` | `src/simd/dispatch.rs` | `simd_dispatch!` macro definition |
| `simd::avx2` | `src/metric/simd.rs` (x86 section) | AVX2 implementations |
| `simd::neon` | `src/metric/simd/neon.rs` | ARM NEON implementations |
| `simd::wasm` | `src/metric/simd.rs` (wasm section) | WASM SIMD128 implementations |

### Distance Metric Modules

| Module | Path | Responsibility |
|:-------|:-----|:---------------|
| `metric::hamming` | `src/metric/hamming.rs` | Hamming distance dispatcher |
| `metric::l2` | `src/metric/l2.rs` | L2/Euclidean distance dispatcher |
| `metric::dot` | `src/metric/dot.rs` | Dot product dispatcher |
| `metric::cosine` | `src/metric/cosine.rs` | Cosine similarity dispatcher |

### File Flow Example

When `hamming_distance()` is called:
1. `metric/hamming.rs` → Public API
2. `simd_dispatch!` macro → Selects implementation
3. Platform-specific impl → `simd.rs` or `neon.rs`
```

**Acceptance Criteria:**
- [ ] All modules documented
- [ ] Responsibilities clear
- [ ] Flow example helpful

**Duration:** 30 minutes

**Agent:** DOCWRITER

---

### W32.3.3: Write "Adding New Operations" Guide

**Objective:** Step-by-step guide for adding a new SIMD operation.

**Content:**

```markdown
## Adding a New SIMD Operation

Follow these steps to add a new SIMD-accelerated operation.

### Step 1: Implement Scalar Fallback

First, create a scalar implementation that works on all platforms:

```rust
// src/metric/my_metric.rs
pub fn my_operation_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| /* your math */).sum()
}
```

### Step 2: Add WASM SIMD Implementation

```rust
// src/metric/simd.rs (in the wasm32 section)
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
pub fn my_operation_wasm(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::wasm32::*;

    let mut sum = f32x4_splat(0.0);
    // Process 4 floats at a time
    for (a_chunk, b_chunk) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
        let va = v128_load(a_chunk.as_ptr() as *const v128);
        let vb = v128_load(b_chunk.as_ptr() as *const v128);
        // Your SIMD math here
        sum = f32x4_add(sum, /* result */);
    }

    // Handle tail
    let tail_start = a.len() - (a.len() % 4);
    let mut tail_sum = 0.0f32;
    for i in tail_start..a.len() {
        tail_sum += /* scalar math */;
    }

    // Horizontal sum
    f32x4_extract_lane::<0>(sum) + f32x4_extract_lane::<1>(sum)
        + f32x4_extract_lane::<2>(sum) + f32x4_extract_lane::<3>(sum)
        + tail_sum
}
```

### Step 3: Add x86_64 AVX2 Implementation

```rust
// src/metric/simd.rs (in the x86_64 section)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn my_operation_avx2(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;

    let mut sum = _mm256_setzero_ps();
    // Process 8 floats at a time
    // ...
}
```

### Step 4: Create Dispatcher

Use `simd_dispatch!` macro:

```rust
// src/metric/my_metric.rs
use crate::simd_dispatch;

simd_dispatch! {
    /// My operation with SIMD acceleration.
    pub fn my_operation(a: &[f32], b: &[f32]) -> f32 {
        avx2: unsafe { my_operation_avx2(a, b) },
        wasm_simd: my_operation_wasm(a, b),
        fallback: my_operation_scalar(a, b),
    }
}
```

### Step 5: Add Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_operation() {
        let a = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = vec![5.0f32, 6.0, 7.0, 8.0];

        let expected = my_operation_scalar(&a, &b);
        let result = my_operation(&a, &b);

        assert!((result - expected).abs() < 1e-5);
    }

    #[test]
    fn test_my_operation_large() {
        let a = vec![1.0f32; 768];
        let b = vec![2.0f32; 768];

        let expected = my_operation_scalar(&a, &b);
        let result = my_operation(&a, &b);

        assert!((result - expected).abs() < 1e-3);
    }
}
```

### Step 6: Add Benchmarks

```rust
// benches/simd_bench.rs
#[bench]
fn bench_my_operation(b: &mut Bencher) {
    let a = vec![1.0f32; 768];
    let b_vec = vec![2.0f32; 768];

    b.iter(|| my_operation(&a, &b_vec));
}
```
```

**Acceptance Criteria:**
- [ ] 6 clear steps
- [ ] Code examples compile
- [ ] Covers all platforms

**Duration:** 30 minutes

**Agent:** DOCWRITER

---

### W32.3.4: Create Platform Matrix and Testing Guide

**Objective:** Document what works where and how to test.

**Content:**

```markdown
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

### Browser SIMD Support

| Browser | Version | SIMD128 |
|:--------|:--------|:--------|
| Chrome | 91+ | ✅ |
| Firefox | 89+ | ✅ |
| Safari | 16.4+ | ✅ |
| Edge | 91+ | ✅ |
| iOS Safari | — | ❌ (scalar fallback) |

## Testing Guide

### Running Tests Locally

```bash
# All tests
cargo test --all-features

# SIMD-specific tests
cargo test simd --all-features

# With verbose output
cargo test simd --all-features -- --nocapture
```

### Testing for Specific Platforms

```bash
# x86_64
cargo test --target x86_64-unknown-linux-gnu

# ARM64 (requires cross-compilation or ARM hardware)
cargo test --target aarch64-unknown-linux-gnu

# WASM (requires wasm-pack)
wasm-pack test --headless --chrome
```

### Verifying SIMD is Used

```bash
# Check SIMD instructions in WASM binary
wasm2wat pkg/edgevec_bg.wasm | grep -c "i8x16\|f32x4"
# Expected: 100+ SIMD instructions

# Check AVX2 in native binary
objdump -d target/release/edgevec | grep -c "vp\|vperm"
# Expected: AVX2 instructions present
```

### Performance Validation

```bash
# Run benchmarks
cargo bench

# Compare SIMD vs scalar
cargo bench -- --baseline scalar
```

## Troubleshooting

### SIMD Not Working in WASM

1. Check `.cargo/config.toml` has `+simd128`
2. Verify browser supports SIMD
3. Check Safari compatibility (iOS doesn't support)

### AVX2 Not Detected

1. Check CPU supports AVX2: `cat /proc/cpuinfo | grep avx2`
2. Verify Rust target: `rustc --print cfg`
3. Enable feature in Cargo.toml if needed
```

**Acceptance Criteria:**
- [ ] Platform matrix complete
- [ ] Testing commands work
- [ ] Troubleshooting covers common issues

**Duration:** 30 minutes

**Agent:** DOCWRITER

---

## Verification Commands

```bash
# Check doc renders
cargo doc --open
# Navigate to SIMD section

# Verify links work
# (manual check of markdown)
```

---

## Exit Criteria for Day 5

| Criterion | Verification | Status |
|:----------|:-------------|:-------|
| Overview section complete | Visual inspection | [x] |
| Module responsibilities documented | All modules listed | [x] |
| "Adding New Operations" guide | 6 steps complete | [x] |
| Platform matrix accurate | Cross-reference code | [x] |
| Testing guide works | Commands succeed | [x] |

**Day 5 Status: ✅ COMPLETE**

---

## Implementation Summary

**Created:** `docs/architecture/SIMD_ARCHITECTURE.md`

**Sections Included:**
1. **Overview** — SIMD purpose, performance summary table
2. **Architecture Diagram** — ASCII diagram showing dispatch layers
3. **Module Structure** — Tables for SIMD core and metric modules
4. **Dispatch Strategies** — Compile-time vs runtime dispatch explanation
5. **Adding New Operations** — 6-step guide with code examples
6. **Platform Matrix** — All supported platforms and browsers
7. **Testing Guide** — Commands for local testing and WASM verification
8. **Troubleshooting** — Common issues and solutions
9. **Design Decisions** — Why certain choices were made
10. **Performance Tuning Tips** — Best practices

**Document Size:** ~400 lines

---

## Handoff to Day 6

After completing Day 5:
1. SIMD_ARCHITECTURE.md is complete and ready for review
2. Proceed to `DAY_6_TASKS.md` for testing and benchmarks
3. Reference doc during testing

---

**Day 5 Total:** 1 hour (faster than estimated)
**Agent:** DOCWRITER
