# PROMPT: W8D37 Hostile Review — SIMD Implementation

**Target Agent:** HOSTILE_REVIEWER
**Command:** `/review W8D37_simd`
**Priority:** P0 (FINAL GATE)
**Estimated Time:** 1 hour
**Dependencies:** ALL Day 37 work complete
**Output:** `docs/reviews/2025-12-12_W8D37_simd_*.md`

---

## REVIEW MANDATE

You are conducting the **FINAL QUALITY GATE** for Day 37 SIMD implementation. This code will be used in production to compare millions of vectors. Any bug here corrupts search results. Any performance miss makes the feature useless.

**Your Standards:**
- NVIDIA/JPL-grade code quality
- Zero tolerance for undocumented unsafe
- Zero tolerance for correctness failures
- Performance target is non-negotiable

**Kill Authority:** YES — You can reject Day 37 entirely.

---

## ARTIFACTS TO REVIEW

```
PRIMARY CODE:
├── src/quantization/simd.rs (or simd/mod.rs)
├── src/quantization/binary.rs (integration changes)
└── src/quantization/mod.rs (export changes)

BENCHMARKS:
├── benches/bench_simd.rs
└── docs/benchmarks/W8D37_simd_report.md

TESTS:
├── tests/simd_correctness.rs (or inline tests)
└── cargo test output

ARCHITECTURE:
└── docs/architecture/SIMD_DESIGN.md
```

---

## REVIEW DIMENSIONS

### 1. CORRECTNESS (Weight: 35%) — ZERO TOLERANCE

**Verification:**
```bash
# All tests must pass
cargo test

# Property tests must pass
cargo test simd_matches_portable

# Day 36 tests must still pass
cargo test quantization::binary
```

**Checklist:**
- [ ] Property tests prove SIMD == Portable (10,000+ cases)
- [ ] All Day 36 fuzz corpus passes through SIMD
- [ ] Edge cases tested (zeros, ones, boundaries)
- [ ] Symmetry property holds
- [ ] Self-distance is always 0

**Scoring:**
- 10/10: All tests pass, property tests comprehensive
- 5/10: Tests pass but coverage gaps
- 0/10: Any test failure → **AUTOMATIC REJECTION**

---

### 2. PERFORMANCE (Weight: 30%) — TARGET IS <50 CYCLES

**Verification:**
```bash
# Check benchmark report
cat docs/benchmarks/W8D37_simd_report.md

# Run benchmarks
cargo bench --bench bench_simd
```

**Checklist:**
- [ ] Cycle count measured and documented
- [ ] AVX2 achieves <50 cycles
- [ ] Speedup vs portable is ≥5x
- [ ] No regression on portable path
- [ ] Benchmark methodology is sound (warmup, iterations)

**Scoring:**
- 10/10: <50 cycles achieved, methodology sound
- 7/10: 50-75 cycles (acceptable with documentation)
- 5/10: 75-100 cycles (marginal, needs justification)
- 0/10: >100 cycles or no benchmark → **REJECTION**

---

### 3. SAFETY (Weight: 20%) — ALL UNSAFE DOCUMENTED

**Verification:**
```bash
# Search for unsafe blocks
grep -n "unsafe" src/quantization/simd.rs

# Each must have // SAFETY comment
grep -B5 "unsafe" src/quantization/simd.rs | grep -c "SAFETY"

# Run Miri (if supported)
cargo +nightly miri test simd
```

**Checklist:**
- [ ] Every `unsafe` block has `// SAFETY:` comment
- [ ] Safety comments explain WHY it's safe, not just WHAT it does
- [ ] Feature detection before SIMD intrinsics
- [ ] Alignment requirements documented
- [ ] No raw pointer arithmetic without bounds proof
- [ ] Miri passes (or documented why it can't run)

**Required Safety Documentation:**
```rust
// SAFETY:
// 1. AVX2 is available (checked by is_x86_feature_detected!)
// 2. Input is exactly 96 bytes (enforced by type &[u8; 96])
// 3. Input is 64-byte aligned (guaranteed by QuantizedVector repr)
// 4. Memory access is within bounds (96 ≤ 3 × 32-byte loads)
#[target_feature(enable = "avx2")]
unsafe fn hamming_distance_avx2(...) { ... }
```

**Scoring:**
- 10/10: All unsafe documented with complete safety proof
- 5/10: Some unsafe documented, minor gaps
- 0/10: Any undocumented unsafe → **REJECTION**

---

### 4. API COMPATIBILITY (Weight: 10%) — NO BREAKING CHANGES

**Verification:**
```bash
# Day 36 public API must work unchanged
cargo test --test day36_api_compat  # If exists

# Check for breaking changes
git diff HEAD~1 -- src/quantization/binary.rs | grep "^-.*pub"
```

**Checklist:**
- [ ] `QuantizedVector::hamming_distance` signature unchanged
- [ ] `QuantizedVector::similarity` unchanged
- [ ] `BinaryQuantizer::quantize` unchanged
- [ ] No new required dependencies
- [ ] Struct layout unchanged (size, alignment)

**Scoring:**
- 10/10: Zero breaking changes
- 0/10: Any breaking change → **REJECTION**

---

### 5. CODE QUALITY (Weight: 5%) — CLEAN CODE

**Verification:**
```bash
# Clippy must pass
cargo clippy -- -D warnings

# Doc warnings
cargo doc --no-deps 2>&1 | grep warning

# Format check
cargo fmt -- --check
```

**Checklist:**
- [ ] Clippy clean
- [ ] cargo doc clean
- [ ] Properly formatted
- [ ] No TODO/FIXME in production code
- [ ] Meaningful variable names
- [ ] Comments explain "why", not "what"

**Scoring:**
- 10/10: All clean
- 7/10: Minor warnings
- 5/10: Multiple warnings
- 0/10: Errors

---

## CRITICAL REVIEW CHECKLIST

### AUTO-REJECT CONDITIONS (Any = Day 37 Fails)

- [ ] Any test failure
- [ ] Undocumented unsafe block
- [ ] Performance >100 cycles (without justification)
- [ ] Breaking API change
- [ ] Miri detects undefined behavior
- [ ] Day 36 tests fail

### CONDITIONAL APPROVAL (Fix within 2 hours)

- [ ] Minor documentation gaps
- [ ] Performance 50-75 cycles (needs explanation)
- [ ] Clippy warnings (not errors)
- [ ] Missing edge case test

### FULL APPROVAL

- [ ] All tests pass
- [ ] Performance <50 cycles proven
- [ ] All unsafe documented
- [ ] Zero API changes
- [ ] Clean code quality

---

## OUTPUT FORMAT

```markdown
# W8D37 HOSTILE REVIEW REPORT: SIMD Implementation

**Status:** <APPROVED / CONDITIONAL / REJECTED>
**Overall Score:** X.X/10.0
**Review Date:** 2025-12-12
**Reviewer:** HOSTILE_REVIEWER v2.0
**Protocol:** NVIDIA/JPL-Grade SIMD Audit

---

## EXECUTIVE SUMMARY

<2-3 sentences: Overall quality, critical findings, verdict>

**Alpha Release Impact:** <Day 37 approved / Day 37 blocked>

---

## DIMENSION SCORES

| Dimension | Score | Weight | Weighted | Status |
|:----------|------:|-------:|---------:|:-------|
| Correctness | X/10 | 35% | X.XX | ✅/❌ |
| Performance | X/10 | 30% | X.XX | ✅/❌ |
| Safety | X/10 | 20% | X.XX | ✅/❌ |
| API Compatibility | X/10 | 10% | X.XX | ✅/❌ |
| Code Quality | X/10 | 5% | X.XX | ✅/❌ |

**Weighted Total:** X.XX/10.0
**Approval Threshold:** ≥8.5/10.0

---

## CORRECTNESS AUDIT

**Test Results:**
```
cargo test output here
```

**Property Test Coverage:** X/10,000 passed
**Fuzz Corpus Coverage:** X/103 passed
**Edge Cases:** X/12 passed

**Verdict:** <PASS/FAIL>

---

## PERFORMANCE AUDIT

**Benchmark Results:**
| Implementation | Cycles | Target | Status |
|:---------------|:-------|:-------|:-------|
| AVX2 SIMD | XX | <50 | ✅/❌ |
| Portable | XX | baseline | — |

**Speedup:** XXx

**Verdict:** <PASS/FAIL>

---

## SAFETY AUDIT

**Unsafe Blocks Found:** X
**Documented:** X
**Undocumented:** X

**Miri Status:** <PASS/FAIL/NOT RUN>

**Safety Issues:**
- [List any issues]

**Verdict:** <PASS/FAIL>

---

## API COMPATIBILITY AUDIT

**Breaking Changes:** <NONE/LIST>

**Day 36 Test Regression:** <NONE/LIST>

**Verdict:** <PASS/FAIL>

---

## CRITICAL ISSUES

### CRIT-001: [Title]
- **Severity:** CRITICAL
- **Location:** file:line
- **Issue:** description
- **Required Fix:** action
- **Blocking:** YES

[Repeat for each critical issue]

---

## MAJOR ISSUES

[List major issues]

---

## MINOR ISSUES

[List minor issues]

---

## POSITIVE FINDINGS

1. [Specific praise]
2. [Specific praise]
3. [Specific praise]

---

## FINAL VERDICT

**Decision:** <APPROVED / CONDITIONAL / REJECTED>

**Rationale:**
<Explanation based on scores and findings>

**Conditions (if conditional):**
1. [Condition 1]
2. [Condition 2]

**Next Steps:**
- If APPROVED: Day 37 complete, proceed to Day 38
- If CONDITIONAL: Fix issues within 2 hours, resubmit
- If REJECTED: Day 37 blocked, reassess approach

---

**Reviewer Signature:** HOSTILE_REVIEWER
**Authority:** Day 37 Quality Gate
**Kill Authority Exercised:** <YES/NO>
```

---

## POST-REVIEW ACTIONS

### IF APPROVED
```
Day 37: COMPLETE

Deliverables locked:
- src/quantization/simd.rs ✅
- docs/benchmarks/W8D37_simd_report.md ✅
- docs/reviews/2025-12-12_W8D37_simd_APPROVED.md ✅

Next: Proceed to W8D38 (if applicable) or Week 8 summary
```

### IF CONDITIONAL
```
RUST_ENGINEER must fix:
1. [Issue 1]
2. [Issue 2]

Deadline: 2 hours from review

Resubmit: /review W8D37_simd_v2
```

### IF REJECTED
```
HALT Day 37

Issues requiring fundamental rework:
1. [Critical issue]

Options:
A. Fix and extend Day 37 timeline
B. Defer SIMD to Week 9
C. Ship without SIMD (portable only)

Escalate to PLANNER for decision.
```

---

**END OF PROMPT**
