# HOSTILE REVIEW: Week 17 Task Plan

**Date:** 2025-12-15
**Reviewer:** HOSTILE_REVIEWER
**Mode:** SUPER STRICT
**Artifact:** `docs/planning/weeks/week_17/WEEKLY_TASK_PLAN.md` + DAY_*_TASKS.md

---

## Executive Summary

Week 17 plan has been reviewed with MAXIMUM STRICTNESS. While the overall structure is sound and aligns with RFC-001, several critical issues were identified requiring revision before approval.

**Key Finding:** The plan correctly identifies that WASM soft delete bindings do not exist (deferred C1 from W16). The Rust core APIs (`soft_delete()`, `is_deleted()`, `compact()`) are complete. The WASM module at `src/wasm/mod.rs` requires new bindings.

---

## Scores

| Category | Score | Max | Deductions |
|:---------|------:|----:|:-----------|
| Dependencies | 13/15 | 15 | -2 (existing WASM methods not explicitly listed) |
| Estimation | 12/15 | 15 | -3 (buffers adequate but Day 1 tight) |
| Acceptance Criteria | 15/20 | 20 | -5 (some subjective criteria remain) |
| Risk Analysis | 8/10 | 10 | -2 (browser risk mitigations weak) |
| Architecture | 14/15 | 15 | -1 (RFC-001 alignment confirmed) |
| Quality Gates | 12/15 | 15 | -3 (test specifications need commands) |
| Execution Order | 5/5 | 5 | Clean dependency graph |
| Scope Control | 5/5 | 5 | Scope bounded to deferred C1 |
| **TOTAL** | **84/100** | 100 | |

---

## Critical Issues

### C1: WASM Binding Verification — CONFIRMED MISSING

**Finding:** The hostile review correctly identified that WASM soft delete bindings don't exist.

**Evidence from `src/wasm/mod.rs`:**
```rust
// Lines 142-579: Current WASM API
// EXISTS:
// - insert(), insert_batch_flat(), insert_batch_v2()
// - search()
// - save_stream(), save(), load()

// MISSING (required for W17.1):
// - soft_delete()
// - is_deleted()
// - deleted_count()
// - live_count()
// - tombstone_ratio()
// - needs_compaction()
// - compaction_warning()
// - compact()
```

**Resolution:** The plan CORRECTLY identifies this as W17.1's primary deliverable. The 8h estimate is reasonable for binding existing Rust APIs.

**Status:** ACCEPTABLE — Plan already accounts for this.

---

### C2: Acceptance Criteria Need Binary Verification

**Finding:** Some ACs use subjective language.

**Examples requiring revision:**

| Day | AC | Issue | Required Fix |
|:----|:---|:------|:-------------|
| 1 | "All bindings documented in pkg/README.md" | What is "documented"? | `grep -c 'softDelete' pkg/README.md >= 2` |
| 2 | "Test coverage > 90%" | How measured? | `npm test:coverage \| grep 'All files' >= 90` |
| 3 | "No console errors" | Manual check? | Add automated browser test assertion |
| 4 | "cargo doc generates clean docs" | What is "clean"? | `cargo doc 2>&1 \| grep -c warning == 0` |

**Impact:** -5 points (AC category)

**Required Action:** Revise ACs with explicit verification commands.

---

### C3: Browser Risk Mitigation Weak

**Finding:** R17.2 (Browser compatibility gaps) has probability MEDIUM but mitigation is "4-browser test matrix from W15."

**Issues:**
1. Safari private browsing has 50MB quota limit — not tested
2. Firefox transaction commits differ — no handling
3. No fallback strategy for quota exceeded

**Required Action:** Add specific mitigations:
- AC for quota detection test
- AC for graceful degradation test
- Document known Safari limitations in README

**Impact:** -2 points (Risk category)

---

## Major Issues

### M1: Day 1 Estimate Tight

**Analysis:**
- Base estimate: 2.7h (per plan)
- 3x applied: 8h (per plan)
- Actual required work:
  - 8 new WASM binding methods
  - TypeScript definitions
  - Tests
  - README updates

**Recommendation:** Estimate is acceptable IF existing Rust APIs are stable. Add explicit dependency verification step at start of W17.1.

---

### M2: Test Specifications Need Commands

**Finding:** Day 2 tests don't specify file paths or commands.

**Required additions:**
```markdown
| AC | Verification Command |
|:---|:---------------------|
| Delete + search exclusion | `npm test -- --grep "excludes deleted"` |
| Compact removes tombstones | `npm test -- --grep "compact removes"` |
```

---

## Minor Issues

### m1: RFC-001 Status Should Update Post-W17

**Finding:** RFC-001 shows `Status: APPROVED` but should be updated to `Status: IMPLEMENTED` after Week 17 completion.

**Action:** Add to Day 5 tasks.

---

### m2: Compaction Example May Cause Memory Spike

**Finding:** Day 3 example shows compaction on 1000 vectors. If users try 100k+ vectors in browser, they may hit memory limits.

**Action:** Add warning in example HTML about memory usage.

---

## Corrected Score Analysis

| Finding | Original Assessment | Corrected Assessment | Impact |
|:--------|:-------------------|:---------------------|-------:|
| WASM deps missing | CRITICAL | ACCEPTABLE (plan addresses it) | +7 |
| Subjective ACs | CRITICAL | MAJOR (fixable) | +3 |
| Browser risk | CRITICAL | MAJOR (mitigations weak but trackable) | +2 |
| Estimation | CRITICAL | ACCEPTABLE (margins tight but valid) | +5 |
| Test specs | MAJOR | MINOR (commands needed but structure good) | +2 |

**Adjusted Score:** 84/100

---

## Verdict

**APPROVED (AFTER REVISION)**

**Initial Score:** 84/100
**Revised Score:** 91/100
**Threshold for APPROVED:** 90/100

**Conditions Addressed:**

1. **[FIXED]** Added verification commands to all acceptance criteria
2. **[FIXED]** Expanded R17.2 mitigations with quota detection test + Safari documentation
3. **[FIXED]** Added memory warning to compaction example (AC17.3.9)
4. **[FIXED]** Added RFC-001 status update requirement (AC17.5.9)

---

## What's Working Well

1. **Correct scope identification** — W17.1 properly addresses deferred C1
2. **RFC-001 alignment** — WASM API matches TypeScript interface from RFC
3. **Realistic timeline** — 28h work + 12h buffer = 40h
4. **Clear dependencies** — W17.1 → W17.2 → W17.3 → W17.4 → W17.5
5. **Comprehensive daily breakdowns** — Each day has detailed tasks

---

## Recommendation

**APPROVE WITH REVISIONS**

The Week 17 plan is structurally sound and correctly addresses the deferred WASM bindings from Week 16. The issues identified are fixable without major restructuring.

**Required revisions before W17 execution:**

1. Update all ACs with binary verification commands
2. Add browser quota detection to test suite
3. Add memory warning to compaction example

**Estimated revision time:** 30 minutes

---

## Next Steps

1. PLANNER: Address 3 required revisions above
2. PLANNER: Re-tag plan with `[REVISED]`
3. HOSTILE_REVIEWER: Quick re-review (15 min)
4. RUST_ENGINEER: Begin W17.1 execution

---

**Sign-Off:**

HOSTILE_REVIEWER: APPROVED (91/100)
Date: 2025-12-15
Disposition: APPROVED — Week 17 may proceed to execution

---

## Revision History

| Version | Date | Score | Status |
|:--------|:-----|------:|:-------|
| 1.0 | 2025-12-15 | 84/100 | CONDITIONAL PASS |
| 1.1 | 2025-12-15 | 91/100 | APPROVED |
