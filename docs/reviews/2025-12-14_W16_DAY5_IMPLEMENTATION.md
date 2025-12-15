# W16.5 Persistence Format v3 Implementation Summary

**Date:** 2025-12-14
**Task:** W16.5 — Persistence Format v0.3
**Author:** RUST_ENGINEER

---

## Implementation Overview

Updated the snapshot persistence format from v0.1 to v0.3 to support soft-delete:
- Added `deleted_count` field to `FileHeader` (was `reserved`)
- `HnswNode.deleted` field is now persisted (was `pad`)
- Forward migration from v0.1/v0.2 → v0.3 is automatic

### Files Modified

1. **`src/persistence/header.rs`**
   - Bumped `VERSION_MINOR` from 1 to 3
   - Added `VERSION_MINOR_MIN = 1` for migration support
   - Renamed `reserved` field to `deleted_count`
   - Added `needs_migration()` and `supports_soft_delete()` methods

2. **`src/persistence/reader.rs`**
   - Updated to read `deleted_count` instead of `reserved`

3. **`src/persistence/chunking.rs`**
   - Updated to write `index.deleted_count` to header

4. **`src/persistence/snapshot.rs`**
   - Updated `read_snapshot()` to restore `deleted_count`
   - Added migration logic for v0.1/v0.2 files
   - Added consistency verification (header vs actual deleted count)

5. **`src/persistence/mod.rs`**
   - Exported `VERSION_MINOR` and `VERSION_MINOR_MIN`

6. **`tests/persistence_v3.rs`** (new)
   - 11 integration tests for v0.3 format

---

## Format Specification

### Header Layout (64 bytes)

```
Offset  Size  Field           Description
------  ----  --------------  ----------------------------------
0       4     magic           "EVEC" = [0x45, 0x56, 0x45, 0x43]
4       1     version_major   0
5       1     version_minor   3 (was 1)
6       2     flags           Feature flags
8       8     vector_count    Total vectors
16      8     index_offset    Offset to index data
24      8     metadata_offset Offset to metadata
32      8     rng_seed        RNG seed
40      4     dimensions      Vector dimensions
44      4     header_crc      CRC32 of header
48      4     hnsw_m          HNSW M parameter
52      4     hnsw_m0         HNSW M0 parameter
56      4     data_crc        CRC32 of data
60      4     deleted_count   Count of deleted nodes (was reserved)
```

### Node Layout (16 bytes)

```
Offset  Size  Field           Description
------  ----  --------------  ----------------------------------
0       8     vector_id       VectorId (u64)
8       4     neighbor_offset Offset into neighbor pool
12      2     neighbor_len    Length of neighbor data
14      1     max_layer       Maximum HNSW layer
15      1     deleted         0=live, 1=deleted (was pad)
```

---

## Migration Path

### v0.1/v0.2 → v0.3 (Automatic)

| Field | v0.1/v0.2 | v0.3 | Migration |
|:------|:----------|:-----|:----------|
| version_minor | 1 | 3 | Updated |
| header.reserved | 0 | deleted_count | Reinterpreted |
| node.pad | 0 | deleted | Reinterpreted |

**Key Insight:** Since v0.1/v0.2 files always had `reserved=0` and `pad=0`, the migration is automatic:
- `deleted_count=0` means no deletions (correct for old files)
- `deleted=0` means node is live (correct for old files)

### v0.3 → v0.1/v0.2 (NOT Supported)

v0.3 files cannot be read by older versions. Downgrade requires re-indexing.

---

## API Summary

### New FileHeader Methods

```rust
impl FileHeader {
    /// Returns true if migration from older format is needed
    pub fn needs_migration(&self) -> bool;

    /// Returns true if this format supports soft-delete (v0.3+)
    pub fn supports_soft_delete(&self) -> bool;
}
```

### New Constants

```rust
/// Current minor version (3)
pub const VERSION_MINOR: u8 = 3;

/// Minimum supported minor version for migration (1)
pub const VERSION_MINOR_MIN: u8 = 1;
```

---

## Test Coverage

| Test | Description | Status |
|:-----|:------------|:------:|
| `test_save_load_v3_roundtrip_no_deletes` | Basic roundtrip without deletions | PASS |
| `test_save_load_v3_roundtrip_with_deletes` | Roundtrip with deleted vectors | PASS |
| `test_save_v3_has_deleted_count_in_header` | Verify header contains deleted_count | PASS |
| `test_deleted_nodes_persist_correctly` | 100 vectors, 10 deleted | PASS |
| `test_crc32_checksum_catches_corruption` | Data corruption detected | PASS |
| `test_header_checksum_catches_corruption` | Header corruption detected | PASS |
| `test_search_works_after_reload_with_deletes` | Search excludes deleted after load | PASS |
| `test_single_vector_persistence` | Single vector roundtrip | PASS |
| `test_all_deleted_persistence` | All vectors deleted | PASS |
| `test_version_is_0_3` | Version constant is 3 | PASS |
| `test_large_deleted_count_persistence` | 1000 vectors, 500 deleted | PASS |

**Total:** 11/11 tests passing

---

## Acceptance Criteria Status

| AC | Description | Status | Evidence |
|:---|:------------|:------:|:---------|
| AC16.5.1 | Version bumped to 3 | PASS | `VERSION_MINOR = 3` |
| AC16.5.2 | `deleted_count` in header | PASS | `test_save_v3_has_deleted_count_in_header` |
| AC16.5.3 | `deleted` field persisted per node | PASS | `test_deleted_nodes_persist_correctly` |
| AC16.5.4 | v2 → v3 migration works | PASS | `needs_migration()` + auto-migration |
| AC16.5.5 | Backward compatibility documented | PASS | Format spec above |
| AC16.5.6 | CRC32 checksum still valid | PASS | `test_crc32_checksum_catches_corruption` |

---

## Full Test Results

```
Test suite: 394+ tests passed
Clippy: CLEAN (0 warnings on library code)
```

---

## Week 16 Completion Status

With W16.5 complete, Week 16 (Soft Delete) is DONE:

| Day | Task | Status |
|:----|:-----|:------:|
| Day 1 | HnswNode.deleted field | ✅ COMPLETE |
| Day 2 | soft_delete(), is_deleted() | ✅ COMPLETE |
| Day 3 | Search tombstone filtering | ✅ COMPLETE |
| Day 4 | compact() + insert_with_id() | ✅ COMPLETE |
| Day 5 | Persistence format v3 | ✅ COMPLETE |

---

**Status:** READY FOR HOSTILE REVIEW
