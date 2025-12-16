# Medium Article: EdgeVec v0.4.0

---
published: true
---

# I Built a Sub-Millisecond Vector Database That Runs Entirely in Your Browser

## How a solo developer used Rust, WebAssembly, and LLM assistance to create production-ready vector search for the edge

---

Vector databases have become the backbone of modern AI applications. From semantic search to retrieval-augmented generation (RAG), they power the contextual intelligence that makes AI assistants useful.

But there's a problem: most vector databases are cloud services or require server infrastructure. What if you need vector search in a browser? What if your users' data can't leave their device? What if you're building for environments without reliable network connectivity?

These questions led me to build **EdgeVec** — a high-performance vector database that compiles to WebAssembly and runs entirely client-side.

**A note on development:** I built EdgeVec as a solo developer with significant LLM assistance (Claude). The AI helped with architecture review, code generation, test cases, and documentation. I made the core algorithmic decisions, handled performance optimization, and verified all outputs. This human-AI collaboration is a model I'd recommend — it accelerates development while maintaining quality through rigorous verification.

---

## The Edge AI Challenge

The AI industry is increasingly moving toward edge deployment. According to Gartner, by 2025, over 50% of enterprise-managed data will be created and processed outside traditional centralized data centers.

For vector search, this creates unique challenges:

**Privacy constraints**: Healthcare, finance, and legal applications often can't send embeddings to external servers. Vector embeddings, while not directly readable, can leak sensitive information about the underlying data.

**Latency requirements**: Real-time applications need sub-10ms responses. A round trip to a cloud vector database adds 50-200ms of network latency.

**Offline operation**: Progressive web apps, mobile applications, and IoT devices need to function without constant connectivity.

Existing solutions fall short. FAISS is excellent but requires Python and a server. Pinecone offers managed convenience but introduces network dependency. Browser-based alternatives like voy exist but suffer from poor performance.

---

## Building the Solution

EdgeVec uses three key technologies:

### Rust for Performance

Rust's zero-cost abstractions and memory safety guarantees make it ideal for performance-critical code. The HNSW (Hierarchical Navigable Small World) algorithm at EdgeVec's core performs millions of floating-point comparisons per search — exactly where Rust shines.

### WebAssembly for Portability

WebAssembly provides near-native execution speed in browsers. The same EdgeVec binary runs in Chrome, Firefox, Safari, Node.js, and Cloudflare Workers without modification.

### IndexedDB for Persistence

Browser storage APIs allow EdgeVec to persist vectors across sessions. Your vector index survives page refreshes and browser restarts, enabling true offline-first applications.

---

## The Numbers

Performance was a primary design goal. Here's what EdgeVec achieves:

| Metric | Value | Context |
|:-------|:------|:--------|
| Search latency | 329µs | 100k vectors, 768 dimensions |
| Bundle size | 227 KB | Gzipped WASM module |
| Memory compression | 3.6x | Via SQ8 quantization |
| Comparative speed | 24x | Faster than voy (pure-WASM) |

These numbers come from benchmarks on consumer hardware (AMD Ryzen 7 5700U). In production browser environments, you can expect similar performance.

---

## v0.4.0: Production Ready

The latest release focuses on production readiness:

**Complete documentation**: Tutorial, performance tuning guide, troubleshooting guide, and integration examples for transformers.js, TensorFlow.js, and OpenAI.

**Migration guides**: Step-by-step instructions for migrating from hnswlib, FAISS, and Pinecone.

**Quality infrastructure**: 15 chaos tests covering edge cases (empty index, maximum dimensions, all-deleted scenarios), load tests at 100k vectors, and P99 latency tracking in CI.

**Soft delete API**: Tombstone-based deletion with O(1) performance and background compaction.

---

## Use Cases

### Privacy-Preserving Search

Legal tech startup needs document similarity search without exposing client data:

```javascript
// All embeddings computed and stored locally
const embedding = await localModel.embed(document);
index.insert(embedding);

// Search never leaves the device
const similar = index.search(queryEmbedding, 10);
```

### Offline-First Applications

Field service app needs technical manual search without cellular connectivity:

```javascript
// Preload embeddings when online
await index.save("manual-embeddings");

// Search works offline
const results = index.search(problemDescription, 5);
```

### Browser-Based AI Assistants

Chatbot with persistent memory, running entirely client-side:

```javascript
// Store conversation context
index.insert(await embed(userMessage));

// Retrieve relevant history
const context = index.search(await embed(newQuestion), 3);
```

---

## The Competitive Landscape

EdgeVec occupies a specific niche: **high-performance vector search in WASM environments**.

Compared to **cloud solutions** (Pinecone, Weaviate, Qdrant): EdgeVec trades managed infrastructure for zero-latency local execution and complete data privacy.

Compared to **native libraries** (FAISS, hnswlib): EdgeVec trades raw performance for browser compatibility and deployment simplicity.

Compared to **other WASM solutions** (voy, usearch-wasm): EdgeVec offers 24x faster search with comparable bundle size.

---

## Getting Started

Installation is straightforward:

```bash
npm install edgevec
```

Basic usage:

```javascript
import init, { EdgeVec, EdgeVecConfig } from 'edgevec';

await init();

const config = new EdgeVecConfig(1536);  // OpenAI embedding dimensions
const index = new EdgeVec(config);

// Insert vectors
const id = index.insert(new Float32Array(embedding));

// Search
const results = index.search(queryVector, 10);

// Persist
await index.save("my-index");
```

---

## Future Direction

Version 0.5.0 will focus on:

- **ARM/NEON optimization**: Verified performance on Apple Silicon and ARM servers
- **Mobile support**: Formalized testing for iOS Safari and Android Chrome
- **Enhanced metadata**: Native support for storing metadata alongside vectors

The longer-term vision is making EdgeVec the default choice for any application needing vector search without server infrastructure.

---

## Conclusion

The AI industry's shift toward edge deployment creates demand for tools that work outside traditional server environments. EdgeVec demonstrates that production-quality vector search is achievable in browsers and edge devices.

If you're building privacy-sensitive applications, offline-first experiences, or low-latency AI features, EdgeVec might be the missing piece.

The project is open source under MIT/Apache-2.0 dual license. I welcome contributions, feedback, and questions.

**Links:**
- GitHub: [github.com/matte1782/edgevec](https://github.com/matte1782/edgevec)
- npm: [npmjs.com/package/edgevec](https://www.npmjs.com/package/edgevec)
- crates.io: [crates.io/crates/edgevec](https://crates.io/crates/edgevec)

---

*Matteo Panzeri is a software developer focused on AI infrastructure and edge computing. EdgeVec is his first major open-source project.*

---

## Suggested Medium Tags

1. Artificial Intelligence
2. Web Development
3. Rust Programming
4. Open Source
5. Machine Learning

## Suggested Publications to Submit To

- **Towards Data Science** — AI/ML focused, high visibility
- **Better Programming** — Software engineering, practical tutorials
- **JavaScript in Plain English** — Web development, practical focus
- **Level Up Coding** — Programming tutorials and projects

