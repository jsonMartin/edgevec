# Hashnode Article: EdgeVec v0.4.0

---
title: EdgeVec: Sub-Millisecond Vector Search in WebAssembly
subtitle: A deep dive into building a production-ready vector database with Rust and WASM
slug: edgevec-vector-search-webassembly
cover: [TODO: Add cover image]
tags: rust, webassembly, ai, database, opensource
published: true
---

# EdgeVec: Sub-Millisecond Vector Search in WebAssembly

## A Deep Dive into Building a Production-Ready Vector Database with Rust and WASM

---

I've been working on a problem that doesn't have many good solutions: **running fast vector search directly in browsers**.

Cloud vector databases like Pinecone are great for server-side applications, but they don't work when you need:

- Vector search without network latency
- Data that never leaves the user's device
- Offline-capable applications

Today I'm releasing **EdgeVec v0.4.0**, a vector database built in Rust that compiles to WebAssembly. This post covers the technical architecture, performance characteristics, and lessons learned.

**Transparency note:** I built EdgeVec as a solo developer with LLM assistance (Claude). The AI helped with architecture review, boilerplate code, test generation, and documentation. I handled core algorithm decisions, performance profiling and optimization, integration testing, and final quality verification. Every significant piece of generated code went through manual review, unit tests, integration tests, and benchmark validation.

---

## Performance First

Let's start with the numbers:

```
Search latency:     329µs (100k vectors, 768 dimensions)
Bundle size:        227 KB gzipped
Memory compression: 3.6x via SQ8 quantization
vs. competitors:    24x faster than voy (pure-WASM)
```

These benchmarks were run on consumer hardware (AMD Ryzen 7 5700U, 16GB RAM). The key insight: WASM + Rust can deliver near-native performance in browsers.

---

## Architecture Overview

EdgeVec uses three core components:

### 1. HNSW Index

Hierarchical Navigable Small World (HNSW) is a graph-based approximate nearest neighbor algorithm. Key properties:

- **O(log n) search complexity**
- **Multi-layer structure**: Upper layers provide fast navigation, lower layers provide accuracy
- **Incremental insertions**: No need to rebuild the index when adding vectors

```rust
// Simplified HNSW search
fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
    let mut current = self.entry_point;

    // Navigate through upper layers
    for layer in (1..=self.max_layer).rev() {
        current = self.greedy_search(query, current, layer);
    }

    // Beam search in layer 0
    self.beam_search(query, current, k)
}
```

### 2. SQ8 Quantization

Full-precision vectors consume significant memory. For 768-dimensional embeddings (OpenAI's default), each vector needs ~3 KB.

Scalar quantization compresses each dimension from 4 bytes (f32) to 1 byte (u8):

```rust
// Quantization: f32 → u8
fn quantize(value: f32, min: f32, max: f32) -> u8 {
    let normalized = (value - min) / (max - min);
    (normalized * 255.0).clamp(0.0, 255.0) as u8
}

// Dequantization: u8 → f32
fn dequantize(value: u8, min: f32, max: f32) -> f32 {
    min + (value as f32 / 255.0) * (max - min)
}
```

This gives 3.6x memory compression with minimal recall loss (~1-2% at k=10).

### 3. IndexedDB Backend

Browser persistence uses IndexedDB via `wasm-bindgen`:

```rust
#[wasm_bindgen]
impl EdgeVec {
    pub async fn save(&self, name: String) -> Result<(), JsValue> {
        let bytes = postcard::to_stdvec(self)?;
        IndexedDbBackend::write(&name, &bytes).await
    }

    pub async fn load(name: String) -> Result<EdgeVec, JsValue> {
        let bytes = IndexedDbBackend::read(&name).await?;
        let index: EdgeVec = postcard::from_bytes(&bytes)?;
        Ok(index)
    }
}
```

---

## WASM-Specific Challenges

Building for WASM introduced several challenges:

### No Threads (Yet)

WASM doesn't support standard threading. This limits parallelization options. EdgeVec is currently single-threaded, but WASM threads are becoming available via `wasm-bindgen-rayon`.

### Memory Constraints

Browsers have stricter memory limits than native environments. EdgeVec uses:
- Streaming serialization for large indices
- Chunked export/import APIs
- Efficient memory layouts (bytemuck for zero-copy operations)

### No File System

Obviously browsers don't have file system access. IndexedDB provides persistent storage, but with different APIs than traditional file I/O.

---

## The Soft Delete Problem

Version 0.3.0 introduced soft delete — marking vectors as deleted without rebuilding the index.

The naive approach (actually removing nodes from the graph) would break HNSW's connectivity guarantees. Instead, EdgeVec uses tombstones:

```rust
pub struct HnswNode {
    vector_offset: u32,
    neighbor_offset: u32,
    max_layer: u8,
    deleted: u8,  // 0 = live, 1 = tombstone
}
```

Tombstones are skipped during search:

```rust
fn is_valid_neighbor(&self, node_id: NodeId) -> bool {
    self.nodes[node_id.0 as usize].deleted == 0
}
```

When tombstones accumulate (>30% by default), `compact()` rebuilds the index:

```rust
pub fn compact(&self) -> Result<(HnswIndex, VectorStorage), GraphError> {
    let live_vectors = self.nodes.iter()
        .filter(|n| n.deleted == 0)
        .collect();

    // Rebuild fresh index with only live vectors
    let mut new_index = HnswIndex::new(self.config)?;
    for vector in live_vectors {
        new_index.insert(vector)?;
    }

    Ok(new_index)
}
```

---

## Quality Infrastructure

v0.4.0 emphasizes production readiness:

### Chaos Testing

15 edge case tests covering scenarios like:
- Empty index behavior
- Single vector index
- All vectors deleted
- Maximum dimensions (4096)
- Duplicate vectors
- Rapid insert-delete cycles

```rust
#[test]
fn test_empty_index_search() {
    let index = HnswIndex::new(HnswConfig::new(128))?;
    let results = index.search(&vec![0.0; 128], 10)?;
    assert!(results.is_empty());
}

#[test]
fn test_all_deleted_search() {
    let mut index = create_index_with_100_vectors();
    for i in 0..100 {
        index.soft_delete(VectorId(i))?;
    }
    let results = index.search(&query, 10)?;
    assert!(results.is_empty());
}
```

### P99 Latency Tracking

Benchmarks report percentile latencies, not just means:

```rust
fn report_percentiles(latencies: &[f64]) {
    latencies.sort();
    let p50 = latencies[latencies.len() * 50 / 100];
    let p99 = latencies[latencies.len() * 99 / 100];
    let p999 = latencies[latencies.len() * 999 / 1000];

    println!("P50: {:.2}µs, P99: {:.2}µs, P999: {:.2}µs", p50, p99, p999);
}
```

### CI Regression Detection

GitHub Actions workflow that fails if P99 latency increases >10%:

```yaml
- name: Run P99 Benchmark
  run: cargo bench --bench p99_bench

- name: Check Regression
  run: |
    if [ "$NEW_P99" -gt "$((OLD_P99 * 110 / 100))" ]; then
      echo "Performance regression detected!"
      exit 1
    fi
```

---

## Quick Start

### Installation

```bash
npm install edgevec
```

### Browser Usage

```javascript
import init, { EdgeVec, EdgeVecConfig } from 'edgevec';

async function main() {
    await init();

    const config = new EdgeVecConfig(128);
    config.metric = 'cosine';
    const index = new EdgeVec(config);

    // Insert
    const vector = new Float32Array(128).fill(0.1);
    const id = index.insert(vector);

    // Search
    const results = index.search(vector, 10);
    console.log(results);  // [{ id: 0, score: 0.0 }]

    // Persist
    await index.save("my-index");
}
```

### Rust Usage

```rust
use edgevec::{HnswConfig, HnswIndex, VectorStorage};

let config = HnswConfig::new(128);
let mut storage = VectorStorage::new(&config, None);
let mut index = HnswIndex::new(config, &storage)?;

let id = index.insert(&vec![1.0; 128], &mut storage)?;
let results = index.search(&vec![1.0; 128], 10, &storage)?;
```

---

## Future Plans

### v0.5.0

- ARM/NEON SIMD verification
- Formalized mobile browser testing
- Enhanced metadata storage

### Longer Term

- WASM threads for parallel search
- Product quantization (PQ) for higher compression
- Streaming indices for very large datasets

---

## Conclusion

EdgeVec demonstrates that production-quality vector search is possible in browser environments. The combination of Rust's performance, WASM's portability, and modern browser APIs creates opportunities for AI applications that weren't feasible a few years ago.

If you're building offline-first AI applications, privacy-sensitive search, or browser-based ML tools, give EdgeVec a try.

**Links:**
- GitHub: [github.com/matte1782/edgevec](https://github.com/matte1782/edgevec)
- npm: [npmjs.com/package/edgevec](https://www.npmjs.com/package/edgevec)
- crates.io: [crates.io/crates/edgevec](https://crates.io/crates/edgevec)
- Documentation: [TUTORIAL.md](https://github.com/matte1782/edgevec/blob/main/docs/TUTORIAL.md)

---

## Hashnode Tags

1. rust
2. webassembly
3. ai
4. database
5. opensource

