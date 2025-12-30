# Hacker News Submission - EdgeVec v0.7.0

**Platform:** Hacker News (news.ycombinator.com)
**Type:** Show HN

---

## Title (80 char limit)

```
Show HN: EdgeVec – Vector search in the browser, no server (Rust/WASM)
```

**Character count:** 69

---

## URL

```
https://github.com/matte1782/edgevec
```

---

## Text (Optional - for Show HN)

```
I built a vector database that runs entirely in WebAssembly. No server, no API calls, no data leaves the browser.

Why? Most vector DBs require a server. If you're building privacy-sensitive apps (medical notes, legal docs, personal journals) or offline-first tools, that's a problem. EdgeVec solves it.

What it does:
- Semantic search on embeddings from OpenAI, Cohere, HuggingFace, etc.
- Binary quantization: 32x memory reduction (1M vectors in ~125MB)
- SIMD acceleration: 8.75x faster Hamming distance
- IndexedDB persistence: index survives page reloads
- Filter expressions: SQL-like metadata queries

Technical details:
- 220KB gzipped, 494KB raw
- Chrome 91+, Firefox 89+, Safari 16.4+
- Brute force search (HNSW coming in v0.8)
- MIT licensed

Live demo: https://matte1782.github.io/edgevec/demo/

This release includes our first community contribution – @jsonMartin implemented the SIMD Hamming kernel that delivered 8.75x speedup.

Happy to discuss architecture decisions, WASM/SIMD tradeoffs, or binary quantization details.
```

**Character count:** 1,051

---

## Posting Guidelines

1. **Submit as "Show HN"** – This is a project showcase
2. **Link to GitHub**, not the demo (HN prefers source)
3. **Don't ask for upvotes** – against HN rules
4. **Be ready to engage** – HN comments can be technical and critical
5. **Best times to post:** 6-9 AM Pacific (weekdays)
6. **Respond to comments quickly** – engagement in first 2 hours matters

---

## Expected Questions to Prepare For

| Question | Answer |
|:---------|:-------|
| "Why not Faiss/Annoy/Qdrant?" | Those require a server or don't compile to WASM. EdgeVec is browser-native. |
| "What's the max vector count?" | ~100k comfortable with brute force, 1M+ with binary quantization. HNSW in v0.8 will improve this. |
| "Why brute force?" | Fast enough for <100k vectors. HNSW adds complexity and we wanted to ship something usable first. |
| "How does BQ quality compare?" | 95%+ recall on standard benchmarks. Good enough for most RAG use cases. |
| "Why Rust for WASM?" | Memory safety, no runtime, excellent WASM toolchain (wasm-bindgen), predictable performance. |
| "What about Web Workers?" | Currently single-threaded. Multi-threading via wasm-bindgen-rayon is planned. |
| "Security implications?" | All computation client-side. No data exfiltration possible. User controls their data entirely. |

---

## Backup Title Options

1. `Show HN: Vector search in the browser, no server (Rust/WASM)`
2. `Show HN: I built a vector DB that runs entirely in WebAssembly`
3. `Show HN: EdgeVec – Offline vector search for privacy-sensitive apps`
4. `Show HN: Browser-native semantic search with 32x memory reduction`
