# PROMPT: SIMD Validation & Verification

**Target Agent:** TEST_ENGINEER + BENCHMARK_SCIENTIST
**Command:** `/test-validate simd` + `/bench-validate simd`
**Priority:** P0 (BLOCKING — Final verification before hostile review)
**Estimated Time:** 2 hours (realistic: 6h with 3x rule)
**Dependencies:**
  - `04_SIMD_HAMMING_IMPL.md` complete (implementation exists)
  - `05_SIMD_QUANTIZE_IMPL.md` complete OR skipped with justification
  - `tests/simd_spec.rs` exists (from 02_SIMD_TEST_SPEC.md)
  - `benches/bench_simd.rs` exists (from 03_SIMD_BENCHMARK_SPEC.md)
**Output:**
  - Test execution results
  - Benchmark validation report
  - `docs/benchmarks/W8D37_VALIDATION_REPORT.md`

---

## MISSION

Execute ALL tests and benchmarks to verify SIMD implementation meets specifications defined in 02_SIMD_TEST_SPEC.md and 03_SIMD_BENCHMARK_SPEC.md.

**Validation-First Principle:**
> "Implementation is not complete until ALL specifications are verified passing."

**CRITICAL REQUIREMENT:** Every claim in this validation MUST be backed by pasted command output.

---

## DEPENDENCY VERIFICATION

Before proceeding, verify ALL dependencies complete:

```bash
# Check implementation exists
test -f src/quantization/simd.rs || { echo "BLOCK: Implementation missing"; exit 1; }

# Check test spec exists
test -f tests/simd_spec.rs || { echo "BLOCK: Test spec missing"; exit 1; }
grep -c "#\[test\]" tests/simd_spec.rs | awk '$1 >= 25 || { print "BLOCK: <25 tests"; exit 1; }'

# Check benchmark spec exists
test -f benches/bench_simd.rs || { echo "BLOCK: Benchmark spec missing"; exit 1; }
test -f docs/benchmarks/SIMD_TARGETS.md || { echo "BLOCK: Targets missing"; exit 1; }

# Check SIMD implementation compiles
cargo build --lib 2>&1 | grep -q error && { echo "BLOCK: Implementation doesn't compile"; exit 1; }

echo "✅ All dependencies verified"
```

**Expected Output:**
```
✅ All dependencies verified
```

**If ANY check fails, STOP and escalate to RUST_ENGINEER.**

---

## VALIDATION PROTOCOL

### Phase 1: Unit Test Execution (TEST_ENGINEER)

**Execute ALL unit tests from simd_spec.rs:**

```bash
cargo test simd_spec --lib -- --nocapture
```

**Expected Outcome:**
- ALL 25+ tests pass
- 0 failures
- 0 panics
- Property tests execute 10,000 cases each

**Evidence Required (paste full output):**

```
running 25 tests
test simd_correctness::test_simd_matches_portable_zeros ... ok
test simd_correctness::test_simd_matches_portable_ones ... ok
test simd_correctness::test_simd_matches_portable_alternating ... ok
[... all 25 tests ...]
test simd_properties::prop_simd_matches_portable ... ok (10000 cases)
test simd_properties::prop_simd_symmetric ... ok (10000 cases)
test simd_properties::prop_simd_self_zero ... ok (10000 cases)
test simd_properties::prop_simd_triangle_inequality ... ok (10000 cases)
test simd_properties::prop_simd_bounded ... ok (10000 cases)

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured

Doc-tests edgevec

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured
```

**Acceptance Criteria:**
- [ ] ALL tests pass (0 failures)
- [ ] Property tests show "(10000 cases)" in output
- [ ] No panics or unwrap violations
- [ ] Full output pasted in validation report

---

### Phase 2: Benchmark Execution (BENCHMARK_SCIENTIST)

**Execute ALL benchmarks from bench_simd.rs:**

```bash
cargo bench --bench bench_simd
```

**Target Verification (from 03_SIMD_BENCHMARK_SPEC.md):**

| Metric | Target | Hard Limit | Actual | Status |
|:-------|:-------|:-----------|:-------|:-------|
| AVX2 Cycles | <50 | <75 | TBD | TBD |
| Speedup vs Portable | >5x | >3x | TBD | TBD |
| Throughput | >1B ops/sec | >500M ops/sec | TBD | TBD |
| Latency P99 | <100ns | <200ns | TBD | TBD |

**Evidence Required (paste criterion output):**

```
simd_hamming_96bytes_cycles
                        time:   [14.2 ns 14.4 ns 14.6 ns]
                        thrpt:  [6.85 Gelem/s 6.94 Gelem/s 7.04 Gelem/s]

simd_vs_portable/simd_avx2
                        time:   [14.2 ns 14.4 ns 14.6 ns]
simd_vs_portable/portable
                        time:   [78.3 ns 78.8 ns 79.3 ns]
                        change: [+445.1% +447.2% +449.4%] (p = 0.00 < 0.05)
                        Performance has regressed.

hamming_ops_per_sec     time:   [14.2 ns 14.4 ns 14.6 ns]
                        thrpt:  [1.12 Gelem/s 1.13 Gelem/s 1.14 Gelem/s]
```

**Cycle Count Measurement (rdtsc):**

```bash
cargo test test_simd_cycle_target --release -- --nocapture
```

**Expected Output:**

```
Measured cycles: 46
test test_simd_cycle_target ... ok
```

**Acceptance Criteria:**
- [ ] Cycle count <50 (target met) OR <75 (hard limit met with justification)
- [ ] Speedup >5x (target met) OR >3x (hard limit met with justification)
- [ ] Throughput >1B ops/sec (target met) OR >500M ops/sec (hard limit met)
- [ ] Latency P99 <100ns (target met) OR <200ns (hard limit met)
- [ ] Full criterion output pasted in validation report
- [ ] rdtsc cycle measurement pasted in validation report

---

### Phase 3: Cross-Platform Verification (TEST_ENGINEER)

**Required Platforms (from 02_SIMD_TEST_SPEC.md - Cross-Platform Test Matrix):**

Execute tests on:
- [x] x86_64 (Linux) with AVX2
- [ ] x86_64 (Linux) without AVX2 (portable fallback)
- [ ] ARM64 (Linux) with NEON (if available)
- [ ] WASM (browser or wasmtime)

**For Each Platform:**

```bash
# x86_64 with AVX2 (already done in Phase 1)

# x86_64 portable (disable AVX2)
RUSTFLAGS="-C target-feature=-avx2" cargo test simd_spec

# ARM64 NEON (if ARM hardware available)
cross test --target aarch64-unknown-linux-gnu simd_spec

# WASM (if wasm-pack available)
wasm-pack test --node
```

**Acceptance Criteria:**
- [ ] x86_64 AVX2: ALL tests pass
- [ ] x86_64 portable: ALL tests pass (may be slower)
- [ ] ARM64 NEON: ALL tests pass OR platform not available (document)
- [ ] WASM: ALL tests pass OR skipped with justification

**Evidence:** Paste test output for each platform attempted.

---

### Phase 4: Regression Detection (BENCHMARK_SCIENTIST)

**Compare Against Day 36 Baseline:**

```bash
# Verify portable implementation performance hasn't regressed
cargo bench --bench bench_quantization -- hamming_distance
```

**Acceptance Criteria:**
- [ ] Portable performance ≤ Day 36 baseline (no regression)
- [ ] SIMD performance > portable performance (speedup achieved)
- [ ] No unexpected performance degradation in other quantization functions

**Evidence:** Paste benchmark comparison output.

---

## ANTI-HALLUCINATION CLAMPS

### Forbidden Phrases

**NEVER claim validation without evidence:**
- ❌ "Tests probably pass"
- ❌ "Should be correct"
- ❌ "Appears to work"
- ❌ "Likely no bugs"
- ❌ "Approximately X tests"
- ❌ "About Y ns latency"

**REQUIRED Evidence Format:**
- ✅ "All 25 tests pass (cargo test output pasted below)"
- ✅ "10,000 property test cases passed (proptest output shows '10000 cases')"
- ✅ "0 failures in test execution (full output: test result: ok. 25 passed; 0 failed)"
- ✅ "Measured: 46 cycles (rdtsc output pasted)"
- ✅ "Benchmark: 5.2x speedup (criterion output: 14.2ns vs 73.8ns)"

### Verification Protocol

Every validation claim MUST include:
1. **Test output:** Full `cargo test` output pasted (not summarized)
2. **Property test stats:** Exact case count from proptest
3. **Benchmark results:** Full criterion output with time/throughput
4. **Cycle measurements:** rdtsc output with exact cycle count
5. **Cross-platform:** Results from each attempted platform

**Example Evidence Format:**

```markdown
## Validation Results

### Unit Tests (Phase 1)

**Command:** `cargo test simd_spec --lib -- --nocapture`

**Output:**
\`\`\`
running 25 tests
test simd_correctness::test_simd_matches_portable_zeros ... ok
[... full output ...]
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured
\`\`\`

**Status:** ✅ PASS (all 25 tests passed, 0 failures)

### Property Tests (Phase 1)

**Output:**
\`\`\`
test simd_properties::prop_simd_matches_portable ... ok (10000 cases)
test simd_properties::prop_simd_symmetric ... ok (10000 cases)
test simd_properties::prop_simd_self_zero ... ok (10000 cases)
test simd_properties::prop_simd_triangle_inequality ... ok (10000 cases)
test simd_properties::prop_simd_bounded ... ok (10000 cases)
\`\`\`

**Status:** ✅ PASS (5 property tests, 10,000 cases each, all passing)

### Cycle Count (Phase 2)

**Command:** `cargo test test_simd_cycle_target --release -- --nocapture`

**Output:**
\`\`\`
Measured cycles: 46
test test_simd_cycle_target ... ok
\`\`\`

**Target:** <50 cycles
**Actual:** 46 cycles
**Status:** ✅ PASS

### Throughput (Phase 2)

**Command:** `cargo bench --bench bench_simd -- hamming_ops_per_sec`

**Output:**
\`\`\`
hamming_ops_per_sec     time:   [14.2 ns 14.4 ns 14.6 ns]
                        thrpt:  [1.12 Gelem/s 1.13 Gelem/s 1.14 Gelem/s]
\`\`\`

**Target:** >1B ops/sec
**Actual:** 1.13B ops/sec
**Status:** ✅ PASS
```

### Rejection Criteria

Validation is REJECTED if:
- [ ] Any test claim lacks pasted output
- [ ] Claims use "approximately", "about", "roughly", "should be", "probably"
- [ ] Benchmark numbers not from actual criterion/rdtsc output
- [ ] Property test case count not verified from output
- [ ] Any test failure not acknowledged and justified
- [ ] Cross-platform execution not attempted or not documented

---

## OUTPUT: VALIDATION REPORT

Create: `docs/benchmarks/W8D37_VALIDATION_REPORT.md`

**Template:**

```markdown
# Week 8 Day 37 — SIMD Validation Report

**Date:** 2025-12-12
**Validator:** TEST_ENGINEER + BENCHMARK_SCIENTIST
**Implementation:** src/quantization/simd.rs
**Status:** [PASS | CONDITIONAL | FAIL]

---

## Executive Summary

- **Unit Tests:** X passed, Y failed
- **Property Tests:** X passed, Y failed, Z total cases
- **Benchmarks:** [ALL TARGETS MET | SOME TARGETS MISSED | FAILED]
- **Cross-Platform:** X platforms verified
- **Overall:** [APPROVED FOR HOSTILE REVIEW | REVISIONS REQUIRED]

---

## Phase 1: Unit Test Results

[Paste full test output here]

**Analysis:**
- [Interpret results]
- [Note any failures or warnings]

---

## Phase 2: Benchmark Results

### Cycle Count

[Paste rdtsc output]

**Analysis:**
- Target: <50 cycles
- Actual: X cycles
- Status: [PASS | FAIL]

### Speedup

[Paste criterion comparison]

**Analysis:**
- Target: >5x
- Actual: Xx speedup
- Status: [PASS | FAIL]

### Throughput

[Paste criterion throughput]

**Analysis:**
- Target: >1B ops/sec
- Actual: X ops/sec
- Status: [PASS | FAIL]

### Latency P99

[Paste criterion P99]

**Analysis:**
- Target: <100ns
- Actual: X ns
- Status: [PASS | FAIL]

---

## Phase 3: Cross-Platform Results

### x86_64 AVX2
[Paste output]

### x86_64 Portable
[Paste output or "Not tested - reason"]

### ARM64 NEON
[Paste output or "Not available - reason"]

### WASM
[Paste output or "Not tested - reason"]

---

## Phase 4: Regression Check

[Paste benchmark comparison]

**Analysis:**
- Portable performance: [NO REGRESSION | REGRESSION DETECTED]
- SIMD vs portable: [SPEEDUP ACHIEVED | NO SPEEDUP]

---

## Issues Found

[List any issues, failures, warnings, or deviations from spec]

OR

No issues found. All specifications met.

---

## Recommendation

**Status:** [APPROVED | CONDITIONAL | REJECTED]

**Justification:**
[Explain recommendation based on evidence]

**Next Step:**
- If APPROVED: Proceed to 07_HOSTILE_REVIEW.md
- If CONDITIONAL: List conditions for approval
- If REJECTED: List required fixes
```

---

## ACCEPTANCE CRITERIA (BINARY)

**Critical (Must Pass for Approval):**

- [ ] File `docs/benchmarks/W8D37_VALIDATION_REPORT.md` created
  - Verify: `test -f docs/benchmarks/W8D37_VALIDATION_REPORT.md`
  - Expected: Exit code 0

- [ ] ALL unit tests pass (25+)
  - Verify: `cargo test simd_spec 2>&1 | grep "test result: ok"`
  - Expected: Output contains "test result: ok. X passed; 0 failed" where X ≥ 25

- [ ] ALL property tests pass with 10,000 cases
  - Verify: `cargo test simd_spec 2>&1 | grep -c "ok (10000 cases)"`
  - Expected: Count ≥ 5

- [ ] Cycle count target met
  - Verify: `cargo test test_simd_cycle_target --release 2>&1 | grep "Measured cycles"`
  - Expected: "Measured cycles: X" where X < 50 (or X < 75 with justification)

- [ ] Speedup target met
  - Verify: Criterion output shows SIMD time < portable time / 5
  - Expected: Calculated speedup > 5x (or > 3x with justification)

- [ ] Throughput target met
  - Verify: Criterion output shows throughput >1B ops/sec
  - Expected: "thrpt: [X Gelem/s ...]" where X > 1.0

- [ ] Full test output pasted in validation report
  - Verify: `grep -c "running .* tests" docs/benchmarks/W8D37_VALIDATION_REPORT.md`
  - Expected: Count ≥ 1 (test output pasted)

- [ ] Full benchmark output pasted in validation report
  - Verify: `grep -c "time:.*ns" docs/benchmarks/W8D37_VALIDATION_REPORT.md`
  - Expected: Count ≥ 4 (benchmark results pasted)

- [ ] Validation report includes recommendation
  - Verify: `grep -E "(APPROVED|CONDITIONAL|REJECTED)" docs/benchmarks/W8D37_VALIDATION_REPORT.md | tail -1`
  - Expected: Clear final status

**Verification Script (Run All Checks):**

```bash
# Check validation report created
test -f docs/benchmarks/W8D37_VALIDATION_REPORT.md || { echo "❌ FAIL: Report missing"; exit 1; }

# Check all tests pass
cargo test simd_spec --lib 2>&1 | grep -q "test result: ok" || { echo "❌ FAIL: Tests not passing"; exit 1; }

# Check property tests executed
PROP_COUNT=$(cargo test simd_spec 2>&1 | grep -c "ok (10000 cases)" || echo 0)
[ "$PROP_COUNT" -ge 5 ] || { echo "❌ FAIL: Only $PROP_COUNT property tests (need 5)"; exit 1; }

# Check benchmark ran
cargo bench --bench bench_simd 2>&1 | grep -q "time:" || { echo "❌ FAIL: Benchmarks not run"; exit 1; }

# Check report has evidence pasted
grep -q "running .* tests" docs/benchmarks/W8D37_VALIDATION_REPORT.md || { echo "❌ FAIL: No test output in report"; exit 1; }
grep -q "time:.*ns" docs/benchmarks/W8D37_VALIDATION_REPORT.md || { echo "❌ FAIL: No benchmark output in report"; exit 1; }

# Check report has final recommendation
grep -E "(APPROVED|CONDITIONAL|REJECTED)" docs/benchmarks/W8D37_VALIDATION_REPORT.md | tail -1 || { echo "❌ FAIL: No final recommendation"; exit 1; }

echo "✅ ALL ACCEPTANCE CRITERIA PASSED"
```

---

## FAILURE PROTOCOL

### Detection

If validation cannot be completed:
- Symptom: Tests fail, benchmarks don't meet targets, or evidence cannot be generated
- Evidence: Failing test output, benchmark results below targets

### Categorization

1. **Type A: Tests failing due to bugs**
   - Action: Document failures, escalate to RUST_ENGINEER
   - Time limit: 30 minutes to triage
   - Evidence required: Full failing test output
   - Escalation: "Implementation has X failing tests: [list]"

2. **Type B: Benchmarks below targets**
   - Action: Verify measurement methodology, rerun with correct environment
   - Time limit: 1 hour
   - Tools:
     ```bash
     # Check CPU governor
     cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
     # Should be: performance

     # Check thermals
     sensors
     # Should be: <80°C
     ```
   - Escalation: If targets still not met after verification, escalate to RUST_ENGINEER with evidence

3. **Type C: Cross-platform failures**
   - Action: Document platform-specific issues
   - Time limit: 1 hour per platform
   - Options:
     - A. Fix platform-specific bug
     - B. Disable SIMD for that platform (fallback to portable)
     - C. Document as known limitation

### Escalation Triggers

Escalate to PLANNER if:
- [ ] >4 hours without validation complete
- [ ] >50% of tests failing
- [ ] All benchmark targets missed by >50%
- [ ] Fundamental spec violation discovered

### Alternative Paths

If validation fails critically:
- Option A: Revise targets in 03_SIMD_BENCHMARK_SPEC.md (requires justification + approval)
- Option B: Ship with portable implementation only (disable SIMD)
- Option C: Defer SIMD to Week 9 for more development time

Document decision in: `docs/planning/weeks/week8/W8D37_VALIDATION_BLOCKER.md`

---

## HANDOFF

```
TEST_ENGINEER + BENCHMARK_SCIENTIST → HOSTILE_REVIEWER

Deliverable: docs/benchmarks/W8D37_VALIDATION_REPORT.md
Status: [APPROVED | CONDITIONAL | REJECTED]
Evidence: Full test and benchmark output pasted in report

Next: 07_HOSTILE_REVIEW.md (if validation APPROVED)

CRITICAL CONSTRAINTS for HOSTILE_REVIEWER:
1. ALL claims in validation report must be backed by pasted evidence
2. NO estimates or approximations allowed
3. Verify test count, property test cases, cycle count, speedup, throughput all meet targets
```

---

**END OF VALIDATION PROMPT**
