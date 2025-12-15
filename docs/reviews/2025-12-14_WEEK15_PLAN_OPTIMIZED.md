# Week 15 Plan — OPTIMIZED (Post-Hostile Review)

**Version:** 1.1 (Optimized)
**Date:** 2025-12-14
**Status:** APPROVED
**Reviewer:** HOSTILE_REVIEWER

---

## Summary of Fixes Applied

All issues from hostile review have been resolved:

| Issue | Severity | Resolution |
|:------|:---------|:-----------|
| M1 | MAJOR | Added 32-bit x86 fallback in SIMD detection |
| M2 | MAJOR | Added bounds checking and error handling to fvecs parser |
| M3 | MAJOR | Changed `index.set_ef_search()` to `index.config.ef_search =` |
| M4 | MAJOR | Added explicit Safari status requirement (TESTED/UNTESTED/BLOCKED) |
| m1 | MINOR | Added MSRV compatibility note for aarch64 |
| m2 | MINOR | Clarified GloVe as stretch goal, SIFT-1M as primary |
| m3 | MINOR | Documented dynamic k adjustment for high tombstone ratios |
| m4 | MINOR | Added WASM compaction warning to RFC |
| m5 | MINOR | Scoped mobile testing as "stretch" with explicit status |

---

## Day 1: SIMD Detection — CORRECTED CODE

### Fix M1 + m1: Complete Architecture Coverage

**File:** `src/simd/detect.rs` (CORRECTED)

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
        // x86_64 (64-bit)
        #[cfg(target_arch = "x86_64")]
        {
            Self {
                avx2: is_x86_feature_detected!("avx2"),
                fma: is_x86_feature_detected!("fma"),
                sse42: is_x86_feature_detected!("sse4.2"),
                neon: false,
            }
        }

        // x86 (32-bit) - FIX M1: Added this block
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
        // NOTE m1: std::arch::is_aarch64_feature_detected! is stable since Rust 1.61
        // MSRV 1.70 is compatible
        #[cfg(target_arch = "aarch64")]
        {
            Self {
                avx2: false,
                fma: false,
                sse42: false,
                neon: std::arch::is_aarch64_feature_detected!("neon"),
            }
        }

        // WASM (compile-time SIMD, not runtime)
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                avx2: false,
                fma: false,
                sse42: false,
                neon: false,
            }
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
    #[must_use]
    pub fn performance_warning(&self) -> Option<String> {
        if self.is_optimal() {
            return None;
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
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

        #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
        { None }
    }
}

/// Global cached capabilities (detected once)
static CAPABILITIES: OnceLock<SimdCapabilities> = OnceLock::new();

/// Get cached SIMD capabilities
#[must_use]
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

        #[cfg(not(feature = "log"))]
        eprintln!("{}", warning);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_detect() {
        let caps = SimdCapabilities::detect();
        // Should not panic on any platform
        let _ = caps.is_optimal();
        let _ = caps.performance_warning();
    }

    #[test]
    fn test_capabilities_cached() {
        let caps1 = capabilities();
        let caps2 = capabilities();
        // Should return same reference
        assert!(std::ptr::eq(caps1, caps2));
    }

    #[test]
    fn test_default_is_all_false() {
        let caps = SimdCapabilities::default();
        assert!(!caps.avx2);
        assert!(!caps.fma);
        assert!(!caps.sse42);
        assert!(!caps.neon);
    }
}
```

### Day 1 Acceptance Criteria — UNCHANGED

All 8 ACs remain valid and measurable.

---

## Day 2: Recall Benchmarks — CORRECTED CODE

### Fix M2: Error Handling in fvecs Parser

**File:** `benches/recall/mod.rs` (CORRECTED)

```rust
//! Recall benchmarks on standard ANN benchmark datasets

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read, Error, ErrorKind};
use std::path::Path;

/// Load SIFT vectors in fvecs format with validation
///
/// # Errors
/// Returns error if file is malformed or dimensions are inconsistent
pub fn load_fvecs(path: &Path) -> std::io::Result<Vec<Vec<f32>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut vectors = Vec::new();
    let mut expected_dim: Option<usize> = None;

    loop {
        // Read dimension (4 bytes, little-endian u32)
        let mut dim_buf = [0u8; 4];
        match reader.read_exact(&mut dim_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
        let dim = u32::from_le_bytes(dim_buf) as usize;

        // FIX M2: Validate dimension consistency
        match expected_dim {
            None => expected_dim = Some(dim),
            Some(expected) if dim != expected => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Dimension mismatch at vector {}: expected {}, got {}",
                        vectors.len(), expected, dim
                    ),
                ));
            }
            _ => {}
        }

        // FIX M2: Validate dimension is reasonable
        if dim == 0 || dim > 10_000 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid dimension {} at vector {}", dim, vectors.len()),
            ));
        }

        // Read vector data
        let mut vec_buf = vec![0u8; dim * 4];
        reader.read_exact(&mut vec_buf)?;

        let vec: Vec<f32> = vec_buf
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        // FIX M2: Validate no NaN/Inf values
        for (i, &val) in vec.iter().enumerate() {
            if !val.is_finite() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Non-finite value at vector {}, component {}: {}",
                        vectors.len(), i, val
                    ),
                ));
            }
        }

        vectors.push(vec);
    }

    Ok(vectors)
}

/// Load ground truth in ivecs format with validation
pub fn load_ivecs(path: &Path) -> std::io::Result<Vec<Vec<u32>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut results = Vec::new();

    loop {
        let mut k_buf = [0u8; 4];
        match reader.read_exact(&mut k_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
        let k = u32::from_le_bytes(k_buf) as usize;

        // FIX M2: Validate k is reasonable
        if k == 0 || k > 1000 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid k {} at result {}", k, results.len()),
            ));
        }

        let mut ids_buf = vec![0u8; k * 4];
        reader.read_exact(&mut ids_buf)?;

        let ids: Vec<u32> = ids_buf
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();

        results.push(ids);
    }

    Ok(results)
}

/// Calculate recall@k
pub fn calculate_recall(
    results: &[u64],
    ground_truth: &[u32],
    k: usize,
) -> f64 {
    let k = k.min(results.len()).min(ground_truth.len());
    if k == 0 {
        return 0.0;
    }

    let result_set: HashSet<u64> = results.iter().take(k).copied().collect();
    let truth_set: HashSet<u64> = ground_truth.iter().take(k).map(|&x| x as u64).collect();

    let intersection = result_set.intersection(&truth_set).count();
    intersection as f64 / k as f64
}
```

### Fix M3: Use Direct Config Access

**File:** `benches/recall/sift.rs` (CORRECTED)

```rust
//! SIFT-1M recall benchmark

use super::*;
use edgevec::{HnswConfig, HnswIndex, VectorStorage};
use std::time::Instant;

pub fn run_sift_benchmark(data_dir: &Path) -> Vec<RecallBenchResult> {
    let mut results = Vec::new();

    // Load data
    println!("Loading SIFT-1M dataset...");
    let base_path = data_dir.join("sift_base.fvecs");
    let query_path = data_dir.join("sift_query.fvecs");
    let gt_path = data_dir.join("sift_groundtruth.ivecs");

    let base_vectors = load_fvecs(&base_path).expect("Failed to load base vectors");
    let queries = load_fvecs(&query_path).expect("Failed to load queries");
    let ground_truth = load_ivecs(&gt_path).expect("Failed to load ground truth");

    println!("  Base vectors: {}", base_vectors.len());
    println!("  Queries: {}", queries.len());
    println!("  Ground truth: {}", ground_truth.len());

    // Build index
    println!("\nBuilding HNSW index...");
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config.clone(), &storage).unwrap();

    let build_start = Instant::now();
    for (i, vec) in base_vectors.iter().enumerate() {
        index.insert(vec, &mut storage).unwrap();
        if (i + 1) % 100_000 == 0 {
            println!("  Inserted {}/{}...", i + 1, base_vectors.len());
        }
    }
    let build_time = build_start.elapsed();
    println!("  Build time: {:.2}s", build_time.as_secs_f64());

    // Run recall benchmarks at various ef_search values
    for ef_search in [10, 50, 100, 200, 500] {
        // FIX M3: Direct config access instead of non-existent method
        index.config.ef_search = ef_search;

        for k in [1, 10, 100] {
            let mut recalls = Vec::new();
            let mut latencies = Vec::new();

            for (query, gt) in queries.iter().zip(ground_truth.iter()) {
                let start = Instant::now();
                let search_results = index.search(query, k, &storage).unwrap();
                let latency = start.elapsed();

                let result_ids: Vec<u64> = search_results.iter()
                    .map(|r| r.vector_id.0)
                    .collect();

                let recall = calculate_recall(&result_ids, gt, k);
                recalls.push(recall);
                latencies.push(latency.as_micros() as f64);
            }

            let avg_recall = recalls.iter().sum::<f64>() / recalls.len() as f64;
            let qps = queries.len() as f64 / latencies.iter().sum::<f64>() * 1_000_000.0;

            latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p50 = latencies[latencies.len() / 2];
            let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];

            results.push(RecallBenchResult {
                dataset: "SIFT-1M".to_string(),
                mode: "float32".to_string(),
                k,
                ef_search: ef_search as usize,
                recall: avg_recall,
                queries_per_second: qps,
                latency_p50_us: p50,
                latency_p99_us: p99,
            });

            println!(
                "  ef={}, k={}: recall={:.4}, QPS={:.0}, p50={:.0}us, p99={:.0}us",
                ef_search, k, avg_recall, qps, p50, p99
            );
        }
    }

    results
}
```

### Fix m2: Clarify GloVe Scope

**Updated AC15.2.3:** GloVe-100 benchmark harness → **STRETCH GOAL**

Primary deliverable is SIFT-1M. GloVe is optional if time permits.

---

## Day 3: Soft Delete RFC — CORRECTIONS

### Fix m3: Document Dynamic K Adjustment

Add to RFC Section "Search Behavior":

```markdown
### Over-Fetch Strategy

When tombstone ratio is significant (>10%), the search over-fetches to ensure k results:

| Tombstone Ratio | Over-Fetch Multiplier |
|:----------------|:----------------------|
| 0-10% | 1.2x |
| 10-20% | 1.5x |
| 20-30% | 2.0x |
| >30% | Trigger compaction warning |

**Pathological Case:** If over-fetch still fails to return k results, return partial results with warning flag.
```

### Fix m4: Add WASM Compaction Warning

Add to RFC Section "Compaction Strategy":

```markdown
### WASM Considerations

**WARNING:** Compaction requires 2x memory during rebuild. On memory-constrained browsers:

1. Monitor `performance.memory` before compaction
2. Warn if available memory < 2x index size
3. Recommend offline compaction for indices > 50k vectors
4. Consider chunked compaction (future enhancement)
```

---

## Day 4: Browser Compatibility — CORRECTIONS

### Fix M4: Explicit Safari Status Requirement

**Updated AC15.4.4:** Safari tested → Safari status documented as one of:

- **TESTED:** Full test suite passed on Safari [version]
- **PARTIAL:** Limited testing, [N] tests passed, [M] skipped
- **UNTESTED:** macOS not available, BrowserStack not used
- **BLOCKED:** Safari has blocking issues preventing testing

**New Mandatory Section in BROWSER_COMPATIBILITY.md:**

```markdown
## Safari Testing Status

**Status:** [TESTED | PARTIAL | UNTESTED | BLOCKED]

**Platform:** [macOS version / BrowserStack / N/A]

**If UNTESTED, reason:**
- [ ] No macOS available
- [ ] BrowserStack account not available
- [ ] Other: _____________

**If BLOCKED, issue:**
- Description: _____________
- Workaround attempted: _____________
```

### Fix m5: Scope Mobile Testing Explicitly

**New Section:**

```markdown
## Mobile Testing Scope

| Platform | Status | Priority |
|:---------|:-------|:---------|
| iOS Safari | STRETCH | P2 |
| Android Chrome | OUT OF SCOPE | P3 |
| Mobile Firefox | OUT OF SCOPE | P3 |

**iOS Safari Testing (if time permits):**
- Memory limits verification
- IndexedDB behavior
- WASM SIMD performance

**Rationale:** Desktop browsers are primary target for v0.2.x. Mobile support formalized in v0.4.0.
```

---

## Day 5: Buffer — NO CHANGES

Buffer day plan is sound as-is.

---

## Optimized Plan Summary

### All Issues Resolved

| Day | Original Issues | Resolution Status |
|:----|:----------------|:------------------|
| Day 1 | M1, m1 | RESOLVED in code |
| Day 2 | M2, M3, m2 | RESOLVED in code + scope |
| Day 3 | m3, m4 | RESOLVED in RFC additions |
| Day 4 | M4, m5 | RESOLVED in requirements |
| Day 5 | None | N/A |

### Quality Gates Unchanged

- 34 Acceptance Criteria (all binary measurable)
- 6 Risks with mitigations
- 32h planned + 12h buffer (27%)
- All tasks < 16 hours

---

## VERDICT

```
+---------------------------------------------------------------------+
|   HOSTILE_REVIEWER: APPROVED                                        |
|                                                                     |
|   Artifact: Week 15 Task Plan v1.1 (Optimized)                      |
|                                                                     |
|   Critical Issues: 0                                                |
|   Major Issues: 0 (all resolved)                                    |
|   Minor Issues: 0 (all resolved)                                    |
|                                                                     |
|   Disposition: APPROVED FOR EXECUTION                               |
|                                                                     |
|   Week 15 may proceed immediately.                                  |
|   First task: /rust-implement W15.1                                 |
+---------------------------------------------------------------------+
```

---

**Reviewer:** HOSTILE_REVIEWER
**Version:** 2.0.0
**Date:** 2025-12-14
**Status:** FINAL APPROVAL
