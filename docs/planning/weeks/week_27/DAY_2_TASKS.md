# Week 27 Day 2: BinaryVectorStorage Implementation

**Date:** 2025-12-23
**Focus:** Storage layer for binary quantized vectors
**Estimated Duration:** 10 hours
**Phase:** RFC-002 Implementation Phase 2 (Binary Quantization)

---

## Tasks

### W27.2.1: Define BinaryVectorStorage Struct

**Objective:** Create a storage container for variable-dimension binary vectors.

**Design Goals:**
- Contiguous memory layout for cache efficiency
- Support for arbitrary vector counts
- Dimension consistency enforced at runtime
- Memory usage = count × (dimension / 8) bytes

**Acceptance Criteria:**
- [ ] `BinaryVectorStorage` struct with Vec<u8> backing store
- [ ] Constructor: `new(dimension: usize) -> Result<Self, StorageError>`
- [ ] Dimension validation: must be divisible by 8
- [ ] Capacity tracking: `len()`, `is_empty()`, `bytes_per_vector()`
- [ ] Memory estimation: `memory_bytes() -> usize`

**Files:**
- `src/storage/binary.rs` (new file)
- `src/storage/mod.rs` (add module)

**Estimated Duration:** 4 hours

**Agent:** RUST_ENGINEER

**API Design:**

```rust
/// Storage for binary quantized vectors.
///
/// All vectors in the storage have the same dimension.
/// Vectors are stored contiguously for cache efficiency.
///
/// # Memory Layout
///
/// ```text
/// |-- Vector 0 --|-- Vector 1 --|-- Vector 2 --|...
/// |<- bytes_per ->|<- bytes_per ->|<- bytes_per ->|
/// ```
///
/// Where `bytes_per = dimension / 8`.
#[derive(Clone, Debug)]
pub struct BinaryVectorStorage {
    /// Packed binary data (all vectors concatenated).
    data: Vec<u8>,

    /// Dimension of each vector (in bits, must be divisible by 8).
    dimension: usize,

    /// Bytes per vector (dimension / 8).
    bytes_per_vector: usize,

    /// Deleted bit flags (tombstones).
    deleted: BitVec<u8, Lsb0>,

    /// Number of vectors stored (including deleted).
    count: usize,

    /// Next vector ID to assign.
    next_id: u64,
}

/// Errors for binary storage operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryStorageError {
    /// Dimension must be divisible by 8.
    InvalidDimension { dimension: usize },
    /// Vector dimension doesn't match storage dimension.
    DimensionMismatch { expected: usize, actual: usize },
    /// Vector ID not found.
    NotFound { id: u64 },
    /// Vector already deleted.
    AlreadyDeleted { id: u64 },
}

impl BinaryVectorStorage {
    /// Creates a new storage for vectors of the given dimension.
    ///
    /// # Arguments
    ///
    /// * `dimension` - Dimension of vectors (must be divisible by 8).
    ///
    /// # Errors
    ///
    /// Returns `BinaryStorageError::InvalidDimension` if dimension % 8 != 0.
    pub fn new(dimension: usize) -> Result<Self, BinaryStorageError>;

    /// Returns the dimension of stored vectors.
    pub fn dimension(&self) -> usize;

    /// Returns the number of bytes per vector.
    pub fn bytes_per_vector(&self) -> usize;

    /// Returns the number of vectors (including deleted).
    pub fn len(&self) -> usize;

    /// Returns true if storage is empty.
    pub fn is_empty(&self) -> bool;

    /// Returns total memory usage in bytes.
    pub fn memory_bytes(&self) -> usize;
}
```

**Dependencies:** W27.1.1 (BinaryVector for type compatibility)

---

### W27.2.2: Insert/Get/Delete Operations

**Objective:** Implement CRUD operations for binary vectors.

**Acceptance Criteria:**
- [ ] `insert(vector: &BinaryVector) -> Result<u64, BinaryStorageError>`
  - Validates dimension matches storage
  - Appends to data Vec
  - Returns assigned ID
- [ ] `get(id: u64) -> Option<BinaryVector>`
  - Returns None if ID out of bounds or deleted
  - Creates BinaryVector from slice
- [ ] `get_raw(id: u64) -> Option<&[u8]>`
  - Returns slice without copying
  - Used for distance calculations
- [ ] `delete(id: u64) -> Result<(), BinaryStorageError>`
  - Marks tombstone (no data modification)
  - Returns error if already deleted or not found
- [ ] `is_deleted(id: u64) -> bool`

**Files:**
- `src/storage/binary.rs` (extend)
- `tests/binary_storage.rs` (new file)

**Estimated Duration:** 3 hours

**Agent:** RUST_ENGINEER

**Implementation:**

```rust
impl BinaryVectorStorage {
    /// Inserts a binary vector and returns its ID.
    ///
    /// # Arguments
    ///
    /// * `vector` - The binary vector to insert.
    ///
    /// # Errors
    ///
    /// Returns `BinaryStorageError::DimensionMismatch` if vector dimension
    /// doesn't match storage dimension.
    ///
    /// # Example
    ///
    /// ```
    /// use edgevec::storage::binary::BinaryVectorStorage;
    /// use edgevec::quantization::variable::BinaryVector;
    ///
    /// let mut storage = BinaryVectorStorage::new(768).unwrap();
    /// let v = vec![1.0f32; 768];
    /// let bv = BinaryVector::quantize(&v).unwrap();
    ///
    /// let id = storage.insert(&bv).unwrap();
    /// assert_eq!(id, 0);
    /// ```
    pub fn insert(&mut self, vector: &BinaryVector) -> Result<u64, BinaryStorageError> {
        if vector.dimension() != self.dimension {
            return Err(BinaryStorageError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.dimension(),
            });
        }

        let id = self.next_id;
        self.next_id += 1;

        self.data.extend_from_slice(vector.data());
        self.deleted.push(false);
        self.count += 1;

        Ok(id)
    }

    /// Retrieves a vector by ID.
    ///
    /// Returns `None` if ID is out of bounds or vector is deleted.
    pub fn get(&self, id: u64) -> Option<BinaryVector> {
        let idx = id as usize;
        if idx >= self.count || self.deleted[idx] {
            return None;
        }

        let start = idx * self.bytes_per_vector;
        let end = start + self.bytes_per_vector;
        let data = self.data[start..end].to_vec();

        BinaryVector::from_bytes(data, self.dimension).ok()
    }

    /// Returns a reference to vector data without copying.
    ///
    /// Returns `None` if ID is out of bounds or vector is deleted.
    pub fn get_raw(&self, id: u64) -> Option<&[u8]> {
        let idx = id as usize;
        if idx >= self.count || self.deleted[idx] {
            return None;
        }

        let start = idx * self.bytes_per_vector;
        let end = start + self.bytes_per_vector;
        Some(&self.data[start..end])
    }

    /// Marks a vector as deleted (soft delete).
    ///
    /// # Errors
    ///
    /// - `BinaryStorageError::NotFound` if ID is out of bounds.
    /// - `BinaryStorageError::AlreadyDeleted` if already deleted.
    pub fn delete(&mut self, id: u64) -> Result<(), BinaryStorageError> {
        let idx = id as usize;
        if idx >= self.count {
            return Err(BinaryStorageError::NotFound { id });
        }
        if self.deleted[idx] {
            return Err(BinaryStorageError::AlreadyDeleted { id });
        }

        self.deleted.set(idx, true);
        Ok(())
    }

    /// Returns true if the vector is deleted.
    pub fn is_deleted(&self, id: u64) -> bool {
        let idx = id as usize;
        idx < self.count && self.deleted[idx]
    }
}
```

**Dependencies:** W27.2.1

---

### W27.2.3: Tombstone Support and Live Count

**Objective:** Track deleted vectors and provide live count.

**Acceptance Criteria:**
- [ ] `live_count() -> usize` — count of non-deleted vectors
- [ ] `deleted_count() -> usize` — count of deleted vectors
- [ ] `compaction_ratio() -> f64` — live_count / len
- [ ] Iterator: `iter_live() -> impl Iterator<Item = (u64, &[u8])>`
  - Skips deleted vectors
  - Yields (id, raw_data) pairs

**Files:**
- `src/storage/binary.rs` (extend)
- `tests/binary_storage.rs` (extend)

**Estimated Duration:** 3 hours

**Agent:** RUST_ENGINEER

**Implementation:**

```rust
impl BinaryVectorStorage {
    /// Returns the number of live (non-deleted) vectors.
    pub fn live_count(&self) -> usize {
        self.count - self.deleted_count()
    }

    /// Returns the number of deleted vectors.
    pub fn deleted_count(&self) -> usize {
        self.deleted.count_ones()
    }

    /// Returns the ratio of live vectors to total vectors.
    ///
    /// Returns 1.0 if storage is empty.
    pub fn compaction_ratio(&self) -> f64 {
        if self.count == 0 {
            1.0
        } else {
            self.live_count() as f64 / self.count as f64
        }
    }

    /// Returns an iterator over live (non-deleted) vectors.
    ///
    /// Yields `(id, raw_data)` pairs.
    pub fn iter_live(&self) -> impl Iterator<Item = (u64, &[u8])> {
        (0..self.count)
            .filter(|&idx| !self.deleted[idx])
            .map(move |idx| {
                let start = idx * self.bytes_per_vector;
                let end = start + self.bytes_per_vector;
                (idx as u64, &self.data[start..end])
            })
    }
}
```

**Test Cases:**

```rust
// tests/binary_storage.rs

mod tombstones {
    use edgevec::storage::binary::BinaryVectorStorage;
    use edgevec::quantization::variable::BinaryVector;

    #[test]
    fn test_live_count_after_delete() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        // Insert 5 vectors
        for i in 0..5 {
            let v = vec![(i as f32); 128];
            let bv = BinaryVector::quantize(&v).unwrap();
            storage.insert(&bv).unwrap();
        }

        assert_eq!(storage.len(), 5);
        assert_eq!(storage.live_count(), 5);
        assert_eq!(storage.deleted_count(), 0);

        // Delete 2 vectors
        storage.delete(1).unwrap();
        storage.delete(3).unwrap();

        assert_eq!(storage.len(), 5);
        assert_eq!(storage.live_count(), 3);
        assert_eq!(storage.deleted_count(), 2);
    }

    #[test]
    fn test_iter_live_skips_deleted() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..5 {
            let v = vec![(i as f32); 128];
            let bv = BinaryVector::quantize(&v).unwrap();
            storage.insert(&bv).unwrap();
        }

        storage.delete(1).unwrap();
        storage.delete(3).unwrap();

        let live_ids: Vec<_> = storage.iter_live().map(|(id, _)| id).collect();
        assert_eq!(live_ids, vec![0, 2, 4]);
    }

    #[test]
    fn test_compaction_ratio() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..10 {
            let v = vec![(i as f32); 128];
            let bv = BinaryVector::quantize(&v).unwrap();
            storage.insert(&bv).unwrap();
        }

        storage.delete(0).unwrap();
        storage.delete(5).unwrap();

        // 8/10 = 0.8
        assert!((storage.compaction_ratio() - 0.8).abs() < 0.001);
    }
}
```

**Dependencies:** W27.2.1, W27.2.2

---

## Day 2 Checklist

- [x] W27.2.1: BinaryVectorStorage struct defined
- [x] W27.2.2: Insert/get/delete operations implemented
- [x] W27.2.3: Tombstone support + live count
- [x] All existing tests pass (`cargo test`)
- [x] New tests pass (`cargo test binary_storage`)
- [x] Clippy clean (`cargo clippy -- -D warnings`)
- [x] Formatted (`cargo fmt --check`)

## Day 2 Exit Criteria

| Criterion | Verification |
|:----------|:-------------|
| `cargo test` passes | CI green |
| Storage CRUD works | Unit tests |
| Tombstone tracking works | Deletion tests |
| Memory usage is correct | Bytes per vector = dim/8 |

## Day 2 Handoff

After completing Day 2:

**Artifacts Generated:**
- `src/storage/binary.rs`
- `tests/binary_storage.rs`

**Status:** COMPLETE

**Next:** Day 3 — HNSW BQ search integration

---

*Agent: RUST_ENGINEER*
*Status: [IMPLEMENTED] (2025-12-21)*

## Implementation Notes (2025-12-21)

### Changes Made

**[C1] Converted storage module to directory structure**
- Moved `src/storage.rs` to `src/storage/mod.rs`
- Created `src/storage/binary.rs` with `BinaryVectorStorage`
- Added `pub mod binary;` to mod.rs

**[C2] Implemented BinaryVectorStorage struct**
- Fields: `data: Vec<u8>`, `dimension: usize`, `bytes_per_vector: usize`, `deleted: BitVec<u8, Lsb0>`, `count: usize`, `next_id: u64`
- Constructor validates dimension divisible by 8
- Memory-efficient contiguous storage layout

**[C3] Implemented CRUD operations**
- `insert(&BinaryVector)` - dimension validation, auto-increment ID
- `insert_raw(&[u8])` - raw bytes insertion
- `get(u64) -> Option<BinaryVector>` - returns None for deleted/OOB
- `get_raw(u64) -> Option<&[u8]>` - zero-copy access for distance calculations
- `delete(u64) -> Result<(), Error>` - soft delete with tombstone

**[C4] Implemented tombstone support**
- `live_count()` - count of non-deleted vectors
- `deleted_count()` - count of deleted vectors
- `compaction_ratio()` - live/total ratio
- `iter_live()` - iterator over live vectors
- `iter_all()` - iterator over all vectors with deleted flag

**[C5] Created comprehensive test suite**
- `tests/binary_storage.rs` with 46 integration tests
- Tests cover: construction, insert, get, delete, tombstones, memory, properties, clone

### Validation Results

| Check | Result |
|:------|:-------|
| `cargo test --lib` | 650 tests passed |
| `cargo test --test binary_storage` | 46 tests passed |
| `cargo test --test binary_vector` | 29 tests passed |
| `cargo clippy -- -D warnings` | Clean |
| `cargo fmt --check` | Clean |
