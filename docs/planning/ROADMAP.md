# EdgeVec Roadmap v1.3

**Date:** 2025-12-11
**Author:** PLANNER
**Status:** [REVISED] (Week 7 Complete - Persistence & HNSW Done)

---

## Executive Summary

**Total Duration:** ~32 Weeks (Dec 2025 – Jul 2026)
**Current Status:** Week 7 Complete (Ahead of Schedule on Core, On Track for Integration).
**Philosophy:** Test-First, WASM-Native, Memory-Constrained.
**Critical Path:** Test Harness → Persistence Layer → **Quantization** → WASM Integration.
**Buffer Strategy:** 4 weeks of explicit contingency allocated.

---

## Phase 1: Foundation (Weeks 1-4)

### Milestone 1: Core Scaffolding & Test Harness
**Dates:** Weeks 1-4 (Dec 8 - Jan 2)
**Status:** ✅ COMPLETE
**Theme:** "Verification infrastructure before feature code."

**Objectives:**
- Establish the repo structure with strict CI checks (Miri, Clippy).
- Implement the "Nvidia Grade" test harness (Proptest, Fuzzing skeleton).
- Implement basic types (`VectorId`, `NodeId`) and `VectorStorage`.

**Deliverables:**
1.  **Repo Setup:** CI pipeline running `cargo test`, `cargo fmt`, `cargo clippy`.
2.  **Test Harness:** `proptest` integration, `cargo-fuzz` setup, `miri` configuration.
3.  **Core Types:** `VectorId`, `NodeId`, `VectorStorage` (Arena-based).
4.  **Verification:** Fuzz target for `VectorStorage` inputs.

---

## Phase 2: Persistence & Compression (Weeks 5-8)

### Milestone 2: Storage Engine, Persistence, & Quantization
**Dates:** Weeks 5-8 (Jan 5 - Jan 30)
**Status:** ✅ COMPLETE
**Theme:** "Data must survive a crash and fit in RAM."

**Objectives:**
- Implement the `StorageBackend` trait and `MemoryBackend`.
- Build the `WriteAheadLog` (WAL) with checksums.
- Implement `SnapshotManager` for atomic checkpoints.
- Define and verify the binary file format (`.evec`).
- **Implement Scalar Quantization (SQ8)** to reduce memory by 4x.

**Deliverables:**
1.  **Storage Layer:** `WriteAheadLog`, `SnapshotManager`, `FileHeader`.
2.  **Serialization:** `postcard` integration for all on-disk structures.
3.  **Quantization:** `ScalarQuantizer` (u8/i8) with SIMD support. [DONE - Week 6]
4.  **Verification:** `PROP-WAL-001` and `PROP-PERSIST-001` tests.

**Outcomes:**
- **Pivot Executed:** Week 6 pivot to SQ8 solved the memory crisis (1M vectors in <1GB).
- **Persistence Hardened:** Week 7 delivered WAL and Snapshots.

---

## [BUFFER] Strategic Contingency 1
**Dates:** Weeks 9-10 (Feb 2 - Feb 13)
**Purpose:** Catch-up time for M1/M2 slippage or unforeseen WASM memory issues.
**Status:** AVAILABLE.

---

## Phase 3: Intelligence (Weeks 11-18)

### Milestone 3: HNSW Graph Implementation
**Dates:** Pulled Forward (Completed in Phase 1/2)
**Status:** ✅ COMPLETE
**Theme:** "Correctness first, then performance."

**Objectives:**
- Implement the HNSW graph algorithms (Insert, Search).
- Implement `NeighborPool` with VByte compression (critical for memory budget).
- Integrate `DeterministicRng` for reproducible builds.
- Verify Recall@10 > 0.95.

**Deliverables:**
1.  **Graph Core:** `HnswIndex`, `HnswNode`, `HnswConfig`.
2.  **Compression:** `NeighborPool` with VByte encoding/decoding.
3.  **Algorithms:** Layer sampling, neighbor selection, heuristic search.
4.  **Verification:** `verify_recall_at_10` test suite.

---

## Phase 4: Integration (Weeks 19-26)

### Milestone 4: WASM Integration & Optimization
**Dates:** Weeks 8-16 (Rescheduled)
**Status:** 🚧 IN PROGRESS
**Theme:** "It must work in the browser."

**Objectives:**
- Expose the public API via `wasm-bindgen`.
- Implement `IndexedDbBackend` for browser persistence.
- Create the TypeScript client library.
- Performance profiling and optimization (SIMD, allocation reduction).

**Deliverables:**
1.  **WASM Bindings:** `edgevec_new`, `edgevec_insert`, `edgevec_search`.
2.  **Browser Storage:** `IndexedDbBackend` implementation.
3.  **JS Client:** TypeScript wrapper with Promise-based API.
4.  **Benchmarks:** Browser-based benchmark suite.

**Risks:**
- [R-M4-1] `IndexedDB` transaction limits blocking large writes. (Mitigation: Chunked writes in the JS layer).
- [R-M4-2] WASM memory growth fragmentation. (Mitigation: Use `wee_alloc` or custom allocator tuning).

---

## [BUFFER] Integration Contingency 2
**Dates:** Weeks 27-28
**Status:** RESERVED.

---

## Phase 5: Release (Weeks 29-32)

### Milestone 5: Production Hardening & Release
**Dates:** Weeks 29-32
**Status:** PENDING
**Theme:** "Polishing the diamond."

**Objectives:**
- Finalize documentation (Rustdoc, README, Architecture details).
- Create comprehensive examples and demos.
- Final code audit and cleanup.
- Publish to crates.io and npm.

---

## Risk Register Summary

| ID | Risk | Impact | Likelihood | Owner | Mitigation |
|:---|:-----|:-------|:-----------|:------|:-----------|
| R1 | WASM Memory Limits (4GB) | HIGH | LOW | ARCHITECT | Graceful errors, sharding (future) |
| R2 | Browser IDB Variability | MEDIUM | HIGH | WASM_DEV | Extensive browser testing matrix |
| R3 | Recall degradation on weird distributions | HIGH | MEDIUM | ALGO_DEV | Test on SIFT/GIST/GloVe datasets |
| R4 | SIMD not portable in WASM | MEDIUM | MEDIUM | RUST_DEV | Feature flags for simd128 |
| R5 | **Memory usage exceeds 1GB for 1M vectors** | HIGH | HIGH | ARCHITECT | **MITIGATED (via Quantization)** |

---

## Approval Status

| Reviewer | Verdict | Date |
|:---------|:--------|:-----|
| HOSTILE_REVIEWER | ✅ APPROVED | 2025-12-05 |
