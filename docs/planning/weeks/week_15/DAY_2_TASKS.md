# Week 15 — Day 2 Tasks (Tuesday, Dec 31)

**Date:** 2025-12-31
**Focus:** Formal Recall Benchmarks on Standard Datasets
**Agent:** BENCHMARK_SCIENTIST, RUST_ENGINEER
**Status:** [PROPOSED]

---

## Day Objective

Establish formal recall benchmarks using industry-standard ANN benchmark datasets (SIFT, GloVe). This addresses limitation #5: "SQ8 quantization trade-offs not formally benchmarked".

**Success Criteria:**
- SIFT-1M recall benchmark implemented
- GloVe-100 recall benchmark implemented
- Recall@10 documented for Float32 and SQ8 modes
- Comparison with published baselines
- Results added to competitive_analysis.md

---

## Tasks

### W15.2: Standard Dataset Recall Benchmarks

**Priority:** P0 (Critical Path)
**Estimate:** 8h (base: 2.7h × 3x)
**Agent:** BENCHMARK_SCIENTIST

#### Scope

- [ ] **AC15.2.1:** Create `benches/recall/` directory structure
- [ ] **AC15.2.2:** Implement SIFT-1M benchmark harness
- [ ] **AC15.2.3:** Implement GloVe-100 benchmark harness
- [ ] **AC15.2.4:** Measure recall@1, recall@10, recall@100
- [ ] **AC15.2.5:** Compare Float32 vs SQ8 quantized recall
- [ ] **AC15.2.6:** Document results in benchmark report

#### Implementation Specification

**File:** `benches/recall/mod.rs`

```rust
//! Recall benchmarks on standard ANN benchmark datasets
//!
//! Datasets:
//! - SIFT-1M: 1M 128-dim vectors from Texmex (ground truth available)
//! - GloVe-100: 1.2M 100-dim word embeddings (ground truth generated)

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Load SIFT vectors in fvecs format with validation
///
/// # Errors
/// Returns error if file is malformed or dimensions are inconsistent
/// [FIX M2: Added validation and proper error handling]
pub fn load_fvecs(path: &Path) -> std::io::Result<Vec<Vec<f32>>> {
    use std::io::{Error, ErrorKind};

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

        // [FIX M2] Validate dimension consistency
        match expected_dim {
            None => expected_dim = Some(dim),
            Some(expected) if dim != expected => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Dimension mismatch at vector {}: expected {}, got {}",
                            vectors.len(), expected, dim),
                ));
            }
            _ => {}
        }

        // [FIX M2] Validate dimension is reasonable
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

        vectors.push(vec);
    }

    Ok(vectors)
}

/// Load ground truth in ivecs format with validation
/// [FIX M2: Added validation]
pub fn load_ivecs(path: &Path) -> std::io::Result<Vec<Vec<u32>>> {
    use std::io::{Error, ErrorKind};

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

        // [FIX M2] Validate k is reasonable
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
///
/// Returns the fraction of ground truth neighbors found in the top-k results.
pub fn calculate_recall(
    results: &[u64],
    ground_truth: &[u32],
    k: usize,
) -> f64 {
    let k = k.min(results.len()).min(ground_truth.len());

    let result_set: HashSet<u64> = results.iter().take(k).copied().collect();
    let truth_set: HashSet<u64> = ground_truth.iter().take(k).map(|&x| x as u64).collect();

    let intersection = result_set.intersection(&truth_set).count();
    intersection as f64 / k as f64
}

/// Recall benchmark configuration
#[derive(Debug, Clone)]
pub struct RecallBenchConfig {
    pub dataset_name: String,
    pub base_vectors_path: String,
    pub query_vectors_path: String,
    pub ground_truth_path: String,
    pub k_values: Vec<usize>,
    pub ef_search_values: Vec<usize>,
}

/// Recall benchmark results
#[derive(Debug, Clone)]
pub struct RecallBenchResult {
    pub dataset: String,
    pub mode: String,  // "float32" or "sq8"
    pub k: usize,
    pub ef_search: usize,
    pub recall: f64,
    pub queries_per_second: f64,
    pub latency_p50_us: f64,
    pub latency_p99_us: f64,
}
```

**File:** `benches/recall/sift.rs`

```rust
//! SIFT-1M recall benchmark
//!
//! Dataset: http://corpus-texmex.irisa.fr/
//! - 1M base vectors (128-dim)
//! - 10K query vectors
//! - 100 ground truth neighbors per query

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
        // [FIX M3] Use direct config access (config is pub)
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
                ef_search,
                recall: avg_recall,
                queries_per_second: qps,
                latency_p50_us: p50,
                latency_p99_us: p99,
            });

            println!(
                "  ef={}, k={}: recall={:.4}, QPS={:.0}, p50={:.0}µs, p99={:.0}µs",
                ef_search, k, avg_recall, qps, p50, p99
            );
        }
    }

    results
}
```

**File:** `benches/recall_bench.rs` (Criterion wrapper)

```rust
//! Recall benchmark runner
//!
//! Run with: cargo bench --bench recall_bench -- --nocapture
//!
//! Requires downloading SIFT-1M dataset:
//! wget ftp://ftp.irisa.fr/local/texmex/corpus/sift.tar.gz
//! tar -xzf sift.tar.gz

mod recall;

use std::path::Path;

fn main() {
    let data_dir = std::env::var("ANN_BENCHMARK_DATA")
        .unwrap_or_else(|_| "./data/sift".to_string());

    let data_path = Path::new(&data_dir);

    if !data_path.exists() {
        eprintln!("Data directory not found: {}", data_dir);
        eprintln!("Download SIFT-1M from: ftp://ftp.irisa.fr/local/texmex/corpus/sift.tar.gz");
        eprintln!("Set ANN_BENCHMARK_DATA environment variable to data location");
        std::process::exit(1);
    }

    println!("=== EdgeVec Recall Benchmark ===\n");

    let results = recall::sift::run_sift_benchmark(data_path);

    println!("\n=== Summary ===\n");
    println!("| ef_search | k | Recall | QPS | P50 (µs) | P99 (µs) |");
    println!("|:----------|:-:|:------:|:---:|:--------:|:--------:|");
    for r in &results {
        println!(
            "| {} | {} | {:.4} | {:.0} | {:.0} | {:.0} |",
            r.ef_search, r.k, r.recall, r.queries_per_second, r.latency_p50_us, r.latency_p99_us
        );
    }
}
```

#### Verification Commands

```bash
# Download SIFT-1M dataset (one-time)
mkdir -p data/sift
cd data/sift
wget ftp://ftp.irisa.fr/local/texmex/corpus/sift.tar.gz
tar -xzf sift.tar.gz
cd ../..

# Run recall benchmark
ANN_BENCHMARK_DATA=./data/sift cargo run --release --bin recall_bench

# Run with quantization comparison
ANN_BENCHMARK_DATA=./data/sift cargo run --release --bin recall_bench --features quantized
```

#### Expected Results (Baseline Targets)

| ef_search | k | Expected Recall (Float32) | Expected Recall (SQ8) |
|:----------|:-:|:-------------------------:|:---------------------:|
| 10 | 1 | >0.85 | >0.82 |
| 50 | 10 | >0.95 | >0.92 |
| 100 | 10 | >0.98 | >0.95 |
| 200 | 100 | >0.99 | >0.97 |

#### Dependencies

- SIFT-1M dataset (~130MB download)
- Optional: GloVe-100 dataset (~1.2GB)

#### Risks

- **R15.2.1:** Dataset download may be slow/unavailable
  - **Mitigation:** Cache locally, provide fallback to synthetic data
- **R15.2.2:** 1M vectors may exceed CI memory
  - **Mitigation:** Run locally or use 100K subset for CI

---

## Day 2 Summary

**Total Effort:** 8h scheduled

**Deliverables:**
1. `benches/recall/` — Recall benchmark module
2. `benches/recall_bench.rs` — SIFT-1M benchmark runner
3. Recall results documented
4. Float32 vs SQ8 comparison

**Day 3 Preview:**
- Soft delete architecture design
- Tombstone system RFC

---

## HOSTILE_REVIEWER Pre-Flight

Before end of day:

- [ ] Recall benchmark compiles and runs
- [ ] SIFT-1M results within expected range
- [ ] Both Float32 and SQ8 modes tested
- [ ] Results documented in markdown table
- [ ] No panics on edge cases (empty queries, etc.)

---

**Status:** [PROPOSED]
**Next:** `/bench-baseline recall`
