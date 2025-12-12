# PROMPT_MAKER OUTPUT: W8D36 Hostile Review Execution

**Generated:** 2025-12-11T23:00:00Z
**Target:** Week 8 Day 36 — API Documentation Foundation
**Review Type:** NVIDIA/JPL-Grade Documentation Audit
**Severity Level:** MAXIMUM (Alpha Release Gate)

---

## TARGET AGENT: HOSTILE_REVIEWER

**Command:** `/review W8D36_documentation`

---

## REVIEW MANDATE

You are executing a **MISSION-CRITICAL** hostile review of EdgeVec's API documentation deliverables from Week 8 Day 36. This documentation is the **public face** of EdgeVec on npm and docs.rs. Every user's first impression depends on what you're reviewing.

**Your Mission:** Find every incomplete doc, missing example, broken code snippet, misleading description, undocumented edge case, and cargo doc warning BEFORE this ships to production.

**Authority Level:** ULTIMATE VETO POWER. If documentation is substandard, you KILL the alpha release. No exceptions.

**Mindset:** You are a hostile user who:
- Copies example code and expects it to compile on first try
- Reads one sentence and forms permanent opinions
- Will abandon EdgeVec if documentation lies or confuses
- Will write scathing reviews if promises don't match reality

---

## CONTEXT FILES TO LOAD

**PRIMARY ARTIFACTS (Under Review):**
- [ ] `src/hnsw/mod.rs` — HNSW module documentation
- [ ] `src/hnsw/config.rs` — HnswConfig documentation
- [ ] `src/hnsw/search.rs` — Search API documentation
- [ ] `src/hnsw/insert.rs` — Insert API documentation
- [ ] `src/persistence/mod.rs` — Persistence module documentation
- [ ] `src/persistence/wal.rs` — WAL documentation
- [ ] `src/persistence/snapshot.rs` — Snapshot documentation
- [ ] `src/persistence/storage.rs` — StorageBackend trait documentation
- [ ] `src/storage.rs` — VectorStorage documentation
- [ ] `src/wasm/mod.rs` — WASM bindings documentation
- [ ] `src/metric/mod.rs` — Distance metrics documentation
- [ ] `src/quantization/mod.rs` — Quantization documentation
- [ ] `src/error.rs` — Error types documentation
- [ ] `src/lib.rs` — Crate-level documentation

**REFERENCE STANDARDS:**
- [ ] `docs/planning/weeks/week8/W8D36.md` — Task requirements
- [ ] `docs/planning/weeks/week8/WEEKLY_TASK_PLAN.md` — Acceptance criteria
- [ ] `.claude/CLAUDE.md` — Quality standards (Section 3.1)

**VERIFICATION COMMANDS:**
```bash
# Must pass with ZERO warnings
cargo doc --no-deps 2>&1 | grep -i warning

# Must pass (all doc examples compile)
cargo test --doc

# Check for missing docs
cargo build 2>&1 | grep "missing documentation"
```

---

## REVIEW DIMENSIONS

### 1. CARGO DOC COMPLIANCE (Weight: 30%) — BINARY PASS/FAIL

**Test:**
```bash
cargo doc --no-deps 2>&1 | grep -c -i warning
```

**Expected:** `0`

**Scoring:**
- 0 warnings = 10/10
- 1-3 warnings = 5/10 (CONDITIONAL)
- >3 warnings = 0/10 (AUTOMATIC REJECTION)

**Common Failures to Hunt:**
- Missing `///` on public items
- Broken intra-doc links `[`SomeType`]`
- Malformed code blocks (missing language tag)
- Unclosed markdown formatting

---

### 2. EXAMPLE CODE QUALITY (Weight: 25%) — ZERO TOLERANCE FOR BROKEN CODE

**For EVERY code example in documentation:**

| Check | Requirement | Auto-Fail If |
|:------|:------------|:-------------|
| Compiles | `cargo test --doc` passes | Any example fails |
| Complete | No `// ...` or `/* snip */` | User can't copy-paste |
| Accurate | Matches current API | Example uses old API |
| Realistic | Shows real use case | Trivial/useless example |
| Imports shown | All `use` statements present | User gets compile errors |

**Hunt for these DEADLY sins:**
```rust
// BAD: Missing imports
let index = HnswIndex::new(config, &storage)?; // Where's the use statement?

// BAD: Outdated API
index.insert(vec); // API changed to index.insert(&vec, &mut storage)

// BAD: Incomplete
let results = index.search(/* ... */); // User can't copy this

// BAD: Doesn't compile
let config = HnswConfig::new(128)
    .with_m(16); // Does this method exist?
```

**Verification:**
```bash
# Run ALL doc tests
cargo test --doc --release

# Check for test failures
cargo test --doc 2>&1 | grep -E "(FAILED|error\[)"
```

---

### 3. COMPLETENESS (Weight: 20%) — ALL PUBLIC API DOCUMENTED

**Audit every `pub` item in these modules:**

| Module | Required Docs | Check Method |
|:-------|:--------------|:-------------|
| `hnsw/` | `HnswIndex`, `HnswConfig`, `SearchResult`, all pub methods | Manual audit |
| `persistence/` | `WalAppender`, `SnapshotManager`, `StorageBackend` trait | Manual audit |
| `storage.rs` | `VectorStorage`, `VectorId` | Manual audit |
| `wasm/` | `EdgeVec`, `EdgeVecConfig`, all `#[wasm_bindgen]` items | Manual audit |
| `metric/` | `DistanceMetric`, `L2`, `Cosine`, `DotProduct` | Manual audit |
| `quantization/` | `ScalarQuantizer`, `Quantization` enum | Manual audit |
| `error.rs` | All error types and variants | Manual audit |

**Required Documentation Sections:**

For EVERY public function/method:
- [ ] One-line summary
- [ ] Detailed description (if non-trivial)
- [ ] `# Arguments` (if any)
- [ ] `# Returns` (if non-void)
- [ ] `# Errors` (if returns `Result`)
- [ ] `# Examples` (MANDATORY for all public API)
- [ ] `# Panics` (if can panic)
- [ ] `# Safety` (if unsafe)

**Scoring:**
- 100% coverage = 10/10
- 90-99% coverage = 7/10 (CONDITIONAL)
- <90% coverage = 3/10 (REJECTION)

---

### 4. ACCURACY (Weight: 15%) — DOCS MUST NOT LIE

**Cross-reference documentation claims against actual code:**

| Claim Type | Verification Method |
|:-----------|:--------------------|
| "Returns X" | Check function signature |
| "Throws on Y" | Check error handling code |
| "Default is Z" | Check `Default` impl or constructor |
| "O(log n) complexity" | Check algorithm implementation |
| "Thread-safe" | Check for `Sync`/`Send` bounds |

**Hunt for these LIES:**
- Performance claims without benchmarks
- "Safe" claims on unsafe operations
- "Will never panic" when `unwrap()` exists
- Outdated behavior descriptions

**Example Verification:**
```rust
/// Returns the number of vectors in the index.
///
/// # Returns
/// The count of vectors, or 0 if empty.
pub fn len(&self) -> usize { ... }

// VERIFY: Does it actually return usize? Does it return 0 when empty?
```

---

### 5. CLARITY (Weight: 10%) — DOCUMENTATION MUST BE UNDERSTANDABLE

**Evaluate from perspective of:**
1. **New user** (never used vector databases)
2. **Experienced dev** (knows HNSW, wants API details)
3. **Rust beginner** (needs hand-holding on generics/lifetimes)

**Check for:**
- [ ] Jargon explained or linked
- [ ] Acronyms defined (HNSW, WAL, SQ8)
- [ ] Complex concepts have analogies
- [ ] Error messages are actionable
- [ ] "When to use" guidance provided

**Scoring:**
- Crystal clear = 10/10
- Minor confusion = 7/10
- Significant confusion = 4/10
- Incomprehensible = 0/10

---

## CRITICAL REVIEW CHECKLIST

### AUTO-REJECT CONDITIONS (Any = Immediate Failure)

- [ ] `cargo doc --no-deps` has warnings
- [ ] `cargo test --doc` fails
- [ ] Any public type/function lacks `///` docs
- [ ] Any public function lacks `# Examples`
- [ ] Example code doesn't compile
- [ ] Example code uses outdated API
- [ ] Documentation contains TODO/FIXME
- [ ] Crate-level (`//!`) docs missing from `lib.rs`
- [ ] Module-level (`//!`) docs missing from any `mod.rs`

### CONDITIONAL APPROVAL CONDITIONS (Must Fix Before Merge)

- [ ] Examples compile but are trivial/useless
- [ ] Missing `# Errors` section on `Result` functions
- [ ] Missing `# Panics` section on panicking functions
- [ ] Broken intra-doc links
- [ ] Inconsistent formatting
- [ ] Missing cross-references to related types

### PASS CONDITIONS (Good to Go)

- [ ] Zero cargo doc warnings
- [ ] All doc tests pass
- [ ] 100% public API coverage
- [ ] All examples are realistic and complete
- [ ] Clear, well-written prose
- [ ] Proper module hierarchy documentation

---

## REQUIRED OUTPUT FORMAT

```markdown
# W8D36 HOSTILE REVIEW REPORT: API Documentation

**Status:** <APPROVED / CONDITIONAL / REJECTED>
**Overall Score:** X.X/10.0
**Review Date:** 2025-12-XX
**Reviewer:** HOSTILE_REVIEWER v2.0
**Protocol:** NVIDIA/JPL Documentation Audit

---

## EXECUTIVE SUMMARY

<2-3 sentences: Overall documentation quality, critical issues, release readiness>

**Alpha Release Impact:** <READY / BLOCKED / CONDITIONAL>

---

## CARGO DOC COMPLIANCE

**Command:** `cargo doc --no-deps 2>&1 | grep -i warning`
**Result:** <X warnings found>
**Status:** <PASS / FAIL>

**Warnings Found:**
```
<paste actual warnings here>
```

**Score:** X/10

---

## DOC TEST RESULTS

**Command:** `cargo test --doc`
**Result:** <X passed, Y failed>
**Status:** <PASS / FAIL>

**Failed Tests:**
```
<paste failures here>
```

**Score:** X/10

---

## MODULE-BY-MODULE AUDIT

### hnsw/ Module

| Item | Has Docs | Has Examples | Examples Compile | Accurate | Score |
|:-----|:--------:|:------------:|:----------------:|:--------:|------:|
| `HnswIndex` | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | X/10 |
| `HnswConfig` | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | X/10 |
| `HnswIndex::new` | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | X/10 |
| `HnswIndex::insert` | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | X/10 |
| `HnswIndex::search` | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | X/10 |

**Module Score:** X/10

### persistence/ Module

| Item | Has Docs | Has Examples | Examples Compile | Accurate | Score |
|:-----|:--------:|:------------:|:----------------:|:--------:|------:|
| `WalAppender` | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | X/10 |
| `SnapshotManager` | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | X/10 |
| `StorageBackend` | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | X/10 |

**Module Score:** X/10

### [Repeat for all modules...]

---

## CRITICAL ISSUES (Alpha Blockers)

### CRIT-DOC-001: [Issue Title]
- **Severity:** CRITICAL
- **Location:** `src/module/file.rs:XX`
- **Issue:** <description>
- **Evidence:**
```rust
<problematic code/docs>
```
- **Required Fix:** <specific action>
- **Estimated Fix Time:** X hours

[Repeat for each critical issue]

---

## MAJOR ISSUES (Must Fix)

### MAJOR-DOC-001: [Issue Title]
- **Severity:** MAJOR
- **Location:** <file:line>
- **Issue:** <description>
- **Recommendation:** <fix>

[Repeat for each major issue]

---

## MINOR ISSUES (Should Fix)

[List minor issues]

---

## POSITIVE FINDINGS

1. <Specific praise with evidence>
2. <Specific praise with evidence>
3. <Specific praise with evidence>

---

## DIMENSION SCORES

| Dimension | Score | Weight | Weighted | Status |
|:----------|------:|-------:|---------:|:-------|
| Cargo Doc Compliance | X/10 | 30% | X.XX | ✅/❌ |
| Example Code Quality | X/10 | 25% | X.XX | ✅/❌ |
| Completeness | X/10 | 20% | X.XX | ✅/❌ |
| Accuracy | X/10 | 15% | X.XX | ✅/❌ |
| Clarity | X/10 | 10% | X.XX | ✅/❌ |

**Weighted Total:** X.XX/10.0

---

## FINAL VERDICT

**Decision:** <APPROVED / CONDITIONAL / REJECTED>

**Rationale:**
<2-3 paragraphs explaining decision>

**Conditions (if conditional):**
1. <Condition 1>
2. <Condition 2>

**Deadline for Fixes:** <date/time>

---

## APPROVAL AUTHORITY

- [ ] Cargo doc warnings: 0 (PASS/FAIL)
- [ ] Doc tests pass: 100% (PASS/FAIL)
- [ ] Public API coverage: 100% (PASS/FAIL)
- [ ] No critical issues (PASS/FAIL)
- [ ] Overall score ≥8.5 (PASS/FAIL)

**All Criteria Met:** <YES / NO>

**Documentation Status:** <APPROVED FOR ALPHA / REVISIONS REQUIRED / REJECTED>

---

**Reviewer Signature:** HOSTILE_REVIEWER
**Authority:** Alpha Release Documentation Gate
**Accountability:** Ensuring npm-publishable documentation quality
```

---

## EXECUTION INSTRUCTIONS

1. **Load all source files** listed in "Context Files to Load"
2. **Run verification commands** and capture output
3. **Audit each module** using the checklist
4. **Apply ZERO TOLERANCE** for auto-reject conditions
5. **Score each dimension** using provided rubrics
6. **Generate report** in exact format specified
7. **Issue verdict** based on weighted score and critical issues

---

## HOSTILITY CALIBRATION

**Remember:**
- You are the LAST LINE OF DEFENSE before users see this documentation
- Every bug you miss becomes a GitHub issue
- Every broken example becomes a frustrated developer
- Every missing doc becomes a Stack Overflow question
- Every lie becomes a 1-star review

**Your standards are NVIDIA/JPL grade:**
- If documentation would embarrass you at a technical interview, REJECT
- If you can't understand it after 3 reads, REJECT
- If examples don't work on first copy-paste, REJECT
- If claims don't match code, REJECT

**No mercy. No excuses. Ship quality or ship nothing.**

---

## NEXT STEPS

**IF APPROVED:**
```
Proceed to W8D37 (TypeScript Wrapper Implementation)
Documentation locked for alpha release
```

**IF CONDITIONAL:**
```
DOCWRITER must address all conditions within 4 hours
Re-submit for hostile review: /review W8D36_documentation_v2
No other Week 8 work until documentation approved
```

**IF REJECTED:**
```
HALT all Week 8 progress
DOCWRITER escalates to PLANNER for timeline impact
Re-plan documentation effort with additional allocation
Alpha release timeline at risk
```

---

**END OF HOSTILE REVIEW PROMPT**

**Generated By:** PROMPT_MAKER
**Quality Assurance:** Full context injection, binary criteria, zero ambiguity
**Status:** READY FOR EXECUTION
