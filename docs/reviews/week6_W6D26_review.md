# HOSTILE_REVIEWER: Rejection — W6D26 (Quantization Foundation)

**Date:** 2025-12-09
**Artifact:** W6D26 (Quantization Foundation)
**Author:** RUST_ENGINEER / TEST_ENGINEER
**Status:** ❌ REJECTED

---

## Summary

Reviewed the initial Scalar Quantization (SQ8) implementation, including the `ScalarQuantizer` struct, training logic, and property tests. Also verified updates to `DATA_LAYOUT.md`.

---

## Findings

### Critical Issues: 1
- [C1] **Unfinished Logic / Dead Code**
  - Description: The `train` function contains an empty `if` block with conversational comments debating the implementation. This indicates unfinished thought processes left in the codebase.
  - Evidence: `src/quantization/scalar.rs:84-89`
    ```rust
    if (max - min).abs() < f32::EPSILON {
         // ... comments ...
         // Actually, `quantize` logic should handle `range == 0`.
    }
    ```
  - Impact: Violation of "No TODO/FIXME without issue reference" (implicit: no half-baked code).
  - Required Action: Remove the dead block and comments. If the logic is handled in `quantize` (as the comment claims), the block is dead noise.

### Major Issues: 1
- [M1] **Test Precision Risk**
  - Description: The test helper `l2_sq_u8` uses `f32` accumulation for squared distances. The maximum possible sum for 768 dimensions is `768 * 255^2 ≈ 50,000,000`. `f32` has 24 bits of significand (~16,777,216 precision). This guarantees precision loss for large distances.
  - Evidence: `tests/proptest_quantization.rs:14-21`
  - Context: `DATA_LAYOUT.md` explicitly advises using `u32` accumulator for this exact reason (Section 5.1). The test ignores this architectural wisdom.
  - Required Action: Update `l2_sq_u8` to use `u32` or `u64` accumulator.

### Minor Issues: 1
- [m1] **NaN Handling in Train**
  - Description: `train` relies on `<` and `>` operators which return false for NaNs. If an input consists entirely of NaNs, `min` remains `f32::MAX` and `max` remains `f32::MIN`, resulting in an inverted range.
  - Required Action: Add a safeguard to `train` to check if `min > max` (indicating no valid values seen) and return default `0.0..1.0` config, similar to the empty slice check.

---

## Verdict

**REJECTED**

This artifact fails 1 critical and 1 major quality gate. The presence of conversational comments in code ("Let's keep min/max as is...") is a clear signal that the code was not scrubbed before submission.

---

## Required Actions Before Resubmission

1. [ ] Remove the empty `if` block and associated commentary in `scalar.rs`.
2. [ ] Update `tests/proptest_quantization.rs` to use `u32` accumulation for `l2_sq_u8`.
3. [ ] (Recommended) Handle the all-NaN edge case in `train`.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-09*
*Verdict: REJECTED*

