# SIMD Performance Targets — EdgeVec Week 8 Day 37

**Version:** 1.0.0
**Author:** BENCHMARK_SCIENTIST
**Date:** 2025-12-12
**Status:** BLOCKING REQUIREMENTS

---

## Executive Summary

This document specifies the non-negotiable performance targets for SIMD Hamming distance implementation. All targets are derived from operation count calculations and must be verified with rdtsc/criterion evidence.

**RUST_ENGINEER MUST hit ALL 4 targets or the implementation is REJECTED.**

---

## Target Summary

| Metric | Target | Hard Limit | Measurement Method | Rationale |
|:-------|:-------|:-----------|:-------------------|:----------|
| AVX2 Hamming (cycles) | **<50** | **<75** | rdtsc | Operation count: 47 calculated |
| Speedup vs Portable | **>5x** | **>3x** | criterion relative | AVX2 processes 256 bits vs 8 bits |
| Throughput | **>1B ops/sec** | **>500M ops/sec** | criterion | Commercial vector DB baseline |
| Latency P99 | **<100ns** | **<200ns** | criterion | Interactive search requirement |

---

## Target 1: AVX2 Cycle Count (<50 cycles)

### Calculation from Operation Count

```
AVX2 Hamming Distance Operation Breakdown (96 bytes = 768 bits):

LOAD Phase (6 loads × 256 bits):
├── _mm256_loadu_si256(a + 0)    →  3 cycles (L1 cache hit)
├── _mm256_loadu_si256(a + 32)   →  3 cycles (pipelined)
├── _mm256_loadu_si256(a + 64)   →  3 cycles (pipelined)
├── _mm256_loadu_si256(b + 0)    →  3 cycles (pipelined)
├── _mm256_loadu_si256(b + 32)   →  3 cycles (pipelined)
└── _mm256_loadu_si256(b + 64)   →  3 cycles (pipelined)
SUBTOTAL: ~18 cycles

XOR Phase (3 × 256-bit XOR):
├── _mm256_xor_si256 (xor0)      →  1 cycle
├── _mm256_xor_si256 (xor1)      →  0 cycles (parallel execution)
└── _mm256_xor_si256 (xor2)      →  0 cycles (parallel execution)
SUBTOTAL: ~1 cycle

POPCOUNT Phase (Lookup Table Method per YMM register × 3):
├── Lookup table constant        →  2 cycles (load once)
├── Nibble extraction (vpand)    →  2 cycles
├── Shift right (vpsrlw)         →  1 cycle
├── Lookup (vpshufb × 2)         →  2 cycles
├── Add nibbles (vpaddb)         →  1 cycle
└── × 3 YMM registers            →  ~5 cycles (pipelined)
SUBTOTAL: ~10 cycles

HORIZONTAL SUM Phase:
├── _mm256_sad_epu8 (× 3)        →  9 cycles (3 cycles each)
├── _mm256_extract_epi64 (× 4)   →  4 cycles
└── Scalar adds                  →  4 cycles
SUBTOTAL: ~13 cycles

DISPATCH OVERHEAD:
├── Branch (predicted)           →  1 cycle
├── Function call overhead       →  4 cycles
SUBTOTAL: ~5 cycles

TOTAL CALCULATED: 47 cycles
TARGET: <50 cycles (includes margin)
HARD LIMIT: <75 cycles (acceptable degraded)
```

### Measurement Protocol

```rust
#[cfg(target_arch = "x86_64")]
pub fn measure_cycles<F>(f: F, iterations: u64) -> u64
where
    F: Fn() -> u32,
{
    use std::arch::x86_64::_rdtsc;

    // Warmup: 1,000 iterations to ensure code in L1 cache
    for _ in 0..1000 {
        std::hint::black_box(f());
    }

    // Measurement: 10,000 iterations for statistical significance
    let start = unsafe { _rdtsc() };
    for _ in 0..iterations {
        std::hint::black_box(f());
    }
    let end = unsafe { _rdtsc() };

    (end - start) / iterations
}
```

### Verification Command

```bash
cargo test --release test_simd_cycle_target -- --nocapture
```

---

## Target 2: Speedup vs Portable (>5x)

### Rationale

- **Portable:** Processes 8 bits per iteration (byte-by-byte)
- **AVX2:** Processes 256 bits per instruction
- **Theoretical max:** 256/8 = 32x
- **Realistic target:** >5x (accounting for setup, horizontal sum overhead)

### Measurement Protocol

```rust
// Run both implementations on identical inputs
// Calculate: speedup = portable_time / simd_time

// Example:
// Portable: 78.5 ns
// SIMD: 14.2 ns
// Speedup: 78.5 / 14.2 = 5.52x ✓
```

### Verification Command

```bash
cargo bench --bench simd_bench -- simd_comparison
```

---

## Target 3: Throughput (>1B ops/sec)

### Rationale

- **At 50 cycles:** 1 / (50 cycles × 0.5ns/cycle) = 40M ops/sec (single core)
- **At <20ns latency:** 1 / 20ns = 50M ops/sec per core
- **With pipeline parallelism:** Target 1B ops/sec on modern multicore

### Measurement Protocol

```rust
// Criterion automatically calculates throughput
// Report: Elements/second from criterion output
```

### Verification Command

```bash
cargo bench --bench simd_bench -- simd_throughput
```

---

## Target 4: Latency P99 (<100ns)

### Rationale

- **Interactive search:** Users expect <100ms total query time
- **100k vectors × 100ns = 10ms:** Acceptable for search phase
- **P99 (not mean):** Ensures consistent user experience

### Measurement Protocol

```rust
// Criterion provides P99 in detailed output
// Use: --sample-size 1000 for statistical significance
```

### Verification Command

```bash
cargo bench --bench simd_bench -- --verbose
```

---

## Measurement Protocols

### Cycle Count (rdtsc)

| Parameter | Value | Rationale |
|:----------|:------|:----------|
| Tool | `_rdtsc` intrinsic | Direct CPU timestamp counter |
| Warmup | 1,000 iterations | Ensure code in L1 cache |
| Measurement | 10,000 iterations | Statistical significance |
| Environment | Intel/AMD with AVX2 | Target platform |

### Time-Based (criterion)

| Parameter | Value | Rationale |
|:----------|:------|:----------|
| Tool | Criterion.rs 0.5.x | Industry standard |
| Sample size | Auto (100+) | Statistical rigor |
| Warmup | Auto | Criterion handles this |
| black_box | Required | Prevent optimization artifacts |

---

## Statistical Validation Requirements

### Required Statistics

1. **Mean:** Average value across iterations
2. **Median:** Robust to outliers
3. **Std Dev:** Measure variability
4. **Min/Max:** Range observed
5. **95% CI:** Confidence interval

### Outlier Detection (IQR Method)

```
Q1 = 25th percentile
Q3 = 75th percentile
IQR = Q3 - Q1
Outliers: < Q1 - 1.5×IQR OR > Q3 + 1.5×IQR
```

### Acceptance Criteria

- [ ] Outlier percentage <5%
- [ ] Mean and median differ by <10%
- [ ] Standard deviation <20% of mean
- [ ] 3 independent runs with <10% variance

---

## Rejection Criteria

Implementation is **REJECTED** if ANY of:

- [ ] AVX2 cycles ≥75 (exceeds hard limit)
- [ ] Speedup <3x (no meaningful improvement)
- [ ] Throughput <500M ops/sec (below hard limit)
- [ ] Latency P99 ≥200ns (exceeds hard limit)
- [ ] Any claim lacks benchmark evidence
- [ ] Estimates used instead of measured values

---

## Approval Criteria

Implementation is **APPROVED** if ALL of:

- [ ] AVX2 cycles <50 (target met)
- [ ] Speedup >5x (target met)
- [ ] Throughput >1B ops/sec (target met)
- [ ] Latency P99 <100ns (target met)
- [ ] All measurements have rdtsc/criterion evidence
- [ ] Statistical validation passed
- [ ] Environment documented

---

## Anti-Hallucination Safeguards

### FORBIDDEN Phrases

- "approximately 50 cycles"
- "should be around 5x faster"
- "estimated throughput"
- "roughly 1 billion ops/sec"

### REQUIRED Evidence Format

```markdown
### Cycle Count (rdtsc)
- **Measured:** 46 cycles (exact number, not estimate)
- **Target:** <50 cycles
- **Status:** ✅ PASS
- **Methodology:** rdtsc, 10,000 iterations, 1,000 warmup
- **Evidence:** [paste cargo test output]

### Speedup (criterion)
- **SIMD:** 14.2 ns (exact from criterion)
- **Portable:** 78.5 ns (exact from criterion)
- **Speedup:** 5.52x (calculated: 78.5/14.2)
- **Target:** >5x
- **Status:** ✅ PASS
- **Evidence:** [paste cargo bench output]
```

---

## Environment Documentation Template

```markdown
## Benchmark Environment

**System:**
- **CPU:** [Model] @ [Base GHz] (Turbo: [Max GHz])
- **Cores:** [Count] ([HT status])
- **Cache:** L1: [KB], L2: [KB], L3: [MB]
- **RAM:** [GB] [Type]-[Speed] MHz
- **OS:** [Name] [Version]

**Configuration:**
- **Governor:** performance (verified)
- **Temperature:** [Idle]°C / [Load]°C
- **Background Load:** [%] (verified with top)

**Tools:**
- **Rust:** [Version]
- **Criterion:** [Version]
- **Cargo:** [Version]
```

---

## Baseline Comparison

### Day 36 Portable Baseline

Before SIMD implementation, record:

```bash
cargo bench --bench bench_quantization -- hamming > docs/benchmarks/W8D36_baseline.txt
```

### Day 37 Post-SIMD Validation

After SIMD implementation:

1. Portable must not regress: ≤ Day 36 baseline
2. SIMD must meet all 4 targets
3. Evidence committed to repo

---

## Handoff Protocol

```
BENCHMARK_SCIENTIST → RUST_ENGINEER

Deliverable: docs/benchmarks/SIMD_TARGETS.md
Status: TARGETS DEFINED
Evidence: All 4 targets documented with rationale

CRITICAL CONSTRAINTS for RUST_ENGINEER:
1. MUST hit ALL 4 targets (not hard limits)
2. Hard limits are MINIMUM acceptable (not goals)
3. Evidence required: rdtsc/criterion output
4. Estimates are FORBIDDEN
```

---

## Revision History

| Version | Date | Change |
|:--------|:-----|:-------|
| 1.0.0 | 2025-12-12 | Initial specification |

---

**END OF SIMD TARGETS SPECIFICATION**
