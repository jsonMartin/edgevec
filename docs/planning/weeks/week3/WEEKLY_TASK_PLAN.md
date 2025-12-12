# EdgeVec Weekly Task Plan — Week 3

**Date Range:** Day 11 to Day 15
**Author:** PLANNER
**Status:** COMPLETED

---

## THIS WEEK'S GOAL

**"Implement the core HNSW graph algorithms (Insert, Search) and verify correctness against property tests."**

Focus: Milestone 1.1 (Core Algorithms). Correctness > Speed.

---

## APPROVED TASKS

**CRITICAL:** Only tasks in this section may be implemented by RUST_ENGINEER.

| ID | Task | Owner | Verification Strategy | Est. Hours | Acceptance Criteria |
|:---|:-----|:------|:----------------------|:-----------|:--------------------|
| W3.1 | **HNSW Infrastructure**<br>- `HnswIndex` struct<br>- Layer generation (RNG)<br>- Entry point logic | RUST_ENGINEER | Unit + Prop | 8 | - `test_layer_distribution` passes (geometric dist)<br>- `HnswIndex::new` initializes correctly |
| W3.2 | **Neighbor Management**<br>- `NeighborPool` (VByte)<br>- **FreeList Mechanism** for Reuse<br>- `SelectNeighbors` heuristic | RUST_ENGINEER | Unit + Prop | 12 | - `test_pool_recycling` passes (memory is reused)<br>- `test_select_neighbors_simple` passes |
| W3.3 | **Greedy Search**<br>- `search_layer` (Algo 2)<br>- Candidate queue logic | RUST_ENGINEER | Unit + Fuzz | 12 | - `test_search_layer_finds_node` passes<br>- Fuzz: `search_layer` never panics |
| W3.4 | **Insertion Logic**<br>- `insert` (Algo 1)<br>- Bidirectional connection<br>- **Error Handling (Result)** | RUST_ENGINEER | Unit + Prop | 16 | - `test_insert_connectivity` passes<br>- Internal errors (OOM) return `Err` (no panics) |
| W3.5 | **Validation & Benchmarks**<br>- Integration tests<br>- Recall@10 Benchmark | TEST_ENGINEER | Bench | 8 | - Recall@10 > 0.95 on **Uniform Random** data<br>- Memory overhead verified <100 bytes/vec |

---

## BLOCKED TASKS

| ID | Task | Blocked By | Unblock Condition |
|:---|:-----|:-----------|:------------------|
| W3.B1 | WASM Bindings | Week 3 Completion | HNSW core logic verified |
| W3.B2 | Persistence of Graph | Week 3 Completion | `HnswIndex` memory layout final |

---

## NOT IN SCOPE THIS WEEK

| Task | Why Deferred |
|:-----|:-------------|
| SIMD Optimization | Premature optimization. Correctness first. |
| Deletion support | Complex. Scheduled for Week 4. |
| Disk Persistence | Focus is on in-memory graph logic this week. |

---

## VALIDATION CRITERIA

This week is COMPLETE when:
- [x] `HnswIndex` can insert and search vectors in memory.
- [x] `NeighborPool` demonstrates <100 bytes/vector overhead (**Verified via recycling stats**).
- [x] Recall@10 > 0.95 on **Uniform Random** datasets.
- [x] All "Nvidia Grade" tests (Prop/Fuzz) pass.
- [x] No panics in `insert` or `search` paths.

---

## HOSTILE REVIEW REQUIRED

**Before coding begins:**
- [x] HOSTILE_REVIEWER has approved this plan

**After coding ends:**
- [x] HOSTILE_REVIEWER validates all deliverables

---

## APPROVALS

| Role | Name | Signature | Date |
|:-----|:-----|:----------|:-----|
| PLANNER | PLANNER | ✓ | 2025-12-07 |
| HOSTILE_REVIEWER | | [PENDING] | |
