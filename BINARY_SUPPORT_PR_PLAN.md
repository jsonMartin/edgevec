# EdgeVec Binary Support PR - Implementation Plan

## Executive Summary

**Goal:** Add binary vector support with Hamming distance to EdgeVec, enabling 1M+ vectors in browser (128 MB vs 832 MB for SQ8).

**Key Features:**
1. `insert_binary(Uint8Array)` - Direct binary insertion (for Turso sync)
2. Binary Quantization mode - Convert f32 → binary on insert
3. Hamming distance metric - For binary vector search
4. Full filtering support - SQL-like filters work with binary

**Estimated Effort:** 2-3 days (10-15 hours)
**Lines Changed:** ~500-800

---

## Part 1: Existing Components to Leverage

### Already Implemented in Rust (Just Needs Wiring)

| Component | Location | Status |
|-----------|----------|--------|
| `BinaryQuantizer` | `src/quantization/binary.rs` | ✅ Exists - f32 → bits |
| `Hamming` metric | `src/metric/hamming.rs` | ✅ Exists - `impl Metric<u8>` |
| Bit packing | `src/quantization/binary.rs` | ✅ Exists - LSB-first |
| SIMD popcount | `src/metric/hamming.rs` | ✅ Exists - AVX2/WASM |
| Filter system | `src/filter/` | ✅ Exists - reusable |
| IndexedDB persistence | `src/wasm/` | ✅ Exists - needs binary type |

### Needs Modification

| Component | Location | Change Needed |
|-----------|----------|---------------|
| `Searcher` | `src/hnsw/search.rs` | Add `BinarySearcher` or make generic |
| `Storage` | `src/storage.rs` | Add `StorageType::Binary` |
| `HnswConfig` | `src/hnsw/config.rs` | Add "hamming" metric option |
| WASM bindings | `src/wasm/mod.rs` | Add `insert_binary()`, expose hamming |

---

## Part 2: Files to Create/Modify

### New Files

```
src/hnsw/binary_search.rs    (~150-200 lines)
├── BinarySearcher struct
├── search() method using Hamming distance
├── search_filtered() method
└── Reuses existing HNSW graph traversal logic

src/storage/binary.rs        (~100-150 lines)
├── BinaryStorage struct
├── Store/retrieve packed bytes
├── No quantization (direct storage)
└── Persistence to IndexedDB
```

### Modified Files

```
src/lib.rs                   (~10 lines)
└── Export new binary types

src/hnsw/mod.rs              (~5 lines)
└── pub mod binary_search;

src/hnsw/config.rs           (~20 lines)
├── Add MetricType::Hamming
└── Add VectorType::Binary

src/storage.rs               (~50 lines)
├── Add StorageType::Binary
├── binary_data: Vec<u8> field
└── get_binary_vector() method

src/wasm/mod.rs              (~100 lines)
├── insert_binary(vector: &[u8], metadata: JsValue)
├── insert_with_bq(vector: &[f32], metadata: JsValue)  // BQ mode
├── search_binary(query: &[u8], k: usize)
├── set_metric("hamming") support
└── set_vector_type("binary") support

src/metric/mod.rs            (~5 lines)
└── Export Hamming for WASM
```

---

## Part 3: Implementation Phases

### Phase 1: Binary Storage (~2-3 hours)

**Goal:** Store and retrieve binary vectors without search.

```rust
// src/storage.rs additions

pub enum StorageType {
    Float32,
    QuantizedU8,
    Binary,  // NEW
}

pub struct Storage {
    // Existing
    data_f32: Vec<f32>,
    quantized_data: Vec<u8>,
    // New
    binary_data: Vec<u8>,
    binary_dimensions: usize,  // In bits (e.g., 1024)
}

impl Storage {
    pub fn insert_binary(&mut self, vector: &[u8]) -> VectorId {
        let id = self.next_id();
        let offset = self.binary_data.len();
        self.binary_data.extend_from_slice(vector);
        self.binary_offsets.push(offset);
        id
    }

    pub fn get_binary_vector(&self, id: VectorId) -> &[u8] {
        let bytes_per_vector = self.binary_dimensions / 8;
        let offset = self.binary_offsets[id as usize];
        &self.binary_data[offset..offset + bytes_per_vector]
    }
}
```

**Test:** Insert binary vector, retrieve it, verify bytes match.

---

### Phase 2: Hamming Distance Integration (~2-3 hours)

**Goal:** Wire existing Hamming metric to search path.

```rust
// src/hnsw/binary_search.rs

use crate::metric::Hamming;

pub struct BinarySearcher<'a> {
    storage: &'a Storage,
    graph: &'a HnswGraph,
    config: &'a HnswConfig,
}

impl<'a> BinarySearcher<'a> {
    pub fn search(&self, query: &[u8], k: usize) -> Vec<SearchResult> {
        let mut candidates = BinaryHeap::new();
        let mut visited = HashSet::new();

        // Start from entry point
        let entry = self.graph.entry_point();

        // Greedy search using Hamming distance
        // (Similar to existing search, but using Hamming::distance)

        self.search_layer(query, entry, k, &mut candidates, &mut visited);

        candidates.into_sorted_vec()
    }

    fn distance(&self, query: &[u8], candidate_id: VectorId) -> f32 {
        let candidate = self.storage.get_binary_vector(candidate_id);
        Hamming::distance(query, candidate)
    }
}
```

**Test:** Insert 1000 binary vectors, search, verify Hamming distances correct.

---

### Phase 3: Binary Quantization Mode (~1-2 hours)

**Goal:** Allow f32 input with automatic BQ.

```rust
// src/wasm/mod.rs additions

#[wasm_bindgen]
impl EdgeVec {
    /// Insert f32 vector, automatically quantize to binary
    pub fn insert_with_bq(&mut self, vector: &[f32], metadata: JsValue) -> u64 {
        // Use existing BinaryQuantizer
        let binary = BinaryQuantizer::quantize(vector);
        self.insert_binary_internal(&binary, metadata)
    }
}
```

**Test:** Insert f32 vector with BQ, verify sign bits match manual calculation.

---

### Phase 4: WASM Bindings (~2-3 hours)

**Goal:** Expose full API to JavaScript.

```rust
// src/wasm/mod.rs

#[wasm_bindgen]
impl EdgeVec {
    /// Direct binary insertion (for pre-quantized data like Turso f1bit_blob)
    #[wasm_bindgen]
    pub fn insert_binary(&mut self, vector: &[u8], metadata: JsValue) -> u64 {
        // Validate dimensions
        let expected_bytes = self.config.dimensions / 8;
        if vector.len() != expected_bytes {
            panic!("Expected {} bytes for {} bits", expected_bytes, self.config.dimensions);
        }

        self.storage.insert_binary(vector);
        // ... handle metadata, update graph
    }

    /// Search with binary query
    #[wasm_bindgen]
    pub fn search_binary(&self, query: &[u8], k: usize) -> JsValue {
        let searcher = BinarySearcher::new(&self.storage, &self.graph, &self.config);
        let results = searcher.search(query, k);
        serde_wasm_bindgen::to_value(&results).unwrap()
    }

    /// Search with binary query and filter
    #[wasm_bindgen]
    pub fn search_binary_filtered(
        &self,
        query: &[u8],
        filter: &str,
        k: usize
    ) -> JsValue {
        // Reuse existing filter parsing and strategy selection
        let filter_expr = parse_filter(filter);
        let searcher = BinarySearcher::new(&self.storage, &self.graph, &self.config);
        let results = searcher.search_filtered(query, &filter_expr, k);
        serde_wasm_bindgen::to_value(&results).unwrap()
    }
}
```

**Test:** Full integration test from JS - insert, search, filter.

---

### Phase 5: Config & Persistence (~1-2 hours)

**Goal:** Configuration options and IndexedDB support.

```rust
// src/hnsw/config.rs additions

#[wasm_bindgen]
pub enum VectorType {
    Float32,
    Quantized,  // SQ8
    Binary,     // NEW
}

#[wasm_bindgen]
pub enum MetricType {
    L2,
    Cosine,
    Dot,
    Hamming,  // NEW
}

impl EdgeVecConfig {
    pub fn set_vector_type(&mut self, vtype: &str) {
        self.vector_type = match vtype {
            "float32" => VectorType::Float32,
            "quantized" | "sq8" => VectorType::Quantized,
            "binary" => VectorType::Binary,
            _ => panic!("Unknown vector type"),
        };
    }

    pub fn set_metric(&mut self, metric: &str) {
        self.metric = match metric {
            "l2" => MetricType::L2,
            "cosine" => MetricType::Cosine,
            "dot" => MetricType::Dot,
            "hamming" => MetricType::Hamming,
            _ => panic!("Unknown metric"),
        };
    }
}
```

**Test:** Create binary index, persist to IndexedDB, reload, verify search works.

---

### Phase 6: Testing & Documentation (~2 hours)

```rust
// tests/binary_test.rs

#[test]
fn test_binary_insert_and_search() {
    let mut db = EdgeVec::new(1024);  // 1024 bits = 128 bytes
    db.config.set_vector_type("binary");
    db.config.set_metric("hamming");

    // Insert 1000 random binary vectors
    for i in 0..1000 {
        let binary = random_binary_vector(128);
        db.insert_binary(&binary, json!({ "id": i }));
    }

    // Search
    let query = random_binary_vector(128);
    let results = db.search_binary(&query, 10);

    assert_eq!(results.len(), 10);
    // Verify Hamming distances are sorted
    for i in 1..results.len() {
        assert!(results[i].distance >= results[i-1].distance);
    }
}

#[test]
fn test_binary_quantization_from_f32() {
    let f32_vec: Vec<f32> = (0..1024).map(|i| if i % 2 == 0 { 0.5 } else { -0.5 }).collect();
    let binary = BinaryQuantizer::quantize(&f32_vec);

    // Verify alternating pattern
    // Even indices: 0.5 > 0 → bit = 1
    // Odd indices: -0.5 < 0 → bit = 0
    assert_eq!(binary[0], 0b01010101);  // LSB-first
}

#[test]
fn test_filtered_binary_search() {
    let mut db = EdgeVec::new(1024);
    db.config.set_vector_type("binary");
    db.config.set_metric("hamming");

    // Insert with metadata
    for i in 0..1000 {
        let binary = random_binary_vector(128);
        let project = if i % 2 == 0 { "A" } else { "B" };
        db.insert_binary(&binary, json!({ "project": project }));
    }

    // Filtered search
    let query = random_binary_vector(128);
    let results = db.search_binary_filtered(&query, "project = 'A'", 10);

    assert_eq!(results.len(), 10);
    // Verify all results have project A
    for r in &results {
        assert_eq!(r.metadata["project"], "A");
    }
}
```

---

## Part 4: JavaScript API

```typescript
// Usage Example: TypeScript/JavaScript

import { EdgeVec, EdgeVecConfig } from 'edgevec';

// === Configuration ===
const config = new EdgeVecConfig();
config.dimensions = 1024;        // 1024 bits
config.vector_type = 'binary';   // NEW
config.metric = 'hamming';       // NEW
config.m = 16;
config.ef_construction = 100;
config.ef_search = 50;

const db = new EdgeVec(config);

// === Path A: Direct binary insertion (from Turso f1bit_blob) ===
const tursoBlob = row.embedding_binary;           // Uint8Array(131)
const binaryVector = tursoBlob.slice(0, 128);     // Strip 3-byte trailer

const id = db.insert_binary(binaryVector, {
  turso_id: row.id,
  project_id: row.project_id,
  tags: row.tags
});

// === Path B: Binary quantization from f32 ===
const f32Embedding = await model.embed(text);     // Float32Array(1024)
const id = db.insert_with_bq(f32Embedding, {
  turso_id: row.id
});

// === Search ===
const queryBinary = getQueryBinary();             // Uint8Array(128)
const results = db.search_binary(queryBinary, 10);

// results = [
//   { id: 42, distance: 156, metadata: { turso_id: "...", ... } },
//   { id: 17, distance: 162, metadata: { ... } },
//   ...
// ]

// === Filtered Search ===
const results = db.search_binary_filtered(
  queryBinary,
  "project_id = 'abc' AND 'work' IN tags",
  10
);

// === Persistence ===
await db.save();                                  // IndexedDB
const db2 = await EdgeVec.load('my-index');       // Restore
```

---

## Part 5: PR Description Template

```markdown
## feat: Add binary vector support with Hamming distance

### Summary

This PR adds native binary vector support to EdgeVec, enabling efficient
storage and search of 1-bit quantized embeddings using Hamming distance.

### Motivation

- **Memory efficiency**: 128 MB for 1M vectors (vs 832 MB for SQ8)
- **Mobile-friendly**: Enables 1M+ vectors in browser/mobile
- **Direct sync**: Compatible with Turso f1bit_blob without conversion
- **No quantization mismatch**: Binary (sign bits) is deterministic across devices

### Features

1. **Direct binary insertion** - `insert_binary(Uint8Array)`
   - For pre-quantized data (e.g., Turso f1bit_blob)
   - No conversion overhead

2. **Binary quantization mode** - `insert_with_bq(Float32Array)`
   - Automatically applies sign() to each dimension
   - For users with f32 embeddings

3. **Hamming distance metric** - `config.metric = 'hamming'`
   - SIMD-optimized popcount
   - O(log n) HNSW search

4. **Full filter support** - Works with existing SQL-like filter system
   - Pre-filter, hybrid, and post-filter strategies all supported

### API

\`\`\`typescript
// Configuration
config.vector_type = 'binary';
config.metric = 'hamming';

// Insertion
db.insert_binary(uint8Array, metadata);      // Direct
db.insert_with_bq(float32Array, metadata);   // With BQ

// Search
db.search_binary(query, k);
db.search_binary_filtered(query, filterExpr, k);
\`\`\`

### Memory Comparison (1M vectors)

| Format | Storage | Mobile Feasible |
|--------|---------|-----------------|
| Float32 | 3.07 GB | ❌ |
| SQ8 | 832 MB | ⚠️ Borderline |
| **Binary** | **128 MB** | ✅ |

### Implementation Details

- Leverages existing `BinaryQuantizer` and `Hamming` metric
- New `BinarySearcher` for u8 vector search
- Reuses existing filter infrastructure
- IndexedDB persistence support

### Testing

- Unit tests for binary storage, BQ, Hamming distance
- Integration tests for filtered search
- Benchmark: 100K vectors, search latency

### Breaking Changes

None. Existing f32 and SQ8 functionality unchanged.
```

---

## Timeline Summary

| Phase | Task | Hours |
|-------|------|-------|
| 1 | Binary Storage | 2-3 |
| 2 | Hamming Integration | 2-3 |
| 3 | Binary Quantization | 1-2 |
| 4 | WASM Bindings | 2-3 |
| 5 | Config & Persistence | 1-2 |
| 6 | Testing & Docs | 2 |
| **Total** | | **10-15 hours** |

---

## Key Files to Read First

Before starting implementation, read these files to understand existing patterns:

1. `src/hnsw/search.rs` - Existing search implementation
2. `src/quantization/binary.rs` - BinaryQuantizer (already exists!)
3. `src/metric/hamming.rs` - Hamming distance (already exists!)
4. `src/wasm/mod.rs` - Existing WASM bindings
5. `src/storage.rs` - Vector storage patterns
6. `src/filter/strategy.rs` - Filter strategies (Pre/Hybrid/Post)

---

## Use Case: Turso f1bit_blob Sync

```typescript
// Sync binary embeddings from Turso to EdgeVec

async function syncEmbeddings() {
  const db = new EdgeVec(config);

  // Fetch from Turso
  const rows = await turso.execute(
    "SELECT id, embedding_binary, project_id, tags FROM notes"
  );

  for (const row of rows) {
    // Strip Turso's 3-byte trailer
    const binary = new Uint8Array(row.embedding_binary).slice(0, 128);

    db.insert_binary(binary, {
      turso_id: row.id,
      project_id: row.project_id,
      tags: row.tags
    });
  }

  await db.save();  // Persist to IndexedDB
}

// Search
async function search(queryText: string) {
  const queryF32 = await model.embed(queryText);
  const queryBinary = binaryQuantize(queryF32);  // sign() each dimension

  const results = db.search_binary_filtered(
    queryBinary,
    "project_id = 'current-project'",
    10
  );

  // Fetch full notes from Turso for rescoring
  const tursoIds = results.map(r => r.metadata.turso_id);
  const notes = await turso.execute(
    `SELECT * FROM notes WHERE id IN (${placeholders})`,
    tursoIds
  );

  return notes;
}
```
