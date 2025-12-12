# EdgeVec Technical Debt Tracker

**Version:** 0.2.0-alpha.1
**Created:** 2025-12-12
**Target Resolution:** v0.3.0

---

## Overview

This document tracks technical debt intentionally deferred from v0.2.0-alpha.1 to ship the alpha release. All items must be resolved before v1.0.0.

---

## Priority Levels

| Priority | Definition | Target |
|:---------|:-----------|:-------|
| **P0** | Security/UB risk | Before v0.2.0 stable |
| **P1** | API/correctness | Before v0.3.0 |
| **P2** | Performance/clarity | Before v1.0.0 |
| **P3** | Style/pedantic | When convenient |

---

## Deferred Clippy Warnings (107 total)

### Summary by Category

| Category | Count | Priority | Notes |
|:---------|:------|:---------|:------|
| `cast_possible_truncation` | ~50 | P2 | Safe on WASM 32-bit, investigate native |
| `similar_names` | ~15 | P3 | Test code, cosmetic |
| `float_cmp` | ~6 | P2 | Test assertions, use approx comparison |
| `uninlined_format_args` | ~10 | P3 | Style preference |
| `cast_lossless` | ~15 | P3 | Explicit casts for clarity |
| `collapsible_else_if` | ~3 | P3 | Style preference |
| Other pedantic | ~8 | P3 | Various style warnings |

### High-Priority Items (P1-P2)

#### 1. `cast_possible_truncation` (~50 warnings)

**Location:** `src/persistence/snapshot.rs`, `src/hnsw/`, `src/storage/`

**Example:**
```rust
// Current
let vec_count = header.vector_count as usize;

// Should be (for explicit handling)
let vec_count = usize::try_from(header.vector_count)
    .expect("vector count exceeds platform capacity");
```

**Risk:** On 32-bit platforms, truncation could occur silently. WASM is 32-bit.

**Action:** Review each cast, add explicit checks where needed, or document why truncation is acceptable.

**Target:** v0.3.0

---

#### 2. `float_cmp` (6 warnings)

**Location:** `src/quantization/scalar.rs:154-198` (test code)

**Example:**
```rust
// Current
assert_eq!(q.config.min, -2.0);

// Should be
assert!((q.config.min - (-2.0)).abs() < f32::EPSILON);
```

**Risk:** Floating-point comparison may fail due to precision.

**Action:** Use approximate comparison macros in tests.

**Target:** v0.3.0

---

### Low-Priority Items (P3)

#### 3. `similar_names` (~15 warnings)

**Location:** `src/quantization/binary.rs:391-604` (test code)

**Example:**
```rust
// Current
let quantizer = BinaryQuantizer::new();
let quantized = quantizer.quantize(&zero);

// Warning: `quantizer` and `quantized` are similar
```

**Risk:** None (readability concern only)

**Action:** Consider renaming to `q` and `result` or allow lint locally.

**Target:** When convenient

---

#### 4. `uninlined_format_args` (~10 warnings)

**Location:** Various test files

**Example:**
```rust
// Current
assert!(diff < 0.05, "Diff too large: {} vs {}", orig, dec);

// Should be
assert!(diff < 0.05, "Diff too large: {orig} vs {dec}");
```

**Risk:** None (style preference)

**Action:** Run `cargo clippy --fix` when ready

**Target:** v0.3.0

---

## Deferred Dependency Updates

### 1. `atomic-polyfill` Unmaintained (RUSTSEC-2023-0089)

**Path:** `heapless 0.7.17 → postcard 1.1.3 → edgevec`

**Risk Level:** LOW (advisory is informational, not vulnerability)

**Description:** `atomic-polyfill` crate is unmaintained since 2023. It provides atomic primitives for platforms without native atomics.

**Options:**
1. **Accept risk** (current) — Advisory is unmaintained notice, not CVE
2. **Update postcard** — Check if newer postcard uses different backend
3. **Replace postcard** — Use different serialization (serde_json, bincode v2)

**Decision:** Accept risk for alpha. Re-evaluate in v0.3.0.

**Target:** Re-evaluate in v0.3.0

---

## Deferred Features

### 1. ARM/NEON SIMD Optimization

**Priority:** P2
**Effort:** 4-8h

**Description:** AVX2 SIMD path exists for x86_64. ARM processors need NEON equivalent.

**Impact:** Performance on Apple Silicon, AWS Graviton

**Target:** v0.4.0

---

### 2. Test Coverage Measurement

**Priority:** P2
**Effort:** 2h

**Description:** Add `cargo tarpaulin` to CI to track test coverage.

**Current:** 16 tests pass, but coverage % unknown.

**Target:** v0.3.0 (CI setup)

---

### 3. Parallel Build/Insert

**Priority:** P2
**Effort:** 16h

**Description:** HNSW graph construction is single-threaded. Parallel construction could speed up batch inserts.

**Target:** v0.4.0

---

## Resolution Tracking

| Item | Priority | Status | Version Resolved | Notes |
|:-----|:---------|:-------|:-----------------|:------|
| `cast_possible_truncation` | P2 | DEFERRED | - | Target v0.3.0 |
| `float_cmp` | P2 | DEFERRED | - | Target v0.3.0 |
| `similar_names` | P3 | DEFERRED | - | Low priority |
| `atomic-polyfill` | P2 | ACCEPTED | - | Re-evaluate v0.3.0 |
| ARM/NEON | P2 | PLANNED | - | Target v0.4.0 |
| Test coverage | P2 | PLANNED | - | Target v0.3.0 |

---

## Review Schedule

| Milestone | Action |
|:----------|:-------|
| v0.2.0 stable | Review P0, P1 items |
| v0.3.0 | Resolve all P1, P2 items |
| v1.0.0 | Clear all debt or document permanent exceptions |

---

**Document Status:** [ACTIVE]
**Last Updated:** 2025-12-12
**Next Review:** Before v0.3.0 release
