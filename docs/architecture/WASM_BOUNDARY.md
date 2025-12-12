# EdgeVec WASM Boundary Specification v1.1

**Date:** 2025-12-05
**Author:** META_ARCHITECT
**Status:** [REVISED]

---

## 0. FFI Safety Principles

> **"What can't cross the WASM boundary doesn't exist."**
> — EdgeVec Design Principle

This document specifies every function, type, and data transfer that crosses the JS ↔ Rust boundary. All designs prioritize **safety over convenience**.

### 0.1 WASM Constraints

| Constraint | Implication | Source |
|:-----------|:------------|:-------|
| `[FACT]` No native threads | Use `wasm-bindgen-rayon` or single-threaded | [MDN](https://developer.mozilla.org/en-US/docs/WebAssembly/JavaScript_interface) |
| `[FACT]` No `std::fs` | Use IndexedDB for browser persistence | WebAssembly spec |
| `[FACT]` 32-bit pointers | Vec is 12 bytes, not 24 | wasm32 target |
| `[FACT]` Linear memory only | No virtual memory, no mmap | WebAssembly spec |
| `[FACT]` Little-endian | All multi-byte values are LE | [WebAssembly spec](https://webassembly.github.io/spec/core/syntax/values.html) |

### 0.2 FFI Safety Rules (Revised v1.1)

1. **No panics across boundary** — All functions return `Result<T, JsValue>`
2. **No `dyn Trait`** — Only concrete types cross the boundary
3. **No raw pointers** — Use `Box`, `Vec`, or `TypedArray`
4. **Explicit String Handling** — Struct fields may use `String` (data), but function signatures should prefer `&str` or `js_sys::JsString`.
5. **All types are `#[wasm_bindgen]`** — Explicit boundary crossing

---

## 1. Exported Types

### 1.1 EdgeVec (Main Handle)

```rust
/// The main EdgeVec database handle.
/// 
/// # WASM Safety
/// - Holds all state internally (no external pointers)
/// - All methods are fallible (return Result)
/// - Implements Drop for cleanup
#[wasm_bindgen]
pub struct EdgeVec {
    inner: EdgeVecInner,  // Not exposed to JS
}

#[wasm_bindgen]
impl EdgeVec {
    /// Create a new EdgeVec database.
    #[wasm_bindgen(constructor)]
    pub fn new(config: &EdgeVecConfig) -> Result<EdgeVec, JsValue> {
        // Implementation
    }
}
```

### 1.2 EdgeVecConfig (Revised v1.1)

```rust
/// Configuration for EdgeVec database.
/// 
/// # Fields (all optional with defaults)
/// - dimensions: Vector dimensionality (required)
/// - m: HNSW M parameter (default: 16)
/// - ef_construction: Build-time ef (default: 200)
/// - ef_search: Query-time ef (default: 50)
#[wasm_bindgen]
pub struct EdgeVecConfig {
    pub dimensions: u32,
    m: Option<u32>,
    m0: Option<u32>,
    ef_construction: Option<u32>,
    ef_search: Option<u32>,
    metric: Option<String>,  // Safe in struct (data carrier)
}

#[wasm_bindgen]
impl EdgeVecConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(dimensions: u32) -> EdgeVecConfig {
        EdgeVecConfig {
            dimensions,
            m: None,
            m0: None,
            ef_construction: None,
            ef_search: None,
            metric: None,
        }
    }
    
    #[wasm_bindgen(setter)]
    pub fn set_metric(&mut self, metric: String) { 
        self.metric = Some(metric); 
    }
}
```

### 1.3 SearchResult

```rust
/// A single search result.
#[wasm_bindgen]
pub struct SearchResult {
    #[wasm_bindgen(readonly)]
    pub id: u64,
    
    #[wasm_bindgen(readonly)]
    pub distance: f32,
}
```

### 1.4 SearchResults (Collection)

```rust
/// Collection of search results.
#[wasm_bindgen]
pub struct SearchResults {
    results: Vec<SearchResult>,
}

#[wasm_bindgen]
impl SearchResults {
    #[wasm_bindgen(getter)]
    pub fn len(&self) -> usize {
        self.results.len()
    }
    
    pub fn to_array(&self) -> js_sys::Array {
        self.results.iter()
            .map(|r| JsValue::from(r.clone()))
            .collect()
    }
}
```

---

## 2. Exported Functions

### 2.1 Core Operations

| Function | Signature | Direction | Notes |
|:---------|:----------|:----------|:------|
| `new` | `(config: EdgeVecConfig) -> Result<EdgeVec>` | JS→Rust | Constructor |
| `insert` | `(&mut self, vector: Float32Array) -> Result<BigInt>` | JS→Rust | Returns VectorId |
| `insert_batch` | `(&mut self, vectors: Float32Array, count: u32) -> Result<BigIntArray>` | JS→Rust | Bulk insert |
| `search` | `(&self, query: Float32Array, k: u32) -> Result<SearchResults>` | JS→Rust | k-NN search |
| `delete` | `(&mut self, id: BigInt) -> Result<bool>` | JS→Rust | Soft delete |
| `len` | `(&self) -> usize` | JS→Rust | Vector count |
| `dimensions` | `(&self) -> u32` | JS→Rust | Configured dims |

### 2.2 Persistence Operations

| Function | Signature | Direction | Notes |
|:---------|:----------|:----------|:------|
| `save` | `(&self, name: &str) -> Promise<()>` | JS→Rust | Save to IndexedDB |
| `load` | `(name: &str) -> Promise<EdgeVec>` | JS→Rust | Load from IndexedDB |
| `export_bytes` | `(&self) -> Result<Uint8Array>` | Rust→JS | Serialize to bytes |
| `import_bytes` | `(bytes: Uint8Array) -> Result<EdgeVec>` | JS→Rust | Deserialize |

---

## 3. Data Transfer Patterns

### 3.1 Vector Input (JS → Rust)

```rust
#[wasm_bindgen]
impl EdgeVec {
    pub fn insert(&mut self, vector: &Float32Array) -> Result<u64, JsValue> {
        // Validate dimensions
        let len = vector.length() as usize;
        if len != self.inner.dimensions() {
            return Err(JsValue::from_str("Dimension mismatch"));
        }
        
        // Copy from JS memory to Rust Vec
        let mut vec = vec![0.0f32; len];
        vector.copy_to(&mut vec);
        
        // Validate no NaN/Infinity
        for (i, &v) in vec.iter().enumerate() {
            if !v.is_finite() {
                return Err(JsValue::from_str("Non-finite value"));
            }
        }
        
        // Insert into index
        let id = self.inner.insert(&vec)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(id.0)
    }
}
```

---

## 4. Error Handling

Errors crossing the boundary are converted to JavaScript Objects with standardized codes.

| Code | Meaning | Source Variant |
|:-----|:--------|:---------------|
| `ERR_IO` | I/O or filesystem failure | `EdgeVecError::Io` |
| `ERR_PERSISTENCE` | General persistence failure | `EdgeVecError::Persistence` |
| `ERR_CORRUPTION` | Data integrity check failed | `PersistenceError::ChecksumMismatch`, `InvalidMagic` |
| `ERR_DIMENSION` | Vector dimension mismatch | `GraphError::DimensionMismatch`, `ConfigMismatch` |
| `ERR_VALIDATION` | Invalid input argument | `EdgeVecError::Validation` |
| `ERR_GRAPH` | Graph algorithm failure | `EdgeVecError::Graph` |

### 4.1 JS Error Structure

All rejected Promises and thrown exceptions follow this shape:

```typescript
interface EdgeVecError {
    code: string;
    message: string;
}
```

Example usage in JS:

```javascript
try {
    db.insert(vector);
} catch (e) {
    if (e.code === 'ERR_DIMENSION') {
        console.error("Wrong size:", e.message);
    }
}
```

---

## 12. Verification Hooks

Each boundary function has associated tests in `TEST_STRATEGY.md`:

| Function | Unit Test | Integration Test | E2E Test |
|:---------|:----------|:-----------------|:---------|
| `new` | UNIT-WASM-001 | INT-WASM-001 | E2E-001 |
| `insert` | UNIT-WASM-002 | INT-WASM-002 | E2E-002 |
| `search` | UNIT-WASM-003 | INT-WASM-003 | E2E-003 |
| `delete` | UNIT-WASM-004 | INT-WASM-004 | - |
| `save` | UNIT-WASM-005 | INT-WASM-005 | E2E-004 |
| `load` | UNIT-WASM-006 | INT-WASM-006 | E2E-005 |

---

*Document Version: 1.1*
*Author: META_ARCHITECT*
*Status: APPROVED*
