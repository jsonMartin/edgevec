# Bundle Size Baseline Report

**Date:** 2025-12-16
**Week:** 20
**Version:** v0.5.0-alpha

---

## Current Bundle Size

| Metric | Value | Notes |
|:-------|:------|:------|
| **Uncompressed** | 227,037 bytes (222 KB) | edgevec_bg.wasm |
| **Gzipped** | 93,579 bytes (91 KB) | Standard HTTP compression |
| **JS Wrapper** | 57,612 bytes (56 KB) | edgevec.js |
| **Total (gzipped)** | ~95 KB | Main deliverable size |

---

## Comparison to v0.4.0

| Version | Uncompressed | Gzipped | Delta |
|:--------|:-------------|:--------|:------|
| v0.4.0 | 227,037 bytes | 93,579 bytes | baseline |
| v0.5.0-alpha | 227,037 bytes | 93,579 bytes | **0 bytes (0%)** |

**Analysis:** Bundle size is unchanged. The NEON SIMD code is conditionally compiled only for `aarch64` targets and is NOT included in the WASM bundle.

---

## Size Target Compliance

| Target | Limit | Actual | Status |
|:-------|:------|:-------|:-------|
| WASM (gzipped) | < 500 KB | 91 KB | PASS |
| JS wrapper | < 100 KB | 56 KB | PASS |
| Total bundle | < 600 KB | ~150 KB | PASS |

---

## Size Breakdown by Feature

### WASM Module (estimated)

| Feature | Contribution | Notes |
|:--------|:-------------|:------|
| HNSW Graph | ~80 KB | Index structure and algorithms |
| Vector Storage | ~40 KB | Contiguous memory layout |
| Persistence | ~30 KB | WAL, snapshots, recovery |
| Quantization | ~25 KB | SQ8, binary quantization |
| Distance Metrics | ~20 KB | L2, cosine, dot product |
| WASM Bindings | ~20 KB | wasm-bindgen glue |
| Other | ~12 KB | Error handling, utilities |

### Not Included in WASM

| Feature | Size | Reason |
|:--------|:-----|:-------|
| NEON SIMD | ~15 KB | ARM64-only (native) |
| x86 SIMD | ~20 KB | x86-only (native) |
| std dependencies | N/A | Uses wasm32 compatible subset |

---

## Optimization Potential

If bundle size reduction is needed:

| Optimization | Estimated Savings | Complexity |
|:-------------|:------------------|:-----------|
| `wasm-opt -Oz` | 5-10% | Low |
| Remove debug info | 2-5% | Low |
| LTO (link-time opt) | 3-7% | Medium |
| Feature gating | Variable | High |

---

## Build Configuration

```toml
# Cargo.toml [profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
```

---

## Measurement Methodology

```bash
# Uncompressed size
wc -c pkg/edgevec_bg.wasm

# Gzipped size
gzip -c pkg/edgevec_bg.wasm | wc -c

# Brotli size (if available)
brotli -c pkg/edgevec_bg.wasm | wc -c
```

---

## Notes

1. **NEON code is NOT in WASM** - The `#[cfg(target_arch = "aarch64")]` attribute ensures NEON-specific code is only compiled for ARM64 native targets
2. **Bundle size unchanged** - Week 20 NEON implementation has zero impact on WASM bundle
3. **Well within limits** - Current 91 KB gzipped is far below the 500 KB target

---

## Historical Trend

| Week | Version | Gzipped Size | Change |
|:-----|:--------|:-------------|:-------|
| W16 | v0.4.0-rc | 90 KB | baseline |
| W17 | v0.4.0 | 91 KB | +1 KB |
| W20 | v0.5.0-alpha | 91 KB | 0 KB |

---

**Author:** RUST_ENGINEER
**Version:** 1.0.0
**Status:** COMPLETE
