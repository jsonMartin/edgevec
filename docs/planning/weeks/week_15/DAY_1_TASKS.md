# Week 15 — Day 1 Tasks (Monday, Dec 30)

**Date:** 2025-12-30
**Focus:** Runtime SIMD Detection & Warnings
**Agent:** RUST_ENGINEER, BENCHMARK_SCIENTIST
**Status:** [PROPOSED]

---

## Day Objective

Implement runtime SIMD capability detection to warn users when optimal performance is not available. This addresses the HIGH-SEVERITY limitation #4: "60-78% performance loss without `-C target-cpu=native`".

**Success Criteria:**
- SIMD detection API implemented
- Performance warnings logged at startup
- Documentation updated with optimization guide
- Benchmarks validate detection accuracy

---

## Tasks

### W15.1: Runtime SIMD Detection System

**Priority:** P0 (Critical Path)
**Estimate:** 6h (base: 2h × 3x)
**Agent:** RUST_ENGINEER

#### Scope

- [ ] **AC15.1.1:** Create `src/simd/detect.rs` module
- [ ] **AC15.1.2:** Implement `SimdCapabilities` struct
- [ ] **AC15.1.3:** Detect AVX2/FMA/SSE4.2 at runtime
- [ ] **AC15.1.4:** Log warning when suboptimal configuration detected
- [ ] **AC15.1.5:** Unit tests for detection on various CPU features

#### Implementation Specification

**File:** `src/simd/detect.rs`

```rust
//! Runtime SIMD capability detection
//!
//! Detects CPU SIMD features at runtime to provide performance warnings
//! and enable feature-appropriate code paths.

use std::sync::OnceLock;

/// SIMD capabilities detected at runtime
#[derive(Debug, Clone, Copy, Default)]
pub struct SimdCapabilities {
    /// AVX2 (256-bit vectors) available
    pub avx2: bool,
    /// FMA (fused multiply-add) available
    pub fma: bool,
    /// SSE 4.2 available
    pub sse42: bool,
    /// NEON (ARM) available
    pub neon: bool,
}

impl SimdCapabilities {
    /// Detect SIMD capabilities for current CPU
    #[must_use]
    pub fn detect() -> Self {
        // x86_64 (64-bit Intel/AMD)
        #[cfg(target_arch = "x86_64")]
        {
            Self {
                avx2: is_x86_feature_detected!("avx2"),
                fma: is_x86_feature_detected!("fma"),
                sse42: is_x86_feature_detected!("sse4.2"),
                neon: false,
            }
        }

        // x86 (32-bit) - [FIX M1: Added for completeness]
        #[cfg(target_arch = "x86")]
        {
            Self {
                avx2: is_x86_feature_detected!("avx2"),
                fma: is_x86_feature_detected!("fma"),
                sse42: is_x86_feature_detected!("sse4.2"),
                neon: false,
            }
        }

        // aarch64 (ARM 64-bit)
        // NOTE: std::arch::is_aarch64_feature_detected! stable since Rust 1.61 (MSRV 1.70 OK)
        #[cfg(target_arch = "aarch64")]
        {
            Self {
                avx2: false,
                fma: false,
                sse42: false,
                neon: std::arch::is_aarch64_feature_detected!("neon"),
            }
        }

        // WASM (compile-time SIMD, not runtime detectable)
        #[cfg(target_arch = "wasm32")]
        {
            Self::default()
        }

        // Fallback for all other architectures
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "wasm32"
        )))]
        {
            Self::default()
        }
    }

    /// Check if optimal performance features are available
    #[must_use]
    pub fn is_optimal(&self) -> bool {
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        { self.avx2 && self.fma }

        #[cfg(target_arch = "aarch64")]
        { self.neon }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
        { true } // No expectations for other platforms
    }

    /// Get human-readable performance warning if suboptimal
    pub fn performance_warning(&self) -> Option<String> {
        if self.is_optimal() {
            return None;
        }

        #[cfg(target_arch = "x86_64")]
        {
            let mut missing = Vec::new();
            if !self.avx2 { missing.push("AVX2"); }
            if !self.fma { missing.push("FMA"); }

            Some(format!(
                "EdgeVec: Suboptimal SIMD configuration detected. Missing: {}. \
                 Expected 60-78% performance loss. \
                 Add `rustflags = [\"-C\", \"target-cpu=native\"]` to .cargo/config.toml",
                missing.join(", ")
            ))
        }
        #[cfg(not(target_arch = "x86_64"))]
        { None }
    }
}

/// Global cached capabilities (detected once)
static CAPABILITIES: OnceLock<SimdCapabilities> = OnceLock::new();

/// Get cached SIMD capabilities
pub fn capabilities() -> &'static SimdCapabilities {
    CAPABILITIES.get_or_init(SimdCapabilities::detect)
}

/// Log performance warning if SIMD is suboptimal
///
/// Call this at library initialization to warn users.
pub fn warn_if_suboptimal() {
    if let Some(warning) = capabilities().performance_warning() {
        #[cfg(feature = "log")]
        log::warn!("{}", warning);

        // Also print to stderr for non-log users
        #[cfg(not(feature = "log"))]
        eprintln!("{}", warning);
    }
}
```

**Integration in `src/lib.rs`:**

```rust
pub mod simd;

// Re-export detection API
pub use simd::detect::{capabilities, warn_if_suboptimal, SimdCapabilities};
```

#### Verification Commands

```bash
# Unit tests
cargo test simd::detect --features simd-detect

# Check detection works
cargo run --example simd_check

# Benchmark with/without native
cargo bench --bench search_bench -- --baseline suboptimal
RUSTFLAGS="-C target-cpu=native" cargo bench --bench search_bench -- --baseline optimal
```

#### Dependencies

- None (new module)

#### Risks

- **R15.1.1:** `is_x86_feature_detected!` may not work in all contexts
  - **Mitigation:** Use `cfg` attributes with graceful fallback

---

### W15.1b: SIMD Detection Benchmarks

**Priority:** P1 (Validates W15.1)
**Estimate:** 2h (base: 0.7h × 3x)
**Agent:** BENCHMARK_SCIENTIST

#### Scope

- [ ] **AC15.1b.1:** Create benchmark comparing detected vs actual performance
- [ ] **AC15.1b.2:** Verify 60-78% delta matches documentation claim
- [ ] **AC15.1b.3:** Add detection example to `examples/`

#### Implementation Specification

**File:** `examples/simd_check.rs`

```rust
//! SIMD capability check example
//!
//! Run with: cargo run --example simd_check

use edgevec::{capabilities, warn_if_suboptimal};

fn main() {
    println!("EdgeVec SIMD Capability Check");
    println!("==============================\n");

    let caps = capabilities();

    println!("Detected capabilities:");
    println!("  AVX2:   {}", if caps.avx2 { "✅" } else { "❌" });
    println!("  FMA:    {}", if caps.fma { "✅" } else { "❌" });
    println!("  SSE4.2: {}", if caps.sse42 { "✅" } else { "❌" });
    println!("  NEON:   {}", if caps.neon { "✅" } else { "❌" });
    println!();

    println!("Optimal configuration: {}", if caps.is_optimal() { "✅ YES" } else { "❌ NO" });
    println!();

    warn_if_suboptimal();
}
```

#### Verification Commands

```bash
# Run check
cargo run --example simd_check

# Compare performance
cargo bench --bench distance_bench 2>&1 | tee baseline.txt
RUSTFLAGS="-C target-cpu=native" cargo bench --bench distance_bench 2>&1 | tee optimized.txt
```

---

## Day 1 Summary

**Total Effort:** 8h scheduled

**Deliverables:**
1. `src/simd/detect.rs` — Runtime SIMD detection module
2. `examples/simd_check.rs` — User-facing capability check
3. Integration tests for detection
4. Performance delta validation

**Day 2 Preview:**
- Formal recall benchmarks on standard datasets
- SIFT/GloVe evaluation

---

## HOSTILE_REVIEWER Pre-Flight

Before end of day:

- [ ] All acceptance criteria verified
- [ ] Unit tests pass
- [ ] Detection works on x86_64, aarch64, wasm32
- [ ] Warning message is clear and actionable
- [ ] No clippy warnings

---

**Status:** [PROPOSED]
**Next:** `/rust-implement W15.1`
