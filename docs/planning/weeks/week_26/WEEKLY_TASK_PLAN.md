# Week 26: Core Metadata Implementation

**Date:** Dec 21-27, 2025
**Focus:** RFC-002 Phase 1 — Core Metadata Storage
**Estimated Duration:** 32 hours
**Target:** v0.6.0-alpha.1

---

## Week Objective

Implement integrated metadata storage in HNSW index with Rust API, following RFC-002 approved design. No WASM bindings yet (deferred to Week 28).

---

## Prerequisites

| Requirement | Status | Reference |
|:------------|:-------|:----------|
| RFC-002 Metadata Storage | APPROVED | `docs/rfcs/RFC-002_METADATA_STORAGE.md` |
| RFC-002 Implementation Plan | APPROVED | `docs/rfcs/RFC-002_IMPLEMENTATION_PLAN.md` |
| RFC-002 Persistence Format | APPROVED | `docs/rfcs/RFC-002_PERSISTENCE_FORMAT.md` |
| Week 25 Gate | COMPLETE | `docs/reviews/2025-12-20_W25_DAY6_DELIVERABLES_APPROVED.md` |

---

## Daily Breakdown

### Day 1: HnswIndex + insert_with_metadata() (8 hours)

#### W26.1.1: Add metadata field to HnswIndex (4 hours)

**Objective:** Integrate existing MetadataStore into HnswIndex struct.

**Acceptance Criteria:**
- [ ] `metadata: MetadataStore` field added to `HnswIndex`
- [ ] `HnswIndex::new()` initializes empty MetadataStore
- [ ] `HnswIndex::with_config()` initializes empty MetadataStore
- [ ] Existing tests still pass

**Files:**
- `src/hnsw/graph.rs`

**Agent:** RUST_ENGINEER

---

#### W26.1.2: Implement insert_with_metadata() (4 hours)

**Objective:** Atomic vector + metadata insert with fail-fast validation.

**Acceptance Criteria:**
- [ ] `insert_with_metadata(&mut self, storage: &mut VectorStorage, vector: &[f32], metadata: HashMap<String, MetadataValue>) -> Result<VectorId, GraphError>` implemented
- [ ] Validates metadata limits (64 keys, 256B key name, 64KB value)
- [ ] Rolls back on metadata validation failure
- [ ] Unit tests for success and failure paths

**Files:**
- `src/hnsw/operations.rs`
- `tests/metadata_insert.rs`

**Agent:** RUST_ENGINEER

---

### Day 2: soft_delete + compact + search_filtered (8 hours)

#### W26.2.1: Modify soft_delete() for metadata cleanup (2 hours)

**Objective:** Automatically remove metadata when vector is soft-deleted.

**Acceptance Criteria:**
- [ ] `soft_delete()` calls `metadata.remove(id)`
- [ ] No orphaned metadata after delete
- [ ] Unit test verifies metadata removal

**Files:**
- `src/hnsw/operations.rs`
- `tests/metadata_delete.rs`

**Agent:** RUST_ENGINEER

---

#### W26.2.2: Modify compact() for metadata (2 hours)

**Objective:** Remove metadata for compacted (tombstoned) vectors.

**Acceptance Criteria:**
- [ ] `compact()` removes metadata for all compacted IDs
- [ ] Metadata store shrinks after compaction
- [ ] Unit test verifies metadata compaction

**Files:**
- `src/hnsw/operations.rs`
- `tests/metadata_compact.rs`

**Agent:** RUST_ENGINEER

---

#### W26.2.3: Implement search_filtered() basic (4 hours)

**Objective:** Post-filtering with adaptive overfetch per RFC-002 §3.2.

**Acceptance Criteria:**
- [ ] `search_filtered(&self, storage: &VectorStorage, query: &[f32], filter: &str, k: usize) -> Result<Vec<(VectorId, f32)>, GraphError>` implemented
- [ ] Parses filter using existing Filter module
- [ ] Applies overfetch factor based on selectivity estimate
- [ ] Returns top-k results passing filter
- [ ] Unit tests for various filter expressions

**Files:**
- `src/hnsw/search.rs`
- `tests/metadata_search.rs`

**Agent:** RUST_ENGINEER

---

### Day 3: Selectivity Estimation + Unit Tests (8 hours)

#### W26.3.1: Implement selectivity estimation (4 hours)

**Objective:** Estimate filter selectivity for adaptive overfetch.

**Acceptance Criteria:**
- [ ] `estimate_selectivity(filter: &Filter) -> f64` implemented
- [ ] Equality: ~0.10, Range: ~0.30, AND: product, OR: sum - product
- [ ] Default: 0.50 for unknown patterns
- [ ] Overfetch factor: min(10, max(2, 1 / selectivity))
- [ ] Unit tests for various filter types

**Files:**
- `src/filter/selectivity.rs`
- `tests/selectivity.rs`

**Agent:** RUST_ENGINEER

---

#### W26.3.2: Unit tests for metadata operations (4 hours)

**Objective:** Comprehensive test coverage for all new metadata APIs.

**Acceptance Criteria:**
- [ ] `insert_with_metadata` tests: success, validation failure, rollback
- [ ] `soft_delete` tests: metadata removed, no orphans
- [ ] `compact` tests: metadata compacted
- [ ] `search_filtered` tests: various filters, edge cases
- [ ] All tests pass with `cargo test`

**Files:**
- `tests/metadata_*.rs`

**Agent:** TEST_ENGINEER

---

### Day 4: Persistence v0.4 Format (6 hours)

#### W26.4.1: Add MetadataSectionHeader struct (2 hours)

**Objective:** Define metadata section header per RFC-002 Persistence Format.

**Acceptance Criteria:**
- [ ] `MetadataSectionHeader` struct (16 bytes) defined
- [ ] `Pod + Zeroable` derives for bytemuck
- [ ] Compile-time size/alignment asserts
- [ ] Magic: "META", version: 1, format: 1 (Postcard)

**Files:**
- `src/persistence/header.rs`

**Agent:** RUST_ENGINEER

---

#### W26.4.2: Implement Postcard serialization (4 hours)

**Objective:** Serialize/deserialize MetadataStore with Postcard.

**Acceptance Criteria:**
- [ ] `MetadataStore::to_postcard() -> Result<Vec<u8>, SerializationError>` implemented
- [ ] `MetadataStore::from_postcard(bytes: &[u8]) -> Result<Self, SerializationError>` implemented
- [ ] Round-trip tests pass
- [ ] CRC32 validation implemented

**Files:**
- `src/metadata/serialize.rs`
- `tests/metadata_serialize.rs`

**Dependencies:** `postcard` crate

**Agent:** RUST_ENGINEER

---

### Day 5: Persistence Read/Write + Migration (8 hours)

#### W26.5.1: Update write_snapshot for v0.4 (4 hours)

**Objective:** Write metadata section to snapshot file.

**Acceptance Criteria:**
- [ ] `version_minor` set to 4
- [ ] `HAS_METADATA` flag set if metadata non-empty
- [ ] Metadata section appended after tombstone bitvec
- [ ] CRC calculated and stored in header

**Files:**
- `src/persistence/snapshot.rs`

**Agent:** RUST_ENGINEER

---

#### W26.5.2: Update read_snapshot for v0.4 (4 hours)

**Objective:** Read metadata section from v0.4 snapshot files.

**Acceptance Criteria:**
- [ ] Detects v0.3 vs v0.4 format
- [ ] Loads metadata section if `HAS_METADATA` flag set
- [ ] Validates CRC before deserializing
- [ ] Returns empty MetadataStore for v0.3 files

**Files:**
- `src/persistence/snapshot.rs`
- `tests/persistence_v04.rs`

**Agent:** RUST_ENGINEER

---

#### W26.5.3: v0.3 → v0.4 migration tests (2 hours)

**Objective:** Verify backward compatibility with v0.3 files.

**Acceptance Criteria:**
- [ ] Load v0.3 file, verify empty metadata
- [ ] Save as v0.4, reload, verify still empty
- [ ] Add metadata, save, reload, verify metadata present
- [ ] Integration test with real v0.3 file

**Files:**
- `tests/migration_v03_v04.rs`

**Agent:** TEST_ENGINEER

---

## Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | CI green |
| v0.3 → v0.4 migration works | `tests/migration_v03_v04.rs` passes |
| `search_filtered()` returns correct results | `tests/metadata_search.rs` passes |
| Clippy clean | `cargo clippy -- -D warnings` |
| Formatted | `cargo fmt --check` |

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|:-----|:------------|:-------|:-----------|
| Postcard compatibility issues | Low | Medium | Use stable version, test round-trip |
| Selectivity estimation inaccuracy | Medium | Low | Conservative default (50%) |
| Migration edge cases | Low | Medium | Extensive test coverage |

---

## Dependencies

| Crate | Version | Purpose |
|:------|:--------|:--------|
| `postcard` | ^1.0 | Binary serialization |
| `crc32fast` | ^1.3 | CRC validation |

---

## Blocked Tasks

None.

---

## Week 26 Checklist

- [x] Day 1: HnswIndex + insert_with_metadata()
- [x] Day 2: soft_delete + compact + search_filtered
- [x] Day 3: Selectivity estimation + unit tests
- [x] Day 4: Persistence v0.4 format
- [x] Day 5: Persistence read/write + migration

---

*Agent: PLANNER*
*Status: [APPROVED] - Week 26 Complete (2025-12-21)*

