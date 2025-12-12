# EdgeVec Fuzzing Strategy

**Version:** 1.0.0
**Created:** 2025-12-11
**Week:** 8 (Binary Quantization & SIMD Foundation)

---

## 1. Overview

This document defines the fuzzing strategy for EdgeVec's binary quantization module and other critical components. Fuzzing is a mandatory quality gate for all security-sensitive operations.

### 1.1 Goals

| Goal | Target | Metric |
|:-----|:-------|:-------|
| Panic-freedom | 100% | No panics on valid/invalid input |
| Memory safety | 100% | No buffer overflows, OOB access |
| Determinism | 100% | Same input always produces same output |
| Invariant preservation | 100% | All mathematical invariants hold |

### 1.2 Scope

**Fuzz Target Status:**

| Target | File | Status | Corpus |
|:-------|:-----|:-------|:-------|
| `fuzz_quantization` | `fuzz_targets/fuzz_quantization.rs` | ✅ Implemented | 103 seeds |
| `header_parse` | `fuzz_targets/header_parse.rs` | ✅ Implemented | — |
| `wal_replay` | `fuzz_targets/wal_replay.rs` | ✅ Implemented | — |
| `hnsw_insert` | `fuzz_targets/hnsw_insert.rs` | ✅ Implemented | — |
| `hnsw_search` | `fuzz_targets/hnsw_search.rs` | ✅ Implemented | — |
| `hnsw_config_init` | `fuzz_targets/hnsw_config_init.rs` | ✅ Implemented | — |
| `graph_ops` | `fuzz_targets/graph_ops.rs` | ✅ Implemented | — |
| `search_robustness` | `fuzz_targets/search_robustness.rs` | ✅ Implemented | — |
| `persistence_load` | `fuzz_targets/persistence_load.rs` | ✅ Implemented | — |
| `scalar_quantization` | — | 🔲 Planned (W9) | — |
| `simd_operations` | — | 🔲 Planned (W9) | — |

**Out of Scope (for now):**
- WASM bindings (requires special fuzzing setup)
- JavaScript API surface

---

## 2. Fuzz Targets

### 2.1 Binary Quantization (`fuzz_quantization`)

**File:** `fuzz/fuzz_targets/fuzz_quantization.rs`

**Purpose:** Verify binary quantization handles all possible f32 inputs without panicking and maintains mathematical invariants.

**Invariants Tested:**
1. **Determinism:** `quantize(v) == quantize(v)` for any vector v
2. **Self-distance:** `hamming_distance(q, q) == 0` for any quantized vector q
3. **Bounded distance:** `hamming_distance(a, b) <= 768` for any a, b
4. **Symmetry:** `hamming_distance(a, b) == hamming_distance(b, a)`
5. **Output size:** `quantized.data().len() == 96` always
6. **Similarity bounds:** `0.0 <= similarity(a, b) <= 1.0` always

**Edge Cases:**
- NaN values
- Positive/negative infinity
- Negative zero
- Subnormal numbers
- All zeros
- All ones
- Alternating patterns

**Corpus:** `fuzz/corpus/fuzz_quantization/` (103 seeds)

### 2.2 Header Parsing (`header_parse`)

**File:** `fuzz/fuzz_targets/header_parse.rs`

**Purpose:** Verify file header parsing handles all possible byte inputs without panicking.

**Invariants Tested:**
1. `FileHeader::from_bytes(data)` returns `Result`, never panics
2. Invalid magic bytes return error
3. Invalid version returns error
4. Invalid checksum returns error

### 2.3 HNSW Operations

**Files:**
- `fuzz/fuzz_targets/hnsw_insert.rs`
- `fuzz/fuzz_targets/hnsw_search.rs`
- `fuzz/fuzz_targets/hnsw_config_init.rs`

**Invariants Tested:**
1. Insert never panics on valid dimension vectors
2. Search returns results in sorted order
3. Config validation catches invalid parameters

### 2.4 WAL Replay (`wal_replay`)

**File:** `fuzz/fuzz_targets/wal_replay.rs`

**Purpose:** Verify WAL iterator handles arbitrary byte sequences without panicking.

**Invariants Tested:**
1. `WalIterator::new(data)` never panics
2. Iteration over corrupted data returns errors, not panics
3. Checksum validation catches corruption

### 2.5 Graph Operations (`graph_ops`)

**File:** `fuzz/fuzz_targets/graph_ops.rs`

**Purpose:** Verify graph structure operations handle edge cases.

**Invariants Tested:**
1. `add_node()` handles arbitrary VectorIds
2. `set_neighbors()` handles arbitrary neighbor lists
3. Graph remains consistent after operations

### 2.6 Search Robustness (`search_robustness`)

**File:** `fuzz/fuzz_targets/search_robustness.rs`

**Purpose:** Verify search handles invalid entry points and malformed queries.

**Invariants Tested:**
1. `search_layer()` handles invalid NodeIds gracefully
2. Query vectors with extreme values don't panic
3. Search terminates on disconnected graphs

### 2.7 Persistence Loading (`persistence_load`)

**File:** `fuzz/fuzz_targets/persistence_load.rs`

**Purpose:** Verify snapshot loading handles corrupted files.

**Invariants Tested:**
1. Corrupted snapshots return errors, not panics
2. Truncated files are handled gracefully
3. Invalid offsets don't cause OOB access

### 2.8 Planned Targets (Week 9+)

| Target | Description | Week |
|:-------|:------------|:-----|
| `scalar_quantization` | SQ8 quantization fuzzing | W9 |
| `simd_operations` | SIMD Hamming distance | W9 |
| `hybrid_search` | Combined binary + SQ8 search | W10 |

---

## 3. Corpus Design

### 3.1 Seed Categories

| Category | Seeds | Purpose |
|:---------|:------|:--------|
| Constant vectors | 7 | Test uniform values |
| Alternating patterns | 6 | Test bit packing logic |
| Special floats | 11 | Test edge cases (NaN, Inf, subnormal) |
| Random distributions | 40 | Cover general case |
| Boundary patterns | 19 | Test byte/bit boundaries |
| Malformed data | 6 | Test error handling |
| Additional patterns | 14 | Ensure 100+ seeds |
| **Total** | **103** | |

### 3.2 Corpus Generation

```bash
# Generate corpus seeds
python scripts/generate_fuzz_corpus.py

# Verify corpus count
ls -1 fuzz/corpus/fuzz_quantization | wc -l
# Expected: 103+
```

### 3.3 Corpus Maintenance

- Seeds that find bugs are promoted to permanent corpus
- Seeds that cause timeouts are investigated and removed
- Corpus is regenerated after major algorithm changes

---

## 4. Execution

### 4.1 Local Fuzzing

```bash
# Install cargo-fuzz (requires nightly)
cargo +nightly install cargo-fuzz

# Run single target
cd fuzz
cargo +nightly fuzz run fuzz_quantization

# Run with corpus
cargo +nightly fuzz run fuzz_quantization corpus/fuzz_quantization

# Run with timeout
cargo +nightly fuzz run fuzz_quantization -- -max_total_time=300
```

### 4.2 Coverage-Guided Fuzzing

```bash
# Run with coverage report
cargo +nightly fuzz coverage fuzz_quantization

# View coverage
llvm-cov show target/x86_64-unknown-linux-gnu/coverage/fuzz_quantization \
  -instr-profile=coverage/fuzz_quantization/coverage.profdata
```

### 4.3 CI Integration

Fuzzing is integrated into CI via GitHub Actions:

```yaml
# .github/workflows/fuzz.yml
name: Fuzz Testing
on:
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight
  push:
    paths:
      - 'src/quantization/**'
      - 'fuzz/**'

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-fuzz
      - run: cargo +nightly fuzz run fuzz_quantization -- -max_total_time=600
```

---

## 5. Bug Triage

### 5.1 Severity Classification

| Severity | Description | Example |
|:---------|:------------|:--------|
| **P0 Critical** | Memory unsafety | Buffer overflow, use-after-free |
| **P1 High** | Panic in production code | Unwrap on None |
| **P2 Medium** | Invariant violation | Wrong distance calculation |
| **P3 Low** | Performance issue | Unexpected O(n²) behavior |

### 5.2 Bug Response

1. **Immediate:** Minimize crash input (`cargo fuzz tmin`)
2. **Investigation:** Create unit test reproducing the bug
3. **Fix:** Implement and verify fix
4. **Regression:** Add minimized input to permanent corpus

---

## 6. Platform Notes

### 6.1 Windows

`cargo-fuzz` uses libFuzzer which requires `clang`. On Windows:

1. Install LLVM: `winget install LLVM`
2. Set `LIBFUZZER_PATH` environment variable
3. Use `cargo +nightly fuzz run --target x86_64-pc-windows-msvc`

**Note:** Windows fuzzing has limited support. CI runs on Linux.

### 6.2 macOS

Full support via clang. Install via:
```bash
xcode-select --install
```

### 6.3 Linux

Full support. Recommended platform for extended fuzzing runs.

---

## 7. Metrics & Reporting

### 7.1 Key Metrics

| Metric | Target | Current |
|:-------|:-------|:--------|
| Corpus size | ≥100 seeds | 103 |
| Coverage | ≥90% line | TBD |
| Fuzzing hours | ≥10 hrs/week | TBD |
| Bugs found | 0 critical | 0 |

### 7.2 Reporting

- Weekly fuzz report in `docs/reviews/`
- Crash artifacts stored in `fuzz/artifacts/`
- Coverage reports stored in `fuzz/coverage/`

---

## 8. References

- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [Google OSS-Fuzz](https://google.github.io/oss-fuzz/)

---

## Revision History

| Version | Date | Change |
|:--------|:-----|:-------|
| 1.0.0 | 2025-12-11 | Initial strategy for Week 8 |
