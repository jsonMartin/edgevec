# EdgeVec Data Layout Specification v1.5

**Date:** 2025-12-09
**Author:** META_ARCHITECT
**Status:** [APPROVED]

---

## 0. Design Principles

1. **Cache-line alignment (64 bytes)** for hot data structures
2. **Contiguous allocation** for SIMD-friendly vector operations
3. **Compressed Neighbor Storage** for <100 bytes/vector overhead
4. **No padding surprises** — all sizes calculated with alignment
5. **WASM-compatible** — no platform-specific assumptions
6. **Quantization-First** — primary storage is u8 (SQ8) for density

---

## 1. Core Identifiers

### 1.1 VectorId

```rust
/// Unique identifier for a vector in the database.
/// 
/// # Size
/// 8 bytes, aligned to 8
/// 
/// # Invariants
/// - IDs are never reused (monotonically increasing)
/// - ID 0 is reserved (invalid sentinel)
/// - Maximum: 2^64 - 1 vectors
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct VectorId(pub(crate) u64);

impl VectorId {
    /// Sentinel value indicating "no vector"
    pub const INVALID: Self = VectorId(0);
    
    /// First valid ID
    pub const FIRST: Self = VectorId(1);
}
// Size: 8 bytes | Alignment: 8 bytes
```

### 1.2 NodeId

```rust
/// Internal node identifier within HNSW graph.
/// 
/// # Size
/// 4 bytes, aligned to 4
/// 
/// # Rationale
/// u32 chosen over u64 to save memory in adjacency lists.
/// Maximum 4 billion nodes is sufficient for edge deployments.
/// 
/// # Invariants
/// - NodeId corresponds 1:1 with VectorId (lower 32 bits)
/// - NodeId 0xFFFFFFFF is reserved (invalid sentinel)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NodeId(pub(crate) u32);

impl NodeId {
    pub const INVALID: Self = NodeId(u32::MAX);
}
// Size: 4 bytes | Alignment: 4 bytes
```

---

## 2. Vector Storage

### 2.1 QuantizerConfig (New for v1.5)

```rust
/// Configuration for Scalar Quantization (SQ8).
/// 
/// # Algorithm: Min-Max Normalization
/// Formula: `u8 = (f32 - min) / (max - min) * 255`
/// 
/// # Storage
/// Global min/max per index for v1. Simplicity over per-segment optimization.
/// 
/// # Zero Mapping
/// If min=0.0 and max=1.0, then 0.0 maps strictly to 0.
#[derive(Clone, Debug, Copy)]
#[repr(C)]
pub struct QuantizerConfig {
    /// Global minimum value observed/configured
    pub min: f32,             // offset 0
    /// Global maximum value observed/configured
    pub max: f32,             // offset 4
}
// Total: 8 bytes | Alignment: 4 bytes
```

### 2.2 QuantizedVector (Transient)

```rust
/// Logical wrapper for a single quantized vector.
/// 
/// Not stored directly in `VectorStorage` (which is flattened), but used
/// for API boundaries and transient operations.
#[derive(Clone, Debug)]
pub struct QuantizedVector {
    pub data: Vec<u8>,
}
```

### 2.3 VectorStorage

```rust
/// Arena-based storage for vector data (Quantized u8).
/// 
/// # Memory Layout
/// 
/// ```text
/// ┌─────────────────────────────────────────────────────────────┐
/// │  data: Vec<u8>                                               │
/// │  ┌─────────┬─────────┬─────────┬───┬─────────┐              │
/// │  │ Vec[0]  │ Vec[1]  │ Vec[2]  │...│ Vec[N]  │              │
/// │  │ D bytes │ D bytes │ D bytes │   │ D bytes │              │
/// │  └─────────┴─────────┴─────────┴───┴─────────┘              │
/// │                                                              │
/// │  Offset of vector i = i * dimensions                        │
/// └─────────────────────────────────────────────────────────────┘
/// ```
/// 
/// # Size Calculation
/// - Header: 24 bytes (Vec pointer + len + cap)
/// - Data: dimensions * num_vectors * 1 byte
/// - Metadata overhead: ~1 bit/vector (tombstones)
pub struct VectorStorage {
    /// Contiguous quantized vector data (cache-friendly)
    /// Layout: [v0_dim0, v0_dim1, ..., v0_dimD, v1_dim0, ...]
    data: Vec<u8>,
    
    /// Soft-delete bitmap: bit i = 1 means vector i is deleted
    /// Size: ceil(num_vectors / 8) bytes
    tombstones: BitVec,
    
    /// Vector dimensionality (immutable after construction)
    dimensions: u32,
    
    /// Number of vectors (including tombstoned)
    count: u32,
    
    /// Next allocation index
    next_slot: u32,

    /// Quantization Parameters (Global)
    /// Used for reconstruction and query quantization.
    quantizer_config: QuantizerConfig,
}

// Size breakdown (on 64-bit):
// - data Vec:       24 bytes
// - tombstones:     24 bytes + ceil(n/8)
// - dimensions:      4 bytes
// - count:           4 bytes
// - next_slot:       4 bytes
// - quantizer:       8 bytes
// Total header:     ~100 bytes + data + tombstone bits
```

### 2.4 Memory Budget Verification

```rust
/// Memory calculation for 1M vectors at 768 dimensions (u8 quantized)
/// 
/// Vector data:     1,000,000 * 768 * 1 = 768,000,000 bytes (768.0 MB)
/// Tombstones:      1,000,000 / 8       =     125,000 bytes (125.0 KB)
/// Struct overhead:                     =         100 bytes
/// ───────────────────────────────────────────────────────────
/// Total storage:                       768,125,100 bytes
/// 
/// Per-vector overhead (excluding vector data):
/// (125,000 + 100) / 1,000,000 = 0.125 bytes/vector ✅ < 100 bytes
```

---

## 3. HNSW Index Structures

### 3.1 HnswConfig

```rust
/// HNSW algorithm parameters.
/// 
/// # Size
/// 32 bytes, aligned to 8
/// 
/// # Parameter Guidelines (from paper)
/// - M: 12-48 for high recall, 4-8 for speed
/// - ef_construction: Higher = better quality, slower build
/// - ef_search: Higher = better recall, slower search
#[derive(Clone, Debug)]
#[repr(C)]
pub struct HnswConfig {
    /// Max connections per node in layers > 0
    /// Typical: 16
    pub m: u32,                    // offset 0, size 4
    
    /// Max connections per node in layer 0 (typically 2*M)
    /// Typical: 32
    pub m0: u32,                   // offset 4, size 4
    
    /// Construction-time candidate list size
    /// Typical: 200
    pub ef_construction: u32,      // offset 8, size 4
    
    /// Search-time candidate list size
    /// Typical: 50
    pub ef_search: u32,            // offset 12, size 4
    
    /// Vector dimensionality
    pub dimensions: u32,           // offset 16, size 4
    
    /// Distance metric (0 = L2, 1 = Cosine, 2 = Dot)
    pub metric: u32,               // offset 20, size 4
    
    /// Reserved for future use
    _reserved: [u32; 2],           // offset 24, size 8
}
// Total: 32 bytes | Alignment: 4 bytes (naturally aligned)
```

### 3.2 HnswNode

```rust
/// A node in the HNSW graph with its adjacency information.
/// 
/// # Memory Layout
/// 
/// We use a FLATTENED representation for cache efficiency:
/// - All nodes at the same layer are stored contiguously
/// - Adjacency lists are stored separately in a pool
/// 
/// # Size Calculation
/// - Fixed part: 8 bytes (id + level)
/// 
/// # Layout
/// ```text
/// ┌────────────────────────────────────────────────────────────────┐
/// │ vector_id (8) | neighbor_offset (4) | neighbor_len (2) | pad(2) │
/// └────────────────────────────────────────────────────────────────┘
/// ```
#[derive(Clone, Debug)]
pub struct HnswNode {
    /// The vector ID this node represents
    pub vector_id: VectorId,       // 8 bytes
    
    /// Offset into COMPRESSED neighbor pool
    pub neighbor_offset: u32,      // 4 bytes
    
    /// Length of neighbor data in bytes
    pub neighbor_len: u16,         // 2 bytes
    
    /// The maximum layer this node appears in
    pub max_layer: u8,             // 1 byte
    
    /// Padding for alignment
    pub _pad: u8,                  // 1 byte
}
// Total: 16 bytes | Alignment: 8 bytes
```

### 3.3 Compressed Neighbor Pool

```rust
/// Byte-pool of neighbor lists using Variable-Byte Encoding.
/// 
/// # Compression Strategy (Delta VByte)
/// 1. Neighbors are sorted: `[n1, n2, n3, ...]` where n1 < n2 < n3
/// 2. Compute deltas: `[n1, n2-n1, n3-n2, ...]`
/// 3. Encode using VByte (Varint):
///    - 0-127: 1 byte
///    - 128-16383: 2 bytes
///    - 16384-2M: 3 bytes
/// 
/// # Memory Calculation (M=16, M0=32)
/// Average Delta Estimation for 100k vectors:
/// - Layer 0 (M0=32): Global connections. Avg delta ≈ 100,000 / 32 ≈ 3,125.
///   - 3,125 fits in 2 bytes (14 bits).
///   - 32 neighbors * 2 bytes = 64 bytes.
/// - Upper Layers (M=16): Sparse connections. Avg delta is larger (often > 16k).
///   - Deltas likely 2-3 bytes.
///   - 16 neighbors * 2.5 bytes = 40 bytes.
/// 
/// # Total Per-Node Size
/// - Layer 0 node (96% of nodes): 64 bytes
/// - Layer 1+ node (4%): 64 + 40 = 104 bytes
/// - Weighted Avg: 0.96*64 + 0.04*104 ≈ 66 bytes
pub struct NeighborPool {
    /// Contiguous compressed data
    data: Vec<u8>,
    
    /// Capacity for growth
    capacity: usize,
}
// Per-vector overhead: ~66 bytes (Compressed) vs 224 bytes (Raw)
```

### 3.4 HnswIndex Complete

```rust
/// Complete HNSW index structure.
/// 
/// # Memory Budget (100k vectors, 768d)
/// 
/// | Component        | Size (bytes)    | Per-vector |
/// |------------------|-----------------|------------|
/// | HnswConfig       | 32              | 0.00       |
/// | nodes Vec        | 100k * 16       | 16.0       |
/// | neighbor_pool    | ~6.6 MB         | 66.0       |
/// | entry_point      | 8               | 0.00       |
/// |------------------|-----------------|------------|
/// | Total index      | ~8.2 MB         | ~82.0      |
/// 
/// **Total Overhead:** 82 bytes < 100 bytes Target ✅
pub struct HnswIndex {
    /// Algorithm configuration
    config: HnswConfig,
    
    /// Node metadata (fixed-size per node)
    nodes: Vec<HnswNode>,
    
    /// Compressed neighbor lists
    neighbors: NeighborPool,
    
    /// Entry point (highest layer node)
    entry_point: Option<NodeId>,

    /// Quantization params (copy from VectorStorage for quick access)
    /// Option<(min, max)>
    quantization_params: Option<(f32, f32)>, 
    
    // --- RUNTIME ONLY FIELDS (See 3.4.1) ---

    /// Maximum layer in the graph
    max_layer: u8,

    /// Level probability multiplier
    level_mult: f32,

    /// Random number generator state for determinism
    rng: ChaCha8Rng,
}
```

#### 3.4.1 Runtime-Only State

The following fields are **transient** and are **NOT persisted** to disk. They are reconstructed or re-initialized upon loading the index.

| Field | Type | Size (Approx) | Purpose |
|:------|:-----|:--------------|:--------|
| `max_layer` | `u8` | 1 byte | Tracks current graph height. Recomputed on load or stored in FileHeader (cached). |
| `level_mult` | `f32` | 4 bytes | Constant derived from `HnswConfig.m`. Recalculated in `new()`. |
| `rng` | `ChaCha8Rng` | ~136 bytes | Deterministic RNG for level generation. Re-seeded from `FileHeader.rng_seed` on load. |

**Persistence Policy:**
- `max_layer`: Recovered from `FileHeader` or by scanning nodes.
- `level_mult`: Recomputed from `config`.
- `rng`: Re-initialized using `FileHeader.rng_seed` (see Section 4.1).

---

## 4. Persistence Structures

### 4.1 File Header (Corrected Alignment)

```rust
/// File header for .evec index files.
/// 
/// # Size
/// 64 bytes (fixed, cache-line aligned)
/// 
/// # Alignment Fix (v1.1)
/// Reordered fields to ensure 8-byte alignment for u64s.
/// 
/// # Layout
/// 00-03: Magic
/// 04-05: Version
/// 06-07: Flags
/// -- 8-byte boundary --
/// 08-15: Vector Count (u64)
/// 16-23: Index Offset (u64)
/// 24-31: Metadata Offset (u64)
/// 32-39: Rng Seed (u64)  <-- NEW for Determinism
/// 40-43: Dimensions (u32)
/// 44-47: Header CRC (u32)
/// 48-51: HNSW M (u32)
/// 52-55: HNSW M0 (u32)
/// 56-63: Reserved (u64)
#[derive(Clone, Debug)]
#[repr(C)]
pub struct FileHeader {
    /// Magic number: "EVEC" = [0x45, 0x56, 0x45, 0x43]
    pub magic: [u8; 4],            // 0
    
    /// Format version (major.minor as u8.u8)
    pub version_major: u8,         // 4
    /// Format version minor
    pub version_minor: u8,         // 5

    pub flags: u16,                // 6
    
    /// Number of vectors in file
    pub vector_count: u64,         // 8
    
    /// Byte offset to index section
    pub index_offset: u64,         // 16
    
    /// Byte offset to metadata section (0 if none)
    pub metadata_offset: u64,      // 24
    
    /// RNG Seed for deterministic replay
    pub rng_seed: u64,             // 32
    
    /// Vector dimensionality
    pub dimensions: u32,           // 40
    
    /// CRC32 of header bytes
    pub header_crc: u32,           // 44
    
    /// HNSW M parameter
    pub hnsw_m: u32,               // 48
    
    /// HNSW M0 parameter
    pub hnsw_m0: u32,              // 52
    
    /// Reserved for future use
    pub reserved: u64,             // 56
}
// Total: 64 bytes | Alignment: 8 bytes
static_assert!(size_of::<FileHeader>() == 64);
```

### 4.2 WAL Entry

```rust
/// Write-ahead log entry format.
/// 
/// # Size
/// Variable: 16 + payload_len bytes
/// 
/// # Encoding
/// All multi-byte integers are little-endian.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct WalEntry {
    /// Sequence number (monotonically increasing)
    pub sequence: u64,             // offset 0
    
    /// Entry type (0=insert, 1=delete, 2=checkpoint)
    pub entry_type: u8,            // offset 8
    
    /// Padding
    _pad: [u8; 3],                 // offset 9
    
    /// Payload length in bytes
    pub payload_len: u32,          // offset 12
    
    // Followed by:
    // - payload: [u8; payload_len]
    // - crc32: u32 (of entire entry including header)
}
// Header: 16 bytes | Total: 16 + payload + 4 = 20 + payload
```

### 4.3 WAL Insert Payload

```rust
/// Payload format for insert operations.
/// 
/// # Size
/// 12 + payload_data_len
/// 
/// # Payload Type
/// - If `flags & 0x1`: Payload is `[u8; dimensions]` (Quantized)
/// - Else: Payload is `[f32; dimensions]` (Raw)
#[repr(C)]
pub struct InsertPayload {
    /// Vector ID being inserted
    pub vector_id: u64,            // offset 0
    
    /// Dimensionality (redundant for validation)
    pub dimensions: u32,           // offset 8
    
    // Followed by vector data
}
// Header: 12 bytes | Total: 12 + D bytes (quantized) or 12 + D*4 (raw)
// For 768d u8: 12 + 768 = 780 bytes per insert
```

### 4.4 Snapshot Section Layout

```rust
/// Snapshot file layout for vectors section.
/// 
/// # Format
/// ```text
/// ┌──────────────────────────────────────────────────────────────┐
/// │ Section Header (24 bytes)                                     │
/// │ ├── section_type: u32 = 0x01 (vectors)                       │
/// │ ├── _pad1: u32 (alignment)                                   │
/// │ ├── section_len: u64                                         │
/// │ ├── reserved: u32                                            │
/// │ └── _pad2: u32 (alignment)                                   │
/// ├──────────────────────────────────────────────────────────────┤
/// │ Vector Data (section_len bytes)                               │
/// │ └── [u8; dimensions * vector_count] contiguous               │
/// └──────────────────────────────────────────────────────────────┘
#[repr(C)]
pub struct SectionHeader {
    pub section_type: u32,         // offset 0
    pub _pad1: u32,                // offset 4
    pub section_len: u64,          // offset 8
    pub reserved: u32,             // offset 16
    pub _pad2: u32,                // offset 20
}
// Size: 24 bytes | Alignment: 8 bytes
```

---

## 5. Distance Calculations

### 5.1 L2 Distance (Euclidean Squared)

```rust
/// Compute squared L2 distance between two vectors.
/// 
/// # Precision
/// Uses f32 accumulator for u8 inputs to prevent overflow.
#[inline]
pub fn l2_squared_u8(a: &[u8], b: &[u8]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    
    // SIMD implementation (conceptual)
    let mut sum: u32 = 0;
    for (x, y) in a.zip(b) {
        let diff = abs_diff(*x, *y) as u32;
        sum += diff * diff;
    }
    sum as f32
}
```

### 5.2 Cosine Similarity (Dot Product)

```rust
/// Compute cosine similarity (via dot product).
/// 
/// # Usage
/// - For SQ8, we assume vectors are normalized before quantization if using Cosine.
#[inline]
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    
    let mut sum: u32 = 0;
    for (x, y) in a.zip(b) {
        sum += (*x as u32) * (*y as u32);
    }
    sum as f32
}
```

---

## 6. Size Calculations Summary

### 6.1 Per-Vector Memory (768 dimensions, M=16, u8 Quantized)

| Component | Bytes | Notes |
|:----------|------:|:------|
| Vector data | 768 | 768 × 1 byte |
| HnswNode | 16 | Fixed metadata (reduced from 24) |
| Neighbors (avg) | 66 | Compressed (VByte Delta) |
| Tombstone bit | 0.125 | 1/8 byte |
| **Total** | **850.1** | |
| **Overhead** | **82** | Excluding vector data |

### 6.2 Total Memory (1M vectors, 768d)

| Component | MB | Notes |
|:----------|---:|:------|
| Vector data | 768.0 | 1M × 768 bytes |
| HNSW nodes | 16.0 | 1M × 16 |
| Neighbor pool | 66.0 | Compressed |
| Tombstones | 0.1 | 1M / 8 |
| **Total** | **850.1** | |

### 6.3 Comparison with Target

```
Target: <100 bytes/vector overhead
Actual: 82 bytes/vector overhead

STATUS: ✅ PASSES TARGET

JUSTIFICATION:
- Pivot to u8 quantization saves 3072 - 768 = 2304 bytes per vector (4x reduction in data).
- Overhead remains constant (HNSW + tombstones).
- Fits 1M vectors in < 1GB.
```

---

## 7. Alignment Verification

```rust
// Compile-time checks (using static_assertions crate)
const_assert!(align_of::<VectorId>() == 8);
const_assert!(align_of::<NodeId>() == 4);
const_assert!(align_of::<HnswConfig>() == 4);
const_assert!(size_of::<HnswConfig>() == 32);
const_assert!(size_of::<FileHeader>() == 64);
const_assert!(size_of::<WalEntry>() == 16);
const_assert!(size_of::<SectionHeader>() == 24);
const_assert!(size_of::<QuantizerConfig>() == 8);
```

---

## 8. WASM Considerations

### 8.1 Memory Model

```
[FACT] WASM uses 32-bit pointers (wasm32 target).
Source: https://webassembly.github.io/spec/core/syntax/types.html

Implications:
- Max addressable memory: 4GB
- Vec header: 12 bytes (not 24 as on 64-bit)
- All u64 IDs fit, but pointers are u32

For 1M vectors at 768d (u8):
- ~850 MB required
- Well within 4GB limit ✅
```

### 8.2 Endianness

```
[FACT] WASM is little-endian.
Source: https://webassembly.github.io/spec/core/syntax/values.html

Implications:
- All #[repr(C)] structs are LE
- No byte-swapping needed
- File format uses LE (matches WASM)
```

---

## 9. Verification Hooks

Each struct has associated property tests defined in `TEST_STRATEGY.md`:

| Struct | Property Test ID |
|:-------|:-----------------|
| VectorId | PROP-ID-001 |
| NodeId | PROP-ID-002 |
| HnswConfig | PROP-CFG-001 |
| FileHeader | PROP-PERSIST-001 |
| WalEntry | PROP-WAL-001 |
| VectorStorage | PROP-STORE-001 |
| QuantizerConfig | PROP-QUANT-001 |

---
