# Week 8 Day 2 Prompt Suite — Remediation Plan

**Version:** 1.0.0
**Date:** 2025-12-12
**Status:** ACTIVE
**Source:** HOSTILE_META_REVIEW.md (Score: 6.83/10.0 — REJECTED)
**Target:** ≥7.0/10.0 for APPROVED status

---

## EXECUTIVE SUMMARY

The Week 8 Day 2 SIMD prompt suite received a REJECTED verdict (6.83/10.0) from hostile meta-review. This plan decomposes the remediation into 4 sequential tasks with binary acceptance criteria.

**Total Issues:** 34 (11 Critical, 14 Major, 9 Minor)
**Estimated Remediation Time:** 4-6 hours
**Blocking Item:** File renaming (requires manual file operations)

---

## REMEDIATION TASKS

### Task R1: Complete File Renaming + Reference Updates

**Priority:** P0 (BLOCKING — Nothing else can proceed)
**Estimated Time:** 30 minutes
**Agent:** PLANNER (requires manual file operations)

**Description:**
Complete the file renaming that PLANNER claimed was done but wasn't. This is NOT a code task — it's a file system operation on prompt markdown files.

**Files to Rename:**
```bash
cd docs/planning/weeks/week8/prompts/day_2/

# Rename files (if they exist with old names)
# Current state unclear - may need to verify what actually exists
mv 02_SIMD_HAMMING_IMPL.md 04_SIMD_HAMMING_IMPL.md (if needed)
mv 03_SIMD_QUANTIZE_IMPL.md 05_SIMD_QUANTIZE_IMPL.md (if needed)
mv 06_SIMD_VALIDATION.md 06_SIMD_VALIDATION.md (no change)
mv 07_HOSTILE_REVIEW.md 07_HOSTILE_REVIEW.md (may already be correct)
mv 08_NVIDIA_GRADE_HOSTILE_REVIEW.md 08_NVIDIA_GRADE_HOSTILE_REVIEW.md (verify)
```

**References to Update:**
1. `00_MASTER_DISPATCH.md` lines 108-117 (table)
2. `00_MASTER_DISPATCH.md` line 61, 68, 93 (descriptions)
3. `PLANNER_FIX_SUMMARY.md` line 220-223 (commands)
4. All internal cross-references within prompts

**Acceptance Criteria (BINARY):**
- [ ] `ls -1 | grep -E "^0[0-8]"` shows: 00, 01, 02, 03, 04, 05, 06, 07, 08
- [ ] No files with old numbering exist: `! ls 02_SIMD_HAMMING_IMPL.md 2>/dev/null`
- [ ] `grep -c "02_SIMD_HAMMING_IMPL" 00_MASTER_DISPATCH.md` returns 0
- [ ] `grep -c "04_SIMD_HAMMING_IMPL" 00_MASTER_DISPATCH.md` returns ≥1
- [ ] `git diff` shows all file moves + reference updates

**Deliverables:**
- Renamed files in correct sequence
- Updated `00_MASTER_DISPATCH.md`
- Updated `PLANNER_FIX_SUMMARY.md`

---

### Task R2: Fix Critical Issues (11 items)

**Priority:** P0 (REQUIRED for approval)
**Estimated Time:** 2-3 hours
**Agent:** PLANNER
**Dependencies:** Task R1 complete

**Critical Issues to Fix:**

#### C1: Subjective Acceptance Criteria (01_SIMD_ARCHITECTURE.md)

**File:** `01_SIMD_ARCHITECTURE.md` lines 363-373

**Current (SUBJECTIVE):**
```markdown
- [ ] Module structure chosen from Options A/B/C
- [ ] Dispatch strategy chosen from Options A/B/C
- [ ] API integration strategy chosen
```

**Required (BINARY):**
```markdown
- [ ] Module structure decision documented in Section 2 with exact option (A/B/C) + justification ≥3 sentences
  - Verify: `grep -E "Chosen: Option [ABC]" docs/architecture/SIMD_DESIGN.md`
- [ ] Dispatch strategy documented in Section 3 with exact option (A/B/C) + cycle overhead calculation
  - Verify: `grep -E "(Compile-Time Only|Runtime Detection|Cached Runtime)" docs/architecture/SIMD_DESIGN.md`
- [ ] API integration documented in Section 6 with exact option (A/B/C) + migration impact analysis
  - Verify: `grep -E "(Transparent Replacement|Explicit SIMD Method|Feature Flag)" docs/architecture/SIMD_DESIGN.md`
```

**Acceptance:**
- [ ] All 3 criteria specify exact verification command
- [ ] All 3 criteria specify expected grep output
- [ ] No uses of "chosen", "justified", "appropriate" without measurement

---

#### C2: Time Estimate Contradiction (00_MASTER_DISPATCH.md)

**File:** `00_MASTER_DISPATCH.md` lines 108-121

**Issue:** Line 119 claims "14 hours" but table sums to different values

**Fix Required:**
1. Recalculate table sum from realistic column
2. Add buffer (e.g., 2.5h) to base sum
3. Update line 119 with correct total
4. Add source attribution: "Source: Sum of realistic estimates + 20% buffer"

**Verification:**
```bash
# Extract realistic times from table, sum them
grep -A 10 "| File | Agent |" 00_MASTER_DISPATCH.md | \
  grep "realistic" | \
  awk '{print $9}' | \
  # Manual sum required
```

**Acceptance:**
- [ ] Line 119 total = sum of realistic column + documented buffer
- [ ] Source attribution present
- [ ] No contradictions between line 119 and table

---

#### C3-C5: Broken File References (00_MASTER_DISPATCH.md)

**File:** `00_MASTER_DISPATCH.md` lines 61, 68, 93, 108-117

**Fix:** Update all references to use new numbering (04, 05, 07, 08)

**Acceptance:**
- [ ] `grep -c "02_SIMD_HAMMING_IMPL" 00_MASTER_DISPATCH.md` = 0
- [ ] `grep -c "03_SIMD_QUANTIZE_IMPL" 00_MASTER_DISPATCH.md` = 0
- [ ] `grep -c "06_HOSTILE_REVIEW" 00_MASTER_DISPATCH.md` = 0 (if renumbered to 07)

---

#### C6: File Renaming Incomplete

**Covered by Task R1** — NO ACTION HERE

---

#### C7: Subjective Acceptance Criteria (02_SIMD_TEST_SPEC.md)

**File:** `02_SIMD_TEST_SPEC.md` lines 363-371

**Current:**
```markdown
- [ ] File compiles
- [ ] All tests currently FAIL
- [ ] Test count ≥25
```

**Issue:** "compiles" is vague, "FAIL" is subjective

**Fix:**
```markdown
- [ ] File compiles: `cargo test --no-run simd_spec 2>&1 | grep -v "error" | grep -v "failed"`
  - Expected: Exit code 0 OR compilation errors only about missing functions (not syntax errors)
- [ ] All tests currently FAIL: `cargo test simd_spec 2>&1 | grep "test result" | grep "0 passed"`
  - Expected: "test result: FAILED. 0 passed; 25 failed" (exact numbers may vary)
- [ ] Test count ≥25: `grep -c "#\[test\]" tests/simd_spec.rs`
  - Expected: Number ≥ 25
```

**Acceptance:**
- [ ] All criteria include exact bash command
- [ ] All criteria include expected output format

---

#### C8: Missing Anti-Hallucination CLAMP (04_SIMD_HAMMING_IMPL.md)

**File:** `04_SIMD_HAMMING_IMPL.md` (after renaming from 02)

**Add Section After Line ~200:**
```markdown
---

## ANTI-HALLUCINATION CLAMPS

### Forbidden Phrases

**NEVER claim performance without measurement:**
- ❌ "This should be fast"
- ❌ "Approximately 50 cycles"
- ❌ "About 5x faster"
- ❌ "Roughly 1 billion ops/sec"

**REQUIRED Evidence Format:**
- ✅ "Measured: 46 cycles (rdtsc, 10k iterations)"
- ✅ "Benchmark: 5.2x speedup (criterion output: 14.2ns vs 73.8ns)"
- ✅ "Measured: 1.12 billion ops/sec (criterion throughput)"

### Verification Protocol

Every performance claim MUST include:
1. **Measurement tool:** rdtsc, criterion, perf, cachegrind
2. **Exact numbers:** Not estimates or ranges
3. **Evidence artifact:** Paste benchmark output or link to report
4. **Methodology:** Iterations count, warmup, environment

**Example:**
```
## Performance Validation

**Cycle Count (rdtsc):**
- Measured: 46 cycles
- Target: <50 cycles
- Status: ✅ PASS
- Methodology: rdtsc with 10,000 iterations, 1,000 warmup
- Evidence:
  ```
  $ cargo test test_simd_cycle_target
  Measured cycles: 46
  test test_simd_cycle_target ... ok
  ```
```

### Rejection Criteria

Implementation REJECTED if:
- [ ] Any performance claim lacks measurement evidence
- [ ] Claims use "approximately", "about", "roughly", "should be"
- [ ] Benchmark output not pasted
- [ ] Cycle count not from rdtsc
```

**Acceptance:**
- [ ] CLAMP section exists in 04_SIMD_HAMMING_IMPL.md
- [ ] Forbidden phrases list ≥4 items
- [ ] Required evidence format specified
- [ ] Rejection criteria enumerated

---

#### C9: Missing Anti-Hallucination CLAMP (05_SIMD_QUANTIZE_IMPL.md)

**File:** `05_SIMD_QUANTIZE_IMPL.md` (after renaming from 03)

**Add Same CLAMP Section as C8** (adapt for quantization performance)

**Acceptance:** Same as C8

---

#### C10: Missing Anti-Hallucination CLAMP (06_SIMD_VALIDATION.md)

**File:** `06_SIMD_VALIDATION.md`

**Add CLAMP Section:**
```markdown
---

## ANTI-HALLUCINATION CLAMPS

### Forbidden Phrases

**NEVER claim validation without evidence:**
- ❌ "Tests probably pass"
- ❌ "Should be correct"
- ❌ "Appears to work"
- ❌ "Likely no bugs"

**REQUIRED Evidence Format:**
- ✅ "All 25 tests pass (cargo test output pasted)"
- ✅ "10,000 property test cases passed (proptest output)"
- ✅ "0 failures in 1M fuzz iterations (cargo fuzz output)"

### Verification Protocol

Every validation claim MUST include:
1. **Test output:** Full `cargo test` output pasted
2. **Property test stats:** Exact case count from proptest
3. **Fuzz results:** Iterations count, coverage %, crashes found
4. **Cross-platform:** Results from x86_64 AND ARM (if applicable)

**Example:**
```
## Validation Results

**Unit Tests:**
```
$ cargo test simd
running 25 tests
test simd_correctness::test_simd_matches_portable_zeros ... ok
[... 23 more ...]
test result: ok. 25 passed; 0 failed; 0 ignored
```

**Property Tests:**
```
$ cargo test prop_simd
test simd_properties::prop_simd_matches_portable ... ok (10000 cases)
test simd_properties::prop_simd_symmetric ... ok (10000 cases)
[...]
```
```

**Acceptance:**
- [ ] CLAMP section exists
- [ ] Forbidden phrases list present
- [ ] Evidence format specified with examples

---

#### C11: Subjective Acceptance Criteria (06_SIMD_VALIDATION.md)

**File:** `06_SIMD_VALIDATION.md` (line numbers TBD)

**Fix:** Add binary criteria with exact verification commands (similar to C7)

**Acceptance:**
- [ ] All acceptance criteria include bash verification command
- [ ] All acceptance criteria include expected output

---

### Task R2 Acceptance Criteria (OVERALL)

- [ ] All 11 critical issues addressed
- [ ] All fixes verified with provided bash commands
- [ ] `git diff` shows changes to affected files
- [ ] No remaining subjective criteria (grep for "chosen", "justified", "appropriate")
- [ ] All CLAMPs include forbidden phrases list + evidence format

**Deliverables:**
- Updated `01_SIMD_ARCHITECTURE.md`
- Updated `00_MASTER_DISPATCH.md`
- Updated `02_SIMD_TEST_SPEC.md`
- Updated `04_SIMD_HAMMING_IMPL.md` (with CLAMP)
- Updated `05_SIMD_QUANTIZE_IMPL.md` (with CLAMP)
- Updated `06_SIMD_VALIDATION.md` (with CLAMP + binary criteria)

---

### Task R3: Fix Major Issues (14 items)

**Priority:** P1 (REQUIRED for approval)
**Estimated Time:** 2-3 hours
**Agent:** PLANNER
**Dependencies:** Task R2 complete

**Major Issues to Fix:**

#### M1: Incomplete Test Matrix (02_SIMD_TEST_SPEC.md)

**File:** `02_SIMD_TEST_SPEC.md` line ~40

**Add Section:**
```markdown
---

## CROSS-PLATFORM TEST MATRIX

### Required Test Coverage

| Platform | Instruction Set | Test Status | CI Job |
|:---------|:----------------|:------------|:-------|
| x86_64 (Linux) | AVX2 | ✅ Required | `test-x86-avx2` |
| x86_64 (Linux) | Portable (no AVX2) | ✅ Required | `test-x86-noavx2` |
| ARM64 (Linux) | NEON | ✅ Required | `test-arm-neon` |
| WASM | WASM SIMD | ⚠️ Optional | `test-wasm-simd` |
| x86_64 (macOS) | AVX2 | ✅ Required | `test-macos-x86` |
| ARM64 (macOS) | NEON | ✅ Required | `test-macos-arm` |

### Test Execution Protocol

**For Each Platform:**
1. Run full test suite: `cargo test --target <target-triple>`
2. Run property tests: `cargo test prop_ --target <target-triple>`
3. Verify SIMD dispatch: Check CPU feature detection logs
4. Compare performance: All platforms must pass <X cycles target (adjusted per platform)

**Acceptance Criteria:**
- [ ] All "Required" platforms show green CI status
- [ ] Property tests pass on all platforms (10,000 cases each)
- [ ] SIMD vs portable correctness verified on all platforms
- [ ] Performance targets met on primary platforms (x86_64 AVX2, ARM64 NEON)

### Platform-Specific Notes

**x86_64 Portable (no AVX2):**
- Uses std::simd fallback or scalar implementation
- Performance target relaxed: <200 cycles (vs <50 for AVX2)

**WASM SIMD:**
- May not support rdtsc cycle measurement
- Use time-based benchmarks only
- Test in actual browser environment

**ARM64 NEON:**
- Different instruction latencies than AVX2
- Target: <60 cycles (vs <50 for AVX2)
- Verify on actual ARM hardware, not QEMU
```

**Acceptance:**
- [ ] Test matrix table present with ≥6 platforms
- [ ] Execution protocol documented
- [ ] Platform-specific notes included

---

#### M2-M5: Missing Statistical Validation (03_SIMD_BENCHMARK_SPEC.md)

**File:** `03_SIMD_BENCHMARK_SPEC.md` line ~180

**Add Section:**
```markdown
---

## STATISTICAL VALIDATION REQUIREMENTS

### Measurement Variability

All benchmark measurements MUST include:
1. **Mean:** Average value across iterations
2. **Median:** Middle value (robust to outliers)
3. **Standard Deviation:** Measure of variability
4. **Min/Max:** Range of observed values
5. **Confidence Interval:** 95% CI for mean

**Example Output:**
```
simd_hamming_96bytes_cycles
  Mean: 46.2 cycles
  Median: 46.0 cycles
  Std Dev: 2.1 cycles
  Min: 43 cycles
  Max: 52 cycles
  95% CI: [45.8, 46.6]
  Target: <50 cycles
  Status: ✅ PASS (mean within target)
```

### Outlier Detection

**Protocol:**
1. Run benchmark 1,000 times (not just criterion's auto-iterations)
2. Detect outliers using IQR method:
   - Q1 = 25th percentile
   - Q3 = 75th percentile
   - IQR = Q3 - Q1
   - Outliers: < Q1 - 1.5×IQR OR > Q3 + 1.5×IQR
3. Report outlier count and percentage
4. If >5% outliers, investigate (thermal throttling, context switches, etc.)

**Acceptance Criteria:**
- [ ] Outlier percentage <5%
- [ ] Mean and median differ by <10%
- [ ] Standard deviation <20% of mean

### Regression Detection

**Baseline Comparison:**
```bash
# Save baseline
cargo bench --bench bench_simd > baseline.txt

# After code changes
cargo bench --bench bench_simd > current.txt

# Compare with criterion's built-in comparison
cargo bench --bench bench_simd --baseline baseline
```

**Acceptance:**
- [ ] No regression >10% from baseline
- [ ] If regression detected, documented justification required
- [ ] Baseline results saved in `docs/benchmarks/W8D37_baseline.txt`

### Environmental Controls

**Required for valid benchmarks:**
- [ ] CPU governor set to `performance`: `cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor`
- [ ] No other heavy processes running: `top` shows <5% CPU usage by others
- [ ] Thermal throttling not active: CPU temp <80°C
- [ ] At least 10 warmup iterations before measurement
- [ ] Benchmark run 3 times, median result reported

**Documentation Required:**
```markdown
## Benchmark Environment

- **CPU:** Intel Core i7-9700K @ 3.6 GHz (Turbo: 4.9 GHz)
- **Governor:** performance
- **Temperature:** 62°C (idle), 74°C (under load)
- **RAM:** 32 GB DDR4-3200
- **OS:** Ubuntu 22.04.3 LTS
- **Kernel:** 6.5.0-15-generic
- **Rust:** 1.75.0
- **Criterion:** 0.5.1
```
```

**Acceptance:**
- [ ] Statistical validation section present in 03_SIMD_BENCHMARK_SPEC.md
- [ ] Outlier detection protocol documented
- [ ] Regression detection protocol documented
- [ ] Environmental controls checklist included
- [ ] Example benchmark environment template provided

---

#### M6-M8: Vague Acceptance Criteria (Multiple Files)

**Files to Fix:**
- `04_SIMD_HAMMING_IMPL.md` (acceptance criteria section)
- `05_SIMD_QUANTIZE_IMPL.md` (acceptance criteria section)
- `06_SIMD_VALIDATION.md` (acceptance criteria section)

**Pattern to Apply:**

**BAD (Vague):**
```markdown
- [ ] Implementation complete
- [ ] Tests pass
- [ ] Performance acceptable
```

**GOOD (Binary):**
```markdown
- [ ] Implementation complete: `test -f src/quantization/simd.rs && grep -c "pub fn hamming" src/quantization/simd.rs` ≥1
  - Expected: File exists with at least 1 public hamming function
- [ ] Tests pass: `cargo test simd 2>&1 | grep "test result: ok"`
  - Expected: "test result: ok. X passed; 0 failed"
- [ ] Performance acceptable: `cargo bench --bench bench_simd 2>&1 | grep "cycles"`
  - Expected: Mean <50 cycles OR documented justification if >50
```

**Acceptance:**
- [ ] All acceptance criteria in 04/05/06 follow binary pattern
- [ ] All criteria include verification command
- [ ] All criteria include expected output

---

#### M9-M11: Missing Dependency Verification (Multiple Files)

**Files:** `04_SIMD_HAMMING_IMPL.md`, `05_SIMD_QUANTIZE_IMPL.md`

**Current:**
```markdown
**Dependencies:** 01, 02, 03
```

**Fix:**
```markdown
---

## DEPENDENCY VERIFICATION

**Before proceeding, verify ALL dependencies complete:**

```bash
# Check architecture approved
test -f docs/architecture/SIMD_DESIGN.md || { echo "BLOCK: Architecture missing"; exit 1; }
grep -q "APPROVED" docs/reviews/*SIMD_DESIGN*.md || { echo "BLOCK: Architecture not approved"; exit 1; }

# Check test spec ready
test -f tests/simd_spec.rs || { echo "BLOCK: Test spec missing"; exit 1; }
grep -c "#\[test\]" tests/simd_spec.rs | awk '$1 >= 25 || { print "BLOCK: <25 tests"; exit 1; }'

# Check benchmark spec ready
test -f benches/bench_simd.rs || { echo "BLOCK: Benchmark spec missing"; exit 1; }
test -f docs/benchmarks/SIMD_TARGETS.md || { echo "BLOCK: Targets missing"; exit 1; }

echo "✅ All dependencies verified"
```

**Expected Output:**
```
✅ All dependencies verified
```

**If ANY check fails, STOP and escalate to PLANNER.**
```

**Acceptance:**
- [ ] Dependency verification section added to 04, 05
- [ ] All checks are bash commands (not prose)
- [ ] Failure cases print clear error messages

---

#### M12-M14: Other Major Issues

*[Detailed fixes for remaining major issues would follow similar pattern]*

### Task R3 Acceptance Criteria (OVERALL)

- [ ] All 14 major issues addressed
- [ ] Test matrix added to 02_SIMD_TEST_SPEC.md
- [ ] Statistical validation added to 03_SIMD_BENCHMARK_SPEC.md
- [ ] Vague criteria replaced with binary in 04, 05, 06
- [ ] Dependency verification added to 04, 05

**Deliverables:**
- Updated `02_SIMD_TEST_SPEC.md` (test matrix)
- Updated `03_SIMD_BENCHMARK_SPEC.md` (statistical validation)
- Updated `04_SIMD_HAMMING_IMPL.md` (binary criteria, deps)
- Updated `05_SIMD_QUANTIZE_IMPL.md` (binary criteria, deps)
- Updated `06_SIMD_VALIDATION.md` (binary criteria)

---

### Task R4: Create Revision Manifest + Optional Minor Fixes

**Priority:** P1 (REQUIRED for resubmission)
**Estimated Time:** 30 minutes
**Agent:** PLANNER
**Dependencies:** Task R2 + R3 complete

**Description:**
Document all fixes applied for hostile meta-review v2.

**Create:** `docs/planning/weeks/week8/prompts/day_2/REVISION_MANIFEST.md`

**Template:**
```markdown
# Week 8 Day 2 Prompt Suite — Revision Manifest

**Version:** 2.0.0
**Date:** 2025-12-12
**Previous Version:** 1.0.0 (REJECTED — 6.83/10.0)
**Status:** PENDING_REVIEW

---

## REVISION SUMMARY

| Metric | Before | After | Change |
|:-------|:-------|:------|:-------|
| Meta-Review Score | 6.83/10.0 | TBD | TBD |
| Critical Issues | 11 | 0 | -11 |
| Major Issues | 14 | 0 | -14 |
| Minor Issues | 9 | TBD | TBD |
| Auto-Reject Triggers | 3 | 0 | -3 |

---

## CRITICAL ISSUES FIXED (11)

### [C1] Subjective Acceptance Criteria (01_SIMD_ARCHITECTURE.md)
- **Status:** ✅ FIXED
- **Change:** Replaced "chosen from Options A/B/C" with specific grep verification commands
- **Files Modified:** `01_SIMD_ARCHITECTURE.md` lines 363-395
- **Verification:** `grep -E "(Option [ABC]|Compile-Time|Runtime Detection)" docs/architecture/SIMD_DESIGN.md`

### [C2] Time Estimate Contradiction (00_MASTER_DISPATCH.md)
- **Status:** ✅ FIXED
- **Change:** Recalculated total from realistic column, added source attribution
- **Files Modified:** `00_MASTER_DISPATCH.md` line 119
- **Verification:** Sum of realistic column matches reported total ± buffer

[... Continue for all 11 critical issues ...]

---

## MAJOR ISSUES FIXED (14)

[... List all major issue fixes ...]

---

## MINOR ISSUES (9)

[... Optionally address minor issues OR document as "DEFERRED" ...]

---

## FILE CHANGE LOG

| File | Lines Changed | Type of Change |
|:-----|:--------------|:---------------|
| `00_MASTER_DISPATCH.md` | 61, 68, 93, 108-121 | References updated, time fixed |
| `01_SIMD_ARCHITECTURE.md` | 363-395 | Binary criteria, verification commands |
| `02_SIMD_TEST_SPEC.md` | 40-80, 363-371 | Test matrix, binary criteria |
| `03_SIMD_BENCHMARK_SPEC.md` | 180-250 | Statistical validation |
| `04_SIMD_HAMMING_IMPL.md` | 200-260 | CLAMP section, binary criteria, deps |
| `05_SIMD_QUANTIZE_IMPL.md` | TBD | CLAMP section, binary criteria, deps |
| `06_SIMD_VALIDATION.md` | TBD | CLAMP section, binary criteria |

---

## VERIFICATION CHECKLIST

Run these commands to verify all fixes:

```bash
cd docs/planning/weeks/week8/prompts/day_2/

# Check file renaming complete
ls -1 | grep -E "^0[0-8]" | wc -l
# Expected: 9 (00, 01, 02, 03, 04, 05, 06, 07, 08)

# Check no old references
grep -r "02_SIMD_HAMMING_IMPL" . | grep -v REVISION_MANIFEST | wc -l
# Expected: 0

# Check all CLAMPs present
grep -c "ANTI-HALLUCINATION CLAMPS" 04_SIMD_HAMMING_IMPL.md
grep -c "ANTI-HALLUCINATION CLAMPS" 05_SIMD_QUANTIZE_IMPL.md
grep -c "ANTI-HALLUCINATION CLAMPS" 06_SIMD_VALIDATION.md
# Expected: 1 for each

# Check test matrix present
grep -c "CROSS-PLATFORM TEST MATRIX" 02_SIMD_TEST_SPEC.md
# Expected: 1

# Check statistical validation present
grep -c "STATISTICAL VALIDATION" 03_SIMD_BENCHMARK_SPEC.md
# Expected: 1

echo "✅ All fixes verified"
```

---

## READY FOR RESUBMISSION

- [ ] All critical issues resolved
- [ ] All major issues resolved
- [ ] Verification checklist passes
- [ ] Git commits document all changes
- [ ] Ready for hostile meta-review v2

**Next Command:** `/review-meta-suite day_2_v2`
```

**Acceptance Criteria:**
- [ ] `REVISION_MANIFEST.md` created
- [ ] All 11 critical issues documented as FIXED
- [ ] All 14 major issues documented as FIXED
- [ ] Verification checklist included
- [ ] File change log complete

---

## REMEDIATION SEQUENCE

```
┌────────────────────────────────────────────────────────────┐
│                  REMEDIATION PIPELINE                      │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Task R1: File Renaming (30 min)                          │
│  ├── Rename 02→04, 03→05, 06→07, 07→08                   │
│  ├── Update all references in 00_MASTER_DISPATCH.md       │
│  └── Verify with ls and grep commands                     │
│                                                            │
│  ▼                                                         │
│                                                            │
│  Task R2: Fix Critical Issues (2-3h)                      │
│  ├── Fix subjective criteria (01, 02, 06)                 │
│  ├── Add CLAMP sections (04, 05, 06)                      │
│  ├── Fix time contradictions (00)                         │
│  └── Update broken references (00)                        │
│                                                            │
│  ▼                                                         │
│                                                            │
│  Task R3: Fix Major Issues (2-3h)                         │
│  ├── Add test matrix (02)                                 │
│  ├── Add statistical validation (03)                      │
│  ├── Fix vague criteria (04, 05, 06)                      │
│  └── Add dependency verification (04, 05)                 │
│                                                            │
│  ▼                                                         │
│                                                            │
│  Task R4: Create Manifest (30 min)                        │
│  ├── Document all fixes                                   │
│  ├── Create verification checklist                        │
│  └── Prepare for resubmission                             │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

---

## APPROVAL CRITERIA

This remediation plan is APPROVED for execution if:
- [ ] All 4 tasks have binary acceptance criteria
- [ ] Dependencies between tasks are clear
- [ ] Estimated times are realistic (3x rule applied: 4-6h total)
- [ ] All 34 issues from HOSTILE_META_REVIEW.md are addressed
- [ ] Verification commands provided for all fixes

**Executor:** PLANNER (with manual file operations for Task R1)
**Timeline:** 4-6 hours total
**Outcome:** Week 8 Day 2 prompt suite ready for hostile meta-review v2 (target score ≥7.0/10.0)

---

**END OF REMEDIATION PLAN**
