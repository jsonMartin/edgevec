# Day 4 Tasks — W17.4: Release Prep

**Date:** Week 17, Day 4
**Task ID:** W17.4
**Agent:** RUST_ENGINEER
**Estimate:** 4h (1.3h base × 3x)
**Priority:** P0
**Status:** PENDING

---

## Objective

Prepare v0.3.0 release: version bumps, changelog, final test pass, and pre-release validation checklist.

---

## Prerequisites

- [ ] W17.1 complete (WASM bindings)
- [ ] W17.2 complete (Integration tests)
- [ ] W17.3 complete (Browser testing)
- [x] All tests passing
- [x] Clippy clean

---

## Implementation Checklist

### 1. Version Bump in `Cargo.toml`

```toml
[package]
name = "edgevec"
version = "0.3.0"  # Bump from 0.2.x
edition = "2021"
description = "High-performance embedded vector database with WASM support"
license = "MIT OR Apache-2.0"
repository = "https://github.com/[user]/edgevec"
keywords = ["vector", "database", "hnsw", "wasm", "embedding"]
categories = ["database", "wasm", "algorithms"]
```

### 2. Version Bump in `pkg/package.json`

```json
{
  "name": "edgevec",
  "version": "0.3.0",
  "description": "High-performance embedded vector database for browser and Node.js",
  "main": "edgevec.js",
  "types": "edgevec.d.ts",
  "files": [
    "edgevec.js",
    "edgevec.d.ts",
    "edgevec_bg.wasm",
    "edgevec_bg.wasm.d.ts"
  ],
  "keywords": [
    "vector",
    "database",
    "hnsw",
    "wasm",
    "embedding",
    "similarity-search",
    "soft-delete"
  ]
}
```

### 3. Update CHANGELOG.md

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-12-XX

### Added

- **Soft Delete API (RFC-001)**
  - `soft_delete(VectorId)` — Mark vector as deleted (O(1))
  - `is_deleted(VectorId)` — Check if vector is deleted
  - `deleted_count()` — Count of tombstoned vectors
  - `live_count()` — Count of live vectors
  - `tombstone_ratio()` — Ratio of deleted to total vectors

- **Compaction API**
  - `compact()` — Rebuild index removing all tombstones
  - `needs_compaction()` — Check if threshold exceeded
  - `compaction_warning()` — Get warning message if compaction recommended
  - `set_compaction_threshold(ratio)` — Configure threshold (default 0.3)
  - `CompactionResult` — Statistics from compaction operation

- **WASM Bindings (v0.3.0)**
  - `softDelete(vectorId)` — JavaScript soft delete
  - `isDeleted(vectorId)` — JavaScript check
  - `deletedCount()` / `liveCount()` — Statistics
  - `tombstoneRatio()` / `needsCompaction()` — Thresholds
  - `compact()` — Browser compaction
  - `compactionWarning()` — User guidance

- **Persistence Format v0.3**
  - `deleted_count` field in snapshot header
  - `deleted` field in HnswNode (replaces padding)
  - Automatic migration from v0.2 snapshots

- **Documentation**
  - `docs/MIGRATION.md` — v0.2 → v0.3 upgrade guide
  - Browser example: `wasm/examples/soft_delete.html`

### Changed

- `HnswNode.pad` renamed to `HnswNode.deleted` (zero memory overhead)
- Snapshot version bumped to v0.3 (reads v0.2, writes v0.3)
- Search internally uses `adjusted_k()` to compensate for tombstones

### Fixed

- N/A (feature release)

### Migration Notes

- v0.2 snapshots are automatically migrated to v0.3 on load
- v0.3 snapshots **cannot** be read by v0.2.x (forward-incompatible)
- Always backup before upgrading
- See `docs/MIGRATION.md` for complete guide

---

## [0.2.1] - 2025-12-14

### Fixed
- Safety hardening based on community feedback
- Improved error messages

## [0.2.0] - 2025-12-XX

### Added
- WASM bindings via wasm-bindgen
- Batch insert API
- Scalar quantization (SQ8)

## [0.1.0] - 2025-12-XX

### Added
- Initial release
- HNSW algorithm implementation
- VectorStorage with arena allocation
- Basic persistence
```

### 4. Final Validation Checklist

Run all validation steps and document results:

```bash
# 1. Run all tests
cargo test --all
# Expected: 400+ tests pass

# 2. Run Clippy
cargo clippy -- -D warnings
# Expected: No warnings

# 3. Check formatting
cargo fmt -- --check
# Expected: Clean

# 4. Build documentation
cargo doc --no-deps
# Expected: No warnings

# 5. Build WASM
wasm-pack build --target web --release
# Expected: Success, bundle < 500KB

# 6. Run WASM tests
cd wasm && npm test
# Expected: All pass

# 7. Dry run crates.io publish
cargo publish --dry-run
# Expected: Success

# 8. Dry run npm publish
cd pkg && npm publish --dry-run
# Expected: Success
```

### 5. Create Pre-Release Validation Document

Create `.claude/RELEASE_VALIDATION_v0.3.0.md`:

```markdown
# Release Validation: v0.3.0

**Date:** 2025-12-XX
**Validator:** RUST_ENGINEER

## Test Results

| Test Suite | Count | Status |
|:-----------|------:|:-------|
| Unit tests | XXX | PASS |
| Integration tests | XX | PASS |
| Property tests | X | PASS |
| WASM tests | XX | PASS |
| **Total** | **XXX** | **PASS** |

## Quality Checks

| Check | Status | Notes |
|:------|:-------|:------|
| cargo clippy | PASS | 0 warnings |
| cargo fmt | PASS | Clean |
| cargo doc | PASS | 0 warnings |
| WASM bundle size | PASS | XXX KB < 500KB |

## Browser Compatibility

| Browser | Version | Status |
|:--------|:--------|:-------|
| Chrome | 90+ | PASS |
| Firefox | 90+ | PASS |
| Safari | 15+ | PASS |
| Edge | 90+ | PASS |

## Publish Dry Run

| Registry | Status |
|:---------|:-------|
| crates.io | PASS |
| npm | PASS |

## Sign-Off

- [ ] All tests pass
- [ ] All quality checks pass
- [ ] All browsers verified
- [ ] Changelog updated
- [ ] Version bumped
- [ ] Ready for publish

**Validator:** _______________
**Date:** _______________
```

---

## Acceptance Criteria Verification

| AC | Verification | Expected |
|:---|:-------------|:---------|
| AC17.4.1 | `grep version Cargo.toml` | "0.3.0" |
| AC17.4.2 | `grep version pkg/package.json` | "0.3.0" |
| AC17.4.3 | Review CHANGELOG.md | Complete |
| AC17.4.4 | `cargo test --all` | All pass |
| AC17.4.5 | `cargo clippy -- -D warnings` | Clean |
| AC17.4.6 | `cargo doc` | No warnings |
| AC17.4.7 | `wasm-pack build --release` | Success |
| AC17.4.8 | Checklist complete | YES |

---

## Output

### Artifacts Generated

- [ ] `Cargo.toml` — Version 0.3.0
- [ ] `pkg/package.json` — Version 0.3.0
- [ ] `CHANGELOG.md` — v0.3.0 section
- [ ] `.claude/RELEASE_VALIDATION_v0.3.0.md` — Validation checklist

### Status After Completion

```
✅ W17.4 COMPLETE
Next: W17.5 (Documentation + Publish)
```

---

**Status:** PENDING
**Next:** `/rust-implement W17.4`
