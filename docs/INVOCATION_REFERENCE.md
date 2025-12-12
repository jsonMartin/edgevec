# INVOCATION REFERENCE: EdgeVec Claude Code Commands

**Version:** 1.0.0
**Purpose:** Quick lookup for all slash commands organized by phase and agent
**Audience:** EdgeVec developers using Claude Code CLI

---

## QUICK NAVIGATION

- [Phase-Based Workflow](#phase-based-workflow)
- [All Commands (Alphabetical)](#all-commands-alphabetical)
- [Commands by Agent](#commands-by-agent)
- [Common Workflows](#common-workflows)
- [Emergency Commands](#emergency-commands)

---

## PHASE-BASED WORKFLOW

### Phase 1: Architecture (No Code Allowed)

**Allowed Commands:**

| Command | Purpose | Example |
|:--------|:--------|:--------|
| `/dispatch` | Analyze request and recommend command | `/dispatch` |
| `/architect-design [component]` | Design architecture for component | `/architect-design hnsw` |
| `/architect-review [document]` | Review existing architecture | `/architect-review ARCHITECTURE.md` |
| `/architect-validate [claim]` | Verify technical claim with sources | `/architect-validate "SharedArrayBuffer requires COOP headers"` |
| `/review [artifact]` | HOSTILE_REVIEWER quality gate | `/review ARCHITECTURE.md` |

**Exit Condition:** GATE_1_COMPLETE.md created after HOSTILE_REVIEWER approval

---

### Phase 2: Planning (No Code Allowed)

**Allowed Commands:**

| Command | Purpose | Example |
|:--------|:--------|:--------|
| `/dispatch` | Analyze request and recommend command | `/dispatch` |
| `/planner-roadmap` | Generate 6-month roadmap | `/planner-roadmap` |
| `/planner-weekly [N]` | Generate weekly task plan | `/planner-weekly 1` |
| `/planner-replan [reason]` | Adjust existing plans | `/planner-replan "WASM constraints changed"` |
| `/review [artifact]` | HOSTILE_REVIEWER quality gate | `/review ROADMAP.md` |

**Exit Condition:** GATE_3_COMPLETE.md created after HOSTILE_REVIEWER approves WEEKLY_TASK_PLAN.md

---

### Phase 3: Implementation (Code Unlocked)

**Allowed Commands:**

| Command | Purpose | Example |
|:--------|:--------|:--------|
| `/dispatch` | Analyze request and recommend command | `/dispatch` |
| `/rust-implement [task-id]` | Implement Rust code for task | `/rust-implement W1.1` |
| `/rust-test [module]` | Write tests for module | `/rust-test hnsw` |
| `/rust-bench [function]` | Create benchmark for function | `/rust-bench search` |
| `/rust-refactor [scope]` | Refactor code (if in plan) | `/rust-refactor hnsw::insert` |
| `/test-prop [invariant]` | Create property tests | `/test-prop "insert is commutative"` |
| `/test-fuzz [module]` | Create fuzz targets | `/test-fuzz parser` |
| `/test-miri [scope]` | Run Miri on unsafe blocks | `/test-miri hnsw` |
| `/test-regression [bug-id]` | Create regression test | `/test-regression issue-42` |
| `/wasm-bind [function]` | Create WASM bindings | `/wasm-bind search` |
| `/wasm-types` | Generate TypeScript definitions | `/wasm-types` |
| `/wasm-test [browser]` | Run browser-specific tests | `/wasm-test safari` |
| `/wasm-compat` | Generate compatibility matrix | `/wasm-compat` |
| `/bench-baseline [component]` | Establish performance baseline | `/bench-baseline hnsw` |
| `/bench-compare [competitor]` | Compare to competitor | `/bench-compare sqlite-vec` |
| `/bench-regression` | Check for performance regressions | `/bench-regression` |
| `/bench-report` | Generate comprehensive report | `/bench-report` |
| `/doc-readme` | Update README.md | `/doc-readme` |
| `/doc-api [module]` | Document API for module | `/doc-api hnsw` |
| `/doc-changelog [version]` | Generate changelog | `/doc-changelog v0.2.0` |
| `/doc-examples` | Create/update examples | `/doc-examples` |
| `/review [artifact]` | HOSTILE_REVIEWER quality gate | `/review src/hnsw.rs` |

---

## ALL COMMANDS (ALPHABETICAL)

| Command | Agent | Phase | Description |
|:--------|:------|:------|:------------|
| `/architect-design [component]` | META_ARCHITECT | 1 | Design architecture for component |
| `/architect-review [document]` | META_ARCHITECT | 1 | Review existing architecture document |
| `/architect-validate [claim]` | META_ARCHITECT | 1 | Verify technical claim with sources |
| `/bench-baseline [component]` | BENCHMARK_SCIENTIST | 3+ | Establish performance baseline |
| `/bench-compare [competitor]` | BENCHMARK_SCIENTIST | 3+ | Compare to competitor system |
| `/bench-regression` | BENCHMARK_SCIENTIST | 3+ | Check for performance regressions |
| `/bench-report` | BENCHMARK_SCIENTIST | 3+ | Generate comprehensive performance report |
| `/dispatch` | WORKFLOW_ROUTER | All | Analyze request and recommend command |
| `/doc-api [module]` | DOCWRITER | 3+ | Generate API documentation |
| `/doc-changelog [version]` | DOCWRITER | 3+ | Generate changelog for version |
| `/doc-examples` | DOCWRITER | 3+ | Create/update code examples |
| `/doc-readme` | DOCWRITER | 3+ | Update README.md |
| `/planner-replan [reason]` | PLANNER | 2 | Adjust existing plans based on changes |
| `/planner-roadmap` | PLANNER | 2 | Generate 6-month roadmap |
| `/planner-weekly [N]` | PLANNER | 2 | Generate weekly task plan for week N |
| `/review [artifact]` | HOSTILE_REVIEWER | All | Quality gate review with approval/rejection |
| `/rust-bench [function]` | RUST_ENGINEER | 3+ | Create benchmark for function |
| `/rust-implement [task-id]` | RUST_ENGINEER | 3+ | Implement Rust code for task |
| `/rust-refactor [scope]` | RUST_ENGINEER | 3+ | Refactor code (must be in plan) |
| `/rust-test [module]` | RUST_ENGINEER | 3+ | Write comprehensive tests |
| `/test-fuzz [module]` | TEST_ENGINEER | 3+ | Create fuzz targets |
| `/test-miri [scope]` | TEST_ENGINEER | 3+ | Run Miri verification on unsafe |
| `/test-prop [invariant]` | TEST_ENGINEER | 3+ | Create property-based tests |
| `/test-regression [bug-id]` | TEST_ENGINEER | 3+ | Create regression test for bug |
| `/wasm-bind [function]` | WASM_SPECIALIST | 3+ | Create WASM bindings |
| `/wasm-compat` | WASM_SPECIALIST | 3+ | Generate browser compatibility matrix |
| `/wasm-test [browser]` | WASM_SPECIALIST | 3+ | Run browser-specific tests |
| `/wasm-types` | WASM_SPECIALIST | 3+ | Generate TypeScript definitions |

---

## COMMANDS BY AGENT

### META_ARCHITECT Commands

**Purpose:** System design and architecture before code

| Command | Description | Usage Example |
|:--------|:------------|:--------------|
| `/architect-design [component]` | Design a new component with data layouts, WASM boundaries, performance budgets | `/architect-design hnsw` |
| `/architect-review [document]` | Review existing architecture document for gaps or inconsistencies | `/architect-review DATA_LAYOUT.md` |
| `/architect-validate [claim]` | Verify a technical claim (e.g., browser API availability) with sources | `/architect-validate "IndexedDB quota is 50MB"` |

**Outputs:** ARCHITECTURE.md, DATA_LAYOUT.md, WASM_BOUNDARY.md, PERSISTENCE_SPEC.md, INVARIANTS.md

**Next Step:** `/review ARCHITECTURE.md` (HOSTILE_REVIEWER approval required)

---

### PLANNER Commands

**Purpose:** Roadmap and weekly task planning

| Command | Description | Usage Example |
|:--------|:------------|:--------------|
| `/planner-roadmap` | Generate comprehensive 6-month roadmap from approved architecture | `/planner-roadmap` |
| `/planner-weekly [N]` | Generate WEEKLY_TASK_PLAN.md for week N with specific tasks | `/planner-weekly 1` |
| `/planner-replan [reason]` | Adjust existing plan based on new information or blockers | `/planner-replan "SharedArrayBuffer unavailable in Safari"` |

**Outputs:** ROADMAP.md, WEEKLY_TASK_PLAN.md, RISK_REGISTER.md, DEPENDENCY_GRAPH.md

**Next Step:** `/review WEEKLY_TASK_PLAN.md` (HOSTILE_REVIEWER approval unlocks coding)

---

### RUST_ENGINEER Commands

**Purpose:** Core Rust implementation

| Command | Description | Usage Example |
|:--------|:------------|:--------------|
| `/rust-implement [task-id]` | Implement a specific task from approved WEEKLY_TASK_PLAN.md | `/rust-implement W1.1` |
| `/rust-test [module]` | Write comprehensive unit tests for a module | `/rust-test hnsw` |
| `/rust-bench [function]` | Create criterion benchmark for a function | `/rust-bench search` |
| `/rust-refactor [scope]` | Refactor code (only if refactor task is in approved plan) | `/rust-refactor hnsw::insert` |

**Outputs:** Rust source files in `src/`, tests in `tests/`, benchmarks in `benches/`

**Requirements:**
- WEEKLY_TASK_PLAN.md must be APPROVED
- Task ID must exist in plan
- All tests must pass before submission

**Next Step:** `/test-prop [task-id]` or `/review [implementation]`

---

### TEST_ENGINEER Commands

**Purpose:** Verification, fuzzing, property testing ("Nvidia Grade")

| Command | Description | Usage Example |
|:--------|:------------|:--------------|
| `/test-prop [invariant]` | Create property-based tests for an invariant | `/test-prop "search results are sorted by score"` |
| `/test-fuzz [module]` | Create fuzz targets for parsers or input handlers | `/test-fuzz parser` |
| `/test-miri [scope]` | Run Miri on unsafe blocks to detect undefined behavior | `/test-miri hnsw` |
| `/test-regression [bug-id]` | Create regression test for a reported bug | `/test-regression issue-42` |

**Outputs:** Property tests in `tests/proptest/`, fuzz targets in `fuzz/`, Miri configs

**Requirements:**
- Implementation must exist (from RUST_ENGINEER)
- ARCHITECTURE.md defines invariants to test

**Next Step:** `/review [test suite]` or proceed to `/wasm-bind` or `/bench-baseline`

---

### WASM_SPECIALIST Commands

**Purpose:** WASM bindings and browser integration

| Command | Description | Usage Example |
|:--------|:------------|:--------------|
| `/wasm-bind [function]` | Create wasm-bindgen bindings for a Rust function | `/wasm-bind search` |
| `/wasm-types` | Generate TypeScript type definitions (.d.ts) | `/wasm-types` |
| `/wasm-test [browser]` | Run tests in specific browser (chrome, firefox, safari, edge) | `/wasm-test safari` |
| `/wasm-compat` | Generate browser compatibility matrix with version support | `/wasm-compat` |

**Outputs:** WASM bindings in `src/*_wasm.rs`, TypeScript types in `pkg/*.d.ts`, browser tests

**Requirements:**
- Core Rust implementation must compile to `wasm32-unknown-unknown`
- WASM_BOUNDARY.md must specify interface

**Next Step:** `/bench-baseline` to measure WASM performance

---

### BENCHMARK_SCIENTIST Commands

**Purpose:** Performance measurement and validation

| Command | Description | Usage Example |
|:--------|:------------|:--------------|
| `/bench-baseline [component]` | Establish baseline metrics for a component | `/bench-baseline hnsw` |
| `/bench-compare [competitor]` | Run fair comparison against competitor system | `/bench-compare sqlite-vec` |
| `/bench-regression` | Check current code against baseline for regressions | `/bench-regression` |
| `/bench-report` | Generate comprehensive performance report with hardware specs | `/bench-report` |

**Outputs:** Benchmark code in `benches/`, reports in `docs/benchmarks/`, baseline data in `baselines/`

**Requirements:**
- Implementation must exist
- ARCHITECTURE.md specifies performance budget
- Hardware specs documented

**Next Step:** `/review benchmark_report.md`

---

### HOSTILE_REVIEWER Commands

**Purpose:** Ultimate quality gate with veto power

| Command | Description | Usage Example |
|:--------|:------------|:--------------|
| `/review [artifact]` | Comprehensive quality gate review with APPROVE or REJECT | `/review ARCHITECTURE.md` |
| `/review-approve [artifact]` | Quick approval for trivial changes (must justify) | `/review-approve typo-fix.md` |
| `/review-reject [artifact] [reason]` | Immediate rejection with stated reason | `/review-reject src/hnsw.rs "unwrap() on line 42"` |
| `/review-status` | Report current review queue and pending approvals | `/review-status` |

**Outputs:** Approval or rejection documents with findings and required actions

**Authority:** ULTIMATE VETO POWER — No artifact proceeds without approval

**Reference:** `.claude/HOSTILE_GATE_CHECKLIST.md` for complete criteria

---

### DOCWRITER Commands

**Purpose:** Developer-facing documentation

| Command | Description | Usage Example |
|:--------|:------------|:--------------|
| `/doc-readme` | Generate/update README.md with viral hook and quick start | `/doc-readme` |
| `/doc-api [module]` | Generate complete API reference for module | `/doc-api hnsw` |
| `/doc-changelog [version]` | Generate changelog for version with all changes | `/doc-changelog v0.2.0` |
| `/doc-examples` | Create/update runnable code examples | `/doc-examples` |

**Outputs:** README.md, docs/API.md, docs/GETTING_STARTED.md, CHANGELOG.md

**Requirements:**
- Implementation must be stable
- All code examples must be tested in CI

**Next Step:** `/review README.md` before public release

---

### PROMPT_MAKER (WORKFLOW_ROUTER) Commands

**Purpose:** Meta-agent dispatcher with quality control, constraint injection, and security hardening

| Command | Description | Usage Example |
|:--------|:------------|:--------------|
| `/dispatch` | Analyze user request, detect phase, recommend command, validate contracts, generate structured prompts (v2.0.0 with security protocols) | `/dispatch` |

**Agent:** PROMPT_MAKER (see `.claude/agents/prompt-maker.md`)
**Version:** 2.0.0 (Security-Hardened)
**Security Score:** 10/10 (Target)

**How It Works — Three Sequential Modes with Security Protocols:**

**MODE 1: Safety Scanner (with Security Hardening)**
- **Security:** Sanitizes user input (blocks path traversal, command injection, null bytes)
- **Security:** Creates atomic snapshot to prevent TOCTOU race conditions
- Detects current project phase (Architecture | Planning | Implementation | Polish)
- Validates all contracts (Architecture > Plan > Code)
- Checks gate completions (GATE_1_COMPLETE.md, etc.)
- Detects scope creep (tasks not in approved plan)
- **Security:** Validates snapshot is still current before proceeding
- Outputs: ✅ VALID or ⛔ VIOLATION or ⚠️ SECURITY ERROR

**MODE 2: Agent Router (with Weighted Classification)**
- **New:** Weighted keyword matching with confidence scoring (0.0-1.0)
- **New:** Phase-based context modifiers for better accuracy
- Classifies intent using advanced algorithm
- Applies context clues (open files, task IDs, last command)
- Maps to correct specialized agent
- Plans multi-agent pipelines for complex tasks
- **New:** Ambiguity detection (confidence < 30% triggers clarification)
- Outputs: Recommended command with confidence score and rationale

**MODE 3: Prompt Generator (with Fault-Tolerant Loading)**
- **Security:** Fault-tolerant file loading with retry logic
- **Security:** Uses canonical file locations (no ambiguity)
- Loads relevant context (architecture, plans, specs)
- **Injects constraints** from ARCHITECTURE.md, CLAUDE.md, DATA_LAYOUT.md, etc.
- Creates structured, atomic task descriptions
- Includes validation criteria and expected outputs
- States next command in pipeline
- Manages context window to prevent overflow

---

**Security Protocols (v2.0.0):**

1. **Protocol 1: Fault-Tolerant File Loading**
   - All file operations use retry logic with exponential backoff (3 attempts)
   - Handles: FileNotFound, PermissionError, UnicodeError, OSError
   - Clear error messages with recovery steps

2. **Protocol 2: TOCTOU Prevention**
   - Atomic snapshots with file hashing prevent race conditions
   - Snapshot staleness detection (max 5 seconds)
   - Concurrent modification protection

3. **Protocol 3: Input Sanitization**
   - Blocks path traversal attacks (`..`, `~/`)
   - Blocks command injection (`;`, `|`, `&`, `` ` ``, etc.)
   - Blocks null byte injection (`\x00`)
   - Enforces 10k character limit

4. **Protocol 4: Canonical File Locations**
   - Authoritative file map (primary + fallback)
   - Conflict detection and migration warnings
   - No ambiguous paths

5. **Protocol 5: Weighted Intent Classification**
   - Keyword scoring with confidence thresholds
   - Phase-based context boosting
   - Ambiguity detection with user clarification

**Attack Vectors Blocked:** 5/5
- Path Traversal ✅
- Command Injection ✅
- Null Byte Injection ✅
- Resource Exhaustion ✅
- TOCTOU Race Conditions ✅

**Outputs:** Detailed analysis with:
- Phase detection and evidence
- Intent classification with keywords
- Contract validation status
- Recommended command with arguments
- Structured prompt with constraints
- Next steps in pipeline
- Alternative options if applicable

**When to Use:**
- Unsure which command to use
- Want to verify no contract violations
- Need a prompt with architectural constraints injected
- Complex request spanning multiple agents
- Need phase validation
- Want to understand current project state

**Example Output:**
```markdown
## MODE 1: Safety Scanner
✅ VALID — Phase 3, all gates passed

## MODE 2: Agent Router
Intent: Implementation
Primary Agent: RUST_ENGINEER
Command: /rust-implement W2.3

## MODE 3: Prompt Generator
[Structured prompt with:]
- Task description
- Context files to load
- Step-by-step instructions
- Constraints from ARCHITECTURE.md §X.Y
- Constraints from CLAUDE.md
- Validation criteria
- Expected outputs

Next: /test-prop W2.3
```

---

## COMMON WORKFLOWS

### Workflow 1: New Feature (Complete Pipeline)

```
Step 1: Analyze request
/dispatch

Step 2: Design architecture
/architect-design [feature-name]

Step 3: Get architecture approved
/review ARCHITECTURE.md

Step 4: Create roadmap
/planner-roadmap

Step 5: Get roadmap approved
/review ROADMAP.md

Step 6: Create weekly plan
/planner-weekly 1

Step 7: Get weekly plan approved
/review WEEKLY_TASK_PLAN.md

Step 8: Implement task
/rust-implement W1.1

Step 9: Test implementation
/test-prop W1.1

Step 10: Fuzz if needed
/test-fuzz [module]

Step 11: Create WASM bindings
/wasm-bind [function]

Step 12: Benchmark performance
/bench-baseline [component]

Step 13: Final review
/review [all-outputs]

Step 14: Document
/doc-api [module]
/doc-readme
```

**Estimated Time:** 2-4 weeks depending on complexity

---

### Workflow 2: Bug Fix

```
Step 1: Verify bug is in current plan
(Check WEEKLY_TASK_PLAN.md or add to plan via /planner-replan)

Step 2: Implement fix
/rust-implement [bug-task-id]

Step 3: Create regression test
/test-regression [bug-id]

Step 4: Verify no performance regression
/bench-regression

Step 5: Review fix
/review [fix-files]
```

**Estimated Time:** 2-8 hours

---

### Workflow 3: Optimization

```
Step 1: Establish baseline (if not exists)
/bench-baseline [component]

Step 2: Implement optimization (must be in plan)
/rust-implement [opt-task-id]

Step 3: Benchmark new version
/bench-compare baseline

Step 4: Verify correctness maintained
/test-prop [invariant]
/test-fuzz [module]

Step 5: Review optimization
/review [optimization-output]
```

**Estimated Time:** 1-3 days

---

### Workflow 4: Documentation Update

```
Step 1: Update README
/doc-readme

Step 2: Update API docs
/doc-api [module]

Step 3: Update examples
/doc-examples

Step 4: Review (optional for trivial)
/review README.md
```

**Estimated Time:** 1-4 hours

---

### Workflow 5: WASM Integration

```
Step 1: Verify Rust code compiles to WASM
(Manual: cargo check --target wasm32-unknown-unknown)

Step 2: Create bindings
/wasm-bind [function]

Step 3: Generate types
/wasm-types

Step 4: Test in browsers
/wasm-test chrome
/wasm-test firefox
/wasm-test safari

Step 5: Generate compatibility matrix
/wasm-compat

Step 6: Review bindings
/review [wasm-files]
```

**Estimated Time:** 1-2 days

---

## EMERGENCY COMMANDS

### When Stuck: Use Dispatch

```
/dispatch
```

**Output:** Detailed analysis of your situation with recommendations

---

### When Gates Are Blocking: Check Status

```
ls .claude/GATE_*.md
```

**Output:** Shows which gates are complete
- GATE_1_COMPLETE.md → Architecture approved
- GATE_2_COMPLETE.md → Roadmap approved
- GATE_3_COMPLETE.md → Weekly plan approved (coding unlocked)

---

### When Unsure About Phase: Check Plan

```
cat WEEKLY_TASK_PLAN.md | grep "Status:"
```

**Output:** Shows if plan is DRAFT, APPROVED, IN_PROGRESS, COMPLETE

---

### When Review Fails: Check Checklist

```
cat .claude/HOSTILE_GATE_CHECKLIST.md
```

**Output:** Complete quality criteria by artifact type

---

### When Command Fails: Check Permissions

```
cat .claude/settings.json | grep -A20 "permissions"
```

**Output:** Shows allowed and denied commands

---

## ARGUMENT REFERENCE

### Task ID Format

**Format:** `W[week].[task]`

**Examples:**
- `W1.1` — Week 1, Task 1
- `W2.3` — Week 2, Task 3

**Where to Find:** WEEKLY_TASK_PLAN.md

---

### Artifact Names

**Valid Artifact Names for `/review`:**
- Architecture: `ARCHITECTURE.md`, `DATA_LAYOUT.md`, `WASM_BOUNDARY.md`
- Planning: `ROADMAP.md`, `WEEKLY_TASK_PLAN.md`
- Code: `src/hnsw.rs`, `tests/hnsw_test.rs`
- Benchmarks: `docs/benchmarks/report.md`
- Documentation: `README.md`, `docs/API.md`

---

### Component Names

**Common Component Names:**
- `hnsw` — HNSW index algorithm
- `quantizer` — Binary quantization
- `persistence` — IndexedDB storage
- `wasm` — WASM bindings
- `search` — Search API

**Where to Find:** ARCHITECTURE.md Component Breakdown section

---

### Browser Names

**Valid Browser Names for `/wasm-test`:**
- `chrome` — Google Chrome
- `firefox` — Mozilla Firefox
- `safari` — Apple Safari
- `edge` — Microsoft Edge

---

## TIPS & TRICKS

### Tip 1: Chain Commands

After each command completes, the output suggests the next command. Follow the chain:

```
/architect-design hnsw
[Output: "Next: /review ARCHITECTURE.md"]

/review ARCHITECTURE.md
[Output: "Status: ✅ APPROVED. Next: /planner-roadmap"]

/planner-roadmap
[And so on...]
```

---

### Tip 2: Use Tab Completion (if available)

Type `/arch` and press TAB to auto-complete to `/architect-design`.

---

### Tip 3: Load Context First

Before running a command, ensure required files are loaded:

**Check what's needed:**
```
cat .claude/commands/[command-name].md | grep "Required Context"
```

**Load files:**
Use file browser or `/add` command for each file.

---

### Tip 4: Bookmark This File

Keep this file open as a reference. Quick lookup beats memorization.

---

### Tip 5: When In Doubt, `/dispatch`

If you're unsure which command to use, `/dispatch` will analyze your request and recommend the correct path.

---

## FREQUENTLY ASKED QUESTIONS

### Q: What's the difference between `/rust-implement` and `/rust-test`?

**A:**
- `/rust-implement` writes production code (`src/*.rs`)
- `/rust-test` writes unit tests (`tests/*.rs`)

Usually you run `/rust-implement` first, then `/rust-test` or `/test-prop`.

---

### Q: Can I skip the architecture phase?

**A:** No. The Supreme Rule (Architecture > Plan > Code) is enforced by the permission system. You'll get an error if you try to code without an approved plan, which requires approved architecture.

**Exception:** Documentation-only changes can skip architecture.

---

### Q: What if HOSTILE_REVIEWER rejects my work?

**A:**
1. Read the rejection document carefully (it lists specific issues)
2. Fix ALL critical issues
3. Fix ALL major issues
4. Tag artifact `[REVISED]`
5. Resubmit for review

The rejection document includes a checklist of required actions.

---

### Q: Can I create custom commands?

**A:** Yes. Add a new file to `.claude/commands/[your-command].md` following the template format. Reference an existing agent in `.claude/agents/`.

---

### Q: How do I know which agent executes a command?

**A:** Each command file (`.claude/commands/*.md`) specifies which agent it invokes. Example:

```markdown
# Command: rust-implement
**Agent:** rust-engineer
```

---

### Q: What's the fastest way to get from idea to code?

**A:** Use `/dispatch` to get the recommended pipeline, then follow it exactly:

```
/dispatch
[Follow recommended commands]
```

Typical pipeline: `/architect-design` → `/review` → `/planner-weekly` → `/review` → `/rust-implement`

**Minimum time:** 1-2 days (including review cycles)

---

## COMMAND SYNTAX PATTERNS

### Pattern 1: Agent Action

Most commands follow: `/[agent]-[action]`

Examples:
- `/rust-implement`
- `/wasm-bind`
- `/bench-baseline`
- `/doc-readme`

---

### Pattern 2: Agent Action Target

Some commands take a target: `/[agent]-[action] [target]`

Examples:
- `/architect-design hnsw`
- `/test-fuzz parser`
- `/doc-api search`

---

### Pattern 3: Special Commands

Some commands have unique syntax:
- `/dispatch` (no arguments — analyzes context)
- `/review [artifact]` (universal review command)

---

## VERSION HISTORY

| Version | Date | Changes |
|:--------|:-----|:--------|
| 1.0.0 | 2025-01-XX | Initial release — Cursor to Claude Code conversion complete |

---

## APPENDIX: COMPLETE COMMAND LIST WITH ONE-LINE DESCRIPTIONS

```
/architect-design [component]     — Design architecture with data layouts and memory budgets
/architect-review [document]      — Review existing architecture for gaps or contradictions
/architect-validate [claim]       — Verify technical claim with cited sources
/bench-baseline [component]       — Establish performance baseline with reproducible metrics
/bench-compare [competitor]       — Fair comparison against competitor (same hardware/dataset)
/bench-regression                 — Check for performance regressions vs baseline
/bench-report                     — Generate comprehensive performance report with specs
/dispatch                         — Analyze request, detect phase, recommend command
/doc-api [module]                 — Generate complete API reference with examples
/doc-changelog [version]          — Generate changelog with all version changes
/doc-examples                     — Create/update runnable, tested code examples
/doc-readme                       — Update README.md with viral hook and quick start
/planner-replan [reason]          — Adjust plans based on new information or blockers
/planner-roadmap                  — Generate 6-month roadmap from approved architecture
/planner-weekly [N]               — Generate weekly task plan with acceptance criteria
/review [artifact]                — HOSTILE_REVIEWER quality gate (APPROVE or REJECT)
/rust-bench [function]            — Create criterion benchmark for function
/rust-implement [task-id]         — Implement Rust code for approved task
/rust-refactor [scope]            — Refactor code (only if in approved plan)
/rust-test [module]               — Write comprehensive unit tests for module
/test-fuzz [module]               — Create fuzz targets for input handlers/parsers
/test-miri [scope]                — Run Miri verification on unsafe blocks
/test-prop [invariant]            — Create property-based tests for invariants
/test-regression [bug-id]         — Create regression test for reported bug
/wasm-bind [function]             — Create wasm-bindgen bindings for Rust function
/wasm-compat                      — Generate browser compatibility matrix with versions
/wasm-test [browser]              — Run tests in specific browser (chrome/firefox/safari/edge)
/wasm-types                       — Generate TypeScript type definitions (.d.ts)
```

---

*Invocation Reference Version: 1.0.0*
*Project: EdgeVec*
*System: Claude Code CLI*
