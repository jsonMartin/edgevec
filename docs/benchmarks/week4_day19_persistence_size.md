# Benchmark: W4.4 Size Check (Persistence)

**Date:** 2025-12-08
**Version:** 0.0.1-alpha
**Commit:** Current

## Results

| Metric | Value | Target | Status |
|:-------|:------|:-------|:-------|
| Bundle Size (WASM) | 133 KB | < 500 KB | ✅ PASS |
| JS Binding Size | 29 KB | N/A | OK |

## Details

- **Configuration:** `release` profile, `opt-level = "z"`, `lto = true`.
- **Optimization:** `wasm-opt` DISABLED (due to bulk-memory validation issue).
- **Impact of Persistence:** Added `serde`, `postcard`, `serde-wasm-bindgen`.
- **Estimate with `wasm-opt`:** Likely < 100 KB.

## Conclusion

The addition of persistence support (async save/load, binary serialization) did not bloat the bundle beyond acceptable limits. We are well within the "Flash Loading" budget.

