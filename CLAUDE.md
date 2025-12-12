- # TASK: Convert EdgeVec Cursor Commands to Claude Code Format

## OBJECTIVE
Convert the existing EdgeVec agent command system from Cursor IDE format to Claude Code CLI format, preserving all rigor, structure, and the "Military Strict Mode" philosophy while adapting to Claude Code's specific capabilities and file structure.

## CONTEXT
You have a mature, battle-tested agent command system designed for Cursor IDE. It includes:
- 10 specialized agent commands (META_ARCHITECT, PLANNER, RUST_ENGINEER, TEST_ENGINEER, WASM_SPECIALIST, BENCHMARK_SCIENTIST, HOSTILE_REVIEWER, DOCWRITER, PROMPT_MAKER, OPTIMIZATION_PROMPT)
- A strict hierarchical approval system with HOSTILE_REVIEWER as the ultimate gate
- "Military Strict Mode" global rules
- A README documenting the agent roster and invocation patterns

## REQUIRED UNDERSTANDING
Before starting, read and understand:
1. Claude Code's native configuration system (settings.json, CLAUDE.md, agents/, commands/, hooks/)
2. The difference between Cursor's @-mention system and Claude Code's /slash commands
3. How Claude Code handles permissions, subagents, and custom commands
4. The role of .claude/ directory structure

## CONVERSION REQUIREMENTS

### 1. PRESERVE CORE PHILOSOPHY
- **DO NOT** dilute the "Architecture > Plan > Code" supreme rule
- **DO NOT** remove the hostile review gate system
- **DO NOT** simplify the anti-hallucination clamps
- **DO NOT** reduce the TDD-first approach
- **PRESERVE** all role boundaries and kill authorities
- **PRESERVE** all chain-of-thought protocols
- **PRESERVE** all output format templates

### 2. ADAPT TO CLAUDE CODE STRUCTURE

#### A. Global Rules → CLAUDE.md
Convert the "GLOBAL USER RULES — MILITARY STRICT MODE" into:
- `~/.claude/CLAUDE.md` (user-level, applies to all projects)
- `.claude/CLAUDE.md` (project-level, EdgeVec-specific context)

**Instructions:**
- Split general rules (safety, conduct, testing) into user-level
- Move EdgeVec-specific rules (architecture gates, agent roster) to project-level
- Maintain the same strictness and completeness

#### B. Agent Commands → Multiple Artifacts

For EACH agent command file (e.g., CMD_RUST_ENGINEER.md), create:

1. **Subagent Definition:** `.claude/agents/<agent-name>.md`
   ```markdown
   ---
   name: rust-engineer
   description: Core Rust implementation with strict TDD
   tools:
     - Read
     - Write
     - Bash(cargo *)
     - Edit
   ---
   
   [Agent mandate, principles, protocols from original CMD file]
   ```

2. **Slash Command(s):** `.claude/commands/<command-name>.md`
   ```markdown
   Execute the RUST_ENGINEER workflow for implementing: $ARGUMENTS
   
   [Include the pre-coding checklist, chain of thought protocol]
   ```

3. **Permission Rules:** Add to `.claude/settings.json`
   ```json
   {
     "permissions": {
       "allow": ["Bash(cargo test)", "Bash(cargo clippy)", ...],
       "deny": ["Bash(rm *)", ...]
     }
   }
   ```

4. **Hooks (if applicable):** Add to `.claude/settings.json`
   ```json
   {
     "hooks": {
       "PostToolUse": {
         "Edit": [{
           "matcher": "**/*.rs",
           "hooks": [{"type": "command", "command": "cargo fmt"}]
         }]
       }
     }
   }
   ```

#### C. HOSTILE_REVIEWER → Special Treatment
Since HOSTILE_REVIEWER has "kill authority," implement as:
1. A subagent in `.claude/agents/hostile-reviewer.md`
2. A mandatory review hook that blocks merges
3. A quality gate checklist in `.claude/HOSTILE_GATE_CHECKLIST.md`
4. Permission settings that enforce review before critical operations

#### D. PROMPT_MAKER → Workflow Dispatcher
Convert PROMPT_MAKER into:
1. A meta-command in `.claude/commands/dispatch.md` that analyzes requests and routes to correct agent
2. A decision tree in `.claude/WORKFLOW_ROUTER.md`

### 3. MAINTAIN INVOCATION PATTERNS

Original Cursor pattern:
```
@META_ARCHITECT design gap_analysis
@PLANNER weekly 1
@RUST_ENGINEER implement W1.1
```

New Claude Code pattern:
```
/agent meta-architect
[prompt about gap analysis]

/planner-weekly 1

/rust-engineer W1.1
```

**Create a mapping document** showing old vs new invocations.

### 4. HANDLE SPECIAL CASES

#### Phase Gates
Original: Enforced via document status checks
Adaptation: Use Claude Code's permission system + custom hooks

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

#### Context Requirements
Original: "Required Before Executing: @file ARCHITECTURE.md"
Adaptation: Add to slash command templates:

```markdown
---
Before executing, ensure these files are loaded:
- ARCHITECTURE.md
- DATA_LAYOUT.md
- WEEKLY_TASK_PLAN.md

Use: /add-dir ../docs if needed
---
```

### 5. OUTPUT STRUCTURE

Produce the following directory structure:

```
.claude/
├── CLAUDE.md                          # Project-level rules
├── HOSTILE_GATE_CHECKLIST.md          # Quality gate criteria
├── WORKFLOW_ROUTER.md                  # Dispatcher logic
├── settings.json                       # Permissions, hooks, env
├── agents/
│   ├── meta-architect.md
│   ├── planner.md
│   ├── rust-engineer.md
│   ├── test-engineer.md
│   ├── wasm-specialist.md
│   ├── benchmark-scientist.md
│   ├── hostile-reviewer.md
│   └── docwriter.md
├── commands/
│   ├── dispatch.md                    # Replaces PROMPT_MAKER
│   ├── architect-design.md
│   ├── planner-roadmap.md
│   ├── planner-weekly.md
│   ├── rust-implement.md
│   ├── test-fuzz.md
│   ├── test-prop.md
│   ├── wasm-bind.md
│   ├── bench-baseline.md
│   ├── review.md
│   └── doc-readme.md
└── hooks/
    └── pre-commit-review.sh           # Enforces hostile review

~/.claude/
└── CLAUDE.md                           # User-level global rules

docs/
├── MIGRATION_GUIDE.md                  # Old vs new invocations
└── INVOCATION_REFERENCE.md             # Quick command reference
```

### 6. VALIDATION CHECKLIST

After conversion, verify:

**Completeness:**
- [ ] All 10 agents have corresponding subagent definitions
- [ ] All agent principles preserved
- [ ] All chain-of-thought protocols preserved
- [ ] All output templates preserved
- [ ] All anti-hallucination clamps preserved

**Functionality:**
- [ ] Permission system blocks code before Gate 3
- [ ] HOSTILE_REVIEWER cannot be bypassed
- [ ] TDD workflow enforced via hooks
- [ ] Phase sequence (Architecture > Plan > Code) maintained

**Usability:**
- [ ] MIGRATION_GUIDE.md clearly explains changes
- [ ] INVOCATION_REFERENCE.md provides quick lookup
- [ ] Example workflows included

**Strictness:**
- [ ] Global rules maintain same rigor
- [ ] No dilution of safety protocols
- [ ] No removal of validation steps

## EXECUTION PLAN

1. **Phase 1: Analysis** (DO THIS FIRST)
   - Read ALL uploaded command files
   - Read the global rules files
   - Read Claude Code documentation (use view tool on /mnt/skills/public/product-self-knowledge/SKILL.md)
   - Create a mapping table: Cursor concept → Claude Code equivalent

2. **Phase 2: Core Conversion**
   - Convert global rules → CLAUDE.md files
   - Convert each agent → subagent + commands + permissions
   - Create settings.json with all permissions and hooks

3. **Phase 3: Special Systems**
   - Implement HOSTILE_REVIEWER gate system
   - Implement PROMPT_MAKER dispatcher
   - Implement phase gate enforcement

4. **Phase 4: Documentation**
   - Create MIGRATION_GUIDE.md
   - Create INVOCATION_REFERENCE.md
   - Update README.md for Claude Code

5. **Phase 5: Validation**
   - Walk through each original use case
   - Verify equivalence
   - Test that gates actually block

## CONSTRAINTS

- **NO SIMPLIFICATION**: If something seems "too strict," that's intentional. Keep it.
- **NO ASSUMPTIONS**: If you're unsure how to map something, ask.
- **NO SHORTCUTS**: Don't skip agents or merge roles.
- **PRESERVE NUMBERING**: Keep task IDs, gate numbers, version numbers.
- **MAINTAIN TONE**: The "military strict" language is intentional.

## DELIVERABLES

1. Complete `.claude/` directory structure (all files)
2. Complete `~/.claude/CLAUDE.md` (user-level rules)
3. `docs/MIGRATION_GUIDE.md`
4. `docs/INVOCATION_REFERENCE.md`
5. Updated `README.md` for agent roster

## SUCCESS CRITERIA

The conversion is successful when:
1. A developer can follow the same workflow as before, using new syntax
2. All safety gates remain intact
3. HOSTILE_REVIEWER maintains veto power
4. Code cannot be written before architecture approval
5. The system feels equally (or more) rigorous than the original

---

## BEGIN CONVERSION

Start by reading the Claude Code documentation, then analyze all uploaded command files, then create the mapping table, then begin systematic conversion.

**Remember:** Strictness is a feature, not a bug. Preserve it completely.