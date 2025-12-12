# W1.5 Persistence — Issue Tracking Sheet

**Date:** 2025-12-06  
**Status:** 🔴 REJECTED  
**Tracking:** 10 issues total (3 critical, 2 major, 5 minor)

---

## Critical Issues

### C1: Duplicate FileHeader Definitions

**Priority:** P0 — BLOCKING  
**Severity:** Critical  
**Status:** 🔴 Open

**Description:**  
Two conflicting `FileHeader` struct definitions exist in codebase.

**Locations:**
- `src/persistence/header.rs:31`
- `src/persistence/writer.rs:25`

**Root Cause:**  
Parallel development without consolidation.

**Impact:**
- Maintenance burden (two sources of truth)
- Potential version conflicts
- API consumer confusion

**Resolution Plan:**
1. Delete `FileHeader` from `writer.rs` (lines 4-68)
2. Update `writer.rs` to import from `header` module:
   ```rust
   use super::header::{FileHeader, MAGIC, VERSION_MAJOR, VERSION_MINOR};
   ```
3. Update `persistence/mod.rs` exports:
   ```rust
   pub use header::{FileHeader, MAGIC, VERSION_MAJOR, VERSION_MINOR};
   ```
4. Verify all tests pass
5. Verify no compilation errors

**Assignee:** RUST_ENGINEER  
**Estimated Time:** 30 minutes  
**Blocked By:** None  
**Blocks:** C2, C3, M1

---

### C2: Version Number Inconsistency

**Priority:** P0 — BLOCKING  
**Severity:** Critical  
**Status:** 🔴 Open

**Description:**  
Version numbers differ between implementations.

**Evidence:**
- `header.rs:9` → `VERSION_MAJOR: u8 = 0`
- `writer.rs:73` → `VERSION_MAJOR: u8 = 1`

**Impact:**  
Files written with one version cannot be read by the other. Data corruption risk.

**Resolution Plan:**
1. Decide on canonical version (recommend `0.1` for pre-release)
2. Update `DATA_LAYOUT.md` to document authoritative version
3. Use single constant from `header.rs`
4. Update tests to verify version matching
5. Add test case for forward compatibility check

**Assignee:** META_ARCHITECT (spec update) + RUST_ENGINEER (code)  
**Estimated Time:** 20 minutes  
**Blocked By:** C1  
**Blocks:** Week 2 Planning

---

### C3: 11 Clippy Errors Block CI

**Priority:** P0 — BLOCKING  
**Severity:** Critical  
**Status:** 🔴 Open

**Description:**  
`cargo clippy -- -D warnings` fails with 11 errors.

**Breakdown:**

| Location | Error | Fix |
|:---------|:------|:----|
| `reader.rs:14` | Missing `# Errors` section | Add documentation |
| `reader.rs:29` | Missing backticks `FileHeader` | Add backticks |
| `reader.rs:38` | Missing `# Panics` section | Add documentation |
| `reader.rs:38` | Missing `# Errors` section | Add documentation |
| `writer.rs:77` | Missing backticks `FileHeader` | Add backticks |
| `writer.rs:78` | Missing `#[must_use]` | Add attribute |
| `writer.rs:97` | Missing `#[must_use]` | Add attribute |
| `writer.rs:145` | Missing `#[must_use]` | Add attribute |
| `hnsw/config.rs:8` | Missing backticks `ef_construction` | Add backticks |
| `hnsw/config.rs:9` | Missing backticks `ef_search` | Add backticks |
| `hnsw/config.rs:41` | Missing `#[must_use]` | Add attribute |

**Resolution Plan:**
1. Fix documentation issues (4 instances)
2. Add `#[must_use]` attributes (4 instances)
3. Fix markdown backticks (3 instances)
4. Verify `cargo clippy -- -D warnings` passes
5. Add CI check to prevent regressions

**Assignee:** RUST_ENGINEER  
**Estimated Time:** 45 minutes  
**Blocked By:** C1 (must fix writer.rs first)  
**Blocks:** CI Merge

---

## Major Issues

### M1: unwrap() in Library Code

**Priority:** P1 — HIGH  
**Severity:** Major  
**Status:** 🔴 Open

**Description:**  
Public library functions contain `.unwrap()` calls that can panic.

**Locations:**
- `reader.rs:47` — `data[0..4].try_into().unwrap()`
- `reader.rs:56` — `u32::from_le_bytes(data[44..48].try_into().unwrap())`
- `reader.rs:83-92` — Multiple additional instances

**Impact:**  
Denial of service via crafted input. Violates "no panics in library code" standard.

**Defense Argument:**  
"Length is checked at line 39, so slices are valid."

**Counter-Argument:**  
`.try_into()` theoretically can fail. Even if unlikely, panic path exists.

**Resolution Plan:**
1. Rewrite slice extraction to be infallible:
   ```rust
   // Before:
   let magic: [u8; 4] = data[0..4].try_into().unwrap();
   
   // After:
   let mut magic = [0u8; 4];
   magic.copy_from_slice(&data[0..4]);
   ```
2. Apply to all 10+ instances
3. Verify no panics possible via proptest

**Assignee:** RUST_ENGINEER  
**Estimated Time:** 1 hour  
**Blocked By:** C1  
**Blocks:** Security Review

---

### M2: Benchmark Results Not Documented

**Priority:** P1 — HIGH  
**Severity:** Major  
**Status:** 🔴 Open

**Description:**  
Benchmark exists but performance not validated against specification.

**Evidence:**
- `benches/persistence_bench.rs` runs successfully
- Comment claims "< 1 microsecond" but no validation
- No benchmark report exists

**Impact:**  
Performance regressions can occur silently.

**Resolution Plan:**
1. Run benchmark: `cargo bench --bench persistence_bench`
2. Capture output
3. Create `docs/benchmarks/week1_persistence_report.md`
4. Include:
   - Hardware specifications
   - Measured latency (P50, P99)
   - Comparison to target (< 1μs)
   - Throughput calculation
5. Add to documentation

**Assignee:** BENCHMARK_SCIENTIST  
**Estimated Time:** 30 minutes  
**Blocked By:** None (can run in parallel)  
**Blocks:** Performance Validation

---

## Minor Issues

### m1: Module Organization Leaks Internal Types

**Priority:** P2 — MEDIUM  
**Severity:** Minor  
**Status:** 🔴 Open

**Location:** `persistence/mod.rs:9`

**Description:**  
Module exports `FileHeader` from `writer`, but `header.rs` also defines it.

**Resolution:**  
Export from canonical location after C1 is resolved.

**Estimated Time:** 5 minutes  
**Blocked By:** C1

---

### m2: Missing Safety Documentation

**Priority:** P2 — MEDIUM  
**Severity:** Minor  
**Status:** 🔴 Open

**Location:** `header.rs:155`

**Description:**  
`bytemuck::try_from_bytes` has unsafe internals but lacks `# Safety` doc.

**Resolution:**  
Add documentation:
```rust
/// # Safety
///
/// This function is safe because:
/// - Buffer length is verified to be exactly 64 bytes
/// - FileHeader is Pod + Zeroable (all bit patterns valid)
/// - bytemuck::try_from_bytes validates alignment
```

**Estimated Time:** 10 minutes  
**Blocked By:** None

---

### m3: Magic Number Constant Duplication

**Priority:** P2 — MEDIUM  
**Severity:** Minor  
**Status:** 🔴 Open

**Locations:**
- `header.rs:6`
- `writer.rs:71`

**Resolution:**  
Will be resolved by C1 (consolidation).

**Estimated Time:** 0 minutes (fixed by C1)  
**Blocked By:** C1

---

### m4: E2E Test Temp File Collision Risk

**Priority:** P3 — LOW  
**Severity:** Minor  
**Status:** 🔴 Open

**Location:** `tests/e2e_empty_file.rs:9`

**Description:**  
Uses process ID for temp file naming, theoretically allowing collisions.

**Resolution:**  
Replace with `tempfile` crate:
```rust
use tempfile::NamedTempFile;

let temp_file = NamedTempFile::new().expect("Failed to create temp file");
let path = temp_file.path();
```

**Estimated Time:** 15 minutes  
**Blocked By:** None

---

### m5: CRC Error Message Inversion

**Priority:** P3 — LOW  
**Severity:** Minor  
**Status:** 🔴 Open

**Location:** `reader.rs:68-71`

**Description:**  
Error message swaps expected/actual semantics.

**Current:**
```rust
Err(PersistenceError::ChecksumMismatch {
    expected: calculated_crc,  // Wrong: this is what we computed
    actual: stored_crc,        // Wrong: this is what we expected
});
```

**Correct:**
```rust
Err(PersistenceError::ChecksumMismatch {
    expected: stored_crc,      // What we expected (from file)
    actual: calculated_crc,    // What we actually got (computed)
});
```

**Estimated Time:** 5 minutes  
**Blocked By:** None

---

## Issue Dependency Graph

```
C1 (FileHeader Consolidation)
 ├─► C2 (Version Conflict)
 ├─► C3 (Clippy Errors)
 ├─► M1 (unwrap removal)
 └─► m1 (Module exports)
     └─► m3 (Magic duplication)

M2 (Benchmark docs) — Independent, can run in parallel

m2 (Safety docs) — Independent
m4 (Temp files) — Independent
m5 (Error message) — Independent
```

---

## Work Breakdown

### Phase 1: Critical Structural (2 hours)
1. C1: Consolidate FileHeader (30 min)
2. C2: Fix version conflict (20 min)
3. C3: Fix clippy errors (45 min)
4. Verify compilation (15 min)

### Phase 2: Safety & Quality (1.5 hours)
5. M1: Remove unwrap() (60 min)
6. M2: Document benchmarks (30 min)

### Phase 3: Polish (30 minutes)
7. m1: Fix exports (5 min)
8. m2: Add safety docs (10 min)
9. m4: Use tempfile crate (15 min)
10. m5: Fix error message (5 min)

**Total Estimated Time:** 4 hours

---

## Testing Checklist

After fixes, verify:
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo bench --bench persistence_bench` runs
- [ ] No `unwrap()` in `src/persistence/`
- [ ] Only one `FileHeader` definition exists
- [ ] All tests still pass after refactor

---

## Resubmission Criteria

**Gate clears when:**
- All 3 critical issues resolved
- Both major issues resolved
- At least 3/5 minor issues resolved (or justified deferrals)
- All tests pass
- Clippy clean

---

*Issue Tracking Document*  
*Generated: 2025-12-06*  
*Next Review: Upon [REVISED] submission*

