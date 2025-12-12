# PROMPT_MAKER: W8D37 Master Dispatch — SIMD Implementation

**Generated:** 2025-12-12T01:30:00Z
**Day:** 37 (Week 8, Day 2)
**Phase:** 5 (Release Polish) — Performance Optimization Track
**Prior Day:** W8D36 Binary Quantization APPROVED (9.6/10.0)
**Target:** <50 CPU cycles per 768-bit Hamming distance

---

## MISSION BRIEFING

Day 37 delivers **SIMD-accelerated Hamming distance** for the binary quantization module. The portable implementation from Day 36 achieves correctness; Day 37 achieves **PERFORMANCE**.

```
┌─────────────────────────────────────────────────────────────────────┐
│  DAY 36 (COMPLETE)          │  DAY 37 (TODAY)                       │
│  ─────────────────          │  ─────────────                        │
│  ✓ Binary quantization      │  → SIMD Hamming distance              │
│  ✓ Portable Hamming         │  → AVX2/NEON acceleration             │
│  ✓ 103 fuzz seeds           │  → <50 cycle target                   │
│  ✓ Property tests           │  → Runtime feature detection          │
│  ✓ Benchmarks (baseline)    │  → Performance validation             │
└─────────────────────────────────────────────────────────────────────┘
```

**Why SIMD Matters:** Binary quantization without fast comparison is useless. SIMD transforms 96 sequential byte operations into 3-6 vectorized operations, delivering **10-30x speedup**.

---

## PERFORMANCE TARGETS

| Metric | Baseline (Day 36) | Target (Day 37) | Speedup |
|:-------|:------------------|:----------------|:--------|
| Hamming (96 bytes) | ~300 cycles | <50 cycles | **6x** |
| Throughput | ~100M cmp/sec | >1B cmp/sec | **10x** |
| Latency P99 | ~500ns | <100ns | **5x** |

---

## PROMPT EXECUTION SEQUENCE

```
┌─────────────────────────────────────────────────────────────────────┐
│                    W8D37 EXECUTION PIPELINE                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ PHASE 1: Architecture (1 hour)                              │   │
│  │ └── 01_SIMD_ARCHITECTURE.md → META_ARCHITECT                │   │
│  │     • Target instruction sets (AVX2, NEON, WASM)            │   │
│  │     • Dispatch strategy (compile-time vs runtime)           │   │
│  │     • Module structure decision                             │   │
│  │     • Safety model definition                               │   │
│  │     OUTPUT: docs/architecture/SIMD_DESIGN.md                │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ PHASE 2: Implementation (4 hours)                           │   │
│  │ ├── 04_SIMD_HAMMING_IMPL.md → RUST_ENGINEER [PRIMARY]       │   │
│  │ │   • AVX2 hamming_distance implementation                  │   │
│  │ │   • Portable SIMD fallback (std::simd)                    │   │
│  │ │   • Runtime feature detection                             │   │
│  │ │   • Integration with QuantizedVector                      │   │
│  │ │   OUTPUT: src/quantization/simd.rs                        │   │
│  │ │                                                           │   │
│  │ └── 05_SIMD_QUANTIZE_IMPL.md → RUST_ENGINEER [SECONDARY]    │   │
│  │     • SIMD sign bit extraction (optional)                   │   │
│  │     • Vectorized bit packing (optional)                     │   │
│  │     OUTPUT: Enhanced quantize() if time permits             │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ PHASE 3: Validation (2 hours) [PARALLEL]                    │   │
│  │ ├── 04_SIMD_BENCHMARKS.md → BENCHMARK_SCIENTIST             │   │
│  │ │   • Cycle count measurement (rdtsc)                       │   │
│  │ │   • Throughput benchmarks (criterion)                     │   │
│  │ │   • Regression detection vs Day 36                        │   │
│  │ │   OUTPUT: docs/benchmarks/W8D37_simd_report.md            │   │
│  │ │                                                           │   │
│  │ └── 05_SIMD_TESTS.md → TEST_ENGINEER                        │   │
│  │     • SIMD vs portable correctness verification             │   │
│  │     • Edge case testing (alignment, special values)         │   │
│  │     • Cross-platform CI setup                               │   │
│  │     OUTPUT: tests/simd_correctness.rs                       │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ PHASE 4: Gate (1 hour)                                      │   │
│  │ └── 07_HOSTILE_REVIEW.md → HOSTILE_REVIEWER                 │   │
│  │     • Performance target validation (<50 cycles)            │   │
│  │     • Unsafe code audit (all unsafe documented)             │   │
│  │     • Fallback correctness verification                     │   │
│  │     • API compatibility check (no breaking changes)         │   │
│  │     OUTPUT: docs/reviews/2025-12-12_W8D37_*.md              │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## PROMPT FILES

| File | Agent | Priority | Est. Time (Optimistic / Realistic) | Dependencies |
|:-----|:------|:---------|:-----------------------------------|:-------------|
| `01_SIMD_ARCHITECTURE.md` | META_ARCHITECT | P0 | 1h / 3h | None |
| `02_SIMD_TEST_SPEC.md` | TEST_ENGINEER | P0 | 1.5h / 4.5h | 01 |
| `03_SIMD_BENCHMARK_SPEC.md` | BENCHMARK_SCIENTIST | P0 | 0.5h / 1.5h | 01 |
| `04_SIMD_HAMMING_IMPL.md` | RUST_ENGINEER | P0 | 3h / 9h | 02, 03 |
| `05_SIMD_QUANTIZE_IMPL.md` | RUST_ENGINEER | P2 | 1h / 3h | 02, 03 |
| `06_SIMD_VALIDATION.md` | TEST_ENGINEER + BENCHMARK_SCIENTIST | P0 | 2h / 6h | 04, 05 |
| `07_HOSTILE_REVIEW.md` | HOSTILE_REVIEWER | P0 | 1h / 3h | All |
| `08_NVIDIA_GRADE_HOSTILE_REVIEW.md` | HOSTILE_REVIEWER | P0 | 1h / 3h | All |

**Total Estimated Time:** 14 hours (phase-level estimate from PLANNER_DAY2_OPTIMIZATION.md)
**Calculation:** 11.5h base (sum of phases A-E) + 2.5h buffer = 14h
**Source:** PLANNER_DAY2_OPTIMIZATION.md lines 666-673 (phase-level breakdown with 3x rule applied)
**Note:** Individual file estimates in table above are more granular and sum to 33h realistic (or 30h excluding P2). The 14h total represents phase-level efficiency gains from parallel work and task overlap, not a direct sum of file estimates. Previous 8h estimate did not apply 3x rule.

**Reconciliation:**
- **Individual Files (Sequential):** 3 + 4.5 + 1.5 + 9 + 3 + 6 + 3 + 3 = 33h realistic (if done sequentially)
- **Phases (Parallel & Optimized):** 11.5h base + 2.5h buffer = 14h (accounts for parallel work in validation phase, reuse of context, and pipelined execution)

---

## CONTEXT FILES (MUST LOAD)

```
DAY 36 BASELINE (APPROVED):
├── src/quantization/binary.rs          # Portable implementation to optimize
├── src/quantization/mod.rs             # Module structure
├── benches/bench_quantization.rs       # Baseline benchmarks
├── fuzz/fuzz_targets/fuzz_quantization.rs
└── docs/reviews/2025-12-12_W8D36_*_APPROVED.md

ARCHITECTURE (FROZEN):
├── docs/architecture/ARCHITECTURE.md
└── docs/architecture/DATA_LAYOUT.md    # 64-byte alignment specified

STANDARDS:
├── .claude/CLAUDE.md                   # Quality standards
└── .claude/HOSTILE_GATE_CHECKLIST.md   # Review criteria
```

---

## SIMD TECHNICAL CONTEXT

### Current Portable Implementation (Day 36)
```rust
// src/quantization/binary.rs:159-170
pub fn hamming_distance(&self, other: &Self) -> u32 {
    let mut distance = 0u32;
    for i in 0..QUANTIZED_VECTOR_SIZE {  // 96 iterations!
        let xor = self.data[i] ^ other.data[i];
        distance += xor.count_ones();
    }
    distance
}
```

**Problem:** 96 sequential iterations = ~300 CPU cycles

### SIMD Opportunity
```
Data size: 96 bytes (768 bits)

AVX2 (256-bit):
├── 3 × ymm registers = 96 bytes exactly
├── vpxor: 3 instructions for XOR
├── vpopcnt (AVX-512 VPOPCNTDQ) or emulated popcnt
└── Target: ~30-50 cycles

AVX-512 (512-bit):
├── 2 × zmm registers = 128 bytes (96 used)
├── vpxorq: 2 instructions
├── vpopcntq: native popcount
└── Target: ~20-30 cycles

NEON (128-bit):
├── 6 × v registers = 96 bytes
├── veor: 6 instructions
├── vcnt: native popcount per byte
└── Target: ~40-60 cycles
```

---

## CONSTRAINT ENFORCEMENT

### ABSOLUTE CONSTRAINTS (VIOLATION = REJECTION)

| Constraint | Reason | Verification |
|:-----------|:-------|:-------------|
| No breaking API changes | Day 36 tests must pass | `cargo test` |
| Preserve struct layout | WASM/FFI compatibility | `size_of`, `align_of` |
| Document all `unsafe` | Safety audit requirement | Code review |
| Portable fallback exists | Not all CPUs have SIMD | Feature flag test |

### PERFORMANCE CONSTRAINTS

| Constraint | Target | Verification |
|:-----------|:-------|:-------------|
| Hamming distance | <50 cycles | rdtsc benchmark |
| No regression on portable | ≤Day 36 baseline | Benchmark comparison |
| Compile time | <30s incremental | CI timing |

---

## DELIVERABLES CHECKLIST

### Code Artifacts
- [ ] `src/quantization/simd.rs` (or `simd/` module)
- [ ] Updated `src/quantization/mod.rs` with SIMD exports
- [ ] Updated `src/quantization/binary.rs` integration
- [ ] `benches/bench_simd.rs` (SIMD-specific benchmarks)
- [ ] `tests/simd_correctness.rs` (SIMD vs portable verification)

### Documentation Artifacts
- [ ] `docs/architecture/SIMD_DESIGN.md`
- [ ] `docs/benchmarks/W8D37_simd_report.md`
- [ ] `docs/reviews/2025-12-12_W8D37_*.md`

### Quality Gates
- [ ] `cargo test` passes (all existing tests)
- [ ] `cargo doc --no-deps` zero warnings
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Benchmark shows <50 cycles achieved
- [ ] HOSTILE_REVIEWER approval

---

## QUICK START COMMANDS

```bash
# Load Day 36 baseline
cat src/quantization/binary.rs

# Run existing benchmarks (baseline)
cargo bench --bench bench_quantization

# Check CPU features available
rustc --print cfg | grep target_feature

# After implementation - verify correctness
cargo test

# After implementation - benchmark SIMD
cargo bench --bench bench_simd
```

---

## RISK MITIGATION

| Risk | Probability | Impact | Mitigation |
|:-----|:------------|:-------|:-----------|
| AVX2 unavailable on CI | Medium | Medium | GitHub Actions has AVX2; add portable-only CI job |
| SIMD bugs vs portable | Medium | High | Exhaustive property tests: SIMD result == portable result |
| Performance target missed | Low | High | Profile with `perf`; optimize hot path; document if <50 unachievable |
| Unsafe introduces UB | Low | Critical | Miri testing; careful safety proofs; code review |
| Compile errors on ARM | Medium | Medium | Conditional compilation; test on ARM CI |

---

## AGENT HANDOFF PROTOCOL

### Start → Architecture
```
User/PLANNER → META_ARCHITECT

Prompt: 01_SIMD_ARCHITECTURE.md
Context: Day 36 approved, need SIMD design
Expected: SIMD_DESIGN.md with module structure, dispatch strategy, safety model
```

### Architecture → Implementation
```
META_ARCHITECT → RUST_ENGINEER

Trigger: SIMD_DESIGN.md approved
Prompt: 04_SIMD_HAMMING_IMPL.md (then optionally 05)
Context: Architecture decisions locked
Expected: Working SIMD implementation
```

### Implementation → Validation
```
RUST_ENGINEER → TEST_ENGINEER + BENCHMARK_SCIENTIST

Trigger: SIMD code compiles, basic tests pass
Prompts: 04_SIMD_BENCHMARKS.md + 05_SIMD_TESTS.md (parallel)
Context: Implementation complete
Expected: Performance report, correctness verification
```

### Validation → Gate
```
TEST_ENGINEER + BENCHMARK_SCIENTIST → HOSTILE_REVIEWER

Trigger: Benchmarks show <50 cycles, tests pass
Prompt: 07_HOSTILE_REVIEW.md
Context: All deliverables ready
Expected: APPROVED/REJECTED verdict
```

---

## SUCCESS CRITERIA (Day 37 Exit)

| Criterion | Target | Verification | Blocking |
|:----------|:-------|:-------------|:---------|
| SIMD correctness | Matches portable | Property tests | YES |
| Performance | <50 cycles | Benchmark | YES |
| Unsafe documented | 100% coverage | Code review | YES |
| API unchanged | Day 36 tests pass | `cargo test` | YES |
| Fallback works | Portable still functional | Feature flag | YES |
| Zero doc warnings | Clean build | `cargo doc` | YES |
| HOSTILE approval | Score ≥8.5 | Review | YES |

---

## VERSION

| Version | Date | Author | Change |
|:--------|:-----|:-------|:-------|
| 1.0.0 | 2025-12-12 | PROMPT_MAKER | Initial Day 2 dispatch |

---

**STATUS: READY FOR EXECUTION**

**Begin with:** `01_SIMD_ARCHITECTURE.md` → META_ARCHITECT
