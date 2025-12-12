# HOSTILE_REVIEWER: Rejection — W7D34 (Error Handling)

**Date:** 2025-12-10
**Artifact:** W7D34 (Error Handling)
**Author:** RUST_ENGINEER / WASM_SPECIALIST
**Status:** ❌ REJECTED

---

## Summary

Reviewed the unified error handling implementation in `src/error.rs` and its integration into the WASM boundary in `src/wasm/mod.rs`. While the unified enum `EdgeVecError` is well-defined, the WASM implementation fails to consistently use it, leaking raw string errors to the JavaScript consumer.

---

## Findings

### Critical Issues: 3
- [C1] **WASM Boundary Violation (Insert/Batch)**
  - Description: `src/wasm/mod.rs` returns raw `JsValue` strings for logic errors instead of `EdgeVecError` objects.
  - Evidence: 
    - Line 233: `return Err(JsValue::from_str("Vector ID overflowed u32"));`
    - Line 264: `return Err(JsValue::from_str(&format!("Batch dimension mismatch...")));`
    - Line 277: `return Err(JsValue::from_str("Vectors contain non-finite values"));`
  - Impact: JS consumers cannot rely on `err.code` existence. Breaking API contract.
  - Required Action: Wrap these in `EdgeVecError::Validation` or specific error variants.

- [C2] **WASM Boundary Violation (Lifecycle)**
  - Description: Constructor and persistence methods return raw strings.
  - Evidence:
    - Line 164 (`new`): `return Err(JsValue::from_str(&format!("Unknown metric: {other}")));`
    - Line 400 (`save`): `.map_err(|e| JsValue::from_str(&e.to_string()))?`
    - Line 428 (`load`): `.map_err(|e| JsValue::from_str(&e.to_string()))?`
  - Impact: Persistence and Config errors are untyped in JS.
  - Required Action: Map `postcard` errors to `EdgeVecError::Persistence` and config errors to `EdgeVecError::Validation`.

- [C3] **Conflicting Error Conversions**
  - Description: `src/wasm/mod.rs` defines a `From<GraphError> for JsValue` that returns a string, while `src/error.rs` defines a structured conversion via `EdgeVecError`.
  - Evidence: `src/wasm/mod.rs` lines 44-48.
  - Impact: Risk of accidental raw string returns if `GraphError` is converted directly instead of wrapping in `EdgeVecError`.
  - Required Action: Remove the local `From<GraphError>` impl in `wasm/mod.rs` and force usage of `EdgeVecError`.

### Major Issues: 0

### Minor Issues: 1
- [m1] **Test Coverage Gaps**
  - Description: `tests/wasm_error.rs` checks `insert` dimension mismatch but does not verify the structured nature of persistence or batch errors (which are currently broken).
  - Required Action: Expand `tests/wasm_error.rs` to cover `insert_batch` and `save/load` error paths once fixed.

---

## Verdict

**REJECTED**

The artifact fails 3 critical quality gates regarding the Unified Error/WASM Boundary contract. The implementation is "half-baked"—defining the type but not using it consistently.

---

## Required Actions Before Resubmission

1. [ ] **Refactor `src/wasm/mod.rs`**: Replace ALL `JsValue::from_str` returns with `EdgeVecError` conversions.
2. [ ] **Remove Ambiguity**: Delete `impl From<GraphError> for JsValue` in `src/wasm/mod.rs`.
3. [ ] **Verify**: Ensure `tests/wasm_error.rs` passes and consider adding a test case for a batch error or persistence error to prove they are structured.

---

*Reviewed by: HOSTILE_REVIEWER*
*Date: 2025-12-10*
*Verdict: REJECTED*

