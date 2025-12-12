# PROMPT: SIMD Benchmark Specification (Benchmark-First)

**Target Agent:** BENCHMARK_SCIENTIST
**Command:** `/bench-spec simd_hamming`
**Priority:** P0 (BLOCKING — Targets must be defined before optimization)
**Estimated Time:** 30 minutes (realistic: 1.5h with 3x rule)
**Dependencies:**
  - `docs/architecture/SIMD_DESIGN.md` exists
  - `docs/reviews/*SIMD_DESIGN*_APPROVED.md` exists
**Output:** `benches/bench_simd.rs` (skeleton with targets), `docs/benchmarks/SIMD_TARGETS.md`

---

## MISSION

Define benchmark targets BEFORE any implementation or optimization begins.

**Benchmark-First Principle:**
> "If you can't measure it, you can't optimize it. Define the target before coding."

**CRITICAL REQUIREMENT:** The RUST_ENGINEER must hit ALL these targets or the implementation is REJECTED.

---

## DEPENDENCY VERIFICATION

```bash
# Check architecture approved
test -f docs/architecture/SIMD_DESIGN.md || echo "BLOCK: No architecture"
grep -q "APPROVED" docs/reviews/*SIMD_DESIGN*.md || echo "BLOCK: Not approved"
```

---

## PERFORMANCE TARGETS (NON-NEGOTIABLE)

### Primary Targets

| Metric | Target | Hard Limit | Measurement Method |
|:-------|:-------|:-----------|:-------------------|
| **AVX2 Hamming (cycles)** | **<50** | **<75** | rdtsc |
| **Speedup vs Portable** | **>5x** | **>3x** | criterion relative |
| **Throughput** | **>1B ops/sec** | **>500M ops/sec** | criterion |
| **Latency P99** | **<100ns** | **<200ns** | criterion |

### Rationale for <50 Cycle Target

**Calculation from Operation Count:**

```
AVX2 Hamming Distance Operation Breakdown:

LOAD Phase:
├── _mm256_loadu_si256(a + 0)    →  ~3 cycles (L1 cache hit)
├── _mm256_loadu_si256(a + 32)   →  ~3 cycles
├── _mm256_loadu_si256(a + 64)   →  ~3 cycles
├── _mm256_loadu_si256(b + 0)    →  ~3 cycles
├── _mm256_loadu_si256(b + 32)   →  ~3 cycles
└── _mm256_loadu_si256(b + 64)   →  ~3 cycles
SUBTOTAL: ~18 cycles

XOR Phase:
├── _mm256_xor_si256 (3×)        →  ~3 cycles (1 cycle each, pipelined)
SUBTOTAL: ~3 cycles

POPCOUNT Phase (Lookup Table Method):
├── Lookup table setup           →  ~2 cycles
├── _mm256_shuffle_epi8 (6×)     →  ~6 cycles
├── _mm256_add_epi8 (3×)         →  ~3 cycles
SUBTOTAL: ~11 cycles

HORIZONTAL SUM Phase:
├── _mm256_sad_epu8 (3×)         →  ~9 cycles
├── _mm256_extracti128_si256 (3×)→  ~3 cycles
├── _mm_add_epi64 (3×)           →  ~3 cycles
SUBTOTAL: ~15 cycles

TOTAL CALCULATED: ~47 cycles
TARGET: <50 cycles (includes margin)
HARD LIMIT: <75 cycles (acceptable degraded performance)
```

---

## BENCHMARK SKELETON

Create `benches/bench_simd.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use edgevec::quantization::QuantizedVector;

/// PRIMARY TARGET: <50 cycles per call
fn bench_simd_hamming_cycles(c: &mut Criterion) {
    let q1 = QuantizedVector::from_bytes([0xAA; 96]);
    let q2 = QuantizedVector::from_bytes([0x55; 96]);

    c.bench_function("simd_hamming_96bytes_cycles", |b| {
        b.iter(|| black_box(&q1).hamming_distance(black_box(&q2)))
    });
}

/// TARGET: >5x speedup over portable
fn bench_simd_vs_portable(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_comparison");
    group.throughput(Throughput::Bytes(96 * 2)); // 2 × 96-byte inputs

    let q1 = QuantizedVector::from_bytes([0xAA; 96]);
    let q2 = QuantizedVector::from_bytes([0x55; 96]);

    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        group.bench_function("simd_avx2", |b| {
            b.iter(|| black_box(&q1).hamming_distance(black_box(&q2)))
        });
    }

    group.bench_function("portable", |b| {
        // Force portable path (implementation will provide this function)
        b.iter(|| {
            // Portable hamming distance call
            black_box(&q1).hamming_distance_portable(black_box(&q2))
        })
    });

    group.finish();
}

/// TARGET: >1 billion operations per second
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_throughput");
    group.throughput(Throughput::Elements(1)); // 1 operation = 1 hamming distance

    let q1 = QuantizedVector::from_bytes([0xAA; 96]);
    let q2 = QuantizedVector::from_bytes([0x55; 96]);

    group.bench_function("hamming_ops_per_sec", |b| {
        b.iter(|| black_box(&q1).hamming_distance(black_box(&q2)))
    });

    group.finish();
}

/// Diverse input patterns for realistic benchmarking
fn bench_diverse_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_patterns");

    let zeros = QuantizedVector::from_bytes([0x00; 96]);
    let ones = QuantizedVector::from_bytes([0xFF; 96]);
    let alt_aa = QuantizedVector::from_bytes([0xAA; 96]);
    let alt_55 = QuantizedVector::from_bytes([0x55; 96]);

    group.bench_function("pattern_zeros", |b| {
        b.iter(|| black_box(&zeros).hamming_distance(black_box(&zeros)))
    });

    group.bench_function("pattern_ones_vs_zeros", |b| {
        b.iter(|| black_box(&ones).hamming_distance(black_box(&zeros)))
    });

    group.bench_function("pattern_alternating", |b| {
        b.iter(|| black_box(&alt_aa).hamming_distance(black_box(&alt_55)))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simd_hamming_cycles,
    bench_simd_vs_portable,
    bench_throughput,
    bench_diverse_patterns
);
criterion_main!(benches);
```

---

## CYCLE MEASUREMENT PROTOCOL

**Mandatory Protocol for <50 Cycle Verification:**

```rust
#[cfg(target_arch = "x86_64")]
pub fn measure_cycles<F>(f: F, iterations: u64) -> u64
where
    F: Fn() -> u32,
{
    use std::arch::x86_64::_rdtsc;

    // Warmup: 1,000 iterations to ensure code in cache
    for _ in 0..1000 {
        std::hint::black_box(f());
    }

    // Measurement: 10,000 iterations minimum for statistical significance
    let start = unsafe { _rdtsc() };
    for _ in 0..iterations {
        std::hint::black_box(f());
    }
    let end = unsafe { _rdtsc() };

    (end - start) / iterations
}

#[test]
fn test_simd_cycle_target() {
    let q1 = QuantizedVector::from_bytes([0xAA; 96]);
    let q2 = QuantizedVector::from_bytes([0x55; 96]);

    let cycles = measure_cycles(|| q1.hamming_distance(&q2), 10_000);

    println!("Measured cycles: {}", cycles);
    assert!(cycles < 50, "FAIL: {} cycles exceeds 50 cycle target", cycles);
}
```

---

## ANTI-HALLUCINATION SAFEGUARDS

**Required Evidence Format:**

```markdown
## Benchmark Results

### Cycle Count (rdtsc)
- **Measured:** 46 cycles (not "~50", exact number required)
- **Target:** <50 cycles
- **Status:** ✅ PASS
- **Methodology:** rdtsc, 10,000 iterations, 1,000 warmup
- **Evidence:** `cargo test test_simd_cycle_target` output

### Speedup (criterion)
- **SIMD:** 14.2 ns
- **Portable:** 78.5 ns
- **Speedup:** 5.5x (not "~5x", calculated value required)
- **Target:** >5x
- **Status:** ✅ PASS
- **Evidence:** `cargo bench --bench bench_simd` output

### Throughput (criterion)
- **Measured:** 1.12 billion ops/sec
- **Target:** >1B ops/sec
- **Status:** ✅ PASS
```

**FORBIDDEN Phrases:**
- ❌ "approximately 50 cycles"
- ❌ "should be around 5x faster"
- ❌ "estimated throughput"
- ❌ "roughly 1 billion ops/sec"

**REQUIRED Evidence:**
- ✅ Exact cycle count from rdtsc
- ✅ Calculated speedup from criterion output
- ✅ Measured throughput from criterion
- ✅ Screenshot or paste of benchmark output

---

## TARGET SPECIFICATION DOCUMENT

Create `docs/benchmarks/SIMD_TARGETS.md`:

```markdown
# SIMD Performance Targets — EdgeVec Day 37

**Version:** 1.0.0
**Date:** 2025-12-12
**Status:** BLOCKING REQUIREMENTS

---

## Target Summary

| Metric | Target | Hard Limit | Rationale |
|:-------|:-------|:-----------|:----------|
| AVX2 Cycles | <50 | <75 | Operation count: 47 calculated |
| Speedup | >5x | >3x | AVX2 processes 3× data per instruction |
| Throughput | >1B ops/sec | >500M ops/sec | Commercial vector DB baseline |
| Latency P99 | <100ns | <200ns | Interactive search requirement |

---

## Measurement Protocols

### Cycle Count
- **Tool:** rdtsc (x86_64 timestamp counter)
- **Warmup:** 1,000 iterations
- **Measurement:** 10,000 iterations
- **Environment:** Intel/AMD CPU with AVX2
- **Command:** `cargo test test_simd_cycle_target`

### Throughput
- **Tool:** Criterion.rs
- **Iterations:** Auto-determined by criterion
- **Environment:** Same as cycle count
- **Command:** `cargo bench --bench bench_simd`

### Speedup
- **Method:** criterion SIMD time / criterion portable time
- **Inputs:** Identical for both (black_box enforced)
- **Command:** `cargo bench --bench bench_simd -- simd_comparison`

---

## Rejection Criteria

Implementation is REJECTED if ANY of:
- [ ] AVX2 cycles ≥75 (exceeds hard limit)
- [ ] Speedup <3x (no meaningful improvement)
- [ ] Throughput <500M ops/sec (below hard limit)
- [ ] Any claim lacks benchmark evidence

---

## Approval Criteria

Implementation is APPROVED if ALL of:
- [ ] AVX2 cycles <50 (target met)
- [ ] Speedup >5x (target met)
- [ ] Throughput >1B ops/sec (target met)
- [ ] Latency P99 <100ns (target met)
- [ ] All measurements have rdtsc/criterion evidence
```

---

## STATISTICAL VALIDATION REQUIREMENTS

### Measurement Variability

All benchmark measurements MUST include statistical analysis to ensure reliability.

**Required Statistics:**
1. **Mean:** Average value across iterations
2. **Median:** Middle value (robust to outliers)
3. **Standard Deviation:** Measure of variability
4. **Min/Max:** Range of observed values
5. **Confidence Interval:** 95% CI for mean

**Example Output Format:**

```markdown
### Cycle Count Statistics

**Raw Measurements:** 46, 45, 47, 46, 45, 48, 46, 45, 47, 46 (10 runs)

**Statistics:**
- Mean: 46.1 cycles
- Median: 46.0 cycles
- Std Dev: 0.99 cycles
- Min: 45 cycles
- Max: 48 cycles
- 95% CI: [45.4, 46.8]
- Target: <50 cycles
- Status: ✅ PASS (mean within target, 95% CI entirely below target)
```

### Outlier Detection

**Protocol:**
1. Run benchmark ≥100 times (not just criterion's auto-iterations)
2. Detect outliers using IQR (Interquartile Range) method:
   - Q1 = 25th percentile
   - Q3 = 75th percentile
   - IQR = Q3 - Q1
   - Outliers: < Q1 - 1.5×IQR OR > Q3 + 1.5×IQR
3. Report outlier count and percentage
4. If >5% outliers, investigate (thermal throttling, context switches, etc.)

**Example:**

```markdown
### Outlier Analysis

**Total Measurements:** 100
**Q1:** 45 cycles
**Q3:** 47 cycles
**IQR:** 2 cycles
**Outlier Threshold:** <42 cycles OR >50 cycles
**Outliers Found:** 2 (2%)
**Outlier Values:** 51 cycles, 52 cycles
**Status:** ✅ ACCEPTABLE (<5% outliers)
**Investigation:** Likely thermal events, not systematic issue
```

**Acceptance Criteria for Outliers:**
- [ ] Outlier percentage <5%
- [ ] Mean and median differ by <10%
- [ ] Standard deviation <20% of mean
- [ ] If >5% outliers, root cause documented

### Regression Detection

**Baseline Comparison Protocol:**

```bash
# Step 1: Establish baseline (before SIMD changes)
git checkout main  # Or appropriate baseline branch
cargo bench --bench bench_quantization -- hamming > baseline.txt
git checkout simd-feature-branch

# Step 2: Run current benchmarks
cargo bench --bench bench_simd > current.txt

# Step 3: Compare using criterion's built-in comparison
cargo bench --bench bench_simd -- --save-baseline simd_v1
# Make changes
cargo bench --bench bench_simd -- --baseline simd_v1

# Step 4: Manual comparison
# Compare portable implementation performance Day 36 vs Day 37
```

**Regression Acceptance Criteria:**
- [ ] Portable performance ≤ Day 36 baseline (no regression allowed)
- [ ] If regression detected, documented justification required
- [ ] Regression >5% requires approval from PLANNER
- [ ] Baseline results saved in `docs/benchmarks/W8D37_baseline.txt`

**Example Comparison:**

```markdown
### Regression Check

**Baseline (Day 36 - portable Hamming):**
- Time: 78.5 ns
- Throughput: 127M ops/sec

**Current (Day 37 - portable Hamming):**
- Time: 78.3 ns
- Throughput: 128M ops/sec

**Change:** -0.25% (improvement)
**Status:** ✅ NO REGRESSION

**SIMD (Day 37 - new):**
- Time: 14.2 ns
- Throughput: 1.13B ops/sec
- Speedup: 5.52x vs portable
**Status:** ✅ TARGET MET
```

### Environmental Controls

**Required for Valid Benchmarks:**

```bash
# 1. CPU Governor (must be "performance")
cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
# Expected: All cores show "performance"
# Fix if needed:
# sudo cpupower frequency-set --governor performance

# 2. CPU Frequency Verification
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq
# Expected: Should be at max frequency

# 3. No Heavy Processes
top -bn1 | head -20
# Expected: <5% CPU usage by other processes

# 4. Thermal Check
sensors
# Expected: CPU temp <80°C
# If >80°C, wait for cooldown

# 5. Isolated Core (optional but recommended for rdtsc)
# Use taskset to pin benchmark to specific core:
taskset -c 0 cargo bench --bench bench_simd
```

**Documentation Required:**

```markdown
## Benchmark Environment

**System:**
- **CPU:** Intel Core i7-9700K @ 3.6 GHz (Turbo: 4.9 GHz)
- **Cores:** 8 (no HT)
- **Cache:** L1: 32KB, L2: 256KB, L3: 12MB
- **RAM:** 32 GB DDR4-3200 MHz
- **OS:** Ubuntu 22.04.3 LTS
- **Kernel:** 6.5.0-15-generic

**Configuration:**
- **Governor:** performance (verified with `cpupower`)
- **Frequency:** 4.9 GHz (turbo active)
- **Temperature:** 62°C (idle), 74°C (under load)
- **Background Load:** 2% CPU (verified with `top`)
- **Process Isolation:** Pinned to core 0 with `taskset`

**Tools:**
- **Rust:** 1.75.0
- **Criterion:** 0.5.1
- **Cargo:** 1.75.0
```

**Acceptance Criteria for Environment:**
- [ ] CPU governor set to `performance`
- [ ] CPU temp <80°C during benchmarking
- [ ] Background CPU usage <5%
- [ ] System specs documented
- [ ] Warmup runs completed before measurement (≥1000 iterations)
- [ ] Measurement runs ≥10,000 iterations for rdtsc, auto for criterion

### Reproducibility Requirements

**For Benchmark Results to be Accepted:**

1. **Multiple Runs:** Benchmark must be run ≥3 times, median result reported
2. **Variability Check:** Standard deviation between runs must be <10%
3. **Environment Documentation:** Full system specs + configuration documented
4. **Baseline Preservation:** Baseline results committed to repo for future comparison
5. **Evidence:** Raw output from all 3 runs pasted in validation report

**Example:**

```markdown
### Reproducibility Verification

**Run 1:** 46.2 cycles (mean), 46.0 cycles (median)
**Run 2:** 46.0 cycles (mean), 46.0 cycles (median)
**Run 3:** 46.3 cycles (mean), 46.0 cycles (median)

**Aggregate:**
- Mean of means: 46.17 cycles
- Std dev between runs: 0.15 cycles
- Variability: 0.33% ✅ (<10%)
- Median: 46.0 cycles
- **Final Result:** 46.0 cycles (median of 3 runs)
```

---

## DELIVERABLES

| Artifact | Status | Content |
|:---------|:-------|:--------|
| `benches/bench_simd.rs` | [ ] | Skeleton with all benchmark functions |
| `docs/benchmarks/SIMD_TARGETS.md` | [ ] | Target specification document |
| Cycle measurement function | [ ] | `measure_cycles` with rdtsc protocol |
| Test for cycle target | [ ] | `test_simd_cycle_target` (will fail until impl) |

---

## ACCEPTANCE CRITERIA (BINARY)

**Critical (Must Pass):**
- [ ] All 4 targets documented in `SIMD_TARGETS.md`
- [ ] File `benches/bench_simd.rs` created
- [ ] Benchmark compiles: `cargo bench --bench bench_simd --no-run 2>&1`
- [ ] Cycle measurement protocol implemented with rdtsc
- [ ] Warmup count = 1,000: `grep -c "1000" benches/bench_simd.rs` ≥ 1
- [ ] Measurement count = 10,000: `grep -c "10_000" benches/bench_simd.rs` ≥ 1
- [ ] black_box usage enforced: `grep -c "black_box" benches/bench_simd.rs` ≥ 4
- [ ] Anti-hallucination guards documented in SIMD_TARGETS.md

**Verification Commands:**
```bash
# Check targets documented
test -f docs/benchmarks/SIMD_TARGETS.md && echo "✅ Targets documented"
grep -E "<50|>5x|>1B|<100ns" docs/benchmarks/SIMD_TARGETS.md | wc -l
# Should be ≥ 4

# Check benchmark compiles
cargo bench --bench bench_simd --no-run 2>&1 | grep -v error && echo "✅ Benchmark compiles"

# Check rdtsc usage
grep -c "rdtsc" benches/bench_simd.rs
# Should be > 0

# Check black_box usage
grep -c "black_box" benches/bench_simd.rs
# Should be ≥ 4 (for all inputs/outputs)
```

---

## HANDOFF

```
BENCHMARK_SCIENTIST → RUST_ENGINEER

Deliverable: Benchmark skeleton with defined targets
Status: TARGETS READY
Evidence: docs/benchmarks/SIMD_TARGETS.md exists

Next: 04_SIMD_HAMMING_IMPL.md (RUST_ENGINEER implements to hit targets)

CRITICAL CONSTRAINT for RUST_ENGINEER:
- Implementation MUST hit ALL 4 targets
- If ANY target missed → Implementation REJECTED
- Evidence required: rdtsc/criterion output, not estimates
```

---

## FAILURE PROTOCOL

### Detection

If benchmark targets cannot be defined:
- Symptom: Uncertainty about realistic cycle count
- Evidence: Stuck on target calculation for >20 minutes

### Categorization

1. **Type A: Targets seem unrealistic**
   - Action: Recalculate from operation count
   - Formula: (loads + XORs + popcounts + sums) × cycles_per_op
   - Time limit: 1 hour
   - Tools: Intel Optimization Manual, LLVM-MCA
   - Example:
     ```bash
     # Use LLVM-MCA to analyze cycle count
     echo "vpxor ymm0, ymm1, ymm2" | llvm-mca -march=x86-64 -mcpu=haswell
     ```

2. **Type B: Measurement methodology unclear**
   - Action: Research rdtsc best practices
   - Resources:
     - Intel® 64 and IA-32 Architectures Software Developer's Manual
     - Agner Fog's optimization guides
   - Time limit: 30 minutes
   - Fallback: Use criterion time estimates only (less precise)

3. **Type C: Architecture doesn't support cycle measurement**
   - Action: Escalate to META_ARCHITECT
   - Issue: "ARM/WASM doesn't have rdtsc equivalent"
   - Options:
     - A. Use criterion time estimates for non-x86
     - B. Defer cycle count requirement to x86 only
     - C. Use platform-specific performance counters

### Escalation Triggers

Escalate to PLANNER if:
- [ ] >1 hour without all 4 targets defined
- [ ] Calculated cycle count >100 (target impossible)
- [ ] Architecture fundamentally incompatible with measurement

### Alternative Paths

If targets unfeasible:
- Option A: Relax <50 to <75 cycles (document justification)
- Option B: Remove cycle count, use time-based metrics only
- Option C: Defer SIMD to Week 9 for more research

Document decision in: `docs/planning/weeks/week8/W8D37_BENCHMARK_SPEC_ISSUE.md`

---

**END OF BENCHMARK SPEC PROMPT**
