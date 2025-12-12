# Week 1 - Day 4: Distance Metrics Implementation

**Objective:** Implement and verify the core mathematical distance metrics required for HNSW.

---

## Morning: RUST_ENGINEER (Implementation)

- [ ] Implement `DistanceMetric` trait in `src/metric/mod.rs`.
    - [ ] Requirement: `no_std` compatible.
    - [ ] Requirement: Trait object safe (optional but good for flexibility).
- [ ] Implement `L2Squared` metric.
    - [ ] Reference: `DATA_LAYOUT.md` Section 5.1.
    - [ ] Input: `&[f32]`.
    - [ ] Output: `f32` (squared distance).
- [ ] Implement `DotProduct` metric.
    - [ ] Reference: `DATA_LAYOUT.md` Section 5.2.
- [ ] Implement `Hamming` metric.
    - [ ] **SALVAGE AUTHORIZED:** `similarity.rs` (lines 286-292).
    - [ ] Input: `&[u8]` (packed binary).
    - [ ] Requirement: Use `count_ones()` intrinsic.

## Afternoon: TEST_ENGINEER (Verification)

- [ ] Implement `TEST-METRIC-001`: L2 Squared properties.
    - [ ] Identity: d(x, x) == 0.
    - [ ] Symmetry: d(x, y) == d(y, x).
    - [ ] Non-negativity: d(x, y) >= 0.
- [ ] Implement `TEST-METRIC-002`: Dot Product properties.
- [ ] Implement `TEST-METRIC-003`: Hamming properties.
    - [ ] Verify against naive bit-loop implementation.
- [ ] **SIMD Verification:** Ensure auto-vectorization (check assembly output if possible, or benchmark).

## Constraints

- [ ] **Handling NaN:** Panics on `NaN` input.
- [ ] **Handling Inf:** Propagates `Inf`.
- [ ] **Dimensions:** Mismatched dimensions must panic (enforce invariant INV-API-2 at low level).

## Deliverables

- [ ] `src/metric/mod.rs`
- [ ] `src/metric/l2.rs`
- [ ] `src/metric/dot.rs`
- [ ] `src/metric/hamming.rs`
- [ ] Tests in `tests/test_metrics.rs`
