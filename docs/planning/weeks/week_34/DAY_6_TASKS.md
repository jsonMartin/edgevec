# Week 34 Day 6: Embedding Integration Guide

**Date:** 2026-01-25
**Focus:** Create embedding integration guide
**Hours:** 2h
**Status:** [ ] PENDING

---

## Objectives

Create comprehensive guide for integrating EdgeVec with popular embedding providers.

---

## Tasks

### W34.3.1 & W34.3.2: Embedding Guide (2h)

**Goal:** Complete embedding integration guide.

**Subtasks:**

- [ ] **6.1** Create document structure (15min)
  - Create `docs/guides/EMBEDDING_GUIDE.md`
  - Add introduction and overview
  - Set up provider sections

- [ ] **6.2** Ollama integration (30min)
  - Installation instructions
  - Model recommendations (nomic-embed-text, all-minilm)
  - Node.js code example
  - Dimension reference table

- [ ] **6.3** transformers.js integration (30min)
  - Installation instructions
  - Model recommendations (gte-small, all-MiniLM-L6-v2)
  - Browser + Node.js examples
  - WASM considerations

- [ ] **6.4** OpenAI integration (25min)
  - API setup instructions
  - Model recommendations (text-embedding-3-small/large)
  - Code example with error handling
  - Cost considerations note

- [ ] **6.5** Decision guide (20min)
  - Comparison table (cost, privacy, quality, speed)
  - Decision tree by use case
  - Recommendations

---

## Document Structure

```markdown
# EdgeVec Embedding Integration Guide

## Overview

EdgeVec stores and searches vector embeddings. This guide shows how to generate
embeddings using popular providers and use them with EdgeVec.

### What You'll Need

- EdgeVec installed (`npm install edgevec`)
- An embedding provider (see options below)
- Vector dimensions that match your model

---

## Provider Comparison

| Provider | Privacy | Cost | Quality | Latency | Dimensions |
|:---------|:--------|:-----|:--------|:--------|:-----------|
| Ollama | Local | Free | Good | Medium | 384-4096 |
| transformers.js | Local | Free | Good | Slow | 384-768 |
| OpenAI | Cloud | Paid | Excellent | Fast | 256-3072 |

---

## Ollama Integration

### Installation

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull an embedding model
ollama pull nomic-embed-text
```

### Recommended Models

| Model | Dimensions | Use Case |
|:------|:-----------|:---------|
| nomic-embed-text | 768 | General purpose, good quality |
| all-minilm | 384 | Smaller, faster |
| mxbai-embed-large | 1024 | Higher quality |

### Code Example

```typescript
import { EdgeVecIndex } from 'edgevec';

const index = new EdgeVecIndex({ dimensions: 768 });

async function getEmbedding(text: string): Promise<number[]> {
  const response = await fetch('http://localhost:11434/api/embeddings', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model: 'nomic-embed-text',
      prompt: text
    })
  });
  const data = await response.json();
  return data.embedding;
}

// Usage
const embedding = await getEmbedding('Your text here');
index.add(embedding, { text: 'Your text here' });
```

---

## transformers.js Integration

### Installation

```bash
npm install @xenova/transformers
```

### Recommended Models

| Model | Dimensions | Size | Use Case |
|:------|:-----------|:-----|:---------|
| Xenova/gte-small | 384 | 67MB | Fast, good quality |
| Xenova/all-MiniLM-L6-v2 | 384 | 90MB | Popular, well-tested |
| Xenova/bge-small-en-v1.5 | 384 | 130MB | High quality |

### Code Example (Browser + Node)

```typescript
import { pipeline } from '@xenova/transformers';
import { EdgeVecIndex } from 'edgevec';

const index = new EdgeVecIndex({ dimensions: 384 });
const extractor = await pipeline('feature-extraction', 'Xenova/gte-small');

async function getEmbedding(text: string): Promise<number[]> {
  const output = await extractor(text, { pooling: 'mean', normalize: true });
  return Array.from(output.data);
}

// Usage
const embedding = await getEmbedding('Your text here');
index.add(embedding, { text: 'Your text here' });
```

---

## OpenAI Integration

### Setup

```bash
npm install openai
```

### Recommended Models

| Model | Dimensions | Cost (per 1M tokens) |
|:------|:-----------|:---------------------|
| text-embedding-3-small | 1536 | $0.02 |
| text-embedding-3-large | 3072 | $0.13 |
| text-embedding-ada-002 | 1536 | $0.10 |

### Code Example

```typescript
import OpenAI from 'openai';
import { EdgeVecIndex } from 'edgevec';

const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
const index = new EdgeVecIndex({ dimensions: 1536 });

async function getEmbedding(text: string): Promise<number[]> {
  const response = await openai.embeddings.create({
    model: 'text-embedding-3-small',
    input: text,
  });
  return response.data[0].embedding;
}

// Usage
const embedding = await getEmbedding('Your text here');
index.add(embedding, { text: 'Your text here' });
```

---

## Choosing the Right Provider

### Decision Tree

1. **Need privacy/offline?** → Ollama or transformers.js
2. **Running in browser?** → transformers.js
3. **Need best quality?** → OpenAI
4. **Budget-constrained?** → Ollama (free) or transformers.js (free)
5. **Need low latency?** → Ollama (local) or OpenAI (fast API)

### Recommendations by Use Case

| Use Case | Recommended | Why |
|:---------|:------------|:----|
| Production RAG | OpenAI | Quality + reliability |
| Privacy-focused | Ollama | Local, no data leaves |
| Browser app | transformers.js | No backend needed |
| Prototyping | Ollama | Free, easy setup |
| Enterprise | OpenAI or Ollama | Depending on policy |
```

---

## Verification

- [ ] Document created at `docs/guides/EMBEDDING_GUIDE.md`
- [ ] Ollama section complete with working example
- [ ] transformers.js section complete
- [ ] OpenAI section complete
- [ ] Decision guide helps users choose
- [ ] All code examples tested

---

## Next

Day 7: Testing & hostile review
