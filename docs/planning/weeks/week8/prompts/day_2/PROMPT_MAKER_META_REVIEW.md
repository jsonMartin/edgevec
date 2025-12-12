# PROMPT_MAKER: Meta-Review of Day 2 Prompt Suite

**Agent:** PROMPT_MAKER → HOSTILE_REVIEWER
**Command:** `/prompt-review day2_suite`
**Priority:** P0 (CRITICAL — Prompt Quality Gate)
**Date:** 2025-12-12
**Purpose:** Audit Day 2 prompts BEFORE execution to prevent workflow failures
**Output:** `docs/reviews/2025-12-12_DAY2_PROMPTS_META_REVIEW.md`

---

## MISSION

You are conducting a **META-REVIEW** of the Day 2 prompt suite itself, NOT the code it will produce. Your job is to find flaws, gaps, contradictions, and ambiguities in the prompts before any agent executes them.

**Why This Matters:**
- A flawed prompt wastes hours of agent time
- Ambiguous acceptance criteria cause rework loops
- Missing anti-hallucination clamps enable false claims
- Unrealistic time estimates cause schedule failures

**Standard:** NASA Pre-Flight Checklist Rigor

**Authority:** REJECT prompts that would lead to failure

---

## HOSTILE REVIEWER MANDATE (Meta-Review Mode)

You are reviewing **PROMPTS** as if they were mission-critical specifications.

**Your Questions:**
- Would this prompt produce the intended artifact?
- Are acceptance criteria measurable and unambiguous?
- Can an agent hallucinate through the safeguards?
- Are time estimates realistic given the constraints?
- Do handoffs specify exactly what the next agent receives?
- Are there conflicting requirements?

**If you answer NO to any question → FLAG the prompt as flawed.**

---

## ARTIFACTS TO REVIEW

```
Prompt Suite Location:
docs/planning/weeks/week8/prompts/day_2/

Files to Audit:
├── 00_MASTER_DISPATCH.md             (Orchestration)
├── 01_SIMD_ARCHITECTURE.md           (META_ARCHITECT)
├── 02_SIMD_HAMMING_IMPL.md           (RUST_ENGINEER - primary)
├── 03_SIMD_QUANTIZE_IMPL.md          (RUST_ENGINEER - optional)
├── 04_SIMD_BENCHMARKS.md             (BENCHMARK_SCIENTIST)
├── 05_SIMD_TESTS.md                  (TEST_ENGINEER)
├── 06_HOSTILE_REVIEW.md              (HOSTILE_REVIEWER - standard)
├── 07_NVIDIA_GRADE_HOSTILE_REVIEW.md (HOSTILE_REVIEWER - enhanced)
├── META_CORRECTION_TEST_FIRST.md     (PROMPT_MAKER - methodology)
└── PLANNER_DAY2_OPTIMIZATION.md      (PLANNER - execution plan)

Total Prompts: 10
```

---

## META-REVIEW DIMENSIONS

### DIMENSION 1: COMPLETENESS (Weight: 20%)

**Audit Protocol:**

For EACH prompt file, verify it contains ALL required sections:

```markdown
Required Sections (ALL prompts):
- [ ] **Target Agent:** Clearly specified
- [ ] **Command:** Slash command syntax
- [ ] **Priority:** P0/P1/P2 classification
- [ ] **Estimated Time:** Realistic time allocation
- [ ] **Dependencies:** Specific prerequisite artifacts
- [ ] **Output:** Exact file paths and artifact names
- [ ] **Mission:** Clear objective statement
- [ ] **Deliverables:** Enumerated list of outputs
- [ ] **Acceptance Criteria:** Binary pass/fail checks
- [ ] **Handoff:** Next agent + status check
```

**Scoring:**
| Condition | Score |
|:----------|:------|
| All 10 prompts have all required sections | 10/10 |
| 1-2 prompts missing sections | 7/10 |
| 3-5 prompts missing sections | 4/10 |
| >5 prompts missing sections | 0/10 → **REJECT SUITE** |

---

### DIMENSION 2: CLARITY (Weight: 15%)

**Audit Protocol:**

Check for ambiguous language that could cause misinterpretation:

```bash
# Search for vague terms
grep -rn "should\|might\|probably\|perhaps\|maybe\|ideally" *.md

# EACH instance is a potential ambiguity

# Check for undefined terms
grep -rn "etc\|and so on\|similar" *.md

# EACH is incomplete specification
```

**Ambiguity Red Flags:**
| Phrase | Problem | Fix |
|:-------|:--------|:----|
| "Should implement..." | Optional? Required? | "MUST implement..." |
| "~50 cycles" | Approximate | "<50 cycles (hard limit: 75)" |
| "Good performance" | Undefined | ">5x speedup vs baseline" |
| "Comprehensive tests" | How many? | "≥25 tests, ≥10k property cases" |
| "Clean code" | Subjective | "0 clippy warnings" |

**Scoring:**
| Condition | Score |
|:----------|:------|
| 0 ambiguous phrases | 10/10 |
| 1-3 ambiguous phrases | 7/10 |
| 4-10 ambiguous phrases | 4/10 |
| >10 ambiguous phrases | 0/10 → **REJECT SUITE** |

---

### DIMENSION 3: ACCEPTANCE CRITERIA RIGOR (Weight: 20%)

**Audit Protocol:**

For EACH prompt, verify acceptance criteria are:
1. **Binary** — Pass/fail, no subjective judgment
2. **Measurable** — Can be verified programmatically
3. **Complete** — Cover all deliverables

**Example Analysis:**

```markdown
BAD (Subjective):
- [ ] Code is well-written
- [ ] Tests are comprehensive
- [ ] Performance is good

GOOD (Binary & Measurable):
- [ ] cargo clippy -- -D warnings returns 0 exit code
- [ ] ≥25 unit tests + ≥10,000 property test cases
- [ ] AVX2 Hamming: <50 cycles (rdtsc measurement)
```

**Checklist for EACH Prompt:**
- [ ] All criteria are binary (yes/no)
- [ ] All criteria specify HOW to verify (command, tool, metric)
- [ ] All criteria have quantified thresholds (numbers, not adjectives)
- [ ] Criteria cover ALL deliverables listed

**Scoring:**
| Condition | Score |
|:----------|:------|
| All prompts have rigorous criteria | 10/10 |
| 1-2 prompts have weak criteria | 6/10 |
| 3-5 prompts have weak criteria | 3/10 |
| >5 prompts have weak criteria | 0/10 → **REJECT SUITE** |

---

### DIMENSION 4: TEST-FIRST ENFORCEMENT (Weight: 15%)

**Audit Protocol:**

Verify the prompt execution order ENFORCES test-first:

```
REQUIRED ORDER:
1. Architecture (design)
2. Test Specification (write failing tests)
3. Benchmark Specification (define targets)
4. Implementation (make tests pass)
5. Validation (verify targets met)
6. Review (approve or reject)

FORBIDDEN PATTERN:
1. Architecture
2. Implementation ← CODE BEFORE TESTS (VIOLATION)
3. Tests ← TESTS AFTER CODE (VIOLATION)
```

**Verification:**
- [ ] Test specification prompt exists
- [ ] Test prompt comes BEFORE implementation prompts (by number/name)
- [ ] Test prompt explicitly states "tests will FAIL initially"
- [ ] Implementation prompt states "make tests from [X] pass"
- [ ] Implementation prompt forbids modifying tests

**Evidence Required:**
```markdown
From Test Prompt (05_SIMD_TESTS.md):
"Write test specifications BEFORE any SIMD code is written.
The tests will initially FAIL (no implementation exists). This is correct."

From Implementation Prompt (02_SIMD_HAMMING_IMPL.md):
"Your mission: Make ALL tests from 05_SIMD_TESTS.md pass.
Do NOT modify existing tests."
```

**Scoring:**
| Condition | Score |
|:----------|:------|
| Test-first order enforced with explicit guards | 10/10 |
| Test-first order enforced but guards missing | 6/10 |
| Order allows tests after code | 0/10 → **AUTO-REJECT** |

---

### DIMENSION 5: ANTI-HALLUCINATION SAFEGUARDS (Weight: 15%)

**Audit Protocol:**

Check that prompts prevent agents from making unverified claims:

**Required Safeguards:**
1. **Evidence Requirement:** "All claims must have benchmark/test proof"
2. **Verification Commands:** Specific bash commands to verify claims
3. **Red Flag Detection:** List of hallucination patterns to avoid
4. **Measurement Protocol:** Exact methodology for measurements

**Example from Benchmark Prompt:**

```markdown
GOOD (Anti-Hallucination):
"Measure CPU cycles using rdtsc:
```rust
let start = unsafe { _rdtsc() };
for _ in 0..10_000 { black_box(f()); }
let end = unsafe { _rdtsc() };
let cycles = (end - start) / 10_000;
```
Document EXACT cycle count, not approximation."

BAD (Enables Hallucination):
"The implementation should be fast."
```

**Check Each Implementation Prompt:**
- [ ] Forbids "TODO" in production code
- [ ] Forbids "approximately X cycles" without measurement
- [ ] Forbids "should work" without tests
- [ ] Requires every unsafe to have safety proof
- [ ] Requires benchmark evidence for performance claims

**Scoring:**
| Condition | Score |
|:----------|:------|
| All prompts have 4+ anti-hallucination safeguards | 10/10 |
| All prompts have 2-3 safeguards | 6/10 |
| Any prompt missing safeguards | 0/10 → **REJECT SUITE** |

---

### DIMENSION 6: TIME REALISM (Weight: 10%)

**Audit Protocol:**

Verify time estimates are realistic given constraints:

**Time Estimation Formula:**
```
Base Estimate = Optimistic time (best case)
3x Rule = Base × 3 (EdgeVec standard)
Actual Allocation = 3x with buffer
```

**Example Audit:**

```markdown
Prompt: 02_SIMD_HAMMING_IMPL.md
Stated Time: 3 hours
Tasks:
  - Create module structure (30 min)
  - Implement AVX2 (1.5 hours)
  - Implement portable fallback (30 min)
  - Implement dispatch (30 min)
  - Unit tests (30 min)
  - Safety documentation (30 min)
  TOTAL: 4 hours BASE

3x Rule: 4 × 3 = 12 hours
Allocated: 3 hours

VERDICT: UNREALISTIC ❌
```

**Check Each Prompt:**
- [ ] Time estimate ≥ sum of subtask times
- [ ] 3x rule applied (or documented exception)
- [ ] Buffer allocated for unexpected issues
- [ ] Dependencies time is NOT double-counted

**Scoring:**
| Condition | Score |
|:----------|:------|
| All estimates realistic with 3x rule | 10/10 |
| Estimates tight but achievable | 7/10 |
| 1-2 estimates clearly unrealistic | 4/10 |
| >2 estimates unrealistic | 0/10 → **REJECT SUITE** |

---

### DIMENSION 7: HANDOFF CLARITY (Weight: 10%)

**Audit Protocol:**

Verify each prompt specifies EXACTLY what the next agent receives:

**Required Handoff Elements:**
1. **Next Agent:** Name of agent to execute next
2. **Deliverables:** Exact files/artifacts to hand off
3. **Status Check:** How to verify readiness for next step
4. **Blocker Conditions:** What prevents handoff

**Example Audit:**

```markdown
GOOD Handoff:
"""
## HANDOFF

RUST_ENGINEER → TEST_ENGINEER

Deliverables:
- src/quantization/simd.rs (implementation complete)
- All tests from tests/simd_spec.rs passing

Status Check:
```bash
cargo test simd 2>&1 | grep "test result: ok"
```

Blocker: If ANY test fails, do not proceed to validation.
Revert to RUST_ENGINEER for fixes.
"""

BAD Handoff:
"Pass work to next agent when done."
```

**Check Each Prompt:**
- [ ] Explicitly names next agent
- [ ] Lists exact file paths to hand off
- [ ] Provides verification command
- [ ] States blocker conditions

**Scoring:**
| Condition | Score |
|:----------|:------|
| All prompts have complete handoffs | 10/10 |
| 1-2 prompts have incomplete handoffs | 6/10 |
| >2 prompts have incomplete handoffs | 0/10 → **REJECT SUITE** |

---

### DIMENSION 8: DEPENDENCY SPECIFICATION (Weight: 5%)

**Audit Protocol:**

Verify dependencies are specific and verifiable:

```markdown
BAD (Vague):
Dependencies: "Architecture design"

GOOD (Specific):
Dependencies:
  - docs/architecture/SIMD_DESIGN.md exists
  - SIMD_DESIGN.md approved by HOSTILE_REVIEWER
  - .claude/GATE_A_COMPLETE.md exists
```

**Check Each Prompt:**
- [ ] Dependencies list specific file paths
- [ ] Dependencies include approval status
- [ ] Dependencies have verification commands
- [ ] Circular dependencies are absent

**Scoring:**
| Condition | Score |
|:----------|:------|
| All dependencies specific and verifiable | 10/10 |
| 1-3 vague dependencies | 7/10 |
| >3 vague dependencies | 0/10 |

---

### DIMENSION 9: INTERNAL CONSISTENCY (Weight: 5%)

**Audit Protocol:**

Check for contradictions across prompts:

**Common Inconsistencies:**
1. **Conflicting Targets:** Prompt A says <50 cycles, Prompt B says <75
2. **File Path Mismatches:** Different prompts reference different paths
3. **Acceptance Criteria Conflicts:** One prompt requires X, another forbids X
4. **Time Allocation Conflicts:** Sum of prompts > total day allocation

**Verification Matrix:**

| Metric | Prompt A | Prompt B | Consistent? |
|:-------|:---------|:---------|:------------|
| Cycle target | <50 | <50 | ✅ |
| Speedup target | >5x | >5x | ✅ |
| Test count | ≥25 | ≥20 | ❌ CONFLICT |

**Scoring:**
| Condition | Score |
|:----------|:------|
| 0 conflicts detected | 10/10 |
| 1-2 minor conflicts | 6/10 |
| >2 conflicts or 1 major conflict | 0/10 → **REJECT SUITE** |

---

### DIMENSION 10: FAILURE PROTOCOLS (Weight: 5%)

**Audit Protocol:**

Verify each prompt specifies what to do if it fails:

**Required Failure Protocol Elements:**
1. **Failure Detection:** How to know it failed
2. **Root Cause Categories:** Common reasons for failure
3. **Fix Strategy:** What to do for each failure type
4. **Escalation Path:** When to escalate to PLANNER
5. **Time Limit:** How long to attempt fixes before escalation

**Example:**

```markdown
## FAILURE PROTOCOL

If cargo test shows failures:

1. Categorize failure:
   - Type A: Logic error → Debug and fix (1 hour max)
   - Type B: Unsafe UB → Rewrite unsafe blocks (2 hours max)
   - Type C: Fundamental approach wrong → Escalate to PLANNER

2. Time limit: 2 hours total for fixes

3. Escalation trigger:
   - >2 hours without resolution
   - >3 failed fix attempts
   - Fundamental architectural issue detected

4. Escalation path: PLANNER reviews, may extend deadline or defer SIMD
```

**Check Each Prompt:**
- [ ] Failure detection method specified
- [ ] Fix strategies enumerated
- [ ] Time limits defined
- [ ] Escalation path clear

**Scoring:**
| Condition | Score |
|:----------|:------|
| All prompts have complete failure protocols | 10/10 |
| 1-3 prompts missing protocols | 5/10 |
| >3 prompts missing protocols | 0/10 |

---

## META-REVIEW SCORING

| Dimension | Weight | Score | Weighted |
|:----------|-------:|------:|---------:|
| 1. Completeness | 20% | X/10 | X.XX |
| 2. Clarity | 15% | X/10 | X.XX |
| 3. Acceptance Criteria | 20% | X/10 | X.XX |
| 4. Test-First Enforcement | 15% | X/10 | X.XX |
| 5. Anti-Hallucination | 15% | X/10 | X.XX |
| 6. Time Realism | 10% | X/10 | X.XX |
| 7. Handoff Clarity | 10% | X/10 | X.XX |
| 8. Dependency Specification | 5% | X/10 | X.XX |
| 9. Internal Consistency | 5% | X/10 | X.XX |
| 10. Failure Protocols | 5% | X/10 | X.XX |
| **TOTAL** | **100%** | — | **X.XX/10** |

**Approval Thresholds:**
- **≥9.0:** APPROVED — Prompt suite ready for execution
- **8.0-8.9:** MINOR REVISIONS — Fix issues within 1 hour
- **7.0-7.9:** MAJOR REVISIONS — Significant rework required
- **<7.0:** REJECTED — Fundamental flaws, redesign needed

**Auto-Reject Conditions (Any = Suite FAILS):**
- Test-first ordering violation
- >5 ambiguous acceptance criteria
- Any prompt missing anti-hallucination safeguards
- >2 unrealistic time estimates
- >2 internal contradictions
- Any prompt missing failure protocol

---

## OUTPUT FORMAT

```markdown
# DAY 2 PROMPT SUITE META-REVIEW

**Status:** <APPROVED / MINOR REVISIONS / MAJOR REVISIONS / REJECTED>
**Overall Score:** X.XX/10.0
**Review Date:** 2025-12-12
**Reviewer:** HOSTILE_REVIEWER (Meta-Review Mode)
**Prompt Count:** 10 prompts audited

---

## EXECUTIVE SUMMARY

<3-4 sentences: Overall quality, critical flaws found, recommendation>

**Execution Readiness:** <READY / NOT READY>

---

## DIMENSION SCORES

| # | Dimension | Score | Weight | Weighted | Status |
|:--|:----------|------:|-------:|---------:|:-------|
| 1 | Completeness | X/10 | 20% | X.XX | ✅/❌ |
| 2 | Clarity | X/10 | 15% | X.XX | ✅/❌ |
| 3 | Acceptance Criteria | X/10 | 20% | X.XX | ✅/❌ |
| 4 | Test-First Enforcement | X/10 | 15% | X.XX | ✅/❌ |
| 5 | Anti-Hallucination | X/10 | 15% | X.XX | ✅/❌ |
| 6 | Time Realism | X/10 | 10% | X.XX | ✅/❌ |
| 7 | Handoff Clarity | X/10 | 10% | X.XX | ✅/❌ |
| 8 | Dependency Specification | X/10 | 5% | X.XX | ✅/❌ |
| 9 | Internal Consistency | X/10 | 5% | X.XX | ✅/❌ |
| 10 | Failure Protocols | X/10 | 5% | X.XX | ✅/❌ |

**Weighted Total:** X.XX/10.0
**Approval Threshold:** ≥9.0

---

## DETAILED FINDINGS

### DIMENSION 1: COMPLETENESS

**Prompts Audited:** 10
**Missing Sections Found:** X

**Issues:**
- [Prompt Name]: Missing [Section Name]
- [Prompt Name]: Missing [Section Name]

**Verdict:** <PASS/FAIL>

---

### DIMENSION 2: CLARITY

**Ambiguous Phrases Detected:** X

**Examples:**
```
File: 02_SIMD_HAMMING_IMPL.md:45
Found: "Should implement AVX2"
Issue: "Should" is ambiguous (optional vs required?)
Fix: "MUST implement AVX2"
```

**Verdict:** <PASS/FAIL>

---

[Continue for all 10 dimensions...]

---

## CRITICAL FLAWS (Auto-Reject)

### FLAW-001: [Title]
- **Severity:** CRITICAL
- **Location:** [Prompt file]:[line]
- **Issue:** [Description]
- **Impact:** [How this would cause failure]
- **Required Fix:** [Specific action]

---

## MAJOR ISSUES

### MAJOR-001: [Title]
- **Severity:** MAJOR
- **Location:** [Prompt file]:[line]
- **Issue:** [Description]
- **Required Fix:** [Action]

---

## MINOR ISSUES

1. [Issue description with location]
2. [Issue description with location]

---

## POSITIVE FINDINGS

1. [Specific praise for well-designed prompt elements]
2. [Specific praise]
3. [Specific praise]

---

## PROMPT-BY-PROMPT AUDIT

### 00_MASTER_DISPATCH.md
- **Completeness:** X/10
- **Clarity:** X/10
- **Issues:** [List]
- **Verdict:** <PASS/FAIL>

### 01_SIMD_ARCHITECTURE.md
- **Completeness:** X/10
- **Clarity:** X/10
- **Issues:** [List]
- **Verdict:** <PASS/FAIL>

[Continue for all 10 prompts...]

---

## EXECUTION READINESS CHECKLIST

- [ ] All prompts have complete sections
- [ ] All acceptance criteria are binary and measurable
- [ ] Test-first ordering is enforced
- [ ] Anti-hallucination safeguards are present
- [ ] Time estimates are realistic (3x rule applied)
- [ ] All handoffs are explicit and verifiable
- [ ] No internal contradictions detected
- [ ] Failure protocols are defined
- [ ] Dependencies are specific
- [ ] Total time ≤ Day 2 allocation (12 hours)

**Readiness Score:** X/10 checks passed

---

## FINAL VERDICT

**Decision:** <APPROVED / MINOR REVISIONS / MAJOR REVISIONS / REJECTED>

**Rationale:**
<Detailed explanation of decision>

**If MINOR REVISIONS — Fix List (1-hour deadline):**
1. [Fix 1]
2. [Fix 2]
3. [Fix 3]

**If MAJOR REVISIONS — Rework Required:**
1. [Major issue 1]
2. [Major issue 2]

**If APPROVED — Execution Clearance:**
```
Day 2 Prompt Suite: CERTIFIED FOR EXECUTION

Prompts are internally consistent, complete, and rigorous.
Test-first methodology enforced.
Anti-hallucination safeguards in place.

CLEARED to execute starting with:
  Phase A: A1_ARCHITECTURE.md (META_ARCHITECT)
```

---

## RECOMMENDATIONS

1. [Recommendation for improving prompt quality]
2. [Recommendation for execution]
3. [Recommendation for future prompt development]

---

**Reviewer Certification:**

I, HOSTILE_REVIEWER, certify that this meta-review was conducted with
maximum rigor. All 10 prompts were audited against 10 dimensions.
All findings are documented with specific locations.

**Reviewer:** HOSTILE_REVIEWER (Meta-Review Mode)
**Authority:** Prompt Quality Gate
**Clearance Granted:** <YES/NO>
```

---

## POST-REVIEW ACTIONS

### IF APPROVED (≥9.0)
```
Prompt Suite: APPROVED FOR EXECUTION

Clearance to begin Day 2:
  1. Execute A1_ARCHITECTURE.md (META_ARCHITECT)
  2. Follow PLANNER_DAY2_OPTIMIZATION.md execution order
  3. Respect all gates and handoffs

Locked Prompts (do not modify during execution):
  - All 10 prompts in day_2/ folder
```

### IF MINOR REVISIONS (8.0-8.9)
```
PROMPT_MAKER must revise within 1 hour:
  1. [List specific fixes]

Resubmit: /prompt-review day2_suite_v2
```

### IF MAJOR REVISIONS (7.0-7.9)
```
Significant rework needed:
  1. [Major issue to address]
  2. [Major issue to address]

Estimated rework time: X hours
Resubmit: /prompt-review day2_suite_v2
```

### IF REJECTED (<7.0)
```
HALT — Fundamental Flaws Detected

Critical Issues:
  1. [Critical flaw]
  2. [Critical flaw]

Recommendation: Redesign prompt suite from scratch
Escalate to: PLANNER for schedule impact assessment
```

---

## VERIFICATION COMMANDS

```bash
# Check all prompts have required sections
for file in docs/planning/weeks/week8/prompts/day_2/*.md; do
  echo "=== $file ==="
  grep -c "Target Agent:" "$file"
  grep -c "Deliverables:" "$file"
  grep -c "Acceptance Criteria:" "$file"
  grep -c "Handoff:" "$file"
done

# Check for ambiguous language
grep -rn "should\|might\|probably\|perhaps\|maybe" \
  docs/planning/weeks/week8/prompts/day_2/ | wc -l

# Check test-first ordering (by filename)
ls -1 docs/planning/weeks/week8/prompts/day_2/ | \
  grep -E "(TEST|IMPL)" | head -5

# Check time estimates sum
grep -rh "Estimated Time:" docs/planning/weeks/week8/prompts/day_2/ | \
  grep -oP '\d+' | awk '{sum+=$1} END {print "Total hours:", sum}'

# Check for contradictions (performance targets)
grep -rn "cycles\|<50\|<75" docs/planning/weeks/week8/prompts/day_2/
```

---

**END OF PROMPT_MAKER META-REVIEW PROMPT**
