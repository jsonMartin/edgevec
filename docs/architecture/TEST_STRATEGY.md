# EdgeVec Test Strategy v1.2 — "Nvidia Grade" Verification

**Date:** 2025-12-05
**Author:** META_ARCHITECT
**Status:** [APPROVED]

---

## 0. Verification Philosophy

> **"Every architectural component must have a corresponding verification strategy."**
> — Nvidia Grade Rule

This document defines the complete verification strategy for EdgeVec. It includes strict requirements for **Property-Based Testing**, **Fuzzing**, **Memory Safety**, **Deterministic Simulation**, and **Recall Verification**.

### 0.1 Test Pyramid

```
                    ┌──────────────────┐
                    │  End-to-End (E2E) │  ← Manual + CI: 10 tests
                    │  Browser + WASM   │
                    ├──────────────────┤
                    │   Integration     │  ← CI: 50 tests
                    │   Component combo │
                ┌───┴──────────────────┴───┐
                │    Recall Verification    │  ← Recall@K > 0.95 check
                │    Ground-truth comparison│
            ┌───┴───────────────────────────┴───┐
            │    Property-Based (PBT)        │  ← proptest: 500+ cases
            │    Invariant verification      │
        ┌───┴────────────────────────────────┴───┐
        │        Fuzzing Campaigns                │  ← cargo-fuzz: 1M+ inputs
        │        Input boundary attacks           │
    ┌───┴────────────────────────────────────────┴───┐
    │            Memory Safety (Miri)                 │  ← miri: All unsafe blocks
    │            UB detection                         │
┌───┴────────────────────────────────────────────────┴───┐
│                Unit Tests                               │  ← cargo test: 200+ tests
│                Function-level correctness               │
└─────────────────────────────────────────────────────────┘
```

### 0.2 Verification Coverage Requirements

| Component | Unit | PBT | Fuzz | Miri | Recall | Int | E2E |
|:----------|:----:|:---:|:----:|:----:|:------:|:---:|:---:|
| VectorId/NodeId | ✅ | ✅ | - | ✅ | - | - | - |
| HnswConfig | ✅ | ✅ | - | - | - | - | - |
| VectorStorage | ✅ | ✅ | ✅ | ✅ | - | ✅ | - |
| HnswIndex | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| NeighborPool (Compressed) | ✅ | ✅ | ✅ | ✅ | - | - | - |
| WAL | ✅ | ✅ | ✅ | - | - | ✅ | - |
| FileHeader | ✅ | ✅ | ✅ | - | - | - | - |
| StorageBackend | ✅ | ✅ | - | - | - | ✅ | ✅ |
| WASM Bindings | ✅ | - | - | - | - | ✅ | ✅ |
| DeterministicRng | ✅ | ✅ | - | - | - | ✅ | - |

---

## 1. Recall Verification (CRITICAL)

**Requirement:** Search quality must be statistically verified against brute-force baseline.

#### RECALL-001: HNSW Accuracy vs Brute Force

```rust
#[test]
fn verify_recall_at_10() {
    // 1. Generate synthetic dataset (10k vectors, 128 dim)
    let vectors = generate_vectors(10_000, 128);
    let queries = generate_vectors(100, 128);
    
    // 2. Build HNSW Index
    let config = HnswConfig {
        m: 16, m0: 32, ef_construction: 200, ef_search: 50, dimensions: 128, ..Default::default()
    };
    let mut index = HnswIndex::new(config);
    for v in &vectors { index.insert(v).unwrap(); }
    
    // 3. Execute Queries
    let mut recall_sum = 0.0;
    
    for query in &queries {
        // Ground Truth (Brute Force)
        let truth = brute_force_search(&vectors, query, 10);
        
        // Approximate Search
        let result = index.search(query, 10).unwrap();
        
        // Calculate Intersection
        let match_count = count_matches(&truth, &result);
        recall_sum += match_count as f32 / 10.0;
    }
    
    // 4. Assert Minimum Quality
    let avg_recall = recall_sum / queries.len() as f32;
    println!("Average Recall@10: {}", avg_recall);
    assert!(avg_recall >= 0.95, "Recall {} too low (<0.95)", avg_recall);
}
```

---

## 2. Property-Based Tests (proptest)

### 2.1 Setup

```toml
# Cargo.toml
[dev-dependencies]
proptest = "1.4"
proptest-derive = "0.4"
```

### 2.2 Identifier Properties

#### PROP-ID-001: VectorId Uniqueness
(Defined in previous iteration)

#### PROP-ID-002: NodeId Bounds (NEW)

```rust
proptest! {
    #[test]
    fn node_id_validity(id in 0u32..u32::MAX) {
        let node = NodeId(id);
        if id == u32::MAX {
            prop_assert_eq!(node, NodeId::INVALID);
        } else {
            prop_assert_ne!(node, NodeId::INVALID);
        }
    }
}
```

### 2.3 Configuration Properties

#### PROP-CFG-001: HnswConfig Validation (NEW)

```rust
proptest! {
    #[test]
    fn hnsw_config_valid(m in 2u32..100, ef_c in 10u32..500) {
        let config = HnswConfig {
            m, m0: m*2, ef_construction: ef_c, ef_search: 50, dimensions: 128, metric: 0, _reserved: [0; 2]
        };
        // Validation logic should pass for valid params
        // Assuming a `validate()` method exists on HnswConfig
        prop_assert!(config.validate().is_ok());
    }
}
```

### 2.4 Storage Properties

#### PROP-STORE-001: VectorStorage Invariants (NEW)

```rust
proptest! {
    #[test]
    fn vector_storage_invariants(
        dims in 1u32..128,
        count in 1u32..100
    ) {
        let mut storage = VectorStorage::new(dims);
        for _ in 0..count {
            storage.insert(&vec![0.0; dims as usize]).unwrap();
        }
        
        prop_assert_eq!(storage.len() as u32, count);
        prop_assert_eq!(storage.data_len(), (count * dims) as usize);
    }
}
```

### 2.5 Persistence Properties

#### PROP-PERSIST-001: FileHeader Round-Trip (NEW)

```rust
proptest! {
    #[test]
    fn file_header_roundtrip(
        dims in 1u32..4096,
        count in 0u64..1_000_000,
        seed in any::<u64>()
    ) {
        let original = FileHeader {
            magic: *b"EVEC",
            version_major: 1, version_minor: 0, flags: 0,
            vector_count: count, index_offset: 100, metadata_offset: 200, rng_seed: seed,
            dimensions: dims, header_crc: 0, hnsw_m: 16, hnsw_m0: 32, reserved: 0
        };
        let mut buf = vec![0u8; 64];
        original.write(&mut buf);
        let decoded = FileHeader::read(&buf).unwrap();
        prop_assert_eq!(original.vector_count, decoded.vector_count);
        prop_assert_eq!(original.rng_seed, decoded.rng_seed);
    }
}
```

#### PROP-WAL-001: WalEntry Integrity (NEW)

```rust
proptest! {
    #[test]
    fn wal_entry_roundtrip(payload in prop::collection::vec(any::<u8>(), 0..1000)) {
        let entry = WalEntry::new(1, 0 /* Insert */, payload.clone());
        let mut buf = Vec::new();
        entry.write(&mut buf).unwrap();
        
        let decoded = WalEntry::read(&buf[..]).unwrap();
        prop_assert_eq!(entry.sequence, decoded.sequence);
        prop_assert_eq!(entry.payload_len, decoded.payload_len);
    }
}
```

### 2.6 Compression Properties

#### PROP-COMP-001: Neighbor Compression Round-Trip
(Defined in previous iteration)

### 2.7 Determinism Properties

#### PROP-DET-001: RNG Determinism
(Defined in previous iteration)

---

## 3. Fuzzing Campaigns (cargo-fuzz)

### 3.1 Setup
(As defined in previous iteration)

### 3.2 Fuzz Targets (Implemented)

#### FUZZ-001: HNSW Insert (NEW Implementation)

```rust
// fuzz/fuzz_targets/hnsw_insert.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use edgevec::index::{HnswIndex, HnswConfig};
use edgevec::types::VectorOp; // Mock type for ops

fuzz_target!(|data: &[u8]| {
    // Decode arbitrary bytes into vector operations
    if let Ok(ops) = bincode::deserialize::<Vec<VectorOp>>(data) {
        let config = HnswConfig::default_with_dim(16);
        let mut index = HnswIndex::new(config);
        for op in ops {
            // Truncate/pad vector to 16 dims
            let mut vec = op.vector;
            vec.resize(16, 0.0);
            
            // Should never panic or corrupt state
            let _ = index.insert(&vec);
        }
        // Invariant check
        assert!(index.validate_graph().is_ok());
    }
});
```

#### FUZZ-002: HNSW Search (NEW Implementation)

```rust
// fuzz/fuzz_targets/hnsw_search.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use edgevec::index::HnswIndex;

// Static index built once
lazy_static::lazy_static! {
    static ref STATIC_INDEX: HnswIndex = {
        let mut idx = HnswIndex::new(Default::default());
        for i in 0..100 { idx.insert(&vec![i as f32; 16]).unwrap(); }
        idx
    };
}

fuzz_target!(|data: &[u8]| {
    // Interpret bytes as query vector
    let query_len = 16 * 4; // 16 f32s
    if data.len() >= query_len {
        let query: Vec<f32> = data[..query_len]
            .chunks(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();
            
        // Should return valid results or empty, never panic
        let _ = STATIC_INDEX.search(&query, 10);
    }
});
```

#### FUZZ-003: Neighbor Decompression
(Defined in previous iteration)

#### FUZZ-004: File Header Parse (NEW Implementation)

```rust
// fuzz/fuzz_targets/header_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use edgevec::persistence::FileHeader;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 64 {
        // Should validate CRC and magic, never panic
        let _ = FileHeader::read(data);
    }
});
```

---

## 4. WASM Boundary Verification

The following tests validate the JS/Rust interface (referenced in WASM_BOUNDARY.md).

### 4.1 Unit Tests (Rust)

- **UNIT-WASM-001**: `EdgeVec::new` handles valid/invalid config objects
- **UNIT-WASM-002**: `insert` rejects wrong-dimension arrays
- **UNIT-WASM-003**: `search` returns `SearchResult` array with correct shape
- **UNIT-WASM-004**: `delete` handles valid/invalid IDs
- **UNIT-WASM-005**: `save` serializes correctly
- **UNIT-WASM-006**: `load` deserializes correctly

### 4.2 Integration/E2E Tests (JS/Playwright)

- **INT-WASM-001**: Full insert/search cycle in Headless Chrome
- **INT-WASM-002**: Error handling propagation
- **INT-WASM-003**: Batch insert performance
- **INT-WASM-004**: Delete consistency
- **INT-WASM-005**: `save/load` persistence to IndexedDB mock
- **INT-WASM-006**: Memory growth handling
- **E2E-001**: End-to-end app load and initialization
- **E2E-002**: Large dataset (>10k) interaction
- **E2E-003**: Search relevance visual check
- **E2E-004**: Persistence across page reloads
- **E2E-005**: Version migration (future proofing)

---

## 5. Verification Matrix (Complete)

| Component | Unit | PBT | Fuzz | Miri | Recall | Int | E2E |
|:----------|:----:|:---:|:----:|:----:|:------:|:---:|:---:|
| VectorId | ✅ | ✅ | - | ✅ | - | - | - |
| NodeId | ✅ | ✅ | - | ✅ | - | - | - |
| HnswConfig | ✅ | ✅ | - | - | - | - | - |
| VectorStorage | ✅ | ✅ | ✅ | ✅ | - | ✅ | - |
| HnswIndex | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| NeighborPool | ✅ | ✅ | ✅ | ✅ | - | - | - |
| WAL | ✅ | ✅ | ✅ | - | - | ✅ | - |
| FileHeader | ✅ | ✅ | ✅ | - | - | - | - |
| StorageBackend | ✅ | ✅ | - | - | - | ✅ | ✅ |
| WASM Bindings | ✅ | - | - | - | - | ✅ | ✅ |
| DeterministicRng | ✅ | ✅ | - | - | - | ✅ | - |

---

## 6. Verification Checklist

Before merging ANY code:

- [ ] All unit tests pass (`cargo test`)
- [ ] Property tests pass (`cargo test --features proptest`)
- [ ] Miri detects no UB (`cargo +nightly miri test`)
- [ ] Fuzzing ran for 5+ minutes per target
- [ ] **Determinism Verified:** Replay test passes with randomized seeds
- [ ] **Recall Verified:** RECALL-001 passes with >0.95
- [ ] **Memory Budget Verified:** Static analysis confirms <100 bytes/vector

---

*Document Version: 1.2*
*Author: META_ARCHITECT*
*Status: APPROVED*
