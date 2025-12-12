# EdgeVec (NPM)

**High-performance vector search for the Browser.**

This package provides the WebAssembly bindings for EdgeVec, allowing you to run a military-grade vector database directly in the browser with persistence via IndexedDB.

## 📦 Installation

```bash
npm install edgevec
```

*Note: You may need to configure your bundler (Webpack/Vite/Rollup) to handle WebAssembly.*

## 🚀 Quick Start (Browser)

EdgeVec uses WebAssembly, so it must be initialized asynchronously.

```javascript
import init, { EdgeVec, EdgeVecConfig } from 'edgevec';

async function main() {
    // 1. Initialize WASM module
    await init();

    // 2. Configure Index
    const config = new EdgeVecConfig(128); // 128 dimensions
    config.metric = "cosine";              // "l2", "cosine", or "dot"
    
    // 3. Create Database
    // Note: IndexedDB backend is automatically initialized
    let db = new EdgeVec(config);

    // 4. Insert Vectors
    const vector = new Float32Array(128).fill(0.1);
    const id = db.insert(vector);
    console.log(`Inserted vector with ID: ${id}`);

    // 5. Search
    const query = new Float32Array(128).fill(0.1);
    const results = db.search(query, 10); // top-k = 10
    
    console.log("Results:", results);
    // Output: [{ id: 123, score: 0.99 }, ...]

    // 6. Persistence (Save/Load)
    // Saves to IndexedDB under the file name "my-vector-db"
    await db.save("my-vector-db");
    console.log("Database saved to IndexedDB");

    // Later... load it back
    // Note: EdgeVec.load is a static method
    const loadedDb = await EdgeVec.load("my-vector-db");
}

main().catch(console.error);
```

## 🛠 Configuration

### `EdgeVecConfig`

| Property | Type | Default | Description |
|:---------|:-----|:--------|:------------|
| `dimensions` | `number` | **Required** | Dimensionality of vectors. |
| `metric` | `string` | `"l2"` | Distance metric: `"l2"`, `"cosine"`, `"dot"`. |
| `m` | `number` | `None` (Auto) | Max connections per layer (HNSW). |
| `ef_construction` | `number` | `None` (Auto) | Search depth during build. |
| `ef_search` | `number` | `None` (Auto) | Search depth during query. |

## 💾 Persistence

EdgeVec automatically bundles an **IndexedDB** backend. 

- **`db.save(name)`**: Serializes the index and stores it in the browser's IndexedDB.
- **`EdgeVec.load(name)`**: Reads from IndexedDB and deserializes the index.

> **Note:** The persistence layer is zero-copy where possible but involves serialization overhead. For large indices (>1M vectors), use the streaming API (advanced).

## ⚠️ Browser Compatibility

Requires a browser with **WebAssembly** and **BigInt** support.

- Chrome 70+
- Firefox 68+
- Safari 14+
- Edge 79+

## 📄 License

MIT

