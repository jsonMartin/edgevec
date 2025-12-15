# Week 17 Task Plan — WASM Bindings & Release Sprint

**Sprint:** Week 17 (Following Week 16 Soft Delete Core)
**Phase:** 4.7 (v0.3.0 Release Preparation)
**Status:** [PROPOSED]
**PLANNER:** Week 17 Planning
**Date Created:** 2025-12-15

---

## Executive Summary

Week 17 completes the v0.3.0 release by adding WASM bindings for soft delete features, comprehensive testing, and release polish. This addresses the deferred C1 item from Week 16 hostile review: "WASM API Missing."

**Goal:** Ship v0.3.0 with full soft delete support in both Rust and WASM.

**Critical Path:** W17.1 (WASM bindings) → W17.2 (Integration tests) → W17.3 (Example app) → W17.4 (Release prep) → W17.5 (Publish)

---

## Week 17 Context

### Previous Week (Week 16) Accomplishments

| Task | Status | Key Deliverable |
|:-----|:-------|:----------------|
| W16.1 | ✅ COMPLETE | `HnswNode.deleted` field (zero overhead) |
| W16.2 | ✅ COMPLETE | `soft_delete()`, `is_deleted()`, `deleted_count()` |
| W16.3 | ✅ COMPLETE | Search tombstone filtering with `adjusted_k()` |
| W16.4 | ✅ COMPLETE | `compact()` returning `(HnswIndex, VectorStorage, CompactionResult)` |
| W16.5 | ✅ COMPLETE | Persistence format v0.3 with `deleted_count` header |

**Gate Status:** `.claude/GATE_16_COMPLETE.md` ✅
**Final Score:** 92/100 (APPROVED)

### Deferred Items from Week 16

| ID | Issue | Resolution |
|:---|:------|:-----------|
| C1 | WASM soft delete bindings | **W17.1** — Full WASM API implementation |
| C2 | Persistent compaction state flag | **DEFERRED v0.4** — Not needed for v0.3.0 |

### RFC-001 Week 17 Specification

From the approved RFC-001:

| Day | Task | Effort (RFC) | Adjusted |
|:----|:-----|:-------------|:---------|
| W17.1 | WASM bindings for delete API | 4h | 12h (3x) |
| W17.2 | Property tests | 4h | 12h (3x) |
| W17.3 | Fuzz tests | 4h | 8h (3x reduced — existing harness) |
| W17.4 | Performance benchmarks | 3h | 6h (3x) |
| W17.5 | Documentation | 3h | 6h (3x) |

---

## Task Overview

| Day | Task ID | Focus | Agent | Hours |
|:----|:--------|:------|:------|:------|
| **Day 1** | W17.1 | WASM soft delete bindings | WASM_SPECIALIST | 8h |
| **Day 2** | W17.2 | TypeScript types + integration tests | TEST_ENGINEER | 6h |
| **Day 3** | W17.3 | Example app + browser testing | WASM_SPECIALIST | 6h |
| **Day 4** | W17.4 | Release prep (version bump, changelog) | RUST_ENGINEER | 4h |
| **Day 5** | W17.5 | Documentation + npm/crates.io publish | DOCWRITER | 4h |

**Total Planned:** 28h + 12h buffer = 40h
**Buffer Allocation:** 30%

---

## Day 1: WASM Bindings (W17.1)

### W17.1: WASM Soft Delete API Implementation

**Agent:** WASM_SPECIALIST
**Estimate:** 8h (2.7h base × 3x)
**Priority:** P0 (Critical — Deferred from W16)

#### Objective

Expose all Week 16 soft delete APIs via WASM bindings, matching the TypeScript interface specified in RFC-001.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC17.1.1 | `soft_delete(vectorId: bigint): boolean` binding | TypeScript compilation |
| AC17.1.2 | `is_deleted(vectorId: bigint): boolean` binding | TypeScript compilation |
| AC17.1.3 | `deleted_count(): number` binding | Unit test |
| AC17.1.4 | `live_count(): number` binding | Unit test |
| AC17.1.5 | `tombstone_ratio(): number` binding | Unit test |
| AC17.1.6 | `needs_compaction(): boolean` binding | Unit test |
| AC17.1.7 | `compact(): CompactionResult` binding | Integration test |
| AC17.1.8 | `compaction_warning(): string \| null` binding | Unit test |
| AC17.1.9 | WASM bundle size < 500KB | `wasm-pack build --release` check |
| AC17.1.10 | All bindings documented in `pkg/README.md` | `grep -c 'softDelete\|isDeleted\|compact' pkg/README.md >= 6` |

#### API Specification (from RFC-001)

```typescript
interface EdgeVecIndex {
    // Existing methods (v0.2.x)
    insert(vector: Float32Array): bigint;
    search(query: Float32Array, k: number): SearchResult[];
    save(): Uint8Array;
    static load(data: Uint8Array): EdgeVecIndex;

    // NEW in v0.3.0 (W17.1)
    softDelete(vectorId: bigint): boolean;
    isDeleted(vectorId: bigint): boolean;
    deletedCount(): number;
    liveCount(): number;
    tombstoneRatio(): number;
    needsCompaction(): boolean;
    compactionWarning(): string | null;
    compact(): CompactionResult;
}

interface CompactionResult {
    tombstonesRemoved: number;
    newSize: number;
    durationMs: number;
}
```

#### Files to Modify

1. `src/wasm/mod.rs` — Add WASM bindings
2. `pkg/edgevec.d.ts` — TypeScript definitions
3. `pkg/README.md` — API documentation
4. `tests/wasm_bench.rs` — WASM test updates

#### Dependencies

- **Rust API:** `soft_delete()`, `is_deleted()`, `compact()` from Week 16 ✅
- **WASM infrastructure:** `wasm-bindgen`, `wasm-pack` ✅

#### Command

```
/wasm-bind soft_delete
```

**Details:** [DAY_1_TASKS.md](./DAY_1_TASKS.md)

---

## Day 2: Integration Tests (W17.2)

### W17.2: TypeScript Types + Integration Tests

**Agent:** TEST_ENGINEER
**Estimate:** 6h (2h base × 3x)
**Priority:** P0

#### Objective

Create comprehensive integration tests for WASM soft delete functionality, covering browser environments and edge cases.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC17.2.1 | TypeScript types compile without errors | `tsc --noEmit` |
| AC17.2.2 | Delete + search exclusion test | Integration test |
| AC17.2.3 | Double-delete idempotency test | Integration test |
| AC17.2.4 | Compact removes all tombstones test | Integration test |
| AC17.2.5 | Persist + reload preserves deletes test | Integration test |
| AC17.2.6 | needs_compaction() threshold test | Integration test |
| AC17.2.7 | All tests pass in Node.js | `npm test` |
| AC17.2.8 | Test coverage > 90% for new bindings | `npm test:coverage \| grep 'All files' >= 90` |

#### Test Cases (from RFC-001)

```typescript
describe('Soft Delete', () => {
    test('soft_delete marks vector as deleted', async () => {
        const index = new EdgeVecIndex(128);
        const id = index.insert(new Float32Array(128).fill(1.0));

        expect(index.isDeleted(id)).toBe(false);
        expect(index.softDelete(id)).toBe(true);
        expect(index.isDeleted(id)).toBe(true);
    });

    test('search excludes deleted vectors', async () => {
        const index = new EdgeVecIndex(128);
        const id1 = index.insert(new Float32Array(128).fill(1.0));
        const id2 = index.insert(new Float32Array(128).fill(0.9));

        index.softDelete(id1);

        const results = index.search(new Float32Array(128).fill(1.0), 10);
        expect(results.find(r => r.vectorId === id1)).toBeUndefined();
        expect(results.find(r => r.vectorId === id2)).toBeDefined();
    });

    test('compact removes tombstones', async () => {
        const index = new EdgeVecIndex(128);
        for (let i = 0; i < 100; i++) {
            index.insert(new Float32Array(128).fill(i / 100));
        }

        for (let i = 0; i < 30; i++) {
            index.softDelete(BigInt(i));
        }

        expect(index.deletedCount()).toBe(30);

        const result = index.compact();

        expect(result.tombstonesRemoved).toBe(30);
        expect(index.deletedCount()).toBe(0);
        expect(index.liveCount()).toBe(70);
    });
});
```

#### Files to Create/Modify

1. `wasm/tests/soft_delete.test.ts` — Integration tests
2. `wasm/tests/compaction.test.ts` — Compaction tests
3. `wasm/package.json` — Test dependencies

#### Command

```
/test-prop soft_delete_wasm
```

**Details:** [DAY_2_TASKS.md](./DAY_2_TASKS.md)

---

## Day 3: Example App (W17.3)

### W17.3: Browser Example + Cross-Browser Testing

**Agent:** WASM_SPECIALIST
**Estimate:** 6h (2h base × 3x)
**Priority:** P1

#### Objective

Create a working browser example demonstrating soft delete functionality and verify cross-browser compatibility.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC17.3.1 | `wasm/examples/soft_delete.html` example | Manual browser test |
| AC17.3.2 | Example shows insert → delete → search flow | Visual inspection |
| AC17.3.3 | Example shows compaction with progress | Visual inspection |
| AC17.3.4 | Works in Chrome 90+ | Browser testing |
| AC17.3.5 | Works in Firefox 90+ | Browser testing |
| AC17.3.6 | Works in Safari 15+ | Browser testing |
| AC17.3.7 | Works in Edge 90+ | Browser testing |
| AC17.3.8 | No console errors in any browser | Automated test: `window.onerror` count === 0 |
| AC17.3.9 | Memory warning for large compaction | Warning shown when vectors > 10k |

#### Example App Features

```html
<!-- wasm/examples/soft_delete.html -->
<h1>EdgeVec Soft Delete Demo</h1>

<section id="controls">
    <button onclick="insertVectors()">Insert 1000 Vectors</button>
    <button onclick="deleteRandom()">Delete Random 30%</button>
    <button onclick="searchSimilar()">Search</button>
    <button onclick="runCompaction()">Compact</button>
</section>

<section id="stats">
    <p>Total: <span id="total">0</span></p>
    <p>Live: <span id="live">0</span></p>
    <p>Deleted: <span id="deleted">0</span></p>
    <p>Tombstone Ratio: <span id="ratio">0%</span></p>
    <p>Needs Compaction: <span id="needs">No</span></p>
</section>

<section id="results">
    <h2>Search Results</h2>
    <ul id="result-list"></ul>
</section>
```

#### Command

```
/wasm-bind example_soft_delete
```

**Details:** [DAY_3_TASKS.md](./DAY_3_TASKS.md)

---

## Day 4: Release Prep (W17.4)

### W17.4: Version Bump + Changelog + Final Validation

**Agent:** RUST_ENGINEER
**Estimate:** 4h (1.3h base × 3x)
**Priority:** P0

#### Objective

Prepare v0.3.0 release: version bumps, changelog, final test pass, pre-release validation.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC17.4.1 | `Cargo.toml` version = "0.3.0" | `grep version Cargo.toml` |
| AC17.4.2 | `pkg/package.json` version = "0.3.0" | `grep version pkg/package.json` |
| AC17.4.3 | `CHANGELOG.md` updated with v0.3.0 section | Manual review |
| AC17.4.4 | All 400+ tests pass | `cargo test --all` |
| AC17.4.5 | Clippy clean | `cargo clippy -- -D warnings` |
| AC17.4.6 | `cargo doc` generates clean docs | `cargo doc 2>&1 \| grep -c warning == 0` |
| AC17.4.7 | WASM bundle builds cleanly | `wasm-pack build --release` |
| AC17.4.8 | Pre-release validation checklist complete | Gate document |

#### Changelog Entry

```markdown
## [0.3.0] - 2025-12-XX

### Added
- **Soft Delete:** `soft_delete()`, `is_deleted()`, `deleted_count()`, `live_count()`
- **Compaction:** `compact()`, `needs_compaction()`, `compaction_warning()`
- **Persistence v0.3:** `deleted_count` in header, `deleted` field per node
- **WASM API:** Full soft delete support in JavaScript/TypeScript

### Changed
- `HnswNode.pad` renamed to `HnswNode.deleted` (zero memory overhead)
- Snapshot format bumped to v0.3 (auto-migration from v0.2)

### Fixed
- N/A (feature release)

### Migration
- See `docs/MIGRATION.md` for v0.2 → v0.3 upgrade guide
- **Warning:** v0.3 snapshots cannot be read by v0.2.x
```

#### Command

```
/rust-implement W17.4
```

**Details:** [DAY_4_TASKS.md](./DAY_4_TASKS.md)

---

## Day 5: Documentation + Publish (W17.5)

### W17.5: Final Documentation + crates.io/npm Publish

**Agent:** DOCWRITER
**Estimate:** 4h (1.3h base × 3x)
**Priority:** P0

#### Objective

Complete documentation and publish v0.3.0 to crates.io and npm.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC17.5.1 | `README.md` updated with soft delete examples | Manual review |
| AC17.5.2 | `docs/API_REFERENCE.md` updated | Manual review |
| AC17.5.3 | `pkg/README.md` updated for npm | Manual review |
| AC17.5.4 | Rustdoc comments complete for all public APIs | `cargo doc` |
| AC17.5.5 | `cargo publish --dry-run` succeeds | Command output |
| AC17.5.6 | `npm publish --dry-run` succeeds | Command output |
| AC17.5.7 | Release tag `v0.3.0` created | `git tag -l` |
| AC17.5.8 | GitHub release with changelog | GitHub UI |
| AC17.5.9 | RFC-001 status updated to IMPLEMENTED | `grep 'Status: IMPLEMENTED' docs/rfcs/RFC-001-soft-delete.md` |

#### Documentation Sections

1. **Soft Delete Guide:**
   - When to use soft delete vs rebuild
   - Performance implications
   - Compaction strategy recommendations

2. **Migration Guide:**
   - v0.2 → v0.3 upgrade steps
   - Breaking changes (none for API)
   - Persistence format changes

3. **WASM Quick Start:**
   - Install from npm
   - Basic usage example
   - Soft delete example

#### Command

```
/doc-readme v0.3.0
```

**Details:** [DAY_5_TASKS.md](./DAY_5_TASKS.md)

---

## Risk Register

| ID | Risk | Probability | Impact | Mitigation |
|:---|:-----|:------------|:-------|:-----------|
| R17.1 | WASM binding compilation issues | LOW | MEDIUM | Existing WASM infrastructure proven |
| R17.2 | Browser compatibility gaps | MEDIUM | HIGH | 4-browser test matrix from W15 + quota detection test + graceful degradation test (Safari 50MB limit documented) |
| R17.3 | npm publish permission issues | LOW | LOW | Verify credentials before Day 5 |
| R17.4 | Integration test failures | MEDIUM | MEDIUM | Property tests catch edge cases |
| R17.5 | WASM bundle size exceeds 500KB | LOW | MEDIUM | Current bundle ~300KB |

---

## Dependencies

### Internal Dependencies

| Task | Depends On | Notes |
|:-----|:-----------|:------|
| W17.1 | W16.* | All Rust APIs complete ✅ |
| W17.2 | W17.1 | Needs WASM bindings |
| W17.3 | W17.1, W17.2 | Needs bindings + tests |
| W17.4 | W17.1, W17.2 | All code complete |
| W17.5 | W17.4 | Release prep complete |

### Execution Order

```
W17.1 ──► W17.2 ──► W17.3
              │         │
              └────┬────┘
                   ▼
              W17.4 ──► W17.5
```

---

## Success Metrics

### Quality Gates

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| Unit Tests | +15 new WASM tests | `npm test` |
| Integration Tests | +8 browser tests | Manual + CI |
| Clippy | 0 warnings | `cargo clippy -- -D warnings` |
| WASM Size | < 500KB | Bundle size check |

### Performance Targets

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| WASM soft_delete() | < 10 μs | Browser benchmark |
| WASM compact() 10k | < 10 s | Browser benchmark |
| TypeScript compilation | < 5 s | `tsc --noEmit` |

### Release Targets

| Target | Status |
|:-------|:-------|
| crates.io v0.3.0 published | PENDING |
| npm edgevec@0.3.0 published | PENDING |
| GitHub release created | PENDING |
| All 4 browsers verified | PENDING |

---

## HOSTILE_REVIEWER Checkpoints

| Day | Artifact | Review Focus |
|:----|:---------|:-------------|
| Day 1 | `src/wasm/mod.rs` | API completeness, error handling |
| Day 2 | `wasm/tests/*.test.ts` | Test coverage, edge cases |
| Day 3 | `wasm/examples/soft_delete.html` | Browser compatibility |
| Day 4 | `CHANGELOG.md`, `Cargo.toml` | Version consistency |
| Day 5 | Final release | Publish validation |

---

## Week 18 Preview

**Theme:** v0.4.0 Feature Planning

**Potential Tasks:**

| Task | Description |
|:-----|:------------|
| W18.1 | Persistent compaction state flag |
| W18.2 | Streaming compaction for large indices |
| W18.3 | Batch delete API optimization |
| W18.4 | Auto-compaction with configurable strategy |
| W18.5 | WASM Worker thread for non-blocking compact |

**Prerequisite:** Week 17 complete and v0.3.0 published

---

## Appendix: Detailed Task Files

- [DAY_1_TASKS.md](./DAY_1_TASKS.md) — WASM Bindings
- [DAY_2_TASKS.md](./DAY_2_TASKS.md) — Integration Tests
- [DAY_3_TASKS.md](./DAY_3_TASKS.md) — Example App
- [DAY_4_TASKS.md](./DAY_4_TASKS.md) — Release Prep
- [DAY_5_TASKS.md](./DAY_5_TASKS.md) — Documentation + Publish

---

**Status:** [REVISED]
**Revision:** 2025-12-15 — Addressed HOSTILE_REVIEWER conditions
**Next:** HOSTILE_REVIEWER re-approval → Week 17 execution
