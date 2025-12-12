# Cursor to Claude Code Mapping Table

**Version:** 1.0.0
**Date:** 2025-12-11
**Purpose:** Systematic conversion reference for EdgeVec agent system

---

## 1. CORE CONCEPT MAPPING

| Cursor Concept | Claude Code Equivalent | Notes |
|:---------------|:----------------------|:------|
| `.cursorrules` | `~/.claude/CLAUDE.md` + `.claude/CLAUDE.md` | Split: user-level vs project-level |
| `.cursor/commands/CMD_*.md` | `.claude/agents/*.md` + `.claude/commands/*.md` | Agent definition + invocation command |
| `@AGENT_NAME invoke` | `/command-name arguments` | Prefix change: `@` → `/` |
| Agent mandate sections | Frontmatter + markdown content | YAML frontmatter for metadata |
| No native permission system | `.claude/settings.json` permissions | New capability for enforcement |
| No native hook system | `.claude/settings.json` hooks | New capability for automation |

---

## 2. FILE STRUCTURE MAPPING

### Cursor Structure
```
edgevec/
├── .cursorrules                    # Global rules
└── .cursor/
    └── commands/
        ├── README.md               # Agent roster
        ├── CMD_PROMPT_MAKER.md
        ├── CMD_META_ARCHITECT.md
        ├── CMD_PLANNER.md
        ├── CMD_RUST_ENGINEER.md
        ├── CMD_TEST_ENGINEER.md
        ├── CMD_WASM_SPECIALIST.md
        ├── CMD_BENCHMARK_SCIENTIST.md
        ├── CMD_HOSTILE_REVIEWER.md
        └── CMD_DOCWRITER.md
```

### Claude Code Structure
```
edgevec/
├── .claude/
│   ├── CLAUDE.md                           # Project rules
│   ├── HOSTILE_GATE_CHECKLIST.md           # Quality gate criteria
│   ├── WORKFLOW_ROUTER.md                   # Dispatcher logic
│   ├── settings.json                        # Permissions, hooks, env
│   ├── agents/
│   │   ├── meta-architect.md
│   │   ├── planner.md
│   │   ├── rust-engineer.md
│   │   ├── test-engineer.md
│   │   ├── wasm-specialist.md
│   │   ├── benchmark-scientist.md
│   │   ├── hostile-reviewer.md
│   │   └── docwriter.md
│   ├── commands/
│   │   ├── dispatch.md                     # PROMPT_MAKER → dispatcher
│   │   ├── architect-design.md
│   │   ├── planner-roadmap.md
│   │   ├── planner-weekly.md
│   │   ├── rust-implement.md
│   │   ├── test-fuzz.md
│   │   ├── test-prop.md
│   │   ├── wasm-bind.md
│   │   ├── bench-baseline.md
│   │   ├── review.md
│   │   └── doc-readme.md
│   └── hooks/
│       └── pre-commit-review.sh            # Enforces hostile review
└── docs/
    ├── MIGRATION_GUIDE.md                  # Conversion guide
    └── INVOCATION_REFERENCE.md             # Quick reference

~/.claude/
└── CLAUDE.md                               # User-level global rules
```

---

## 3. AGENT-BY-AGENT MAPPING

### 3.1 PROMPT_MAKER (Dispatcher)

| Aspect | Cursor Implementation | Claude Code Implementation |
|:-------|:---------------------|:---------------------------|
| File | `.cursor/commands/CMD_PROMPT_MAKER.md` | `.claude/commands/dispatch.md` + `.claude/WORKFLOW_ROUTER.md` |
| Invocation | `@PROMPT_MAKER` | `/dispatch [task]` |
| Role | Generate prompts for other agents | Route user requests to correct agent/command |
| Implementation | Single command file with decision tree | Command file + separate router logic document |
| Output | Formatted prompt block | Direct invocation of target command |

**Conversion Strategy:**
- Extract decision tree logic → `WORKFLOW_ROUTER.md`
- Create simple dispatch command that references router
- Router becomes reference doc for users

---

### 3.2 META_ARCHITECT

| Aspect | Cursor | Claude Code |
|:-------|:-------|:------------|
| File | `CMD_META_ARCHITECT.md` | Agent: `agents/meta-architect.md`<br>Commands: `commands/architect-design.md` |
| Invocation | `@META_ARCHITECT design [component]` | `/architect-design [component]` |
| Mandate Preservation | ✅ All sections | ✅ All sections in agent file |
| Chain of Thought | ✅ 5-step protocol | ✅ Preserved in agent definition |
| Output Templates | ✅ ARCHITECTURE.md, DATA_LAYOUT.md | ✅ Same, referenced in agent |
| Anti-Hallucination Clamps | ✅ 3 clamps | ✅ Preserved |
| Hostile Gate Protocol | ✅ Self-review checklist | ✅ Preserved + hook enforcement |

**Tools Required:**
- Read (architecture docs)
- Write (create architecture files)
- Edit (update architecture)
- Grep (search existing patterns)

---

### 3.3 PLANNER

| Aspect | Cursor | Claude Code |
|:-------|:-------|:------------|
| File | `CMD_PLANNER.md` | Agent: `agents/planner.md`<br>Commands: `commands/planner-roadmap.md`, `commands/planner-weekly.md` |
| Invocation | `@PLANNER roadmap`<br>`@PLANNER weekly [N]` | `/planner-roadmap`<br>`/planner-weekly [N]` |
| Gate Dependency | Must check GATE_1_COMPLETE.md | Hook enforces architecture approval |
| Unlocks Coding | WEEKLY_TASK_PLAN.md approval | Hook enforces plan approval before Write(src/**) |
| Estimation Rules | 3x rule, <16hr tasks | ✅ Preserved |

**Tools Required:**
- Read (architecture, previous plans)
- Write (roadmap, weekly plans)
- Edit (update plans)

**Hook Requirement:**
```json
{
  "hooks": {
    "PreToolUse": {
      "Write(src/**)": [{
        "matcher": "**",
        "hooks": [{
          "type": "command",
          "command": "test -f docs/planning/weeks/*/WEEKLY_TASK_PLAN.md && grep -q APPROVED docs/planning/weeks/*/WEEKLY_TASK_PLAN.md || (echo 'GATE 2 NOT PASSED: No approved plan' && exit 1)"
        }]
      }]
    }
  }
}
```

---

### 3.4 RUST_ENGINEER

| Aspect | Cursor | Claude Code |
|:-------|:-------|:------------|
| File | `CMD_RUST_ENGINEER.md` | Agent: `agents/rust-engineer.md`<br>Commands: `commands/rust-implement.md` |
| Invocation | `@RUST_ENGINEER implement W[N].[X]` | `/rust-implement W[N].[X]` |
| Pre-Coding Checklist | Manual verification | Hook enforcement |
| TDD Workflow | Self-enforced | Hook: cargo fmt after Edit(*.rs) |
| Forbidden Actions | 6 rules (no panics, unwrap, etc.) | Enforced via HOSTILE_REVIEWER gate |

**Tools Required:**
- Read (plan, architecture, existing code)
- Write (new Rust files)
- Edit (modify Rust files)
- Bash (cargo test, cargo clippy, cargo fmt)

**Hook Requirement:**
```json
{
  "hooks": {
    "PostToolUse": {
      "Edit": [{
        "matcher": "**/*.rs",
        "hooks": [
          {"type": "command", "command": "cargo fmt"},
          {"type": "command", "command": "cargo clippy -- -D warnings || echo 'Clippy warnings detected'"}
        ]
      }]
    }
  }
}
```

---

### 3.5 TEST_ENGINEER

| Aspect | Cursor | Claude Code |
|:-------|:-------|:------------|
| File | `CMD_TEST_ENGINEER.md` | Agent: `agents/test-engineer.md`<br>Commands: `commands/test-fuzz.md`, `commands/test-prop.md` |
| Invocation | `@TEST_ENGINEER fuzz [module]`<br>`@TEST_ENGINEER prop [invariant]` | `/test-fuzz [module]`<br>`/test-prop [invariant]` |
| Role Separation | QA, fuzzing, property tests | ✅ Same |
| Invariant Extraction | 4-step protocol | ✅ Preserved |

**Tools Required:**
- Read (architecture for invariants, code to test)
- Write (test files, fuzz targets)
- Bash (cargo test, cargo fuzz)

---

### 3.6 WASM_SPECIALIST

| Aspect | Cursor | Claude Code |
|:-------|:-------|:------------|
| File | `CMD_WASM_SPECIALIST.md` | Agent: `agents/wasm-specialist.md`<br>Commands: `commands/wasm-bind.md` |
| Invocation | `@WASM_SPECIALIST bind [function]` | `/wasm-bind [function]` |
| Browser Gotchas | 3 documented gotchas | ✅ Preserved in agent definition |
| Build Verification | Manual wasm-pack builds | Hook: PostToolUse for wasm files |

**Tools Required:**
- Read (Rust code, WASM_BOUNDARY.md)
- Write (WASM bindings, TypeScript types)
- Edit (update bindings)
- Bash (wasm-pack build, wasm-pack test)

**Hook Requirement:**
```json
{
  "hooks": {
    "PostToolUse": {
      "Write": [{
        "matcher": "**/*_wasm.rs",
        "hooks": [
          {"type": "command", "command": "wasm-pack build --target web || echo 'WASM build failed'"}
        ]
      }]
    }
  }
}
```

---

### 3.7 BENCHMARK_SCIENTIST

| Aspect | Cursor | Claude Code |
|:-------|:-------|:------------|
| File | `CMD_BENCHMARK_SCIENTIST.md` | Agent: `agents/benchmark-scientist.md`<br>Commands: `commands/bench-baseline.md`, `commands/bench-compare.md` |
| Invocation | `@BENCHMARK_SCIENTIST baseline [component]` | `/bench-baseline [component]`<br>`/bench-compare [competitor]` |
| Role Separation | Performance only (not correctness) | ✅ Preserved |
| Report Format | Comprehensive template | ✅ Preserved |
| Anti-Hallucination Clamps | 3 clamps (hardware spec, no cherry-picking, fair comparisons) | ✅ Preserved |

**Tools Required:**
- Read (code, architecture, previous benchmarks)
- Write (benchmark code, reports)
- Bash (cargo bench, memory profiling)

---

### 3.8 HOSTILE_REVIEWER

| Aspect | Cursor | Claude Code |
|:-------|:-------|:------------|
| File | `CMD_HOSTILE_REVIEWER.md` | Agent: `agents/hostile-reviewer.md`<br>Command: `commands/review.md`<br>Checklist: `HOSTILE_GATE_CHECKLIST.md` |
| Invocation | `@HOSTILE_REVIEWER review [artifact]` | `/review [artifact]` |
| Kill Authority | YES - Ultimate veto | ✅ Enforced via hooks + permission system |
| Attack Vectors | 4 types (architecture, plans, code, benchmarks) | ✅ All preserved in agent definition |
| Verdict Format | Binary: APPROVE/REJECT | ✅ Same |
| Enforcement | Manual adherence | **NEW:** Hooks prevent bypass |

**Special Implementation:**

1. **Quality Gate Checklist** (`.claude/HOSTILE_GATE_CHECKLIST.md`):
   - Extracted from hostile review standards
   - Referenced by hooks
   - Used for all gate checks

2. **Hooks for Enforcement:**
```json
{
  "hooks": {
    "PreToolUse": {
      "Write(src/**)": [{
        "matcher": "**",
        "hooks": [{
          "type": "command",
          "command": "test -f .claude/GATE_3_COMPLETE.md || (echo 'HOSTILE_REVIEWER must approve before coding' && exit 1)"
        }]
      }]
    }
  }
}
```

3. **No Bypass:**
   - Permission system denies dangerous operations
   - Hooks check for approval files
   - Only [HUMAN_OVERRIDE] can bypass

**Tools Required:**
- Read (all artifacts for review)
- Write (review documents - approvals/rejections)
- Grep (search for violations)

---

### 3.9 DOCWRITER

| Aspect | Cursor | Claude Code |
|:-------|:-------|:------------|
| File | `CMD_DOCWRITER.md` | Agent: `agents/docwriter.md`<br>Commands: `commands/doc-readme.md`, `commands/doc-api.md` |
| Invocation | `@DOCWRITER readme` | `/doc-readme`<br>`/doc-api [module]` |
| README Template | Viral hook format | ✅ Preserved |
| Anti-Hallucination Clamps | 3 clamps (tested examples, verified claims, current info) | ✅ Preserved |

**Tools Required:**
- Read (code, benchmarks, architecture)
- Write (README, docs)
- Edit (update docs)

---

## 4. GLOBAL RULES DECOMPOSITION

### 4.1 User-Level Rules (`~/.claude/CLAUDE.md`)

**Content:** General development philosophy applicable to ALL projects

- Supreme Mandate (Design > Code)
- Forbidden Actions (universal)
- Quality Standards (general coding standards)
- Communication Protocol (agent handoffs)
- Emergency Procedures

**Why User-Level:**
- Not EdgeVec-specific
- Apply to any project with this workflow
- User's personal development philosophy

---

### 4.2 Project-Level Rules (`.claude/CLAUDE.md`)

**Content:** EdgeVec-specific context and configuration

- The Agent Roster (10 agents for EdgeVec)
- The Workflow (Genesis Sequence for EdgeVec)
- Directory Structure (EdgeVec layout)
- Technical Constraints (Rust/WASM specific)
- Salvaged Code Policy (binary_semantic_cache reference)
- Hostile Gate Protocol (EdgeVec gates 1-4)

**Why Project-Level:**
- EdgeVec-specific agents
- EdgeVec architecture constraints
- Project-specific workflow

---

## 5. INVOCATION PATTERN MAPPING

| Cursor Pattern | Claude Code Pattern | Example |
|:---------------|:--------------------|:--------|
| `@META_ARCHITECT design gap_analysis` | `/architect-design gap_analysis` | Design new component |
| `@PLANNER roadmap` | `/planner-roadmap` | Create 6-month plan |
| `@PLANNER weekly 1` | `/planner-weekly 1` | Create Week 1 plan |
| `@RUST_ENGINEER implement W1.1` | `/rust-implement W1.1` | Implement task W1.1 |
| `@TEST_ENGINEER fuzz parser` | `/test-fuzz parser` | Create fuzz target |
| `@TEST_ENGINEER prop hnsw` | `/test-prop hnsw` | Create property tests |
| `@WASM_SPECIALIST bind search` | `/wasm-bind search` | Create WASM binding |
| `@BENCHMARK_SCIENTIST baseline hnsw` | `/bench-baseline hnsw` | Establish baseline |
| `@HOSTILE_REVIEWER review ARCHITECTURE.md` | `/review ARCHITECTURE.md` | Review artifact |
| `@DOCWRITER readme` | `/doc-readme` | Generate README |
| `@PROMPT_MAKER` | `/dispatch [task description]` | Route task to agent |

---

## 6. PERMISSION SYSTEM MAPPING

**Cursor:** No native permission system (rely on agent discipline)

**Claude Code:** Explicit permission rules in `.claude/settings.json`

### Required Permissions

```json
{
  "permissions": {
    "allow": [
      "Read(**/*)",
      "Write(tests/**/*)",
      "Write(benches/**/*)",
      "Write(docs/**/*)",
      "Write(examples/**/*)",
      "Edit(tests/**/*)",
      "Edit(benches/**/*)",
      "Edit(docs/**/*)",
      "Bash(cargo test)",
      "Bash(cargo clippy)",
      "Bash(cargo fmt)",
      "Bash(cargo bench)",
      "Bash(cargo fuzz *)",
      "Bash(wasm-pack *)",
      "Grep(*)",
      "Glob(*)"
    ],
    "deny": [
      "Bash(rm -rf *)",
      "Bash(cargo publish)",
      "Write(Cargo.toml)",
      "Edit(.claude/*)"
    ],
    "conditional": {
      "Write(src/**)": {
        "requires": "file_exists(.claude/GATE_3_COMPLETE.md)"
      }
    }
  }
}
```

**Notes:**
- Read is always allowed (agents need context)
- Write to `src/` is conditional on gate passage
- Dangerous operations are denied
- Test/bench/doc writes are allowed (pre-implementation artifacts)

---

## 7. HOOK SYSTEM MAPPING

**Cursor:** No native hook system

**Claude Code:** Hooks in `.claude/settings.json`

### Hook Categories

#### 7.1 Quality Hooks (Post-Edit)
```json
{
  "PostToolUse": {
    "Edit": [{
      "matcher": "**/*.rs",
      "hooks": [
        {"type": "command", "command": "cargo fmt"},
        {"type": "command", "command": "cargo clippy -- -D warnings || true"}
      ]
    }]
  }
}
```

#### 7.2 Gate Enforcement Hooks (Pre-Write)
```json
{
  "PreToolUse": {
    "Write(src/**)": [{
      "matcher": "**",
      "hooks": [{
        "type": "command",
        "command": "test -f .claude/GATE_3_COMPLETE.md || (echo 'GATE 3 NOT PASSED' && exit 1)"
      }]
    }]
  }
}
```

#### 7.3 Build Verification Hooks
```json
{
  "PostToolUse": {
    "Write(**/*_wasm.rs)": [{
      "matcher": "**",
      "hooks": [
        {"type": "command", "command": "wasm-pack build --target web || echo 'WASM build failed'"}
      ]
    }]
  }
}
```

---

## 8. PHASE GATE MAPPING

| Gate | Cursor Implementation | Claude Code Implementation |
|:-----|:----------------------|:---------------------------|
| **GATE 1:** Architecture → Planning | Manual check of GATE_1_COMPLETE.md | Hook checks file existence before planner commands |
| **GATE 2:** Planning → Implementation | Manual check of approved WEEKLY_TASK_PLAN.md | Hook blocks Write(src/**) without approved plan |
| **GATE 3:** Implementation → Merge | Manual HOSTILE_REVIEWER approval | Hook checks GATE_3_COMPLETE.md before critical ops |
| **GATE 4:** Documentation → Release | Manual HOSTILE_REVIEWER approval | Permission blocks Bash(cargo publish) |

### Gate File Pattern

Each gate completion creates a marker file:
- `.claude/GATE_1_COMPLETE.md` (architecture approved)
- `.claude/GATE_2_COMPLETE.md` (planning approved)
- `.claude/GATE_3_COMPLETE.md` (implementation approved)
- `.claude/GATE_4_COMPLETE.md` (documentation approved)

**HOSTILE_REVIEWER** creates these files upon approval.

---

## 9. CONTEXT REQUIREMENTS MAPPING

**Cursor Pattern:**
```markdown
## INPUT REQUIREMENTS

**Required Before Executing:**
- @file ARCHITECTURE.md
- @file DATA_LAYOUT.md
```

**Claude Code Pattern:**
```markdown
## INPUT REQUIREMENTS

**Required Context (use /add if needed):**
- docs/architecture/ARCHITECTURE.md
- docs/architecture/DATA_LAYOUT.md
- docs/planning/WEEKLY_TASK_PLAN.md

**Verification:**
Agent should use Read tool to verify these files exist before proceeding.
```

---

## 10. SLASH COMMAND NAMING CONVENTION

| Agent | Command Type | Naming Pattern | Example |
|:------|:-------------|:---------------|:--------|
| META_ARCHITECT | Design | `/architect-[action]` | `/architect-design` |
| PLANNER | Planning | `/planner-[type]` | `/planner-roadmap`, `/planner-weekly` |
| RUST_ENGINEER | Implementation | `/rust-[action]` | `/rust-implement` |
| TEST_ENGINEER | Testing | `/test-[type]` | `/test-fuzz`, `/test-prop` |
| WASM_SPECIALIST | WASM | `/wasm-[action]` | `/wasm-bind` |
| BENCHMARK_SCIENTIST | Benchmarking | `/bench-[type]` | `/bench-baseline`, `/bench-compare` |
| HOSTILE_REVIEWER | Review | `/review` | `/review` |
| DOCWRITER | Documentation | `/doc-[type]` | `/doc-readme`, `/doc-api` |
| PROMPT_MAKER | Dispatcher | `/dispatch` | `/dispatch` |

**Rationale:**
- Prefix groups related commands
- Action/type suffix provides clarity
- No ambiguity in invocation

---

## 11. ANTI-HALLUCINATION CLAMP PRESERVATION

All agents have anti-hallucination clamps that MUST be preserved:

| Agent | Clamps | Preservation Strategy |
|:------|:-------|:----------------------|
| META_ARCHITECT | 3 clamps (no magic numbers, no unverified claims, no optimistic sizing) | Copy verbatim to agent definition |
| PLANNER | 3 clamps (no invented deps, no vague acceptance, no optimistic timelines) | Copy verbatim to agent definition |
| RUST_ENGINEER | 3 clamps (no code without plan, no unverified perf claims, no unsafe without justification) | Copy verbatim to agent definition |
| WASM_SPECIALIST | 3 clamps (no assumed browser support, no untested WASM builds, no optimistic memory) | Copy verbatim to agent definition |
| BENCHMARK_SCIENTIST | 3 clamps (no benchmark without hardware spec, no cherry-picked results, no unfair comparisons) | Copy verbatim to agent definition |
| HOSTILE_REVIEWER | 3 clamps (evidence required, no subjective criteria, no improvement suggestions) | Copy verbatim to agent definition |
| DOCWRITER | 3 clamps (no untested examples, no unverified claims, no outdated info) | Copy verbatim to agent definition |

**CRITICAL:** These clamps are **intentional strictness**, not verbosity. They prevent AI hallucination and enforce rigor.

---

## 12. CONVERSION CHECKLIST

### Per-Agent Conversion
For EACH of the 10 agents:

- [ ] Create `.claude/agents/[agent-name].md`
  - [ ] Add YAML frontmatter (name, description, tools)
  - [ ] Copy Mandate section
  - [ ] Copy Principles
  - [ ] Copy Chain of Thought Protocol
  - [ ] Copy Output Formats
  - [ ] Copy Anti-Hallucination Clamps
  - [ ] Copy Hostile Gate Protocol
  - [ ] Copy Forbidden Actions
  - [ ] Copy Handoff template

- [ ] Create `.claude/commands/[command-name].md` for each invocation pattern
  - [ ] Reference the agent file
  - [ ] Include pre-execution checklist
  - [ ] Include context requirements
  - [ ] Include expected arguments

- [ ] Add tool permissions to `.claude/settings.json`
  - [ ] Read permissions
  - [ ] Write permissions (conditional if needed)
  - [ ] Bash command permissions
  - [ ] Deny dangerous operations

- [ ] Add hooks to `.claude/settings.json` (if applicable)
  - [ ] PostToolUse hooks (e.g., cargo fmt after Edit)
  - [ ] PreToolUse hooks (e.g., gate checks before Write)

---

## 13. SUCCESS CRITERIA

The conversion is successful when:

1. **Functional Equivalence:**
   - [ ] All 10 agents are represented
   - [ ] All invocation patterns mapped to slash commands
   - [ ] All workflows still possible

2. **Rigor Preservation:**
   - [ ] All anti-hallucination clamps preserved
   - [ ] All quality gates preserved
   - [ ] HOSTILE_REVIEWER still has veto power

3. **Enhancement:**
   - [ ] Hooks enforce gates automatically (better than Cursor)
   - [ ] Permission system prevents violations (better than Cursor)
   - [ ] Clear separation of user vs project rules

4. **Usability:**
   - [ ] MIGRATION_GUIDE.md explains old→new
   - [ ] INVOCATION_REFERENCE.md provides quick lookup
   - [ ] Users can follow same workflow with new syntax

---

**End of Mapping Table**
