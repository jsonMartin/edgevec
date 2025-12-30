# Twitter/X Thread - EdgeVec v0.7.0

**Platform:** Twitter/X
**Format:** Thread (7 tweets)

---

## Tweet 1 (Hook)

```
I built a vector database that runs entirely in your browser.

No server. No API calls. No data leaving your device.

EdgeVec v0.7.0 is out. Here's what's new 🧵
```

**Character count:** 175

---

## Tweet 2 (The Problem)

```
The problem: most vector DBs require a server.

If you're building privacy-sensitive apps (medical notes, legal docs, personal journals) or offline-first tools, that's a blocker.

EdgeVec runs 100% client-side via WebAssembly.
```

**Character count:** 248

---

## Tweet 3 (Binary Quantization)

```
32x memory reduction with Binary Quantization.

Store 1M vectors in ~125MB instead of 4GB.

95%+ recall retention. Perfect for browser memory constraints.
```

**Character count:** 167

---

## Tweet 4 (SIMD - Community Contribution)

```
8.75x faster Hamming distance via WASM SIMD.

This came from our first community contributor @jsonMartin.

One PR. Massive impact. Open source works.
```

**Character count:** 161

---

## Tweet 5 (Code Example)

```
The API:

import init, { EdgeVec, EdgeVecConfig } from 'edgevec';
await init();

const db = new EdgeVec(new EdgeVecConfig(768));
db.insert(new Float32Array(embedding));
const results = db.search(new Float32Array(query), 10);

That's it.
```

**Character count:** 237

---

## Tweet 6 (Use Cases)

```
Use cases:

→ Browser RAG with Transformers.js or Ollama
→ Offline semantic search
→ Privacy-preserving AI (data never leaves device)
→ Local codebase search

Works with OpenAI, Cohere, HuggingFace embeddings.
```

**Character count:** 232

---

## Tweet 7 (Links + CTA)

```
Try it:

🔗 Demo: https://matte1782.github.io/edgevec/demo/
📦 npm install edgevec
🦀 cargo add edgevec
⭐ https://github.com/matte1782/edgevec

220KB gzipped. MIT licensed.

What would make this useful for your workflow?
```

**Character count:** 243

---

## Hashtags (Optional - add to Tweet 1)

```
#rustlang #webassembly #vectordatabase
```

---

## Posting Strategy

1. **Post as thread** using Twitter's thread feature
2. **Best time:** 9 AM - 12 PM EST (weekdays) or 6-9 AM Pacific
3. **Engage with replies** within first hour – algorithm rewards quick responses
4. **Quote tweet** if Rust community accounts engage
5. **Don't over-hashtag** – 1-3 max, or none

---

## Image Suggestions

| Tweet | Image |
|:------|:------|
| 1 | Demo screenshot showing "WASM READY" status |
| 3 | Memory comparison visual (F32 vs BQ bar chart) |
| 4 | Benchmark showing 8.75x speedup |
| 7 | GitHub repo card or demo interface |

---

## Alternative Hook (Tweet 1)

If first hook doesn't resonate:

```
What if your vector database ran entirely in the browser?

No server costs. No network latency. No data leaving the device.

EdgeVec v0.7.0 does exactly that.

Here's what we shipped 🧵
```
