# MIGRATION GUIDE: Cursor to Claude Code

**Version:** 1.0.0
**Audience:** EdgeVec developers transitioning from Cursor IDE to Claude Code CLI
**Purpose:** Complete reference for converting workflows and invocations

---

## EXECUTIVE SUMMARY

EdgeVec's agent command system has been converted from Cursor IDE's `@-mention` system to Claude Code's `/slash` command system. **All core principles remain unchanged**: the Military Strict Mode philosophy, the Supreme Rule (Architecture > Plan > Code), and the HOSTILE_REVIEWER gate system are fully preserved.

**What's Different:**
- Invocation syntax: `@AGENT` → `/command`
- File structure: `.cursor/commands/` → `.claude/agents/` + `.claude/commands/`
- New capabilities: Hooks, permissions, native workflow routing

**What's The Same:**
- All 10 agents with identical mandates
- All quality gates and review criteria
- All chain-of-thought protocols
- All output templates and standards

---

## QUICK REFERENCE: OLD VS NEW

### Agent Invocation Changes

| Cursor (Old) | Claude Code (New) | Notes |
|:-------------|:------------------|:------|
| `@META_ARCHITECT design gap_analysis` | `/architect-design gap_analysis` | Subagent + command |
| `@PLANNER roadmap` | `/planner-roadmap` | Subagent + command |
| `@PLANNER weekly 1` | `/planner-weekly 1` | Subagent + command |
| `@RUST_ENGINEER implement W1.1` | `/rust-implement W1.1` | Subagent + command |
| `@TEST_ENGINEER fuzz parser` | `/test-fuzz parser` | Subagent + command |
| `@TEST_ENGINEER prop W1.1` | `/test-prop W1.1` | Subagent + command |
| `@WASM_SPECIALIST bind search` | `/wasm-bind search` | Subagent + command |
| `@BENCHMARK_SCIENTIST baseline hnsw` | `/bench-baseline hnsw` | Subagent + command |
| `@BENCHMARK_SCIENTIST compare sqlite-vec` | `/bench-compare sqlite-vec` | Subagent + command |
| `@HOSTILE_REVIEWER review ARCHITECTURE.md` | `/review ARCHITECTURE.md` | Subagent + command |
| `@DOCWRITER readme` | `/doc-readme` | Subagent + command |
| `@DOCWRITER api hnsw` | `/doc-api hnsw` | Subagent + command |
| `@PROMPT_MAKER` (implicit dispatcher) | `/dispatch` | Explicit command |

### File Structure Changes

| Cursor (Old) | Claude Code (New) | Purpose |
|:-------------|:------------------|:--------|
| `.cursorrules` | `.claude/CLAUDE.md` (project) + `~/.claude/CLAUDE.md` (user) | Global rules |
| `.cursor/commands/CMD_*.md` | `.claude/agents/*.md` + `.claude/commands/*.md` | Agent definitions + commands |
| `.cursor/commands/README.md` | `docs/INVOCATION_REFERENCE.md` | Command quick reference |
| N/A | `.claude/settings.json` | Permissions, hooks, environment |
| N/A | `.claude/HOSTILE_GATE_CHECKLIST.md` | Quality criteria reference |
| N/A | `.claude/WORKFLOW_ROUTER.md` | Dispatcher decision tree |

---

## DETAILED MIGRATION WALKTHROUGH

### 1. Understanding the New Structure

Claude Code uses a multi-file system instead of Cursor's monolithic command files:

#### Old Structure (Cursor)
```
.cursor/
├── commands/
│   ├── CMD_META_ARCHITECT.md       # All-in-one: agent + invocation
│   ├── CMD_PLANNER.md
│   ├── CMD_RUST_ENGINEER.md
│   └── ...
└── .cursorrules                     # Global rules
```

#### New Structure (Claude Code)
```
.claude/
├── CLAUDE.md                        # Project-level rules
├── settings.json                    # Permissions, hooks, env vars
├── HOSTILE_GATE_CHECKLIST.md       # Quality gate criteria
├── WORKFLOW_ROUTER.md               # Dispatcher logic
├── agents/
│   ├── meta-architect.md            # Agent definition (principles, mandate)
│   ├── planner.md
│   ├── rust-engineer.md
│   └── ...
├── commands/
│   ├── dispatch.md                  # Workflow router command
│   ├── architect-design.md          # Specific invocation
│   ├── planner-roadmap.md
│   ├── planner-weekly.md
│   ├── rust-implement.md
│   └── ...
└── hooks/
    └── pre-commit-review.sh         # Automated quality checks

~/.claude/
└── CLAUDE.md                        # User-level global rules
```

**Why the split?**
- **Agents** define WHO the agent is (principles, mandate, authority)
- **Commands** define WHAT the agent does (specific invocations, workflows)
- **Settings** define HOW the system enforces rules (permissions, hooks)

---

### 2. Agent Definitions (`.claude/agents/`)

Each agent from Cursor's `CMD_*.md` is now split into:

#### Example: RUST_ENGINEER

**Old (Cursor):** `.cursor/commands/CMD_RUST_ENGINEER.md`
- All-in-one file: mandate, principles, protocols, templates, execution triggers

**New (Claude Code):**
1. **Agent Definition:** `.claude/agents/rust-engineer.md`
   - Mandate, principles, role boundaries
   - Tool permissions (Read, Write, Edit, Bash)
   - Chain-of-thought protocols

2. **Commands:**
   - `.claude/commands/rust-implement.md` — Implementation workflow
   - `.claude/commands/rust-test.md` — Test writing workflow
   - `.claude/commands/rust-bench.md` — Benchmark creation workflow

3. **Permissions:** In `.claude/settings.json`
   ```json
   {
     "permissions": {
       "allow": [
         "Bash(cargo test)",
         "Bash(cargo clippy)",
         "Bash(cargo fmt)",
         "Write(src/**/*.rs)",
         "Write(tests/**/*.rs)"
       ],
       "deny": [
         "Bash(rm *)",
         "Bash(git push --force)"
       ]
     }
   }
   ```

**Migration Action:**
- Read `.claude/agents/rust-engineer.md` to understand role (same as before)
- Use `/rust-implement W[N].[X]` to invoke (instead of `@RUST_ENGINEER implement W[N].[X]`)

---

### 3. Slash Commands (`.claude/commands/`)

Commands in Claude Code are **explicit workflows** that reference agents.

#### Command Structure

**Template:**
```markdown
# Command: [command-name]

**Purpose:** [One-line description]
**Agent:** [Which agent executes this]

---

## EXECUTION

[Step-by-step protocol for this specific workflow]

### Step 1: [Action]
[Instructions]

### Step 2: [Action]
[Instructions]

---

## REQUIRED CONTEXT

Before executing, ensure these files are loaded:
- [ ] [file1.md]
- [ ] [file2.md]

---

## EXPECTED OUTPUT

[What artifacts will be produced]

---

## NEXT STEP

After completion:
```
/[next-command] [args]
```
```

#### Example Mapping

**Old (Cursor):** `@PLANNER weekly 1`

**New (Claude Code):** `/planner-weekly 1`

**Behind the scenes:**
1. User types `/planner-weekly 1`
2. Claude Code loads `.claude/commands/planner-weekly.md`
3. Command file invokes the `planner` agent (defined in `.claude/agents/planner.md`)
4. Command provides the "weekly planning" workflow instructions
5. Agent executes with access to specified tools (defined in settings.json)

**Migration Action:**
- Replace `@PLANNER weekly 1` with `/planner-weekly 1`
- Same workflow, new syntax

---

### 4. The Dispatcher (`/dispatch`)

The PROMPT_MAKER agent is now a **command** called `/dispatch`.

#### Old (Cursor)
PROMPT_MAKER was implicitly invoked when you weren't sure which agent to use. It analyzed your request and generated the correct `@AGENT` invocation.

#### New (Claude Code)
`/dispatch` is explicitly invoked to analyze your request.

**Usage:**
```
User: "I want to implement the HNSW search algorithm"
User: /dispatch

Output:
## WORKFLOW_ROUTER: Request Analysis
[Detailed analysis with phase detection, intent classification, recommended command]

Recommended Command: /rust-implement W1.1
```

**When to use:**
- Unsure which command to use
- Complex request spanning multiple agents
- Need phase validation before proceeding

**Migration Action:**
- Use `/dispatch` whenever you're unsure
- Replaces the "guess which agent" workflow from Cursor

---

### 5. HOSTILE_REVIEWER Gate System

The HOSTILE_REVIEWER remains the ultimate quality gate, but now has additional enforcement mechanisms.

#### Old (Cursor)
- Manual invocation: `@HOSTILE_REVIEWER review [artifact]`
- Enforcement: Social contract (developers honor rejections)

#### New (Claude Code)
- **Manual invocation:** `/review [artifact]` (same as before)
- **Automated enforcement:** Hooks and permissions prevent bypassing

**New Enforcement Mechanisms:**

1. **Pre-Commit Hook** (`.claude/hooks/pre-commit-review.sh`)
   - Blocks commits without HOSTILE_REVIEWER approval
   - Checks for approval markers in commit messages

2. **Permission System** (`.claude/settings.json`)
   ```json
   {
     "hooks": {
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
   }
   ```
   - Prevents writing code files until GATE_3_COMPLETE.md exists
   - Enforces phase sequence programmatically

3. **Quality Gate Checklist** (`.claude/HOSTILE_GATE_CHECKLIST.md`)
   - Comprehensive checklist organized by artifact type
   - Critical/Major/Minor severity levels
   - Used as reference during `/review` command

**Migration Action:**
- Use `/review [artifact]` (instead of `@HOSTILE_REVIEWER review [artifact]`)
- System now enforces gates automatically (can't bypass accidentally)
- Review `.claude/HOSTILE_GATE_CHECKLIST.md` for complete criteria

---

### 6. Permissions System (New Feature)

Claude Code introduces a **permission system** that Cursor didn't have.

#### What It Does
Controls which tools each agent can use and when.

#### Example Configuration

**In `.claude/settings.json`:**
```json
{
  "permissions": {
    "allow": [
      "Bash(cargo test)",
      "Bash(cargo clippy)",
      "Bash(cargo bench)",
      "Bash(cargo fmt)",
      "Bash(wasm-pack build)",
      "Bash(git status)",
      "Bash(git diff)",
      "Bash(git log)",
      "Read(**/*)",
      "Write(src/**/*.rs)",
      "Write(tests/**/*.rs)",
      "Write(benches/**/*.rs)",
      "Write(docs/**/*.md)",
      "Edit(src/**/*.rs)",
      "Edit(tests/**/*.rs)"
    ],
    "deny": [
      "Bash(rm -rf *)",
      "Bash(git push --force)",
      "Bash(cargo publish)",
      "Write(Cargo.toml)",
      "Write(.claude/**/*)"
    ]
  }
}
```

**What This Means:**
- Agents can't accidentally `rm -rf` your codebase
- Agents can't force-push to git
- Agents can't modify the command system itself
- Agents CAN run tests, format code, build WASM

**Migration Action:**
- Review `.claude/settings.json` to understand boundaries
- Adjust permissions if your workflow needs different access

---

### 7. Hooks System (New Feature)

Claude Code supports **hooks** that automatically run commands before/after tool use.

#### Use Cases in EdgeVec

**1. Auto-format Rust code after editing:**
```json
{
  "hooks": {
    "PostToolUse": {
      "Edit": [{
        "matcher": "**/*.rs",
        "hooks": [{
          "type": "command",
          "command": "cargo fmt"
        }]
      }]
    }
  }
}
```

**2. Enforce architecture approval before coding:**
```json
{
  "hooks": {
    "PreToolUse": {
      "Write(src/**)": [{
        "matcher": "**",
        "hooks": [{
          "type": "command",
          "command": "test -f .claude/GATE_3_COMPLETE.md || (echo 'ERROR: Code cannot be written until GATE_3 is passed' && exit 1)"
        }]
      }]
    }
  }
}
```

**3. Run tests after implementation:**
```json
{
  "hooks": {
    "PostToolUse": {
      "Write": [{
        "matcher": "src/**/*.rs",
        "hooks": [{
          "type": "command",
          "command": "cargo test"
        }]
      }]
    }
  }
}
```

**Migration Action:**
- Hooks automate what was manual in Cursor
- Review `.claude/settings.json` for active hooks
- These enforce quality gates automatically

---

## WORKFLOW EXAMPLES

### Example 1: Starting a New Feature (Full Pipeline)

#### Old Workflow (Cursor)
```
1. @META_ARCHITECT design feature_name
2. [Review output manually]
3. @HOSTILE_REVIEWER review ARCHITECTURE.md
4. [If approved, continue]
5. @PLANNER roadmap
6. @HOSTILE_REVIEWER review ROADMAP.md
7. @PLANNER weekly 1
8. @HOSTILE_REVIEWER review WEEKLY_TASK_PLAN.md
9. @RUST_ENGINEER implement W1.1
10. @TEST_ENGINEER prop W1.1
11. @HOSTILE_REVIEWER review [all outputs]
```

#### New Workflow (Claude Code)
```
1. /dispatch
   [Analyzes request, recommends /architect-design feature_name]

2. /architect-design feature_name
   [Generates ARCHITECTURE.md, DATA_LAYOUT.md, etc.]

3. /review ARCHITECTURE.md
   [HOSTILE_REVIEWER validates; produces GATE_1_COMPLETE.md if approved]

4. /planner-roadmap
   [Blocked by hook if GATE_1 not complete; succeeds if approved]

5. /review ROADMAP.md
   [Produces GATE_2_COMPLETE.md if approved]

6. /planner-weekly 1
   [Generates WEEKLY_TASK_PLAN.md]

7. /review WEEKLY_TASK_PLAN.md
   [Produces GATE_3_COMPLETE.md if approved]

8. /rust-implement W1.1
   [Blocked by permission system if GATE_3 not complete; auto-formats after edit]

9. /test-prop W1.1
   [TEST_ENGINEER creates property tests]

10. /review [all W1.1 outputs]
    [Final approval gate]
```

**Key Differences:**
- `/dispatch` explicitly recommends next step (was implicit in Cursor)
- Hooks automatically enforce gate sequence (was manual in Cursor)
- Auto-formatting after edits (was manual in Cursor)

---

### Example 2: Quick Documentation Fix

#### Old Workflow (Cursor)
```
1. @DOCWRITER readme
2. [Edit README.md]
3. @HOSTILE_REVIEWER review README.md (optional for trivial changes)
```

#### New Workflow (Claude Code)
```
1. /doc-readme
   [Updates README.md; auto-runs linter via hook]

2. /review README.md (optional for trivial changes)
```

**Key Differences:**
- Shorter command syntax
- Hooks ensure quality (linting, link checking)

---

### Example 3: Benchmark Comparison

#### Old Workflow (Cursor)
```
1. @BENCHMARK_SCIENTIST baseline hnsw
2. [Wait for completion]
3. @BENCHMARK_SCIENTIST compare sqlite-vec
4. [Review output]
5. @HOSTILE_REVIEWER review benchmark_report.md
```

#### New Workflow (Claude Code)
```
1. /bench-baseline hnsw
   [Establishes baseline; saves to baselines/]

2. /bench-compare sqlite-vec
   [Runs comparison; auto-checks for fair comparison criteria via hook]

3. /review benchmark_report.md
```

**Key Differences:**
- Same workflow, different syntax
- Hooks validate comparison fairness automatically

---

## COMMON MIGRATION PITFALLS

### Pitfall 1: Forgetting to Use `/dispatch`

**Symptom:** You're not sure which command to use.

**Solution:** Use `/dispatch` to analyze your request and get a recommendation.

**Example:**
```
User: "I want to optimize the search algorithm"
User: /dispatch

Output: [Analysis showing you need to /bench-baseline first, then /rust-implement optimization-task-id]
```

---

### Pitfall 2: Trying to Bypass Gates

**Symptom:** Error message like "GATE 3 NOT PASSED — cannot write code"

**Cause:** Trying to run `/rust-implement` before plan is approved.

**Solution:** Follow the pipeline sequence:
1. `/architect-design` → `/review` → (Gate 1 passes)
2. `/planner-roadmap` → `/review` → (Gate 2 passes)
3. `/planner-weekly 1` → `/review` → (Gate 3 passes)
4. NOW you can `/rust-implement`

**Note:** This is now enforced by the permission system (was honor system in Cursor).

---

### Pitfall 3: Using Old Syntax

**Symptom:** Typing `@RUST_ENGINEER implement W1.1` out of habit.

**Solution:** Replace `@AGENT` with `/command`:
- `@RUST_ENGINEER implement W1.1` → `/rust-implement W1.1`
- `@PLANNER weekly 1` → `/planner-weekly 1`

**Tip:** Create shell aliases if you keep making this mistake:
```bash
alias @RUST_ENGINEER='/rust-implement'
alias @PLANNER='/planner-roadmap'  # or /planner-weekly
```

---

### Pitfall 4: Forgetting Context Files

**Symptom:** Agent says "I need ARCHITECTURE.md loaded"

**Cause:** Commands require specific files to be in context.

**Solution:** Each command lists required files. Load them first:
```
Before running /rust-implement W1.1, load:
- WEEKLY_TASK_PLAN.md
- ARCHITECTURE.md
- DATA_LAYOUT.md
```

**Tip:** Use the file browser or `/add` command to load files.

---

### Pitfall 5: Modifying `.claude/` Files Directly

**Symptom:** Your changes to `.claude/settings.json` aren't taking effect, or system behaves unexpectedly.

**Cause:** `.claude/` files are system configuration; changes require restart or reload.

**Solution:**
- Don't modify `.claude/` files unless you know what you're doing
- If you must, restart Claude Code to pick up changes
- Prefer using commands to modify behavior (e.g., `/dispatch` to route, not editing WORKFLOW_ROUTER.md)

---

## INSTALLING THE GIT PRE-COMMIT HOOK

EdgeVec includes a Git pre-commit hook that enforces quality gates at the VCS level. This provides an additional layer of protection beyond the runtime hooks in `settings.json`.

### What the Pre-Commit Hook Does

The hook (`.claude/hooks/pre-commit-review.sh`) automatically checks:

| Files Being Committed | Required Gate | Error if Missing |
|:----------------------|:-------------|:-----------------|
| `docs/architecture/**` | GATE 1 | Architecture must be reviewed before committing |
| `src/**/*.rs` | GATE 2 | Plan must be approved before committing code |
| `Cargo.toml` | GATE 2 | Dependency changes require approved plan |
| `tests/`, `benches/`, `fuzz/` | GATE 2 | Tests require approved plan |
| `README.md`, `CHANGELOG.md` | GATE 3 | Release docs require implementation approval |

**Key Feature:** The hook blocks commits that would bypass the Military-Grade Development Protocol.

### Installation Instructions

#### Step 1: Verify Hook Exists

```bash
# Check that the hook script exists
ls .claude/hooks/pre-commit-review.sh

# Expected output:
# .claude/hooks/pre-commit-review.sh
```

#### Step 2: Make Hook Executable

```bash
# On Unix-like systems (Linux, macOS, WSL)
chmod +x .claude/hooks/pre-commit-review.sh

# Verify it's executable
ls -l .claude/hooks/pre-commit-review.sh
# Expected: -rwxr-xr-x (note the 'x' flags)
```

#### Step 3: Create Symlink to Git Hooks Directory

```bash
# Create symlink from .git/hooks/pre-commit to .claude/hooks/pre-commit-review.sh
ln -sf ../../.claude/hooks/pre-commit-review.sh .git/hooks/pre-commit

# Verify symlink
ls -l .git/hooks/pre-commit
# Expected: .git/hooks/pre-commit -> ../../.claude/hooks/pre-commit-review.sh
```

**Why symlink?** This keeps the authoritative hook in `.claude/hooks/` (version controlled) while activating it in `.git/hooks/` (not version controlled).

#### Step 4: Test the Hook

**Test blocking behavior (should fail):**
```bash
# Try to commit code without GATE_2_COMPLETE.md
echo "// test" >> src/lib.rs
git add src/lib.rs
git commit -m "test commit"

# Expected output:
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#                     COMMIT BLOCKED BY HOSTILE GATE
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#
# ┌─────────────────────────────────────────────────────────────────────┐
# │ GATE 2 NOT PASSED                                                   │
# ├─────────────────────────────────────────────────────────────────────┤
# │ Files being committed:                                              │
# │   src/lib.rs                                                        │
# │                                                                     │
# │ Required:                                                           │
# │   .claude/GATE_2_COMPLETE.md must exist                             │
# │                                                                     │
# │ Remediation:                                                        │
# │   Run /planner-weekly [N] and /review WEEKLY_TASK_PLAN.md          │
# └─────────────────────────────────────────────────────────────────────┘
```

**Test passing behavior (should succeed):**
```bash
# Commit non-code files (allowed without gates)
echo "# Test" >> docs/notes.md
git add docs/notes.md
git commit -m "Add notes"

# Expected output:
# [EdgeVec Pre-Commit] ✅ All gates passed. Commit allowed.
# [branch] abc1234 Add notes
```

### Windows Installation (Alternative)

If you're on Windows without WSL/Git Bash, the Bash script may not work. Use this PowerShell alternative:

#### Option 1: Install WSL or Git Bash (Recommended)
```powershell
# Install Git Bash (if not already installed)
# Git for Windows includes Git Bash
# Download from: https://git-scm.com/download/win

# Then follow Unix instructions above in Git Bash
```

#### Option 2: PowerShell Hook (Advanced)
Create `.git/hooks/pre-commit.ps1` with equivalent logic, then configure Git:
```powershell
git config core.hooksPath .git/hooks
```

**Note:** The Bash script is recommended as it's more portable and tested.

### Bypassing the Hook (Emergency Only)

If you **MUST** bypass the hook (not recommended):

```bash
git commit --no-verify -m "[HUMAN_OVERRIDE] reason for bypass"
```

**When to use:**
- Emergency hotfix (document in commit message)
- Prototyping (will throw away code)
- Testing (not production code)

**NEVER bypass for production code.** The gates exist to protect quality.

### Uninstalling the Hook

If you need to disable the hook:

```bash
# Remove symlink
rm .git/hooks/pre-commit

# Or rename to deactivate
mv .git/hooks/pre-commit .git/hooks/pre-commit.disabled
```

**The hook script itself (`.claude/hooks/pre-commit-review.sh`) remains in the repo for version control.**

### Troubleshooting

**Problem:** Hook doesn't run / always succeeds
```bash
# Check symlink exists
ls -l .git/hooks/pre-commit

# Check hook is executable
ls -l .claude/hooks/pre-commit-review.sh

# Manually test hook
./.claude/hooks/pre-commit-review.sh
```

**Problem:** `command not found: test`
- **Cause:** Running on Windows without Bash
- **Solution:** Install Git Bash or WSL

**Problem:** Hook blocks valid commits
- **Cause:** GATE_*_COMPLETE.md files missing despite approval
- **Solution:** Ensure HOSTILE_REVIEWER creates gate files:
  ```bash
  # After /review approval, verify:
  ls .claude/GATE_*.md

  # If missing, rerun /review with explicit approval
  ```

---

## BACKWARDS COMPATIBILITY

### Can I Still Use Cursor?

**Yes**, but not simultaneously on the same project.

**Reason:** The file structures are different:
- Cursor uses `.cursor/commands/CMD_*.md`
- Claude Code uses `.claude/agents/*.md` + `.claude/commands/*.md`

**If you need to switch back to Cursor:**
1. The original `.cursor/` directory still exists (unchanged)
2. Delete or rename `.claude/` to avoid confusion
3. Use `@AGENT` syntax again

**If you want both:**
- Use Cursor for one branch
- Use Claude Code for another branch
- Don't merge `.claude/` and `.cursor/` changes together

---

## ADVANCED FEATURES (New in Claude Code)

### 1. Environment Variables

You can inject environment variables via `.claude/settings.json`:

```json
{
  "env": {
    "RUST_BACKTRACE": "1",
    "CARGO_TERM_COLOR": "always"
  }
}
```

**Use Case:** Ensure all agents run cargo commands with consistent settings.

---

### 2. Custom Hooks for Project-Specific Workflows

**Example:** Auto-deploy to staging after HOSTILE_REVIEWER approval.

```json
{
  "hooks": {
    "PostCommand": {
      "/review": [{
        "matcher": "**",
        "hooks": [{
          "type": "command",
          "command": "grep -q 'Status: ✅ APPROVED' && ./scripts/deploy-staging.sh || true"
        }]
      }]
    }
  }
}
```

**Use Case:** Automate deployment pipelines based on approval state.

---

### 3. Agent-Specific Tool Restrictions

**Example:** DOCWRITER can only write to `docs/`, not `src/`.

```json
{
  "agents": {
    "docwriter": {
      "permissions": {
        "allow": ["Write(docs/**/*)", "Read(**/*)", "Bash(prettier *)"],
        "deny": ["Write(src/**/*)", "Write(tests/**/*)", "Bash(cargo *)"]
      }
    }
  }
}
```

**Use Case:** Enforce strict role boundaries programmatically.

---

## MIGRATION CHECKLIST

Use this checklist to verify your migration is complete:

### Phase 1: Setup
- [ ] Install Claude Code CLI
- [ ] Verify `.claude/` directory structure exists
- [ ] Read `.claude/CLAUDE.md` (project rules)
- [ ] Read `~/.claude/CLAUDE.md` (user rules, if exists)

### Phase 2: Learning New Syntax
- [ ] Review `docs/INVOCATION_REFERENCE.md` (this file)
- [ ] Practice with `/dispatch` to understand routing
- [ ] Test simple command: `/doc-readme`
- [ ] Test full pipeline: `/architect-design` → `/review` → `/planner-roadmap`

### Phase 3: Understanding New Features
- [ ] Review `.claude/settings.json` permissions
- [ ] Understand hooks (pre/post tool use)
- [ ] Test gate enforcement (try to `/rust-implement` without plan)
- [ ] Verify hooks work (edit Rust file, see auto-format)

### Phase 4: Workflow Validation
- [ ] Run through full feature pipeline (architecture → plan → code)
- [ ] Verify HOSTILE_REVIEWER gates still enforce quality
- [ ] Test emergency `/dispatch` when stuck
- [ ] Confirm all 10 agents work with new syntax

### Phase 5: Documentation
- [ ] Update team wiki with new commands (if applicable)
- [ ] Share this MIGRATION_GUIDE.md with team
- [ ] Create project-specific examples (if needed)

---

## GETTING HELP

### If Commands Aren't Working

1. **Check phase status:**
   ```
   ls .claude/GATE_*.md
   ```
   - If no gates exist, you're in Phase 1 (Architecture only)

2. **Use `/dispatch` for diagnosis:**
   ```
   /dispatch
   ```
   - It will tell you what phase you're in and what you can do

3. **Check permissions:**
   ```
   cat .claude/settings.json
   ```
   - Ensure the command you're trying to run is in `allow` list

4. **Review hooks:**
   ```
   grep -A5 "hooks" .claude/settings.json
   ```
   - Check if a hook is blocking your action

### If Quality Gates Are Too Strict

**Remember:** Strictness is a feature, not a bug.

**But if you MUST bypass temporarily:**
1. Use `[HUMAN_OVERRIDE]` tag in your request
2. Document why override is needed
3. Track technical debt in an issue

**Example:**
```
User: [HUMAN_OVERRIDE] I need to test a quick prototype without going through full architecture.

Reason: Validating WASM SharedArrayBuffer support before committing to design.

Acknowledged risks:
- This code will be thrown away
- No production use
- Will redo properly after validation
```

**System will:**
- Allow the action
- Log the override
- Remind you to follow proper process for production code

---

## SUMMARY

**What Changed:**
- Syntax: `@AGENT` → `/command`
- File structure: Monolithic → Modular (agents + commands + settings)
- Enforcement: Honor system → Programmatic (hooks + permissions)

**What Stayed The Same:**
- All 10 agents with identical roles
- Supreme Rule: Architecture > Plan > Code
- HOSTILE_REVIEWER ultimate veto power
- Military Strict Mode philosophy
- Chain-of-thought protocols
- Quality standards

**Key Takeaway:**
Claude Code makes EdgeVec's strict workflow **enforceable** instead of just **documented**. You can't accidentally bypass gates anymore. The system protects you from yourself.

**Next Steps:**
1. Read `docs/INVOCATION_REFERENCE.md` for quick command lookup
2. Try `/dispatch` to route your next request
3. Follow a full pipeline (architecture → plan → code) to internalize new syntax

---

*Migration Guide Version: 1.0.0*
*Project: EdgeVec*
*Converted: Cursor → Claude Code*
