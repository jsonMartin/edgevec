# EdgeVec Roadmap v2.0

**Date:** 2025-12-16
**Author:** PLANNER
**Status:** [REVISED] — Week 19 Reconciliation Update
**Current Version:** v0.3.0 (released)
**Next Version:** v0.4.0 (in progress)

---

## Executive Summary

**Total Duration:** ~32 Weeks (Dec 2025 – Jul 2026)
**Current Status:** Week 19 — v0.4.0 Release Sprint
**Philosophy:** Test-First, WASM-Native, Memory-Constrained
**Critical Path:** v0.4.0 Release → v0.5.0 (ARM/NEON) → v1.0 (Production)

---

## Phase 1: Foundation (Weeks 1-4) — COMPLETE

### Milestone 1: Core Scaffolding & Test Harness
**Status:** COMPLETE
**Gate:** `.claude/GATE_2_COMPLETE.md`

**Deliverables:**
- Repo setup with CI pipeline
- Test harness (proptest, cargo-fuzz, miri)
- Core types (VectorId, NodeId, VectorStorage)

---

## Phase 2: Persistence & Compression (Weeks 5-8) — COMPLETE

### Milestone 2: Storage Engine, Persistence, & Quantization
**Status:** COMPLETE
**Gate:** `.claude/GATE_WEEK8_COMPLETE.md`

**Deliverables:**
- WriteAheadLog, SnapshotManager
- Scalar Quantization (SQ8) — 4x memory reduction
- Binary file format (.evec)

---

## Phase 3: Intelligence (Weeks 9-15) — COMPLETE

### Milestone 3: HNSW Graph + SIMD + RFC-001
**Status:** COMPLETE
**Gates:**
- `.claude/GATE_9_COMPLETE.md` through `.claude/GATE_15_COMPLETE.md`

**Deliverables:**
- HNSW graph algorithms (Insert, Search)
- NeighborPool with VByte compression
- Runtime SIMD detection
- RFC-001 Soft Delete design (approved)

---

## Phase 4: Feature Development (Weeks 16-18) — COMPLETE

### Milestone 4.1: Soft Delete (Week 16)
**Status:** COMPLETE
**Gate:** `.claude/GATE_16_COMPLETE.md`
**Score:** 92/100

**Deliverables:**
- `soft_delete()`, `is_deleted()`, `deleted_count()`, `live_count()`
- `compact()`, `needs_compaction()`, `compaction_warning()`
- Persistence format v0.3 with tombstone support
- Zero memory overhead (reuses padding byte)

### Milestone 4.2: v0.3.0 Release (Week 17)
**Status:** COMPLETE
**Gate:** `.claude/GATE_17_COMPLETE.md`
**Release:** v0.3.0 on crates.io and npm

**Deliverables:**
- WASM soft delete bindings
- TypeScript definitions
- Browser demo (`wasm/examples/soft_delete.html`)
- Documentation update

### Milestone 4.3: Process Hardening & Batch Delete (Week 18)
**Status:** COMPLETE
**Gate:** `.claude/GATE_18_COMPLETE.md`

**Deliverables:**
- CI hardening with `cargo xtask ci-check`
- P99 latency tracking infrastructure
- `soft_delete_batch()` API
- WASM batch delete bindings with Safari fallback
- Dual-license (MIT OR Apache-2.0)

---

## Phase 5: v0.4.0 Release (Week 19) — IN PROGRESS

### Milestone 5: Documentation & Release Polish
**Status:** IN PROGRESS
**Target:** v0.4.0 release
**Gate:** `.claude/GATE_19_WEEK_PLAN_APPROVED.md`

**Week 19 Tasks:**

| Day | Task | Status |
|:----|:-----|:-------|
| Day 1 | Week 16-18 Reconciliation | IN PROGRESS |
| Day 2 | Benchmark Dashboard | PENDING |
| Day 3 | User Documentation Sprint | PENDING |
| Day 4 | Test Hardening & CI | PENDING |
| Day 5 | v0.4.0 Release Prep | PENDING |

**Deliverables:**
- Reconciliation docs for Weeks 16-18
- Benchmark visualization dashboard
- TUTORIAL.md, PERFORMANCE_TUNING.md, TROUBLESHOOTING.md, INTEGRATION_GUIDE.md
- Chaos tests and P99 benchmarks
- CHANGELOG.md, RELEASE_CHECKLIST, CONTRIBUTING.md

---

## Phase 6: v0.5.0+ (Future)

### Milestone 6: Performance Optimization
**Status:** PLANNED
**Target:** v0.5.0+

**Planned Features:**
- ARM/NEON SIMD optimization
- Mobile support (iOS Safari, Android Chrome)
- Memory profiling and optimization
- Sharding for >1M vectors

---

## Version History

| Version | Date | Highlights |
|:--------|:-----|:-----------|
| v0.1.0 | 2025-12-05 | Initial alpha (HNSW, SQ8) |
| v0.2.0 | 2025-12-10 | Batch API, WASM bindings |
| v0.2.1 | 2025-12-14 | Safety hardening (bytemuck) |
| v0.3.0 | 2025-12-15 | Soft Delete API (RFC-001) |
| v0.4.0 | TBD | Documentation, Dashboard, CI |

---

## Risk Register Summary

| ID | Risk | Status |
|:---|:-----|:-------|
| R1 | WASM Memory Limits (4GB) | MITIGATED |
| R2 | Browser IDB Variability | TESTED |
| R3 | Recall degradation | TESTED (>0.95) |
| R4 | SIMD portability | RUNTIME DETECTION |
| R5 | Memory usage >1GB | MITIGATED (SQ8) |

---

## Approval Status

| Reviewer | Verdict | Date |
|:---------|:--------|:-----|
| HOSTILE_REVIEWER | APPROVED | 2025-12-05 (v1.0) |
| HOSTILE_REVIEWER | APPROVED | 2025-12-14 (Week 16) |
| HOSTILE_REVIEWER | APPROVED | 2025-12-15 (Week 17) |
| HOSTILE_REVIEWER | APPROVED | 2025-12-16 (Week 19 Plan) |

---

## Revision History

| Version | Date | Change |
|:--------|:-----|:-------|
| v1.0 | 2025-12-05 | Initial roadmap |
| v1.3 | 2025-12-11 | Week 7 update |
| v2.0 | 2025-12-16 | Week 19 reconciliation — Weeks 16-18 complete |

---

**END OF ROADMAP**
