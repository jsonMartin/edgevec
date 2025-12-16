# Dev.to Article: EdgeVec v0.4.0

---
title: Building Production-Ready Vector Search for the Browser with Rust and WebAssembly
published: true
description: How I built EdgeVec, a sub-millisecond vector database that runs entirely in your browser, as a solo developer with LLM assistance.
tags: rust, webassembly, ai, opensource
cover_image: [TODO: Add cover image URL]
canonical_url: https://github.com/matte1782/edgevec
---

## The Problem: Vector Search at the Edge

If you've worked with AI applications, you've probably used vector databases like Pinecone, Weaviate, or Qdrant. They're excellent for server-side deployments, but what happens when you need vector search:

- **In a browser** — without sending user data to external servers?
- **Offline** — where network connectivity isn't guaranteed?
- **At the edge** — where latency to cloud services is unacceptable?

This was the problem I set out to solve with **EdgeVec**.

## What is EdgeVec?

EdgeVec is an embedded vector database built in Rust that compiles to WebAssembly. It runs sub-millisecond nearest neighbor search directly in browsers, Node.js, and edge devices.

**Full disclosure:** I built this as a solo developer with LLM assistance (Claude). The AI helped with architecture review, boilerplate code, test generation, and documentation. I handled core algorithm decisions, performance optimization, and quality verification. This collaboration model accelerated development significantly.

**Key numbers:**
- **329µs** search latency at 100k vectors (768 dimensions)
- **227 KB** gzipped bundle size
- **3.6x** memory compression via scalar quantization
- **24x faster** than voy (the fastest pure-WASM alternative)

```javascript
import init, { EdgeVec, EdgeVecConfig } from 'edgevec';

await init();
const config = new EdgeVecConfig(128);  // 128 dimensions
const index = new EdgeVec(config);

// Insert vectors
const embedding = new Float32Array(128).fill(0.1);
const id = index.insert(embedding);

// Search
const results = index.search(embedding, 10);
console.log(results);  // [{ id: 0, score: 0.0 }, ...]

// Persist to IndexedDB
await index.save("my-vectors");
```

## Why WebAssembly?

I chose WebAssembly (WASM) for three reasons:

### 1. Performance

JavaScript is fast for most tasks, but vector similarity search involves millions of floating-point operations. WASM runs at near-native speed, and Rust's zero-cost abstractions mean no runtime overhead.

### 2. Portability

The same compiled WASM module runs in:
- Chrome, Firefox, Safari, Edge
- Node.js
- Deno
- Cloudflare Workers
- Any WASM runtime

One codebase, deploy anywhere.

### 3. Security

WASM runs in a sandboxed environment with no access to the file system or network by default. User embeddings never leave the device unless you explicitly send them.

## The Architecture

EdgeVec uses three core components:

### HNSW Index (Hierarchical Navigable Small World)

HNSW is a graph-based algorithm for approximate nearest neighbor search with O(log n) query complexity. Each vector becomes a node in a multi-layer graph:

- **Layer 0**: Contains all vectors, dense connections
- **Higher layers**: Contain fewer vectors, act as "express lanes"

Search starts at the top layer and descends, following the greedy path to the nearest neighbor.

### SQ8 Quantization

Full-precision vectors use 4 bytes per dimension. For 768-dimensional embeddings (like OpenAI's), that's ~3 KB per vector.

EdgeVec's scalar quantization (SQ8) compresses each dimension to 1 byte:
- **3.6x memory reduction**
- **Minimal recall loss** (~1-2% at k=10)
- **SIMD-accelerated** distance calculations

### IndexedDB Persistence

In browsers, EdgeVec stores data in IndexedDB — the same storage API used by offline-first web apps. Your vectors survive page refreshes and browser restarts.

```javascript
// Save
await index.save("my-index");

// Load
const loaded = await EdgeVec.load("my-index");
```

## What's New in v0.4.0

Version 0.4.0 focuses on production readiness:

### Documentation Suite
- **Tutorial**: Step-by-step getting started guide
- **Performance Tuning**: HNSW parameter optimization
- **Troubleshooting**: Top 10 common errors and solutions
- **Integration Guide**: Works with transformers.js, TensorFlow.js, OpenAI API

### Migration Guides
Detailed instructions for migrating from:
- **hnswlib** (Python/C++)
- **FAISS** (Python)
- **Pinecone** (Cloud)

### Quality Infrastructure
- **15 chaos tests**: Empty index, max dimensions (4096), all deleted, etc.
- **Load tests**: 100k vector stress tests
- **P99 latency tracking**: Percentile benchmarks in CI
- **Regression detection**: 10% threshold enforcement

## Performance Comparison

Benchmarked on AMD Ryzen 7 5700U, 16GB RAM:

| Library | Search P50 | Type | Notes |
|:--------|:-----------|:-----|:------|
| **EdgeVec** | **0.20ms** | WASM | Fastest pure-WASM |
| hnswlib-node | 0.05ms | Native C++ | Requires compilation |
| voy | 4.78ms | WASM | KD-tree algorithm |

EdgeVec is **24x faster than voy** while both run as pure WASM. Native bindings (hnswlib-node) are faster but don't work in browsers and require a C++ toolchain.

## Development Philosophy

LLMs are tools, like IDEs and linters. The key is understanding what you're building and verifying the output. Every piece of generated code went through manual review, unit tests, integration tests, and benchmark validation.

## Use Cases

### Offline-First Semantic Search

Build search features that work without internet:

```javascript
// User searches their local notes
const queryEmbedding = await embed("meeting notes from last week");
const results = index.search(queryEmbedding, 5);
```

### Privacy-Preserving RAG

Run retrieval-augmented generation without sending documents to external APIs:

```javascript
// All embeddings stay on device
const context = index.search(questionEmbedding, 3);
const answer = await localLLM.generate(question, context);
```

### Browser-Based AI Assistants

Add semantic memory to chatbots running entirely client-side:

```javascript
// Store conversation embeddings
index.insert(await embed(userMessage));

// Retrieve relevant context
const relevant = index.search(await embed(newMessage), 5);
```

## Getting Started

### Installation

```bash
npm install edgevec
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

### With OpenAI Embeddings

```javascript
import init, { EdgeVec, EdgeVecConfig } from 'edgevec';
import OpenAI from 'openai';

const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });

async function getEmbedding(text) {
    const response = await openai.embeddings.create({
        model: "text-embedding-3-small",
        input: text,
    });
    return new Float32Array(response.data[0].embedding);
}

await init();
const index = new EdgeVec(new EdgeVecConfig(1536));

// Index your documents
const docs = ["Hello world", "Vector search is cool"];
for (const doc of docs) {
    index.insert(await getEmbedding(doc));
}

// Search
const query = await getEmbedding("greeting");
const results = index.search(query, 5);
```

## Future Roadmap (v0.5.0+)

- **ARM/NEON optimization**: Verified cross-platform SIMD
- **Mobile support**: Formalized iOS Safari and Android Chrome testing
- **Enhanced metadata**: Native metadata storage alongside vectors
- **CLI tools**: Optional developer command-line interface

## Links

- **GitHub**: [github.com/matte1782/edgevec](https://github.com/matte1782/edgevec)
- **crates.io**: [crates.io/crates/edgevec](https://crates.io/crates/edgevec)
- **npm**: [npmjs.com/package/edgevec](https://www.npmjs.com/package/edgevec)
- **Documentation**: [Tutorial](https://github.com/matte1782/edgevec/blob/main/docs/TUTORIAL.md) | [API Reference](https://github.com/matte1782/edgevec/blob/main/docs/API_REFERENCE.md)

## Conclusion

EdgeVec proves that high-performance vector search doesn't require cloud infrastructure. With Rust and WebAssembly, we can bring AI capabilities directly to users' devices — faster, more private, and more reliable.

If you're building offline-first AI applications, privacy-preserving search, or browser-based ML tools, give EdgeVec a try. It's MIT/Apache-2.0 dual-licensed, and I welcome contributions and feedback.

---

**Questions?** Drop a comment below or open an issue on GitHub. I'm happy to discuss the implementation details or help troubleshoot integration issues.

---

## References

- [The Rise of WASM in 2024](https://dev.to/codesolutionshub/the-rise-of-wasm-webassembly-in-2024-why-every-developer-should-care-6i0) - Dev.to
- [Top 5 Vector Databases in 2024](https://bhavikjikadara.medium.com/top-5-vector-databases-in-2024-6af8ebde3f09) - Medium
- [How to Get the Right Vector Embeddings](https://medium.com/vector-database/how-to-get-the-right-vector-embeddings-83295ced7f35) - Milvus

