# Week 27: Binary Quantization Implementation

**Date:** 2025-12-22 to 2025-12-28
**Focus:** Binary Quantization (BQ) for HNSW Search
**Estimated Duration:** 48 hours
**Phase:** RFC-002 Implementation Phase 2
**Previous:** Week 26 — Core Metadata (APPROVED)

---

## 1. Executive Summary

Week 27 implements **Binary Quantization (BQ)** for EdgeVec, enabling:
- 32x memory reduction (768 f32s → 96 bytes)
- 3-5x search speedup via SIMD popcount
- Recall recovery through F32 rescoring

**Foundation Status:**
- ✅ `BinaryQuantizer` exists (`src/quantization/binary.rs`)
- ✅ `QuantizedVector` with Hamming distance exists
- ✅ SIMD detection exists (`src/simd/mod.rs`)
- ❌ `BinaryVectorStorage` — needs implementation
- ❌ SIMD popcount optimization — needs implementation
- ❌ BQ search in HNSW — needs implementation
- ❌ Rescoring layer — needs implementation

---

## 2. Week 27 Objectives

| ID | Objective | Hours | Risk |
|:---|:----------|:------|:-----|
| W27.1 | Variable-dimension BQ + SIMD popcount | 12 | Medium |
| W27.2 | BinaryVectorStorage implementation | 10 | Medium |
| W27.3 | BQ search integration with HNSW | 14 | High |
| W27.4 | Rescoring layer for recall recovery | 8 | Medium |
| W27.5 | Benchmarks + validation tests | 4 | Low |

**Total:** 48 hours

---

## 3. Daily Task Breakdown

### Day 1: Variable-Dimension BQ + SIMD Popcount (12 hours)

| ID | Task | Hours |
|:---|:-----|:------|
| W27.1.1 | Generalize `BinaryQuantizer` to accept any dimension (not just 768) | 3 |
| W27.1.2 | Implement `VariableQuantizedVector` for dynamic dimension | 3 |
| W27.1.3 | SIMD popcount for x86 (SSE4.1/AVX2 `popcnt`) | 3 |
| W27.1.4 | SIMD popcount for ARM (NEON `vcntq_u8`) | 3 |

**Exit Criteria:**
- Quantization works for any dimension divisible by 8
- Hamming distance uses SIMD popcount when available
- Benchmark shows >2x speedup vs scalar fallback

### Day 2: BinaryVectorStorage (10 hours)

| ID | Task | Hours |
|:---|:-----|:------|
| W27.2.1 | Define `BinaryVectorStorage` struct with Vec<u8> backing | 4 |
| W27.2.2 | Implement insert/get/delete operations | 3 |
| W27.2.3 | Add tombstone support (reuse deleted bitvec pattern) | 3 |

**Exit Criteria:**
- Storage can hold arbitrary number of quantized vectors
- Delete marks as tombstone (no resize)
- Memory usage = dimension/8 bytes per vector

### Day 3: HNSW BQ Search Integration (14 hours)

| ID | Task | Hours |
|:---|:-----|:------|
| W27.3.1 | Add BQ storage field to `HnswIndex` (optional, dual mode) | 4 |
| W27.3.2 | Implement `insert_bq()` that quantizes and stores | 4 |
| W27.3.3 | Implement `search_bq()` using Hamming distance | 6 |

**Exit Criteria:**
- Index can operate in F32-only, BQ-only, or hybrid mode
- search_bq returns candidates sorted by Hamming distance
- All existing tests still pass

### Day 4: Rescoring Layer (8 hours)

| ID | Task | Hours |
|:---|:-----|:------|
| W27.4.1 | Implement `rescore()` that reruns top-N with F32 distance | 4 |
| W27.4.2 | Implement `search_bq_rescored()` combining BQ + rescore | 4 |

**Exit Criteria:**
- Rescoring loads original F32 vectors for top-N candidates
- Recall@10 > 0.90 with 3x overfetch + rescore
- API is ergonomic (`search_bq_rescored(query, k, rescore_factor)`)

### Day 5: Benchmarks + Validation (4 hours)

| ID | Task | Hours |
|:---|:-----|:------|
| W27.5.1 | Create benchmark suite comparing F32 vs BQ vs BQ+rescore | 2 |
| W27.5.2 | Property tests for BQ invariants | 2 |

**Exit Criteria:**
- Benchmark report with speedup measurements
- Property tests ensure BQ is deterministic
- All tests pass (`cargo test`)

---

## 4. Technical Design

### 4.1 Variable-Dimension Quantization

The current `QuantizedVector` is fixed at 768 dimensions. Week 27 adds support for arbitrary dimensions:

```rust
/// Variable-dimension binary quantized vector.
pub struct BinaryVector {
    data: Vec<u8>,
    dimension: usize,
}

impl BinaryVector {
    /// Creates a new binary vector from f32 data.
    ///
    /// # Errors
    /// Returns `QuantizationError::InvalidDimension` if dimension % 8 != 0.
    pub fn quantize(vector: &[f32]) -> Result<Self, QuantizationError> {
        if vector.len() % 8 != 0 {
            return Err(QuantizationError::InvalidDimension {
                dimension: vector.len(),
            });
        }
        let bytes = vector.len() / 8;
        let mut data = vec![0u8; bytes];
        for (i, &v) in vector.iter().enumerate() {
            if v > 0.0 {
                data[i / 8] |= 1 << (i % 8);
            }
        }
        Ok(Self { data, dimension: vector.len() })
    }

    /// Hamming distance to another vector.
    pub fn hamming_distance(&self, other: &Self) -> u32 {
        simd_popcount_xor(&self.data, &other.data)
    }
}
```

### 4.2 SIMD Popcount

```rust
// x86_64 with AVX2/SSE4.1
#[cfg(target_arch = "x86_64")]
pub fn simd_popcount_xor(a: &[u8], b: &[u8]) -> u32 {
    if is_x86_feature_detected!("avx2") {
        simd_popcount_avx2(a, b)
    } else if is_x86_feature_detected!("popcnt") {
        simd_popcount_native(a, b)
    } else {
        scalar_popcount_xor(a, b)
    }
}

// ARM64 with NEON
#[cfg(target_arch = "aarch64")]
pub fn simd_popcount_xor(a: &[u8], b: &[u8]) -> u32 {
    simd_popcount_neon(a, b)
}
```

### 4.3 BinaryVectorStorage

```rust
/// Storage for binary quantized vectors.
pub struct BinaryVectorStorage {
    /// Packed binary data (all vectors concatenated).
    data: Vec<u8>,
    /// Dimension of each vector.
    dimension: usize,
    /// Bytes per vector (dimension / 8).
    bytes_per_vector: usize,
    /// Deleted bit flags.
    deleted: BitVec<u8, Lsb0>,
    /// Number of stored vectors (including deleted).
    count: usize,
}
```

### 4.4 Dual-Mode Index

```rust
pub struct HnswIndex {
    // Existing fields...
    pub(crate) metadata: MetadataStore,

    // NEW: Binary quantization storage (optional)
    pub(crate) bq_storage: Option<BinaryVectorStorage>,
}
```

### 4.5 BQ Search with Rescoring

```rust
impl HnswIndex {
    /// Search using binary quantization with rescoring.
    ///
    /// 1. Quantize query to binary
    /// 2. Search using Hamming distance (fast)
    /// 3. Rescore top-N candidates using F32 distance (accurate)
    /// 4. Return final top-k
    pub fn search_bq_rescored(
        &self,
        query: &[f32],
        k: usize,
        rescore_factor: usize, // Typically 3-10
    ) -> Result<Vec<(VectorId, f32)>, GraphError> {
        // Step 1: BQ search for k * rescore_factor candidates
        let candidates = self.search_bq(query, k * rescore_factor)?;

        // Step 2: Rescore with F32
        let mut rescored: Vec<_> = candidates
            .iter()
            .map(|(id, _)| (*id, self.f32_distance(query, *id)))
            .collect();

        // Step 3: Sort by F32 distance and return top-k
        rescored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        rescored.truncate(k);
        Ok(rescored)
    }
}
```

---

## 5. Acceptance Criteria

### 5.1 Functional Requirements

| Requirement | Verification |
|:------------|:-------------|
| Variable dimension BQ (any multiple of 8) | Unit test with 128, 384, 768, 1024 dims |
| SIMD popcount for x86 | Runtime detection + benchmark |
| SIMD popcount for ARM | Compile-time test (CI matrix) |
| BinaryVectorStorage CRUD | Unit tests for insert/get/delete |
| search_bq returns Hamming-sorted results | Unit test |
| search_bq_rescored improves recall | Recall benchmark > 0.90 |

### 5.2 Performance Requirements

| Metric | Target | Verification |
|:-------|:-------|:-------------|
| BQ memory usage | 32x reduction vs F32 | Memory benchmark |
| BQ search speedup | 3-5x vs F32 | Latency benchmark |
| BQ+rescore recall | >0.90 @ k=10 | Recall benchmark |
| SIMD vs scalar | >2x speedup | Popcount benchmark |

### 5.3 Quality Requirements

| Requirement | Verification |
|:------------|:-------------|
| All existing tests pass | `cargo test` |
| No new clippy warnings | `cargo clippy -- -D warnings` |
| Formatted | `cargo fmt --check` |
| No panics in library code | No unwrap() except with proven invariants |

---

## 6. Risk Analysis

### 6.1 High Risk: BQ Recall Degradation

**Risk:** Binary quantization loses precision, reducing search quality.
**Probability:** Medium
**Impact:** High
**Mitigation:**
- Rescoring layer with 3-5x overfetch
- Benchmark recall at each step
- Allow fallback to F32-only mode

### 6.2 Medium Risk: SIMD Portability

**Risk:** SIMD code may not work on all platforms.
**Probability:** Low
**Impact:** Medium
**Mitigation:**
- Runtime feature detection
- Scalar fallback always available
- CI matrix tests multiple architectures

### 6.3 Low Risk: Memory Alignment

**Risk:** Misaligned data causes performance degradation.
**Probability:** Low
**Impact:** Low
**Mitigation:**
- Use 64-byte alignment for SIMD data
- Bytemuck for safe casting

---

## 7. Dependencies

```
W27.1 (Variable BQ + SIMD)
    ↓
W27.2 (BinaryVectorStorage)
    ↓
W27.3 (HNSW BQ Search) ← depends on both W27.1 and W27.2
    ↓
W27.4 (Rescoring) ← depends on W27.3
    ↓
W27.5 (Benchmarks) ← depends on all
```

---

## 8. Testing Strategy

### 8.1 Unit Tests

| Module | Test File | Coverage |
|:-------|:----------|:---------|
| BinaryVector | `tests/binary_vector.rs` | quantize, hamming_distance |
| BinaryVectorStorage | `tests/binary_storage.rs` | insert, get, delete |
| BQ Search | `tests/bq_search.rs` | search_bq, search_bq_rescored |
| SIMD Popcount | `src/simd/popcount.rs` | SIMD vs scalar equivalence |

### 8.2 Property Tests

```rust
proptest! {
    #[test]
    fn prop_hamming_symmetric(a in any_vector(), b in any_vector()) {
        let qa = BinaryVector::quantize(&a);
        let qb = BinaryVector::quantize(&b);
        assert_eq!(qa.hamming_distance(&qb), qb.hamming_distance(&qa));
    }

    #[test]
    fn prop_hamming_identity(v in any_vector()) {
        let qv = BinaryVector::quantize(&v);
        assert_eq!(qv.hamming_distance(&qv), 0);
    }

    #[test]
    fn prop_quantize_deterministic(v in any_vector()) {
        let q1 = BinaryVector::quantize(&v);
        let q2 = BinaryVector::quantize(&v);
        assert_eq!(q1, q2);
    }
}
```

### 8.3 Benchmarks

| Benchmark | File | Metrics |
|:----------|:-----|:--------|
| BQ quantization | `benches/bq_quantize.rs` | ns/vector |
| Hamming distance | `benches/bq_hamming.rs` | ns/comparison |
| BQ search vs F32 | `benches/bq_search.rs` | latency, throughput |
| BQ+rescore recall | `benches/bq_recall.rs` | recall@k |

---

## 9. Files to Create/Modify

### 9.1 New Files

| File | Description |
|:-----|:------------|
| `src/quantization/variable.rs` | Variable-dimension BinaryVector |
| `src/storage/binary.rs` | BinaryVectorStorage |
| `src/hnsw/search_bq.rs` | BQ search algorithms |
| `src/hnsw/rescore.rs` | F32 rescoring layer |
| `src/simd/popcount.rs` | SIMD popcount implementations |
| `tests/binary_vector.rs` | BinaryVector tests |
| `tests/binary_storage.rs` | Storage tests |
| `tests/bq_search.rs` | BQ search tests |
| `benches/bq_quantize.rs` | Quantization benchmarks |
| `benches/bq_search.rs` | Search benchmarks |

### 9.2 Modified Files

| File | Changes |
|:-----|:--------|
| `src/quantization/mod.rs` | Add `variable` module |
| `src/storage/mod.rs` | Add `binary` module |
| `src/hnsw/graph.rs` | Add `bq_storage` field |
| `src/hnsw/mod.rs` | Add `search_bq`, `rescore` modules |
| `src/simd/mod.rs` | Add `popcount` module |
| `src/lib.rs` | Export new types |

---

## 10. Week 27 Checklist

- [ ] Day 1: Variable-dimension BQ + SIMD popcount
- [ ] Day 2: BinaryVectorStorage implementation
- [ ] Day 3: HNSW BQ search integration
- [ ] Day 4: Rescoring layer
- [ ] Day 5: Benchmarks + validation

---

## 11. Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| All tests pass | `cargo test` |
| BQ search works | `tests/bq_search.rs` |
| Recall > 0.90 | Benchmark report |
| Speedup 3-5x | Benchmark report |
| No clippy warnings | `cargo clippy -- -D warnings` |
| Formatted | `cargo fmt --check` |

---

## 12. Handoff

After completing Week 27:

**Artifacts Generated:**
- Variable-dimension BinaryVector
- BinaryVectorStorage
- HNSW BQ search integration
- Rescoring layer
- Benchmark suite

**Status:** PENDING_HOSTILE_REVIEW

**Next:** Week 28 — WASM Bindings + Integration

---

*Agent: PLANNER*
*Status: [APPROVED] (2025-12-21)*
*Review: `docs/reviews/2025-12-21_W27_PLAN_APPROVED.md`*
