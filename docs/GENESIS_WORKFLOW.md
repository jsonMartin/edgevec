# EdgeVec Genesis Workflow

**Version:** 1.0.0
**Date:** 2025-12-04
**Author:** META_ARCHITECT (Genesis Phase)

---

## Overview

This document defines the **rigid sequence of operations** for EdgeVec development. No phase may begin until the previous phase is complete and approved.

---

## Phase 0: Environment Setup ✅

**Status:** COMPLETE

### Deliverables

| Artifact | Status | Notes |
|:---------|:-------|:------|
| `.cursorrules` | ✅ | Supreme development protocol |
| `CMD_META_ARCHITECT.md` | ✅ | System design agent |
| `CMD_PLANNER.md` | ✅ | Roadmap & task planning agent |
| `CMD_RUST_ENGINEER.md` | ✅ | Core implementation agent |
| `CMD_WASM_SPECIALIST.md` | ✅ | WASM integration agent |
| `CMD_BENCHMARK_SCIENTIST.md` | ✅ | Performance testing agent |
| `CMD_HOSTILE_REVIEWER.md` | ✅ | Quality gate agent |
| `CMD_DOCWRITER.md` | ✅ | Documentation agent |

### Unlock Condition

- [x] All agent commands created
- [x] .cursorrules established
- [x] Genesis workflow documented

**→ PROCEED TO PHASE 1**

---

## Phase 1: Gap Analysis & Architecture

**Status:** IN PROGRESS — Architecture & Verification Strategy Drafted

### Purpose

Define what EdgeVec is, what it contains, and how it works — before any code is written.

### Step 1.1: Gap Analysis

**Input:** `ASSET_FIT_REPORT.md` from binary_semantic_cache audit

**Output:** `docs/GAP_ANALYSIS.md`

**Content:**

| Aspect | binary_semantic_cache | EdgeVec Requirement | Gap |
|:-------|:----------------------|:--------------------|:----|
| Index Type | Linear scan | HNSW | **BUILD FROM SCRATCH** |
| Vector Dims | 256-bit binary only | Generic (128-3072) | **BUILD FROM SCRATCH** |
| Persistence | Python pickle | Rust/IndexedDB | **BUILD FROM SCRATCH** |
| WASM | PyO3 (incompatible) | Native wasm-bindgen | **BUILD FROM SCRATCH** |
| Quantization | Binary only | Float + Binary | **EXTEND** |
| Hamming Distance | ✅ Salvageable | ✅ Keep | **SALVAGE** |

**Agent:** META_ARCHITECT
**Reviewer:** HOSTILE_REVIEWER

### Step 1.2: Architecture Blueprint

**Input:** Gap Analysis

**Outputs:**
- `docs/architecture/ARCHITECTURE.md` — System overview
- `docs/architecture/DATA_LAYOUT.md` — Memory layouts
- `docs/architecture/WASM_BOUNDARY.md` — JS/Rust interface
- `docs/architecture/PERSISTENCE_SPEC.md` — Storage format
- `docs/architecture/INVARIANTS.md` — System invariants

**Key Decisions to Make:**

1. **Vector Storage:**
   - Float32 arrays? Quantized binary? Both?
   - Memory layout: SOA vs AOS?

2. **HNSW Parameters:**
   - M (connections per node)?
   - ef_construction?
   - ef_search (default)?

3. **WASM Boundary:**
   - How are vectors passed (copy vs view)?
   - How are results returned (JSON vs ArrayBuffer)?
   - Async vs sync API?

4. **Persistence:**
   - File format (custom binary vs MessagePack)?
   - Incremental save vs full dump?
   - IndexedDB structure (single store vs multiple)?

**Agent:** META_ARCHITECT
**Reviewer:** HOSTILE_REVIEWER

### Gate 1 Checkpoint

```
┌─────────────────────────────────────────────────────────────┐
│   GATE 1: Architecture Approval                             │
│                                                             │
│   Required for APPROVAL:                                    │
│   - [x] ARCHITECTURE.md complete                            │
│   - [x] DATA_LAYOUT.md complete                             │
│   - [x] WASM_BOUNDARY.md complete                           │
│   - [x] TEST_STRATEGY.md complete (Nvidia Grade)            │
│   - [x] All struct sizes calculated                         │
│   - [x] Performance budget defined                          │
│   - [ ] No [UNKNOWN] items remaining                        │
│                                                             │
│   HOSTILE_REVIEWER Verdict: ❌ REJECTED                     │
│   Reason: Alignment bugs, Memory budget violation, RNG gap  │
└─────────────────────────────────────────────────────────────┘
```

**→ BLOCKED UNTIL GATE 1 PASSES**

---

## Phase 2: Master Roadmap

**Status:** BLOCKED (Requires Gate 1)

### Purpose

Transform approved architecture into a phased execution plan with milestones.

### Step 2.1: Roadmap Creation

**Input:** Approved architecture documents

**Output:** `docs/planning/ROADMAP.md`

**Content:**

```markdown
# EdgeVec Roadmap

## Phase 1: Foundation (Weeks 1-4)
- Milestone 1.1: HNSW Core (in-memory)
- Milestone 1.2: WASM Bindings (basic)
- Milestone 1.3: Browser Demo

## Phase 2: Persistence (Weeks 5-8)
- Milestone 2.1: IndexedDB Integration
- Milestone 2.2: Save/Load API
- Milestone 2.3: WAL Implementation

## Phase 3: Performance (Weeks 9-12)
- Milestone 3.1: SIMD Optimization
- Milestone 3.2: Multi-threading (if SAB available)
- Milestone 3.3: Benchmark Suite

## Phase 4: Polish (Weeks 13-16)
- Milestone 4.1: TypeScript Types
- Milestone 4.2: Documentation
- Milestone 4.3: npm/crates.io Publish

## Phase 5: Advanced (Weeks 17-24)
- Milestone 5.1: Quantization Options
- Milestone 5.2: Batch Operations
- Milestone 5.3: v1.0.0 Release
```

**Agent:** PLANNER
**Reviewer:** HOSTILE_REVIEWER

### Step 2.2: Risk Register

**Output:** `docs/planning/RISK_REGISTER.md`

**Content:**

| ID | Risk | Impact | Likelihood | Mitigation | Owner |
|:---|:-----|:-------|:-----------|:-----------|:------|
| R1 | SharedArrayBuffer unavailable in Safari | HIGH | MEDIUM | Single-threaded fallback | WASM_SPECIALIST |
| R2 | IndexedDB quota exceeded | MEDIUM | LOW | Warn user, provide export | RUST_ENGINEER |
| R3 | HNSW recall too low | HIGH | LOW | Tune ef_search, document tradeoffs | BENCHMARK_SCIENTIST |

**Agent:** PLANNER
**Reviewer:** HOSTILE_REVIEWER

### Gate 2 Checkpoint

```
┌─────────────────────────────────────────────────────────────┐
│   GATE 2: Roadmap Approval                                  │
│                                                             │
│   Required for APPROVAL:                                    │
│   - [ ] ROADMAP.md complete                                 │
│   - [ ] All milestones have Definition of Done              │
│   - [ ] Critical path identified                            │
│   - [ ] Risk register complete                              │
│   - [ ] 30% buffer included in timeline                     │
│                                                             │
│   HOSTILE_REVIEWER Verdict: [PENDING]                       │
└─────────────────────────────────────────────────────────────┘
```

**→ BLOCKED UNTIL GATE 2 PASSES**

---

## Phase 3: Tactical Planning

**Status:** BLOCKED (Requires Gate 2)

### Purpose

Break roadmap into weekly task plans with specific acceptance criteria.

### Step 3.1: Week 1 Task Plan

**Input:** Approved ROADMAP.md

**Output:** `docs/planning/WEEKLY_TASK_PLAN.md`

**Content:**

```markdown
# Weekly Task Plan — Week 1

## Goal: In-Memory HNSW Prototype

## Approved Tasks

| ID | Task | Owner | Hours | Acceptance |
|:---|:-----|:------|:------|:-----------|
| W1.1 | HNSW insert implementation | RUST_ENGINEER | 16 | `test_insert_1000` passes |
| W1.2 | HNSW search implementation | RUST_ENGINEER | 16 | `test_search_recall` > 0.9 |
| W1.3 | Basic Rust tests | RUST_ENGINEER | 8 | All unit tests pass |

## NOT This Week
- WASM bindings (depends on core)
- Persistence (Phase 2)
```

**Agent:** PLANNER
**Reviewer:** HOSTILE_REVIEWER

### Gate 3 Checkpoint (Per Week)

```
┌─────────────────────────────────────────────────────────────┐
│   GATE 3: Weekly Plan Approval                              │
│                                                             │
│   Required for APPROVAL:                                    │
│   - [ ] All tasks have acceptance criteria                  │
│   - [ ] No task > 16 hours                                  │
│   - [ ] Dependencies verified                               │
│   - [ ] Owners assigned                                     │
│                                                             │
│   HOSTILE_REVIEWER Verdict: [PENDING]                       │
│                                                             │
│   *** CODING UNLOCKED ONLY AFTER APPROVAL ***               │
└─────────────────────────────────────────────────────────────┘
```

**→ CODING UNLOCKED UPON GATE 3 APPROVAL**

---

## Phase 4: Implementation

**Status:** BLOCKED (Requires Gate 3)

### Weekly Cycle

```
Monday:
├── PLANNER presents WEEKLY_TASK_PLAN.md
├── HOSTILE_REVIEWER approves plan
└── → CODING UNLOCKED

Tuesday-Thursday:
├── RUST_ENGINEER implements approved tasks
├── WASM_SPECIALIST creates bindings (if applicable)
└── Tests written alongside code

Friday:
├── BENCHMARK_SCIENTIST runs performance validation
├── HOSTILE_REVIEWER reviews all deliverables
└── → GATE 4 (code merge)
```

### Gate 4 Checkpoint (Per Task)

```
┌─────────────────────────────────────────────────────────────┐
│   GATE 4: Code Merge Approval                               │
│                                                             │
│   Required for APPROVAL:                                    │
│   - [ ] All acceptance criteria met                         │
│   - [ ] All tests pass                                      │
│   - [ ] No clippy warnings                                  │
│   - [ ] Code formatted                                      │
│   - [ ] Documentation complete                              │
│   - [ ] No performance regression                           │
│                                                             │
│   HOSTILE_REVIEWER Verdict: [PENDING]                       │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 5: Documentation & Release

**Status:** BLOCKED (Requires Phase 4 milestones)

### Step 5.1: Documentation

**Agent:** DOCWRITER

**Outputs:**
- `README.md` — The viral hook
- `docs/GETTING_STARTED.md` — Quick start
- `docs/API.md` — API reference
- `CHANGELOG.md` — Version history

### Step 5.2: Release Preparation

**Checklist:**
- [ ] All tests pass
- [ ] All benchmarks meet targets
- [ ] README is compelling
- [ ] npm package configured
- [ ] crates.io package configured
- [ ] GitHub Actions CI/CD

### Gate 5 Checkpoint

```
┌─────────────────────────────────────────────────────────────┐
│   GATE 5: Release Approval                                  │
│                                                             │
│   Required for APPROVAL:                                    │
│   - [ ] Documentation complete                              │
│   - [ ] Examples tested                                     │
│   - [ ] CHANGELOG updated                                   │
│   - [ ] Version number set                                  │
│   - [ ] License file present                                │
│                                                             │
│   HOSTILE_REVIEWER Verdict: [PENDING]                       │
│                                                             │
│   *** v0.1.0 RELEASE UPON APPROVAL ***                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Execution Timeline (Estimated)

| Week | Phase | Focus | Gate |
|:-----|:------|:------|:-----|
| 0 | Setup | Environment, agents | — |
| 1 | Architecture | Design docs | Gate 1 |
| 2 | Planning | Roadmap, risks | Gate 2 |
| 3-4 | Implementation | HNSW Core | Gate 3, 4 |
| 5-6 | Implementation | WASM Bindings | Gate 3, 4 |
| 7-8 | Implementation | Persistence | Gate 3, 4 |
| 9-10 | Implementation | Performance | Gate 3, 4 |
| 11-12 | Documentation | README, API | Gate 5 |
| 13+ | Release | v0.1.0 | Ship! |

---

## Current Status

```
┌─────────────────────────────────────────────────────────────┐
│   CURRENT PHASE: 1 (Architecture)                           │
│   STATUS: REJECTED by Hostile Reviewer                      │
│   NEXT ACTION: META_ARCHITECT to fix critical flaws         │
│                                                             │
│   Invoke: @META_ARCHITECT fix architecture                  │
│                                                             │
│   Blocking: All coding is FORBIDDEN until Gate 3            │
└─────────────────────────────────────────────────────────────┘
```

### Phase 1 Log Entry (2025-12-05) - Iteration 3

**Architecture & Verification Strategy Hardened**

| Document | Status | Key Changes |
|:---------|:-------|:------------|
| `TEST_STRATEGY.md` | ✅ PROPOSED | Added Recall@K check, Fuzzing code, Property definitions |
| `DATA_LAYOUT.md` | ✅ PROPOSED | Fixed `SectionHeader` & `HnswConfig` alignment |
| `WASM_BOUNDARY.md` | ✅ PROPOSED | Fixed String type safety |
| `ARCHITECTURE.md` | ✅ PROPOSED | Clarified Q3 (IndexedDB limits) |

**Hostile Review Fixes:**
1. **[C1] Fuzzing:** Implemented 4 specific fuzz targets.
2. **[C2] Recall:** Added `RECALL-001` mandatory statistical check.
3. **[C3-C6] Coverage:** Defined all referenced tests and fixed alignment math.
4. **[M1-M3] Consistency:** Resolved String types and Alignment docs.

**Next:** Run `/CMD_HOSTILE_REVIEWER` for Gate 1 Final Approval

---

## Quick Reference: What's Allowed Now?

| Action | Allowed? | Why |
|:-------|:---------|:----|
| Create architecture docs | ✅ YES | Phase 1 |
| Write Rust code | ❌ NO | Requires Gate 3 |
| Create roadmap | ❌ NO | Requires Gate 1 |
| Create weekly plan | ❌ NO | Requires Gate 2 |
| Write tests | ❌ NO | Requires approved code |
| Write README | ❌ NO | Requires working code |

---

## Invoke Next Phase

To begin Phase 1:

```
@META_ARCHITECT design gap_analysis
```

This will:
1. Analyze binary_semantic_cache vs EdgeVec requirements
2. Identify gaps to fill
3. Begin ARCHITECTURE.md drafting

---

*END OF GENESIS WORKFLOW*

