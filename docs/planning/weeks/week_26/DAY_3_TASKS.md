# Week 26 Day 3: Selectivity Estimation + Unit Tests

**Date:** 2025-12-23
**Focus:** Adaptive overfetch tuning and comprehensive test coverage
**Estimated Duration:** 8 hours
**Phase:** RFC-002 Core Metadata (Phase 1)

---

## Tasks

### W26.3.1: Implement selectivity estimation

**Objective:** Estimate filter selectivity for adaptive overfetch per RFC-002 §3.2.

**Acceptance Criteria:**
- [ ] `estimate_selectivity(filter: &Filter) -> f64` implemented
- [ ] Selectivity heuristics:
  - Equality: ~0.10 (10% of vectors match)
  - Range: ~0.30 (30% of vectors match)
  - AND compound: product of individual selectivities
  - OR compound: sum - product (union formula)
  - Default/Unknown: 0.50 (conservative)
- [ ] Return value always in range [0.0, 1.0]
- [ ] Overfetch factor formula: `min(10, max(2, 1.0 / selectivity))`
- [ ] Unit tests for each filter type

**Files:**
- `src/filter/selectivity.rs` (new file — selectivity estimation)
- `src/filter/mod.rs` (add module export)
- `tests/selectivity.rs` (new file — unit tests)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**API Signature:**
```rust
/// Estimates the selectivity of a filter expression.
///
/// Selectivity is the fraction of vectors expected to pass the filter.
/// - 0.0 means no vectors pass (very selective)
/// - 1.0 means all vectors pass (not selective)
///
/// # Arguments
/// * `filter` - The parsed filter expression
///
/// # Returns
/// Estimated selectivity in range [0.0, 1.0]
pub fn estimate_selectivity(filter: &Filter) -> f64 {
    match filter {
        Filter::Equals { .. } => 0.10,
        Filter::NotEquals { .. } => 0.90,
        Filter::LessThan { .. } | Filter::GreaterThan { .. } => 0.30,
        Filter::LessThanOrEqual { .. } | Filter::GreaterThanOrEqual { .. } => 0.35,
        Filter::Contains { .. } => 0.20,
        Filter::And(filters) => filters.iter()
            .map(estimate_selectivity)
            .product(),
        Filter::Or(filters) => {
            // P(A ∪ B) = P(A) + P(B) - P(A ∩ B)
            // For simplicity: sum - product (approximation)
            let sum: f64 = filters.iter().map(estimate_selectivity).sum();
            let product: f64 = filters.iter().map(estimate_selectivity).product();
            (sum - product).min(1.0)
        }
        Filter::Not(inner) => 1.0 - estimate_selectivity(inner),
        _ => 0.50, // Default for unknown patterns
    }
}

/// Calculates overfetch factor from selectivity.
///
/// Returns how many times k to fetch during HNSW search.
pub fn overfetch_factor(selectivity: f64) -> usize {
    let factor = (1.0 / selectivity).max(2.0).min(10.0);
    factor.ceil() as usize
}
```

**Implementation Notes:**
1. Selectivity is PESSIMISTIC — assume filter is more selective than it might be
2. This avoids under-fetching which would return fewer than k results
3. Conservative default (0.50) means 2x overfetch minimum
4. Cap at 10x overfetch to prevent memory issues on very selective filters

**Dependencies:** Existing Filter enum from `src/filter/`

---

### W26.3.2: Unit tests for metadata operations

**Objective:** Comprehensive test coverage for all new metadata APIs.

**Acceptance Criteria:**
- [ ] `insert_with_metadata` tests:
  - Success path: insert with valid metadata, verify retrieval
  - Failure path: too many keys (>64), verify error returned
  - Failure path: key name too long (>256 bytes), verify error
  - Failure path: value too large (>64KB), verify error
  - Rollback verification: on failure, no partial state (vector not inserted)
- [ ] `soft_delete` tests:
  - Metadata removed after delete
  - No orphans: delete vector without metadata works
  - Re-insert after delete: fresh metadata
- [ ] `compact` tests:
  - Metadata compacted with vectors
  - Multiple deletions then compact
  - Verify metadata.len() equals remaining vector count
- [ ] `search_filtered` tests:
  - Equality filter: `category = "books"`
  - Range filter: `price < 50`
  - AND compound: `category = "books" AND price < 50`
  - OR compound: `category = "books" OR category = "movies"`
  - Empty result: filter matches nothing
  - All match: filter matches everything
  - Edge case: filter on non-existent key
- [ ] All tests pass with `cargo test`
- [ ] Coverage report shows >90% for new code

**Files:**
- `tests/metadata_insert.rs` (extend from Day 1)
- `tests/metadata_delete.rs` (extend from Day 2)
- `tests/metadata_compact.rs` (extend from Day 2)
- `tests/metadata_search.rs` (extend from Day 2)
- `tests/selectivity.rs` (new — selectivity tests)

**Estimated Duration:** 4 hours

**Agent:** TEST_ENGINEER

**Test Structure:**
```rust
// tests/metadata_insert.rs
mod insert_with_metadata {
    #[test]
    fn success_with_valid_metadata() { ... }

    #[test]
    fn fails_with_too_many_keys() { ... }

    #[test]
    fn fails_with_key_too_long() { ... }

    #[test]
    fn fails_with_value_too_large() { ... }

    #[test]
    fn rollback_on_validation_failure() { ... }
}

// tests/metadata_delete.rs
mod soft_delete_metadata {
    #[test]
    fn removes_metadata_on_delete() { ... }

    #[test]
    fn handles_vector_without_metadata() { ... }

    #[test]
    fn fresh_metadata_after_reinsert() { ... }
}

// tests/metadata_compact.rs
mod compact_metadata {
    #[test]
    fn compacts_metadata_with_vectors() { ... }

    #[test]
    fn handles_multiple_deletions() { ... }

    #[test]
    fn metadata_count_matches_vector_count() { ... }
}

// tests/metadata_search.rs
mod search_filtered {
    #[test]
    fn filters_by_equality() { ... }

    #[test]
    fn filters_by_range() { ... }

    #[test]
    fn filters_by_and_compound() { ... }

    #[test]
    fn filters_by_or_compound() { ... }

    #[test]
    fn returns_empty_when_no_match() { ... }

    #[test]
    fn returns_all_when_all_match() { ... }

    #[test]
    fn handles_nonexistent_key() { ... }
}

// tests/selectivity.rs
mod selectivity {
    #[test]
    fn equality_is_selective() { ... }

    #[test]
    fn range_is_moderately_selective() { ... }

    #[test]
    fn and_multiplies_selectivities() { ... }

    #[test]
    fn or_uses_union_formula() { ... }

    #[test]
    fn overfetch_clamps_to_range() { ... }
}
```

**Dependencies:**
- W26.1.1, W26.1.2 (metadata field, insert_with_metadata)
- W26.2.1, W26.2.2, W26.2.3 (soft_delete, compact, search_filtered)
- W26.3.1 (selectivity estimation)

---

## Day 3 Checklist

- [ ] W26.3.1: Selectivity estimation implemented
- [ ] W26.3.2: Comprehensive unit tests written
- [ ] All existing tests pass (`cargo test`)
- [ ] All new tests pass (`cargo test selectivity metadata_`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Formatted (`cargo fmt --check`)

## Day 3 Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | CI green |
| Selectivity in [0, 1] | Property tests |
| >90% coverage for new code | `cargo tarpaulin` |
| Overfetch adaptive | search_filtered tests |

## Day 3 Handoff

After completing Day 3:

**Artifacts Generated:**
- New `src/filter/selectivity.rs`
- Modified `src/filter/mod.rs` (module export)
- Extended `tests/metadata_*.rs`
- New `tests/selectivity.rs`

**Status:** PENDING_DAY_4

**Next:** Day 4 — Persistence v0.4 format (MetadataSectionHeader + Postcard)

---

*Agent: RUST_ENGINEER + TEST_ENGINEER*
*Status: [PROPOSED]*
