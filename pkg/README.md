# 🚀 `EdgeVec`

**High-performance vector search for Browser, Node, and Edge**

> 🚧 **STATUS: Phase 4 (WASM Integration) COMPLETE** — Ready for Final Review.

---

## What is `EdgeVec`?

`EdgeVec` is an embedded vector database built in Rust with first-class WASM support. It's designed to run anywhere: browsers, Node.js, mobile apps, and edge devices.

### Key Features

- **HNSW Indexing** — O(log n) approximate nearest neighbor search
- **WASM-First** — Native browser support via WebAssembly
- **Persistent Storage** — `IndexedDB` in browser, file system elsewhere
- **Minimal Dependencies** — No C compiler required, WASM-ready
- **Tiny Bundle** — Target <500KB gzipped

---

## ⚡ Quick Start

### Installation

```bash
npm install @edgevec/core
```

### Browser Usage

```javascript
import { EdgeVecClient } from '@edgevec/core';

async function main() {
    // 1. Create Index (auto-initializes WASM)
    const client = await EdgeVecClient.create({ dimensions: 128 });

    // 2. Insert Vectors (synchronous)
    const vector = new Float32Array(128).fill(0.1);
    const id = client.insert(vector);
    console.log(`Inserted vector with ID: ${id}`);

    // 3. Search (synchronous)
    const query = new Float32Array(128).fill(0.1);
    const results = client.search(query, 10);
    console.log("Results:", results);

    // 4. Save to IndexedDB
    await client.save("my-vector-db");
}

main().catch(console.error);
```

### Node.js Usage

```javascript
import { EdgeVecClient } from '@edgevec/core';

// Create and use synchronously after initialization
const client = await EdgeVecClient.create({
    dimensions: 128,
    metric: 'cosine' // Optional: 'l2', 'cosine', or 'dot'
});

// Insert vectors (synchronous)
const vector1 = new Float32Array(128).fill(0.1);
const vector2 = new Float32Array(128).fill(0.2);
const id1 = client.insert(vector1);
const id2 = client.insert(vector2);

// Search (synchronous)
const results = client.search(vector1, 10);
console.log(`Found ${results.length} results`);
console.log(`Top result: ID=${results[0].id}, distance=${results[0].distance}`);

// Persistence
await client.save("my-db");
const loadedClient = await EdgeVecClient.load("my-db", { dimensions: 128 });
```

### Rust Usage

```rust,no_run
use edgevec::{HnswConfig, HnswIndex, VectorStorage};
use edgevec::persistence::{write_snapshot, MemoryBackend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create Config & Storage
    let config = HnswConfig::new(128);
    let mut storage = VectorStorage::new(&config, None);

    // 2. Create Index
    let mut index = HnswIndex::new(config, &storage)?;

    // 3. Insert Vectors
    let vec1 = vec![1.0; 128];
    let _id1 = index.insert(&vec1, &mut storage)?;

    // 4. Search
    let query = vec![1.0; 128];
    let results = index.search(&query, 10, &storage)?;
    println!("Found {} results", results.len());

    // 5. Save Snapshot
    let mut backend = MemoryBackend::new();
    write_snapshot(&index, &storage, &mut backend)?;

    Ok(())
}
```

---

## Development Status

`EdgeVec` follows a **military-grade development protocol**. No code is written without an approved plan.

### Current Phase: Phase 4 Complete — WASM Integration

**What Works Now:**
- ✅ **HNSW Indexing** (Insertion & Search)
- ✅ **Scalar Quantization (SQ8)** (4x Memory Reduction)
- ✅ **Crash Recovery (WAL)** (Log-based replay)
- ✅ **Atomic Snapshots** (Safe background saving)
- ✅ **Browser Integration** (WASM Bindings + IndexedDB)
- ✅ **Persistence** (Automatic IndexedDB Adapter)

**Progress:**
- Phase 0: Environment Setup — ✅ COMPLETE
- Phase 1: Architecture — ✅ COMPLETE
- Phase 2: Planning — ✅ COMPLETE
- Phase 3: Implementation — ✅ COMPLETE
- Phase 4: WASM Integration — ✅ COMPLETE
- Phase 5: Release Polish — 🚧 PENDING

### What's Built So Far

- [x] Agent system (7 specialized AI agents)
- [x] Development protocol (`.cursorrules`)
- [x] **Distance Metrics** (L2, Cosine, Dot Product)
- [x] **HNSW Indexing** (Insert, Search, Heuristics)
- [x] **Vector Storage** (Contiguous Memory Layout)
- [x] **Write-Ahead Log (WAL)** (Durability & Crash Recovery)
- [x] **Atomic Snapshots** (Compression & Fast Load)
- [x] **WASM Bindings** (Core functionality exposed)
- [x] **NPM Packaging** (Bundled JS adapter)

> **Note:** The core engine is now feature-complete and hardening is underway.

### What's Next (Phase 5: Release Polish)

1. **Documentation** — Finalize API docs and examples
2. **NPM Package** — Release to npm registry
3. **Performance Tuning** — Final benchmarks and optimizations
4. **v1.0.0 Launch**

---

## 📊 Performance (Week 7)

Benchmarked on AMD Ryzen 7, 100k vectors (128d):

| Metric | Result | Notes |
|:-------|:-------|:------|
| **Snapshot Load** | **51 ms** | Cold start from disk |
| **Snapshot Save** | **65 ms** | Atomic background save |
| **Search P99** | **< 3.5 ms** | k=10, 128d |
| **WAL Overhead** | **~50 ns** | Memory-backed append |

[Full benchmarks →](docs/benchmarks/)

---

## Development Protocol

### The Agents

| Agent | Role |
|:------|:-----|
| **META_ARCHITECT** | System design, data layouts |
| **PLANNER** | Roadmaps, weekly task plans |
| **`RUST_ENGINEER`** | Core Rust implementation |
| **`WASM_SPECIALIST`** | WASM bindings, browser integration |
| **`BENCHMARK_SCIENTIST`** | Performance testing |
| **HOSTILE_REVIEWER** | Quality gate (has veto power) |
| **DOCWRITER** | Documentation, README |

---

## Origins

`EdgeVec` builds upon lessons learned from [binary_semantic_cache](../binary_semantic_cache/), a high-performance semantic caching library. Specifically:

**Salvaged (MIT Licensed):**
- Hamming distance implementation (~10 lines)
- Binary quantization math (~100 lines)

**Built Fresh:**
- HNSW graph indexing
- WASM-native architecture
- `IndexedDB` persistence
- Everything else

---

## License

MIT — See [LICENSE](https://github.com/anthropics/edgevec/blob/main/LICENSE)

---

<div align="center">

**Built with 🦀 Rust + 🕸️ WebAssembly**

*Correctness by Construction*

</div>
