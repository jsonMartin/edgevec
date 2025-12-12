# HOSTILE_REVIEWER: Rejection — W6D28 (Quantized Storage)

**Date:** 2025-12-09
**Artifact:** W6D28 (Quantized Storage)
**Author:** RUST_ENGINEER
**Status:** ❌ REJECTED

---

## Summary

Review of the `VectorStorage` refactoring to support 8-bit quantization (`QuantizedU8`). Evaluated `src/storage.rs`, `tests/proptest_storage_quantized.rs`, and benchmark reports against the requirement for 4x memory reduction and dequantization support.

---

## Findings

### Critical Issues: 2
- [C1] **`get_vector` Panics in Quantized Mode**
  - **Description:** `VectorStorage::get_vector` explicitly panics when `StorageType::QuantizedU8` is active. This violates the `VectorProvider` contract and the explicit review requirement: "Does `get_vector` return valid data (dequantized)?".
  - **Evidence:** `src/storage.rs:469` — `panic!("get_vector called on storage without f32 data");`.
  - **Impact:** `HnswIndex::search` relies on `VectorProvider::get_vector`. Using quantized storage will cause the application to crash during search.
  - **Required Action:** Implement on-the-fly dequantization in `get_vector` instead of panicking.

- [C2] **`VectorProvider` Trait Incompatibility**
  - **Description:** The `VectorProvider` trait signature `fn get_vector(&self, id: VectorId) -> &[f32]` requires returning a reference to existing `f32` data. This makes on-the-fly dequantization (which produces new `f32` values) impossible without memory leaks or internal buffering.
  - **Evidence:** `src/hnsw/graph.rs:429` defines the trait returning `&[f32]`.
  - **Impact:** Blocks the fix for [C1].
  - **Required Action:** Refactor `VectorProvider` to return `Cow<[f32]>` or `Vec<f32>` to allow returning computed (dequantized) data.

### Major Issues: 1
- [M1] **Missing Dequantization Test Coverage**
  - **Description:** `tests/proptest_storage_quantized.rs` verifies `get_quantized_vector` and manually dequantizes in the test to check accuracy. It does NOT test `get_vector` for correctness (it currently asserts it panics).
  - **Required Action:** Add tests verifying `get_vector` automatically dequantizes and returns data within expected error bounds.

### Minor Issues: 1
- [m1] **No Auto-Switching for `insert_quantized`**
  - **Description:** Calling `insert_quantized` on a `Float32` storage returns an error instead of automatically switching mode or handling it gracefully.
  - **Status:** Acceptable for strict mode, but worth noting.

---

## Verdict

**REJECTED**

The artifact fails a critical functional requirement: `get_vector` does not return valid dequantized data, and in fact panics, rendering the storage unusable for HNSW search in quantized mode. The underlying architecture (`VectorProvider` trait) prevents a direct fix without refactoring.

---

## Required Actions Before Resubmission

1. [ ] Refactor `VectorProvider::get_vector` to return `Cow<[f32]>` (or similar) to support both zero-copy (f32 mode) and owned (quantized mode) returns.
2. [ ] Update `VectorStorage::get_vector` to dequantize on the fly when in `QuantizedU8` mode.
3. [ ] Update `HnswIndex` and `Searcher` to handle the new `VectorProvider` return type.
4. [ ] Verify `get_vector` correctness via property tests.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-09*
*Verdict: REJECTED*

