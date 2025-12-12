# EdgeVec Agent Commands

**Version:** 1.1.0

This directory contains the system prompts for all EdgeVec development agents.

---

## Agent Roster

| Agent | Command File | Role | Kill Authority |
|:------|:-------------|:-----|:---------------|
| **PROMPT_MAKER** | `CMD_PROMPT_MAKER.md` | **Dispatcher & Meta-Agent** | NO |
| **META_ARCHITECT** | `CMD_META_ARCHITECT.md` | System design, data layouts | NO |
| **PLANNER** | `CMD_PLANNER.md` | Roadmaps, weekly tasks | NO |
| **RUST_ENGINEER** | `CMD_RUST_ENGINEER.md` | Core Rust implementation | NO |
| **TEST_ENGINEER** | `CMD_TEST_ENGINEER.md` | **QA, Fuzzing, Property Tests** | NO |
| **WASM_SPECIALIST** | `CMD_WASM_SPECIALIST.md` | WASM bindings, browser | NO |
| **BENCHMARK_SCIENTIST** | `CMD_BENCHMARK_SCIENTIST.md` | Performance testing | NO |
| **HOSTILE_REVIEWER** | `CMD_HOSTILE_REVIEWER.md` | Quality gate | **YES** |
| **DOCWRITER** | `CMD_DOCWRITER.md` | Documentation | NO |

---

## Quick Invocation Guide

### Phase 1: Architecture (No Code Allowed)

```
@META_ARCHITECT design gap_analysis
@META_ARCHITECT design architecture
@HOSTILE_REVIEWER review ARCHITECTURE.md
```

### Phase 2: Planning (No Code Allowed)

```
@PLANNER roadmap
@HOSTILE_REVIEWER review ROADMAP.md
@PLANNER weekly 1
@HOSTILE_REVIEWER review WEEKLY_TASK_PLAN.md
```

### Phase 3: Implementation (Code Allowed After Gate 3)

```
@RUST_ENGINEER implement W1.1
@TEST_ENGINEER prop W1.1
@TEST_ENGINEER fuzz W1.1
@WASM_SPECIALIST bind search
@BENCHMARK_SCIENTIST baseline hnsw
@HOSTILE_REVIEWER review [task_output]
```

### Phase 4: Documentation

```
@DOCWRITER readme
@HOSTILE_REVIEWER review README.md
```

---

## The Supreme Rule

**No code is written without an approved plan.**
**No plan is created without approved architecture.**
**No artifact proceeds without HOSTILE_REVIEWER approval.**

See `../.cursorrules` for complete protocol.
