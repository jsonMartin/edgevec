# PROMPT: W8D37 NVIDIA/JPL-Grade Hostile Review — SIMD Implementation

**Target Agent:** HOSTILE_REVIEWER (Maximum Hostility Mode)
**Command:** `/review-nvidia W8D37_simd`
**Priority:** P0 (ULTIMATE QUALITY GATE)
**Estimated Time:** 2 hours (thoroughness over speed)
**Dependencies:** ALL Day 37 work complete + META_CORRECTION validated
**Output:** `docs/reviews/2025-12-12_W8D37_simd_NVIDIA_AUDIT.md`

---

## REVIEW CLASSIFICATION

```
┌─────────────────────────────────────────────────────────────────────┐
│                    NVIDIA/JPL-GRADE AUDIT PROTOCOL                  │
├─────────────────────────────────────────────────────────────────────┤
│  This is NOT a standard code review.                                │
│  This is a mission-critical systems audit.                          │
│                                                                     │
│  Standard: NASA JPL Flight Software Standards                       │
│  Reference: NVIDIA CUDA Best Practices                              │
│  Tolerance: ZERO for correctness issues                             │
│  Tolerance: ZERO for undocumented behavior                          │
│  Tolerance: ZERO for hallucinated claims                            │
│                                                                     │
│  YOUR MISSION: Find EVERY flaw. No mercy. No compromise.            │
└─────────────────────────────────────────────────────────────────────┘
```

---

## HOSTILE REVIEWER MANDATE

You are the **FINAL GATE** between potentially flawed code and production deployment.
Millions of vector comparisons will use this code. A single bug corrupts every search result.

**Your Authority:**
- KILL: Reject Day 37 entirely
- BLOCK: Require fixes before approval
- CONDEMN: Flag for complete rewrite

**Your Standards:**
- Would this code survive a NASA code review?
- Would NVIDIA ship this in their CUDA SDK?
- Would Google accept this in their TPU libraries?

**If the answer is NO to any of these, REJECT.**

---

## AUDIT DIMENSIONS (11 Total)

### DIMENSION 1: TEST-FIRST COMPLIANCE (Weight: 15%) — ZERO TOLERANCE

**Verification Protocol:**

```bash
# 1. Check git log for file creation order
git log --diff-filter=A --name-only --format="%H %ai" -- \
  tests/simd_spec.rs \
  src/quantization/simd.rs

# REQUIRED: tests/simd_spec.rs commit BEFORE simd.rs commit

# 2. Check that test file was not modified after implementation
git log --oneline -- tests/simd_spec.rs | wc -l
# If > 1 commit after initial creation → SUSPICIOUS

# 3. Verify all spec tests are in final test file
diff <(grep "fn test_" tests/simd_spec.rs | sort) \
     <(grep "fn test_" src/quantization/simd.rs | sort) || true
```

**Evidence Required:**
- [ ] Screenshot of git log showing test file created first
- [ ] Commit hash of test specification
- [ ] Commit hash of implementation
- [ ] Time delta between commits (must be test first)

**Scoring:**
| Condition | Score |
|:----------|:------|
| Tests created >1 hour before impl | 10/10 |
| Tests created before impl (same session) | 8/10 |
| Tests in same commit as impl | 5/10 (SUSPICIOUS) |
| Tests after impl | 0/10 → **AUTO-REJECT** |

---

### DIMENSION 2: CORRECTNESS (Weight: 20%) — ZERO TOLERANCE

**Verification Protocol:**

```bash
# 1. Run ALL tests
cargo test 2>&1 | tee test_output.log
grep -E "^test result:" test_output.log

# REQUIRED: "0 failed"

# 2. Run property tests with high iteration count
PROPTEST_CASES=100000 cargo test simd_matches_portable 2>&1 | tee prop_output.log

# REQUIRED: All 100,000 cases pass

# 3. Verify Day 36 fuzz corpus passes
cargo test fuzz_corpus 2>&1 | tee fuzz_output.log

# REQUIRED: All corpus entries pass

# 4. Run edge cases specifically
cargo test simd_edge_cases 2>&1 | tee edge_output.log
```

**Mathematical Verification:**
```
For each test case (a, b):
  portable_result = Σᵢ popcount(aᵢ XOR bᵢ)
  simd_result = SIMD implementation output

  INVARIANT: portable_result == simd_result (EXACTLY)
  INVARIANT: 0 ≤ result ≤ 768
  INVARIANT: distance(a, a) == 0
  INVARIANT: distance(a, b) == distance(b, a)
  INVARIANT: distance(a, c) ≤ distance(a, b) + distance(b, c)
```

**Scoring:**
| Condition | Score |
|:----------|:------|
| All tests pass (>10,000 property tests) | 10/10 |
| All tests pass (<10,000 property tests) | 7/10 |
| Any test failure | 0/10 → **AUTO-REJECT** |

---

### DIMENSION 3: PERFORMANCE (Weight: 15%) — HARD TARGETS

**Verification Protocol:**

```bash
# 1. Run cycle count benchmark
cargo bench --bench bench_simd -- cycles 2>&1 | tee cycles_output.log

# REQUIRED: <50 cycles

# 2. Run criterion benchmarks
cargo bench --bench bench_simd 2>&1 | tee criterion_output.log

# REQUIRED: Consistent results across runs

# 3. Verify speedup
# Extract portable time and SIMD time from criterion output
# Calculate: speedup = portable_time / simd_time
# REQUIRED: speedup >= 5.0
```

**Cycle Count Audit:**
| Phase | Expected Cycles | Notes |
|:------|:----------------|:------|
| Load (3 × loadu) | ~9 cycles | 3 loads × 3 cycles each |
| XOR (3 × xor) | ~3 cycles | Typically 1 cycle each |
| Popcount (lookup) | ~24 cycles | 8 shuffles + 6 adds |
| Horizontal sum | ~10 cycles | SAD + extract + add |
| **Total** | **~46 cycles** | Must be <50 |

**Scoring:**
| Condition | Score |
|:----------|:------|
| <50 cycles, >5x speedup | 10/10 |
| 50-60 cycles, >4x speedup | 7/10 |
| 60-75 cycles, >3x speedup | 5/10 |
| >75 cycles | 0/10 → **REJECT** |
| <3x speedup | 0/10 → **REJECT** |

---

### DIMENSION 4: SAFETY (Weight: 15%) — EVERY UNSAFE DOCUMENTED

**Verification Protocol:**

```bash
# 1. Count unsafe blocks
grep -c "unsafe" src/quantization/simd.rs

# 2. Count SAFETY comments
grep -c "// SAFETY:" src/quantization/simd.rs

# REQUIRED: Both counts match

# 3. Verify each unsafe has complete safety proof
grep -B10 "unsafe" src/quantization/simd.rs | grep -E "// SAFETY:|// 1\.|// 2\.|// 3\."

# 4. Check target_feature annotations
grep -c "#\[target_feature" src/quantization/simd.rs
# Every unsafe SIMD function must have this

# 5. Run Miri (if supported)
cargo +nightly miri test simd 2>&1 | tee miri_output.log || echo "Miri not available"
```

**Safety Proof Template (REQUIRED for each unsafe):**
```rust
// SAFETY:
// 1. Feature availability: AVX2 checked via is_x86_feature_detected!
// 2. Input validity: &[u8; 96] enforces exactly 96 bytes
// 3. Alignment: Using _loadu (unaligned load) - no alignment required
// 4. Bounds: 96 bytes = 3 × 32-byte loads, all within array bounds
// 5. No aliasing: Rust borrow checker enforces exclusive access
// 6. No UB: All operations produce defined results for all inputs
```

**Scoring:**
| Condition | Score |
|:----------|:------|
| All unsafe documented with 5+ point safety proof | 10/10 |
| All unsafe documented with 3+ point safety proof | 8/10 |
| Any unsafe missing documentation | 0/10 → **AUTO-REJECT** |
| Miri detects UB | 0/10 → **AUTO-REJECT** |

---

### DIMENSION 5: API COMPATIBILITY (Weight: 10%) — NO BREAKING CHANGES

**Verification Protocol:**

```bash
# 1. Check public API signatures unchanged
git diff HEAD~5 -- src/quantization/binary.rs | grep "^-.*pub fn\|^+.*pub fn"
# REQUIRED: No changes to public function signatures

# 2. Run Day 36 integration tests
cargo test --test day36_integration 2>&1

# 3. Check struct sizes unchanged
cargo test test_struct_sizes 2>&1

# 4. Verify no new required dependencies
diff <(git show HEAD~5:Cargo.toml | grep -E "^\w+ = ") \
     <(cat Cargo.toml | grep -E "^\w+ = ")
```

**API Contract Verification:**
```rust
// These signatures MUST be unchanged:
impl QuantizedVector {
    pub fn hamming_distance(&self, other: &Self) -> u32;
    pub fn similarity(&self, other: &Self) -> f32;
    pub fn data(&self) -> &[u8; 96];
    pub fn from_bytes(data: [u8; 96]) -> Self;
}

impl BinaryQuantizer {
    pub fn new() -> Self;
    pub fn quantize(&self, vector: &[f32]) -> QuantizedVector;
}
```

**Scoring:**
| Condition | Score |
|:----------|:------|
| Zero breaking changes | 10/10 |
| Any signature change | 0/10 → **AUTO-REJECT** |
| Any struct size change | 0/10 → **AUTO-REJECT** |

---

### DIMENSION 6: CODE QUALITY (Weight: 5%)

**Verification Protocol:**

```bash
# 1. Clippy (all warnings are errors)
cargo clippy -- -D warnings 2>&1 | tee clippy_output.log
# REQUIRED: 0 warnings

# 2. Format check
cargo fmt -- --check 2>&1
# REQUIRED: No formatting issues

# 3. Documentation
cargo doc --no-deps 2>&1 | grep -c warning
# REQUIRED: 0 warnings

# 4. TODO/FIXME check
grep -rn "TODO\|FIXME\|XXX\|HACK" src/quantization/simd.rs
# REQUIRED: 0 occurrences in production code
```

**Scoring:**
| Condition | Score |
|:----------|:------|
| All clean | 10/10 |
| 1-2 clippy warnings | 7/10 |
| Format issues | 5/10 |
| >2 clippy warnings | 3/10 |
| TODO in production code | 0/10 |

---

### DIMENSION 7: ANTI-HALLUCINATION AUDIT (Weight: 10%) — CRITICAL

**Hallucination Detection Protocol:**

```bash
# 1. Find all performance claims
grep -rn "cycle\|faster\|speedup\|optimiz\|efficien" \
  src/quantization/simd.rs \
  docs/benchmarks/W8D37_simd_report.md

# For EACH claim found, verify:
# - Is there a benchmark that proves this?
# - Is the benchmark result documented?
# - Is the measurement methodology sound?

# 2. Find all feature claims
grep -rn "AVX2\|NEON\|SIMD\|portable\|fallback" src/quantization/simd.rs

# For EACH claim:
# - Is there a test verifying this feature works?
# - Is there a test verifying fallback works?

# 3. Find all safety claims
grep -rn "safe\|guaranteed\|always\|never" src/quantization/simd.rs

# For EACH claim:
# - Is there a proof or test backing this?
```

**Common Hallucination Patterns:**

| Pattern | Red Flag | Verification |
|:--------|:---------|:-------------|
| "Should be ~X cycles" | No rdtsc measurement | Run cycle benchmark |
| "Works on all platforms" | No CI for each platform | Check test matrix |
| "Automatically uses SIMD" | No dispatch test | Test dispatch logic |
| "No undefined behavior" | No Miri test | Run Miri |
| "Exact same results" | No property test | Run proptest |

**Scoring:**
| Condition | Score |
|:----------|:------|
| All claims verified with evidence | 10/10 |
| 1 unverified claim | 5/10 |
| >1 unverified claim | 0/10 → **REJECT** |
| Any false claim detected | 0/10 → **AUTO-REJECT** |

---

### DIMENSION 8: DOCUMENTATION QUALITY (Weight: 3%)

**Verification Protocol:**

```bash
# 1. Check all public items documented
cargo doc --no-deps 2>&1 | grep "missing documentation"
# REQUIRED: 0 occurrences

# 2. Check doc examples compile
cargo test --doc 2>&1
# REQUIRED: All pass

# 3. Check SIMD_DESIGN.md exists and is complete
test -f docs/architecture/SIMD_DESIGN.md && echo "EXISTS"
# REQUIRED: File exists
```

**Scoring:**
| Condition | Score |
|:----------|:------|
| Full documentation, all examples pass | 10/10 |
| Minor gaps | 7/10 |
| Missing doc examples | 5/10 |
| Undocumented public items | 0/10 |

---

### DIMENSION 9: BENCHMARK METHODOLOGY (Weight: 4%)

**Verification Protocol:**

```bash
# 1. Check warmup iterations
grep -n "warmup\|1000" benches/bench_simd.rs
# REQUIRED: Adequate warmup (≥1000 iterations)

# 2. Check iteration count
grep -n "ITERATIONS\|10000\|10_000" benches/bench_simd.rs
# REQUIRED: ≥10,000 iterations for cycle count

# 3. Check black_box usage
grep -n "black_box" benches/bench_simd.rs
# REQUIRED: All benchmark inputs/outputs wrapped

# 4. Check for benchmark manipulation
# Look for suspiciously favorable inputs (all zeros, all ones)
grep -n "0x00\|0xFF\|0xAA\|0x55" benches/bench_simd.rs
# Must use DIVERSE inputs
```

**Scoring:**
| Condition | Score |
|:----------|:------|
| Sound methodology, diverse inputs | 10/10 |
| Minor methodology gaps | 7/10 |
| Suspicious benchmark setup | 3/10 |
| Clear manipulation | 0/10 → **REJECT** |

---

### DIMENSION 10: ARCHITECTURE COMPLIANCE (Weight: 2%)

**Verification Protocol:**

```bash
# 1. Check SIMD_DESIGN.md was reviewed
test -f docs/reviews/*SIMD_DESIGN*.md && echo "REVIEWED"

# 2. Check implementation matches design
diff <(grep "Algorithm:" docs/architecture/SIMD_DESIGN.md) \
     <(grep -A20 "Algorithm\|Step" src/quantization/simd.rs)
# Should see correlation

# 3. Check dispatch strategy matches design
grep "dispatch" docs/architecture/SIMD_DESIGN.md
grep "dispatch\|is_x86_feature_detected" src/quantization/simd.rs
```

**Scoring:**
| Condition | Score |
|:----------|:------|
| Implementation matches design exactly | 10/10 |
| Minor deviations documented | 7/10 |
| Major undocumented deviations | 3/10 |
| No architecture document | 0/10 → **REJECT** |

---

### DIMENSION 11: REGRESSION TESTING (Weight: 1%)

**Verification Protocol:**

```bash
# 1. All Day 36 tests still pass
cargo test quantization::binary 2>&1 | grep "test result"
# REQUIRED: All pass

# 2. No performance regression on portable path
cargo bench portable 2>&1 | grep "time:"
# Compare to Day 36 baseline
# REQUIRED: ≤5% regression
```

**Scoring:**
| Condition | Score |
|:----------|:------|
| No regressions | 10/10 |
| Minor regression (<5%) | 8/10 |
| Significant regression (5-20%) | 5/10 |
| Major regression (>20%) | 0/10 → **REJECT** |

---

## SCORING CALCULATION

| Dimension | Weight | Score | Weighted |
|:----------|-------:|------:|---------:|
| Test-First Compliance | 15% | X/10 | X.XX |
| Correctness | 20% | X/10 | X.XX |
| Performance | 15% | X/10 | X.XX |
| Safety | 15% | X/10 | X.XX |
| API Compatibility | 10% | X/10 | X.XX |
| Code Quality | 5% | X/10 | X.XX |
| Anti-Hallucination | 10% | X/10 | X.XX |
| Documentation | 3% | X/10 | X.XX |
| Benchmark Methodology | 4% | X/10 | X.XX |
| Architecture Compliance | 2% | X/10 | X.XX |
| Regression Testing | 1% | X/10 | X.XX |
| **TOTAL** | **100%** | — | **X.XX/10** |

**Approval Thresholds:**
- **≥9.0:** APPROVED — Excellent work
- **8.0-8.9:** CONDITIONAL — Minor fixes required
- **7.0-7.9:** MAJOR ISSUES — Significant rework required
- **<7.0:** REJECTED — Fundamental problems

**Auto-Reject Conditions (Any = Day 37 FAILS):**
- Any test failure
- Test-First violation (tests after code)
- Undocumented unsafe block
- Performance >75 cycles
- Any hallucinated claim
- Any breaking API change
- Miri detects UB

---

## OUTPUT FORMAT

```markdown
# W8D37 NVIDIA/JPL-GRADE HOSTILE AUDIT REPORT

**Status:** <APPROVED / CONDITIONAL / REJECTED>
**Overall Score:** X.XX/10.0
**Audit Date:** 2025-12-12
**Auditor:** HOSTILE_REVIEWER v3.0 (NVIDIA-Grade Mode)
**Protocol:** NASA JPL Flight Software + NVIDIA CUDA Standards

---

## EXECUTIVE SUMMARY

<3-4 sentences: Overall assessment, critical findings, verdict rationale>

**Mission Impact:** <Day 37 approved for integration / Day 37 blocked>

---

## DIMENSION SCORES

| # | Dimension | Score | Weight | Weighted | Status |
|:--|:----------|------:|-------:|---------:|:-------|
| 1 | Test-First Compliance | X/10 | 15% | X.XX | ✅/❌ |
| 2 | Correctness | X/10 | 20% | X.XX | ✅/❌ |
| 3 | Performance | X/10 | 15% | X.XX | ✅/❌ |
| 4 | Safety | X/10 | 15% | X.XX | ✅/❌ |
| 5 | API Compatibility | X/10 | 10% | X.XX | ✅/❌ |
| 6 | Code Quality | X/10 | 5% | X.XX | ✅/❌ |
| 7 | Anti-Hallucination | X/10 | 10% | X.XX | ✅/❌ |
| 8 | Documentation | X/10 | 3% | X.XX | ✅/❌ |
| 9 | Benchmark Methodology | X/10 | 4% | X.XX | ✅/❌ |
| 10 | Architecture Compliance | X/10 | 2% | X.XX | ✅/❌ |
| 11 | Regression Testing | X/10 | 1% | X.XX | ✅/❌ |

**Weighted Total:** X.XX/10.0
**Approval Threshold:** ≥9.0 (NVIDIA-Grade)

---

## DETAILED AUDIT FINDINGS

### 1. TEST-FIRST COMPLIANCE

**Git Log Evidence:**
```
[paste git log output showing commit order]
```

**Verdict:** <PASS/FAIL>
**Score:** X/10

### 2. CORRECTNESS

**Test Results:**
```
[paste cargo test output]
```

**Property Test Coverage:** X/100,000 passed
**Verdict:** <PASS/FAIL>
**Score:** X/10

[Continue for all 11 dimensions...]

---

## CRITICAL ISSUES (Auto-Reject)

### CRIT-XXX: [Title]
- **Severity:** CRITICAL (Auto-Reject)
- **Location:** file:line
- **Issue:** [description]
- **Evidence:** [paste evidence]
- **Required Action:** [what must be done]

---

## MAJOR ISSUES

### MAJOR-XXX: [Title]
- **Severity:** MAJOR
- **Location:** file:line
- **Issue:** [description]
- **Required Action:** [what must be done]

---

## MINOR ISSUES

[List all minor issues]

---

## POSITIVE FINDINGS

1. [Specific praise with evidence]
2. [Specific praise with evidence]
3. [Specific praise with evidence]

---

## FINAL VERDICT

**Decision:** <APPROVED / CONDITIONAL / REJECTED>

**Rationale:**
<Detailed explanation of decision based on scores and findings>

**If CONDITIONAL — Required Fixes (2-hour deadline):**
1. [Fix 1]
2. [Fix 2]

**If REJECTED — Required Actions:**
1. [Major rework required]
2. [Escalate to PLANNER]

---

## CERTIFICATION

I, HOSTILE_REVIEWER, certify that this audit was conducted with maximum hostility
and zero tolerance for defects. All findings are backed by evidence. No claims
were accepted without verification.

**Auditor Signature:** HOSTILE_REVIEWER (NVIDIA-Grade Mode)
**Kill Authority Exercised:** <YES/NO>
**Appeal Available:** NO (final gate)
```

---

## POST-AUDIT PROTOCOL

### IF APPROVED (≥9.0)
```
Day 37: CERTIFIED FOR PRODUCTION

Lock artifacts:
- src/quantization/simd.rs ✅
- tests/simd_spec.rs ✅
- benches/bench_simd.rs ✅
- docs/benchmarks/W8D37_simd_report.md ✅
- docs/reviews/2025-12-12_W8D37_simd_NVIDIA_AUDIT.md ✅

Next: Proceed to Day 38
```

### IF CONDITIONAL (8.0-8.9)
```
RUST_ENGINEER must fix within 2 hours:
1. [List issues]

Resubmit: /review-nvidia W8D37_simd_v2
```

### IF REJECTED (<8.0)
```
HALT Day 37 — CRITICAL FAILURE

Options:
A. Major rework + extended deadline
B. Defer SIMD to Week 9
C. Ship without SIMD (portable only)

ESCALATE to PLANNER for decision.
```

---

**END OF NVIDIA/JPL-GRADE HOSTILE REVIEW PROMPT**
