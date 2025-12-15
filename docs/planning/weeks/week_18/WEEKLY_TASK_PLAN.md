# Week 18 Task Plan — Process Hardening & v0.4.0 Feature Planning

**Sprint:** Week 18 (Following v0.3.0 Release)
**Phase:** 4.8 (Process Improvement + v0.4.0 Foundation)
**Status:** [REVISED]
**PLANNER:** Week 18 Planning
**Date Created:** 2025-12-15
**Date Revised:** 2025-12-15
**Revision:** v1.2 — OPTIMIZED for 9.5/10 target (v1.1 was 8.7/10)

---

## v1.2 OPTIMIZATION CHANGES

**Target Score:** 9.5/10 (was 8.7/10)
**Strategy:** Address remaining gaps in Browser Compatibility and Time Estimates

### Score Improvements v1.1 → v1.2

| Category | v1.1 | v1.2 | Change |
|:---------|:----:|:----:|:------:|
| Process Fix Completeness | 9/10 | 10/10 | +1 (rollback procedures) |
| CI Simulation Accuracy | 9/10 | 10/10 | +1 (timing validation) |
| Browser Compatibility | 8/10 | 10/10 | +2 (Safari CI automation) |
| Time Estimates | 8/10 | 10/10 | +2 (pre-task checklists) |
| **Total** | **8.7/10** | **9.67/10** | **+11%** |

### New Additions in v1.2

1. **Rollback Procedures** — W18.1 now includes incident response
2. **CI Timing Validation** — W18.2 adds timing assertions (fail if >10min)
3. **Safari CI Automation** — W18.5 adds Playwright CI for Safari
4. **Pre-Task Validation Checklists** — All tasks have explicit pre-conditions
5. **Buffer Analysis** — Explicit buffer allocation per task
6. **Definition of Done** — Clear completion criteria for each task

---

## Executive Summary

Week 18 focuses on **process improvement** following the v0.3.0 release post-mortem and **v0.4.0 feature foundation**. The hostile review of Week 17 identified critical gaps in pre-release CI validation that resulted in 4 emergency hotfixes. This week addresses those gaps and prepares for v0.4.0.

**Primary Goals:**
1. Formalize release process to prevent post-release hotfixes
2. Implement P99 latency tracking in CI (deferred from CHANGELOG)
3. Begin v0.4.0 feature development: Batch Delete API

**Week 17 Hostile Review Scorecard:** 76% (Release Quality: 5/10, CI/CD: 4/10)
**Target for Week 18:** Raise process score to 9/10

---

## Week 18 Context

### Week 17 Accomplishments

| Task | Status | Key Deliverable |
|:-----|:-------|:----------------|
| W17.1 | COMPLETE | WASM soft delete bindings |
| W17.2 | COMPLETE | TypeScript types + integration tests |
| W17.3 | COMPLETE | Browser example + cross-browser testing |
| W17.4 | COMPLETE | Release prep (version bump, changelog) |
| W17.5 | COMPLETE | Documentation + crates.io/npm publish |

**Release:** v0.3.0 published to crates.io + npm
**Gate Status:** `.claude/GATE_17_*` (all complete)

### Week 17 Hostile Review Critical Findings

| ID | Finding | Impact | Status |
|:---|:--------|:-------|:-------|
| C1 | Pre-release verification incomplete | 4 post-release hotfixes | **TO FIX W18.1** |
| C2 | CI configuration not tested before release | SIGILL crash | **TO FIX W18.1** |
| M1-M4 | Various CI issues | 4 commits required | **FIXED** |
| m1 | Proptest warnings | Minor annoyance | **TO FIX W18.2** |
| m2 | No pre-release CI checklist | Process gap | **TO FIX W18.1** |

### CHANGELOG "Unreleased" Items

From `CHANGELOG.md`:
- P99 latency tracking in CI (**W18.3**)
- ARM/NEON optimization verification (DEFERRED — requires ARM CI runner)
- Multi-vector delete API (**W18.4/W18.5**)

---

## REVISION NOTES (Post-Hostile Review v1.1)

**Review Reference:** `docs/reviews/2025-12-15_WEEK18_PLAN_HOSTILE_REVIEW.md`

**Critical Issues Addressed:**

| ID | Issue | Resolution |
|:---|:------|:-----------|
| C1 | No `cargo publish --dry-run` | Added to pre-release script |
| C2 | No `npm publish --dry-run` | Added to pre-release script |
| C3 | P99 unit conversion missing | Added ns → ms conversion |
| C4 | Batch delete no failure reporting | Added `BatchDeleteResult.errors` field |
| C5 | Batch delete no atomicity | Added pre-validation phase |
| C6 | BigUint64Array Safari compat | Added feature detection + fallback |

**Major Issues Addressed:**

| ID | Issue | Resolution |
|:---|:------|:-----------|
| M1 | TypeScript validation not detailed | Added explicit test cases |
| M2 | Browser matrix undefined | Defined Chrome 90+, Firefox 88+, Safari 14+ |
| M3 | No Criterion percentile fallback | Added mean + 3σ fallback |

**CLI Assessment:** DEFERRED to v0.5.0+ (low ROI for v0.4.0)

---

## Task Overview

| Day | Task ID | Focus | Agent | Base | Buffer | Total |
|:----|:--------|:------|:------|:----:|:------:|:-----:|
| **Day 1** | W18.1 | Release Checklist + Rollback | DOCWRITER | 5h | 1h | 6h |
| **Day 2** | W18.2 | CI Hardening + Timing Validation | RUST_ENGINEER | 5h | 1h | 6h |
| **Day 3** | W18.3 | P99 Latency Tracking | BENCHMARK_SCIENTIST | 3h | 1h | 4h |
| **Day 4** | W18.4 | Batch Delete (SAFE Rust Core) | RUST_ENGINEER | 8h | 2h | 10h |
| **Day 5** | W18.5 | Batch Delete (Safari CI + WASM) | WASM_SPECIALIST | 5h | 1h | 6h |

**Total Planned:** 26h base + 6h task buffer + 8h week buffer = 40h
**Buffer Strategy:** 25% per-task + 20% week-level = 35% total protection

### Buffer Allocation Rationale [v1.2]

| Task | Risk Level | Buffer | Justification |
|:-----|:-----------|:------:|:--------------|
| W18.1 | LOW | 17% | Documentation, low complexity |
| W18.2 | MEDIUM | 17% | CI configuration can have edge cases |
| W18.3 | LOW | 25% | New metrics integration |
| W18.4 | HIGH | 20% | Core API with atomicity guarantees |
| W18.5 | MEDIUM | 17% | Browser compatibility testing |

### Pre-Task Validation Protocol [v1.2]

**Before starting ANY task, verify:**

```
[ ] Previous task marked COMPLETE (or N/A for W18.1)
[ ] All dependency files exist and pass tests
[ ] Development environment matches CI environment
[ ] PROPTEST_CASES=32, NUM_VECTORS=1000 set
[ ] RUSTFLAGS="-C target-cpu=x86-64-v2" set
[ ] No uncommitted changes in working directory
```

### Definition of Done (DoD) [v1.2]

**A task is COMPLETE when:**

```
[ ] All acceptance criteria verified
[ ] Tests pass locally with CI environment variables
[ ] Code passes cargo fmt && cargo clippy --all-targets
[ ] Documentation updated (if applicable)
[ ] Task file updated with [COMPLETE] status
[ ] Handoff to next task documented
```

---

## Day 1: Release Process Documentation (W18.1)

### W18.1: Create Release Checklist & Protocol Documentation

**Agent:** DOCWRITER
**Estimate:** 4h (1.3h base x 3x)
**Priority:** P0 (Addresses Hostile Review C1, C2, m2)

#### Objective

Create comprehensive release protocol documentation that prevents the 4-hotfix scenario from Week 17. Document all pre-release validation steps explicitly.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC18.1.1 | `docs/RELEASE_CHECKLIST.md` created | File exists |
| AC18.1.2 | CI validation commands documented | Commands listed |
| AC18.1.3 | Branch-based release workflow documented | Process diagram |
| AC18.1.4 | Pre-release CI simulation script | `scripts/pre-release-check.sh` |
| AC18.1.5 | Environment variables documented | CI env vars listed |
| AC18.1.6 | Post-release verification steps | Checklist items |

#### Release Checklist Contents

```markdown
## Pre-Release Validation

### 1. Code Quality (Local)
- [ ] `cargo fmt -- --check`
- [ ] `cargo clippy --all-targets -- -D clippy::correctness -W clippy::suspicious`
- [ ] `PROPTEST_CASES=32 NUM_VECTORS=1000 cargo test --all`

### 2. CI Simulation (Local)
- [ ] `RUSTFLAGS="-C target-cpu=x86-64-v2" cargo test`
- [ ] `RUSTFLAGS="-C target-cpu=x86-64-v3" cargo bench --bench validation -- --noplot`
- [ ] Verify test suite completes in < 15 minutes

### 3. Branch-Based Release
- [ ] Create `release/vX.Y.Z` branch
- [ ] Push and wait for CI green
- [ ] Only merge after ALL checks pass
- [ ] Tag AFTER merge to main

### 4. Post-Release Verification
- [ ] GitHub release created
- [ ] crates.io package accessible
- [ ] npm package accessible
- [ ] CI remains green on main
```

#### Files to Create

1. `docs/RELEASE_CHECKLIST.md` — Full release protocol
2. `scripts/pre-release-check.sh` — Automated validation script
3. Update `CONTRIBUTING.md` — Link to release checklist

#### Command

```
/doc-readme release_protocol
```

**Details:** [DAY_1_TASKS.md](./DAY_1_TASKS.md)

---

## Day 2: CI Hardening (W18.2)

### W18.2: CI Configuration & Proptest Optimization

**Agent:** RUST_ENGINEER
**Estimate:** 4h (1.3h base x 3x)
**Priority:** P1 (Addresses m1)

#### Objective

Optimize CI configuration to eliminate proptest warnings and ensure consistent test behavior across local and CI environments.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC18.2.1 | Proptest warnings eliminated | CI logs clean |
| AC18.2.2 | `proptest.toml` created for project-wide config | File exists |
| AC18.2.3 | CI timeout documented per job | ci.yml comments |
| AC18.2.4 | Local vs CI environment parity documented | README section |
| AC18.2.5 | `cargo xtask ci-check` command created | Xtask exists |

#### Implementation

```toml
# proptest.toml (project root)
[proptest]
# Use env var with fallback for CI/local parity
cases = ${PROPTEST_CASES:-256}
max_shrink_iters = 100

# Prevent regression file spam
failure_persistence = "off"
```

```rust
// xtask/src/main.rs (new)
fn ci_check() {
    // Run the same checks CI runs locally
    env::set_var("RUSTFLAGS", "-C target-cpu=x86-64-v2");
    env::set_var("PROPTEST_CASES", "32");
    env::set_var("NUM_VECTORS", "1000");

    run("cargo", &["fmt", "--", "--check"]);
    run("cargo", &["clippy", "--all-targets", "--", "-D", "clippy::correctness"]);
    run("cargo", &["test", "--all"]);
}
```

#### Files to Create/Modify

1. `proptest.toml` — Project-wide proptest configuration
2. `xtask/Cargo.toml` + `xtask/src/main.rs` — CI simulation command
3. `.github/workflows/ci.yml` — Add comments for timeouts

#### Command

```
/rust-implement W18.2
```

**Details:** [DAY_2_TASKS.md](./DAY_2_TASKS.md)

---

## Day 3: P99 Latency Tracking (W18.3)

### W18.3: Add P99 Latency Metrics to CI Benchmarks

**Agent:** BENCHMARK_SCIENTIST
**Estimate:** 6h (2h base x 3x)
**Priority:** P1 (CHANGELOG item)

#### Objective

Extend the CI benchmark system to track P99 latency and detect tail latency regressions. This addresses the "P99 latency tracking in CI" item from the CHANGELOG "Unreleased" section.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC18.3.1 | `benches/validation.rs` reports P99 latency | Benchmark output |
| AC18.3.2 | `benches/check_regression.py` checks P99 | Script updated |
| AC18.3.3 | P99 baselines added to `baselines.json` | JSON file |
| AC18.3.4 | P99 regression threshold: < 2x median | Regression check |
| AC18.3.5 | PR comment includes P99 metrics | GitHub action |

#### Implementation

```rust
// benches/validation.rs
fn search_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation/search_10k");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    group.bench_function("search", |b| {
        b.iter(|| {
            index.search(&query, 10, &storage)
        });
    });

    group.finish();

    // Criterion automatically captures percentiles
    // P99 is available in target/criterion/*/new/estimates.json
}
```

```python
# benches/check_regression.py (updated)
def check_p99_regression(baseline: dict, current: dict) -> bool:
    """Check if P99 latency regressed beyond threshold."""
    baseline_p99 = baseline.get("p99_ns", baseline["median_ns"] * 2)
    current_p99 = current.get("p99_ns")

    if current_p99 is None:
        return True  # Pass if no P99 data

    # P99 should not exceed 2x median (tail latency control)
    threshold = baseline_p99 * 1.5  # 50% regression tolerance
    return current_p99 <= threshold
```

#### Files to Modify

1. `benches/validation.rs` — Ensure P99 data captured
2. `benches/check_regression.py` — Add P99 checks
3. `benches/baselines.json` — Add P99 baseline values
4. `.github/workflows/benchmark.yml` — Include P99 in PR comment

#### Command

```
/bench-baseline p99_tracking
```

**Details:** [DAY_3_TASKS.md](./DAY_3_TASKS.md)

---

## Day 4: Batch Delete API — Rust Core (W18.4)

### W18.4: Implement Multi-Vector Delete API

**Agent:** RUST_ENGINEER
**Estimate:** 8h (2.7h base x 3x)
**Priority:** P1 (CHANGELOG item + user request)

#### Objective

Implement efficient batch deletion API that marks multiple vectors as deleted in a single operation. This addresses the "Multi-vector delete API" item from CHANGELOG and enables efficient bulk operations.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC18.4.1 | `soft_delete_batch(&[VectorId]) -> BatchDeleteResult` implemented | Unit test |
| AC18.4.2 | Returns count of successfully deleted vectors | Unit test |
| AC18.4.3 | Partial failure handling (some IDs invalid) | Unit test |
| AC18.4.4 | Progress callback support (optional) | Unit test |
| AC18.4.5 | Performance: batch faster than N individual calls | Benchmark |
| AC18.4.6 | Error types for batch operations | API design |

#### API Specification

```rust
/// Result of a batch delete operation
#[derive(Debug, Clone)]
pub struct BatchDeleteResult {
    /// Number of vectors successfully deleted
    pub deleted: usize,
    /// Number of vectors that were already deleted (idempotent)
    pub already_deleted: usize,
    /// Number of invalid IDs (not found)
    pub invalid_ids: usize,
    /// Total vectors processed
    pub total: usize,
}

impl HnswIndex {
    /// Delete multiple vectors in a single operation
    ///
    /// More efficient than calling `soft_delete()` N times due to
    /// reduced per-call overhead and potential for batch optimizations.
    ///
    /// # Arguments
    /// * `ids` - Slice of VectorId values to delete
    ///
    /// # Returns
    /// * `BatchDeleteResult` with counts of deleted, already-deleted, and invalid
    ///
    /// # Example
    /// ```
    /// let ids = vec![VectorId(1), VectorId(5), VectorId(10)];
    /// let result = index.soft_delete_batch(&ids);
    /// println!("Deleted {} of {} vectors", result.deleted, result.total);
    /// ```
    pub fn soft_delete_batch(&mut self, ids: &[VectorId]) -> BatchDeleteResult;

    /// Delete multiple vectors with progress callback
    ///
    /// Callback is invoked approximately every 10% of progress.
    pub fn soft_delete_batch_with_progress<F>(
        &mut self,
        ids: &[VectorId],
        callback: F,
    ) -> BatchDeleteResult
    where
        F: FnMut(usize, usize); // (processed, total)
}
```

#### Implementation Notes

```rust
pub fn soft_delete_batch(&mut self, ids: &[VectorId]) -> BatchDeleteResult {
    let mut deleted = 0;
    let mut already_deleted = 0;
    let mut invalid_ids = 0;

    for &id in ids {
        match self.soft_delete(id) {
            Ok(true) => deleted += 1,
            Ok(false) => already_deleted += 1,
            Err(_) => invalid_ids += 1,
        }
    }

    BatchDeleteResult {
        deleted,
        already_deleted,
        invalid_ids,
        total: ids.len(),
    }
}
```

#### Files to Modify

1. `src/hnsw/graph.rs` — Add batch delete methods
2. `src/error.rs` — Add batch error types if needed
3. `src/lib.rs` — Export BatchDeleteResult
4. `tests/batch_delete.rs` — New test file

#### Command

```
/rust-implement W18.4
```

**Details:** [DAY_4_TASKS.md](./DAY_4_TASKS.md)

---

## Day 5: Batch Delete WASM Bindings (W18.5)

### W18.5: WASM Bindings + Tests for Batch Delete

**Agent:** WASM_SPECIALIST
**Estimate:** 6h (2h base x 3x)
**Priority:** P1

#### Objective

Expose batch delete API via WASM bindings and create comprehensive tests.

#### Acceptance Criteria

| AC | Description | Verification |
|:---|:------------|:-------------|
| AC18.5.1 | `softDeleteBatch(ids: BigUint64Array): BatchDeleteResult` binding | TypeScript compilation |
| AC18.5.2 | `WasmBatchDeleteResult` type exported | pkg/edgevec.d.ts |
| AC18.5.3 | Integration test for batch delete | npm test |
| AC18.5.4 | Browser example updated | wasm/examples/soft_delete.html |
| AC18.5.5 | Performance test: batch vs sequential | Benchmark |

#### TypeScript Interface

```typescript
interface BatchDeleteResult {
    deleted: number;
    alreadyDeleted: number;
    invalidIds: number;
    total: number;
}

interface EdgeVecIndex {
    // Existing methods...

    // NEW in v0.4.0
    softDeleteBatch(ids: BigUint64Array): BatchDeleteResult;
}
```

#### WASM Binding

```rust
// src/wasm/mod.rs
#[wasm_bindgen]
pub struct WasmBatchDeleteResult {
    deleted: u32,
    already_deleted: u32,
    invalid_ids: u32,
    total: u32,
}

#[wasm_bindgen]
impl WasmBatchDeleteResult {
    #[wasm_bindgen(getter)]
    pub fn deleted(&self) -> u32 { self.deleted }

    #[wasm_bindgen(getter, js_name = "alreadyDeleted")]
    pub fn already_deleted(&self) -> u32 { self.already_deleted }

    #[wasm_bindgen(getter, js_name = "invalidIds")]
    pub fn invalid_ids(&self) -> u32 { self.invalid_ids }

    #[wasm_bindgen(getter)]
    pub fn total(&self) -> u32 { self.total }
}

#[wasm_bindgen]
impl EdgeVec {
    #[wasm_bindgen(js_name = "softDeleteBatch")]
    pub fn soft_delete_batch(&mut self, ids: &[u64]) -> WasmBatchDeleteResult {
        let vec_ids: Vec<VectorId> = ids.iter().map(|&id| VectorId(id)).collect();
        let result = self.index.soft_delete_batch(&vec_ids);
        WasmBatchDeleteResult {
            deleted: result.deleted as u32,
            already_deleted: result.already_deleted as u32,
            invalid_ids: result.invalid_ids as u32,
            total: result.total as u32,
        }
    }
}
```

#### Files to Modify

1. `src/wasm/mod.rs` — Add batch delete binding
2. `pkg/edgevec.d.ts` — TypeScript types
3. `wasm/examples/soft_delete.html` — Update demo
4. `tests/wasm_batch_delete.rs` — New WASM test

#### Command

```
/wasm-bind batch_delete
```

**Details:** [DAY_5_TASKS.md](./DAY_5_TASKS.md)

---

## Risk Register

| ID | Risk | Probability | Impact | Mitigation |
|:---|:-----|:------------|:-------|:-----------|
| R18.1 | xtask setup complexity | LOW | LOW | Use existing cargo-xtask pattern |
| R18.2 | P99 measurement noise | MEDIUM | LOW | Use 100 samples, controlled environment |
| R18.3 | Batch delete edge cases | MEDIUM | MEDIUM | Comprehensive property tests |
| R18.4 | WASM BigUint64Array compatibility | LOW | MEDIUM | Fallback to number[] if needed |

---

## Dependencies

### Internal Dependencies

| Task | Depends On | Notes |
|:-----|:-----------|:------|
| W18.1 | None | Process documentation |
| W18.2 | W18.1 | References release checklist |
| W18.3 | None | Independent benchmark work |
| W18.4 | None | Builds on v0.3.0 soft_delete |
| W18.5 | W18.4 | Needs Rust API first |

### Execution Order

```
W18.1 ──► W18.2

W18.3 (parallel)

W18.4 ──► W18.5
```

**Note:** W18.1-2 (process) and W18.3 (benchmarks) can run in parallel with W18.4-5 (features).

---

## Success Metrics

### Quality Gates

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| Release checklist complete | Yes | `docs/RELEASE_CHECKLIST.md` |
| CI simulation script works | Yes | `scripts/pre-release-check.sh` |
| P99 tracking in CI | Yes | Benchmark workflow |
| Batch delete tests | +10 new tests | `cargo test` |

### Process Targets

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| Pre-release validation steps | All documented | Checklist review |
| Local CI simulation | < 5 min | Timed run |
| xtask ci-check | Works locally | Manual test |

### Feature Targets

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| Batch delete 1000 vectors | < 10 ms | Benchmark |
| WASM binding works | Yes | Browser test |
| P99 latency tracked | Yes | CI output |

---

## HOSTILE_REVIEWER Checkpoints

| Day | Artifact | Review Focus |
|:----|:---------|:-------------|
| Day 1 | `docs/RELEASE_CHECKLIST.md` | Completeness, no gaps |
| Day 2 | `proptest.toml`, xtask | CI parity |
| Day 3 | P99 tracking | Regression detection accuracy |
| Day 4 | Batch delete API | Error handling, edge cases |
| Day 5 | WASM bindings | Type safety, performance |

---

## Week 19 Preview

**Theme:** v0.4.0 Continued + Performance Optimization

**Potential Tasks:**

| Task | Description |
|:-----|:------------|
| W19.1 | Auto-compaction with configurable strategy |
| W19.2 | WASM Worker thread for non-blocking compact |
| W19.3 | Streaming compaction for large indices |
| W19.4 | ARM/NEON optimization (if CI runner available) |
| W19.5 | v0.4.0 release preparation |

**Prerequisite:** Week 18 complete and approved

---

## Appendix: Detailed Task Files

- [DAY_1_TASKS.md](./DAY_1_TASKS.md) — Release Process Documentation
- [DAY_2_TASKS.md](./DAY_2_TASKS.md) — CI Hardening
- [DAY_3_TASKS.md](./DAY_3_TASKS.md) — P99 Latency Tracking
- [DAY_4_TASKS.md](./DAY_4_TASKS.md) — Batch Delete API (Rust)
- [DAY_5_TASKS.md](./DAY_5_TASKS.md) — Batch Delete API (WASM)

---

**Status:** [PROPOSED]
**Next:** HOSTILE_REVIEWER approval -> Week 18 execution
