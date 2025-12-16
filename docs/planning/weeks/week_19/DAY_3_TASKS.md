# Week 19 Day 3: User Documentation Sprint

**Task ID:** W19.3
**Date:** 2025-12-18
**Estimated Hours:** 8 hours (3x rule: 2.67h optimistic × 3 = 8h)
**Base Estimate:** 2.67 hours (4 docs × 40 min each = 2.67h)
**Risk Buffer:** +5.33 hours (example debugging, testing in both environments)
**Contingency:** If running late, prioritize TUTORIAL.md (highest value), defer TROUBLESHOOTING.md refinement to post-v0.4.0
**Dependencies:** W19.1 (Reconciliation)
**Priority:** HIGH

---

## Objective

Create comprehensive user-facing documentation for v0.4.0 launch. This includes a getting started tutorial, performance tuning guide, troubleshooting reference, and integration guide for common ML libraries. All documentation must be tested and verified to work when copy-pasted.

---

## Background

**Existing Documentation:**
- `README.md` - Basic usage examples
- `docs/API_REFERENCE.md` - Complete API documentation
- `docs/BROWSER_COMPATIBILITY.md` - Browser support matrix
- `docs/benchmarks/competitive_analysis.md` - Performance analysis

**Missing Documentation:**
- Step-by-step tutorial for beginners
- Parameter tuning guide for optimization
- Troubleshooting guide for common issues

**Target Audience:** Developers new to EdgeVec or vector databases in general.

---

## Deliverables

| # | Deliverable | Path | Type |
|:--|:------------|:-----|:-----|
| 1 | Getting Started Tutorial | `docs/TUTORIAL.md` | Doc |
| 2 | Performance Tuning Guide | `docs/PERFORMANCE_TUNING.md` | Doc |
| 3 | Troubleshooting Guide | `docs/TROUBLESHOOTING.md` | Doc |
| 4 | Integration Guide | `docs/INTEGRATION_GUIDE.md` | Doc |

---

## Acceptance Criteria

- [ ] AC1: Tutorial can be followed start-to-finish by someone unfamiliar with EdgeVec
- [ ] AC2: All code examples in tutorial compile and run without modification
- [ ] AC3: Performance tuning guide covers all 6 HNSW parameters (M, efConstruction, ef, etc.)
- [ ] AC4: Troubleshooting guide covers 10+ common error scenarios
- [ ] AC5: Each document has table of contents and clear section headers
- [ ] AC6: Examples work in both Node.js and browser environments
- [ ] AC7: Integration guide covers transformers.js, @xenova/transformers, and TensorFlow.js

---

## Implementation Steps

### Step 1: Create TUTORIAL.md (3 hours)

**Structure:**
```markdown
# EdgeVec Tutorial: Getting Started

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Your First Vector Index](#your-first-vector-index)
4. [Inserting Vectors](#inserting-vectors)
5. [Searching for Similar Vectors](#searching)
6. [Persistence (Save/Load)](#persistence)
7. [Soft Delete and Compaction](#soft-delete)
8. [Batch Operations](#batch-operations)
9. [Next Steps](#next-steps)

---

## Prerequisites

- Node.js 18+ or modern browser (Chrome 90+, Firefox 90+, Safari 15+)
- npm or yarn package manager
- Basic understanding of vectors and similarity search

---

## Installation

### npm (Node.js and bundlers)
```bash
npm install edgevec
```

### Browser (ES Modules)
```html
<script type="module">
import init, { EdgeVec, EdgeVecConfig } from 'https://unpkg.com/edgevec/edgevec.js';
</script>
```

---

## Your First Vector Index

Let's create a simple vector index and perform a search:

```javascript
import init, { EdgeVec, EdgeVecConfig } from 'edgevec';

async function main() {
    // 1. Initialize WASM module (required once)
    await init();

    // 2. Create configuration for 128-dimensional vectors
    const config = new EdgeVecConfig(128);
    config.metric = 'cosine'; // Options: 'l2', 'cosine', 'dot'

    // 3. Create the index
    const index = new EdgeVec(config);

    console.log('EdgeVec index created!');
    console.log('Dimensions:', index.dimensions());
    console.log('Vector count:', index.len());
}

main();
```

**Expected Output:**
```
EdgeVec index created!
Dimensions: 128
Vector count: 0
```

[Continue with more sections...]
```

### Step 2: Create PERFORMANCE_TUNING.md (2.5 hours)

**Structure:**
```markdown
# EdgeVec Performance Tuning Guide

## Table of Contents
1. [Overview](#overview)
2. [HNSW Parameters](#hnsw-parameters)
3. [Quantization Options](#quantization)
4. [Memory vs Speed Tradeoffs](#tradeoffs)
5. [Recommended Configurations](#recommendations)
6. [Benchmarking Your Workload](#benchmarking)

---

## Overview

EdgeVec uses the HNSW (Hierarchical Navigable Small World) algorithm for approximate nearest neighbor search. Performance can be tuned by adjusting several parameters.

**Key Metrics:**
- **Search Latency:** Time to find k nearest neighbors
- **Insert Latency:** Time to add a vector to the index
- **Recall:** Accuracy of approximate search vs brute force
- **Memory Usage:** RAM consumed by the index

---

## HNSW Parameters

### M (Max Connections)

**Default:** 16
**Range:** 4-64
**Impact:** Memory usage and recall

| M Value | Memory Impact | Recall Impact | Best For |
|:--------|:--------------|:--------------|:---------|
| 8 | Low | Lower | Memory-constrained, low accuracy OK |
| 16 | Medium | Good | General purpose (recommended) |
| 32 | High | Higher | High accuracy requirements |
| 64 | Very High | Highest | Maximum accuracy, memory not a concern |

```javascript
const config = new EdgeVecConfig(128);
config.m = 32; // Higher M for better recall
```

### efConstruction (Build-time Search Width)

**Default:** 200
**Range:** 100-500
**Impact:** Index quality and build time

Higher values = better quality index, slower build.

```javascript
config.efConstruction = 400; // Higher quality, slower build
```

### ef (Search-time Beam Width)

**Default:** 50
**Range:** 10-500
**Impact:** Search accuracy and latency

```javascript
// Set at search time
const results = index.search(query, 10, { ef: 100 });
```

[Continue with more parameters...]

---

## Recommended Configurations

### Use Case: Real-time Semantic Search
```javascript
const config = new EdgeVecConfig(768);
config.m = 16;
config.efConstruction = 200;
// ef = 50-100 at search time
```

### Use Case: High-Accuracy Recommendation
```javascript
const config = new EdgeVecConfig(384);
config.m = 32;
config.efConstruction = 400;
// ef = 200-300 at search time
```

### Use Case: Memory-Constrained Edge Device
```javascript
const config = new EdgeVecConfig(128);
config.m = 8;
config.efConstruction = 100;
config.quantized = true; // 4x memory reduction
```
```

### Step 3: Create TROUBLESHOOTING.md (2.5 hours)

**Structure:**
```markdown
# EdgeVec Troubleshooting Guide

## Table of Contents
1. [Installation Issues](#installation-issues)
2. [Initialization Errors](#initialization-errors)
3. [Insert Errors](#insert-errors)
4. [Search Issues](#search-issues)
5. [Performance Problems](#performance-problems)
6. [Persistence Errors](#persistence-errors)
7. [Browser-Specific Issues](#browser-issues)
8. [Memory Issues](#memory-issues)

---

## Installation Issues

### Error: "Cannot find module 'edgevec'"

**Cause:** Package not installed or wrong import path.

**Solution:**
```bash
# Verify installation
npm list edgevec

# Reinstall if needed
npm install edgevec
```

### Error: "WebAssembly is not defined"

**Cause:** Running in environment without WASM support.

**Solution:** Ensure you're using:
- Node.js 12+ (WASM support)
- Modern browser (Chrome 57+, Firefox 52+, Safari 11+)
- NOT running in a Web Worker without proper WASM import

---

## Initialization Errors

### Error: "WASM module not initialized"

**Cause:** Using EdgeVec before calling `init()`.

**Solution:**
```javascript
import init, { EdgeVec } from 'edgevec';

// WRONG - init not awaited
const index = new EdgeVec(config); // Error!

// CORRECT - await init first
await init();
const index = new EdgeVec(config); // Works!
```

### Error: "Dimension mismatch"

**Cause:** Inserting vector with wrong dimension.

**Solution:**
```javascript
const config = new EdgeVecConfig(128); // 128 dimensions
const index = new EdgeVec(config);

// WRONG - 256 dimensions
const badVector = new Float32Array(256);
index.insert(badVector); // Error!

// CORRECT - 128 dimensions
const goodVector = new Float32Array(128);
index.insert(goodVector); // Works!
```

---

## Search Issues

### Problem: Search returns empty results

**Possible Causes:**
1. Index is empty
2. Query vector is all zeros
3. ef parameter too low

**Debugging Steps:**
```javascript
console.log('Index size:', index.len());
console.log('Query has values:', query.some(v => v !== 0));

// Try higher ef
const results = index.search(query, 10, { ef: 200 });
```

### Problem: Search results have poor accuracy

**Solution:** Increase ef (search-time accuracy):
```javascript
// Default ef = 50
const lowAccuracy = index.search(query, 10);

// Higher ef = better accuracy, slower
const highAccuracy = index.search(query, 10, { ef: 300 });
```

[Continue with more troubleshooting scenarios...]

---

## Common Error Messages Reference

| Error | Cause | Solution |
|:------|:------|:---------|
| `DimensionMismatch` | Vector size != config dimensions | Use correct dimension |
| `IndexEmpty` | Searching empty index | Insert vectors first |
| `InvalidId` | ID not found in index | Check ID exists |
| `WasmNotInitialized` | init() not called | await init() first |
| `SerializationError` | Corrupt save data | Use fresh index |
```

### Step 4: Create INTEGRATION_GUIDE.md (2 hours)

**Structure:**
```markdown
# EdgeVec Integration Guide

## Table of Contents
1. [Overview](#overview)
2. [Transformers.js (@xenova/transformers)](#transformersjs)
3. [TensorFlow.js](#tensorflowjs)
4. [OpenAI Embeddings](#openai-embeddings)
5. [Custom Embedding Models](#custom-models)

---

## Overview

EdgeVec is a **storage layer** for vector embeddings. It does NOT generate embeddings itself.
You need to pair EdgeVec with an embedding model/library.

**Common Workflow:**
```
Text/Image → Embedding Model → Float32Array → EdgeVec → Search Results
```

---

## Transformers.js (@xenova/transformers)

### Installation

```bash
npm install @xenova/transformers edgevec
```

### Usage

```javascript
import { pipeline } from '@xenova/transformers';
import init, { EdgeVec, EdgeVecConfig } from 'edgevec';

// Initialize both libraries
await init();
const embedder = await pipeline('feature-extraction', 'Xenova/all-MiniLM-L6-v2');

// Create EdgeVec index (384 dimensions for MiniLM)
const config = new EdgeVecConfig(384);
const index = new EdgeVec(config);

// Generate embedding and store
async function addDocument(text) {
    const result = await embedder(text, { pooling: 'mean', normalize: true });
    const embedding = new Float32Array(result.data);
    return index.insert(embedding);
}

// Search
async function search(query, k = 10) {
    const result = await embedder(query, { pooling: 'mean', normalize: true });
    const embedding = new Float32Array(result.data);
    return index.search(embedding, k);
}

// Example
await addDocument("EdgeVec is a fast vector database");
await addDocument("Machine learning is awesome");
const results = await search("vector search engine");
```

### Dimension Reference

| Model | Dimensions | Use Case |
|:------|:-----------|:---------|
| `Xenova/all-MiniLM-L6-v2` | 384 | General purpose, fast |
| `Xenova/all-mpnet-base-v2` | 768 | Higher quality |
| `Xenova/gte-small` | 384 | General purpose |
| `Xenova/bge-small-en-v1.5` | 384 | High quality, English |

---

## TensorFlow.js

### Installation

```bash
npm install @tensorflow/tfjs @tensorflow-models/universal-sentence-encoder edgevec
```

### Usage

```javascript
import * as use from '@tensorflow-models/universal-sentence-encoder';
import init, { EdgeVec, EdgeVecConfig } from 'edgevec';

await init();
const model = await use.load();

// USE outputs 512 dimensions
const config = new EdgeVecConfig(512);
const index = new EdgeVec(config);

async function addDocument(text) {
    const embeddings = await model.embed([text]);
    const embedding = await embeddings.data();
    return index.insert(new Float32Array(embedding));
}
```

---

## OpenAI Embeddings

For server-side usage with OpenAI API:

```javascript
import OpenAI from 'openai';
import init, { EdgeVec, EdgeVecConfig } from 'edgevec';

await init();
const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });

// text-embedding-3-small outputs 1536 dimensions
const config = new EdgeVecConfig(1536);
const index = new EdgeVec(config);

async function addDocument(text) {
    const response = await openai.embeddings.create({
        model: 'text-embedding-3-small',
        input: text
    });
    return index.insert(new Float32Array(response.data[0].embedding));
}
```

**Note:** OpenAI API requires network calls. For fully offline usage, prefer transformers.js.

---

## Custom Models

EdgeVec accepts any Float32Array embedding:

```javascript
// Your custom embedding function
function myEmbed(text) {
    // Returns Float32Array of fixed dimensions
    return new Float32Array(128).fill(0.1); // placeholder
}

const config = new EdgeVecConfig(128); // Match your model's dimensions
const index = new EdgeVec(config);

const embedding = myEmbed("Hello world");
index.insert(embedding);
```

---

## Best Practices

1. **Always match dimensions:** Config dimension must equal embedding dimension
2. **Normalize if using cosine:** Most models output normalized vectors
3. **Batch when possible:** Use `batchInsert` for large datasets
4. **Persist both:** Save EdgeVec index AND your document metadata separately

---

## Troubleshooting

### Dimension Mismatch Error
**Solution:** Check your model's output dimension and update `EdgeVecConfig(dim)`

### Poor Search Quality
**Solutions:**
- Ensure you're using the same model for indexing and querying
- Check if model requires normalization
- Increase `ef` parameter for better recall
```

---

## Test Requirements

- [ ] All code examples in TUTORIAL.md run without errors
- [ ] All code examples in PERFORMANCE_TUNING.md are syntactically correct
- [ ] All error scenarios in TROUBLESHOOTING.md are reproducible
- [ ] Documentation renders correctly in GitHub markdown preview

---

## Review Gate

**Artifacts for Review:**
1. `docs/TUTORIAL.md`
2. `docs/PERFORMANCE_TUNING.md`
3. `docs/TROUBLESHOOTING.md`
4. `docs/INTEGRATION_GUIDE.md`

**Command:** `/review docs/TUTORIAL.md`

---

## Quality Checklist

Before submitting for review:

- [ ] All code examples tested in Node.js
- [ ] All code examples tested in browser
- [ ] Table of contents links work
- [ ] No broken internal links
- [ ] Consistent formatting throughout
- [ ] Clear, beginner-friendly language
- [ ] No jargon without explanation

---

## Exit Criteria

Day 3 is **COMPLETE** when:
- [ ] All four documentation files created (TUTORIAL, TUNING, TROUBLESHOOTING, INTEGRATION)
- [ ] All code examples verified working
- [ ] Documentation follows EdgeVec style guide
- [ ] `/review` approved for all docs

---

**Next:** Proceed to W19.4 (Test Hardening) after review approval
