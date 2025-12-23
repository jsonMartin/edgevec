//! Binary Vector Storage Module.
//!
//! Storage layer for binary quantized vectors with tombstone support.
//!
//! # Memory Layout
//!
//! All vectors are stored contiguously in a single `Vec<u8>`:
//!
//! ```text
//! |-- Vector 0 --|-- Vector 1 --|-- Vector 2 --|...
//! |<- bytes_per ->|<- bytes_per ->|<- bytes_per ->|
//! ```
//!
//! Where `bytes_per = dimension / 8`.
//!
//! # Tombstones
//!
//! Deleted vectors are tracked via a `BitVec`. Deletion is a soft delete
//! that marks the tombstone bit without modifying the data. This allows
//! O(1) deletion at the cost of memory fragmentation.
//!
//! # Example
//!
//! ```
//! use edgevec::storage::binary::BinaryVectorStorage;
//! use edgevec::quantization::variable::BinaryVector;
//!
//! let mut storage = BinaryVectorStorage::new(768).unwrap();
//!
//! // Insert a vector
//! let v = vec![1.0f32; 768];
//! let bv = BinaryVector::quantize(&v).unwrap();
//! let id = storage.insert(&bv).unwrap();
//!
//! // Retrieve it
//! let retrieved = storage.get(id).unwrap();
//! assert_eq!(retrieved.dimension(), 768);
//!
//! // Delete it
//! storage.delete(id).unwrap();
//! assert!(storage.get(id).is_none());
//! ```

use crate::quantization::variable::BinaryVector;
use bitvec::order::Lsb0;
use bitvec::vec::BitVec;
use std::fmt;

/// Errors for binary storage operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryStorageError {
    /// Dimension must be divisible by 8.
    InvalidDimension {
        /// The invalid dimension provided.
        dimension: usize,
    },
    /// Vector dimension doesn't match storage dimension.
    DimensionMismatch {
        /// Expected dimension.
        expected: usize,
        /// Actual dimension provided.
        actual: usize,
    },
    /// Vector ID not found.
    NotFound {
        /// The ID that was not found.
        id: u64,
    },
    /// Vector already deleted.
    AlreadyDeleted {
        /// The ID that was already deleted.
        id: u64,
    },
}

impl fmt::Display for BinaryStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimension { dimension } => {
                write!(
                    f,
                    "dimension must be divisible by 8, got {dimension}. \
                     Try using a dimension like 128, 384, 768, 1024, or 1536."
                )
            }
            Self::DimensionMismatch { expected, actual } => {
                write!(f, "dimension mismatch: expected {expected}, got {actual}")
            }
            Self::NotFound { id } => {
                write!(f, "vector with id {id} not found")
            }
            Self::AlreadyDeleted { id } => {
                write!(f, "vector with id {id} is already deleted")
            }
        }
    }
}

impl std::error::Error for BinaryStorageError {}

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
///
/// # Example
///
/// ```
/// use edgevec::storage::binary::BinaryVectorStorage;
/// use edgevec::quantization::variable::BinaryVector;
///
/// let mut storage = BinaryVectorStorage::new(128).unwrap();
/// let v = vec![1.0f32; 128];
/// let bv = BinaryVector::quantize(&v).unwrap();
///
/// let id = storage.insert(&bv).unwrap();
/// assert_eq!(storage.len(), 1);
/// assert_eq!(storage.memory_bytes(), 16); // 128 / 8
/// ```
#[derive(Clone)]
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

impl BinaryVectorStorage {
    /// Creates a new storage for vectors of the given dimension.
    ///
    /// # Arguments
    ///
    /// * `dimension` - Dimension of vectors (must be divisible by 8).
    ///
    /// # Errors
    ///
    /// Returns `BinaryStorageError::InvalidDimension` if dimension % 8 != 0
    /// or dimension is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use edgevec::storage::binary::BinaryVectorStorage;
    ///
    /// let storage = BinaryVectorStorage::new(768).unwrap();
    /// assert_eq!(storage.dimension(), 768);
    /// assert_eq!(storage.bytes_per_vector(), 96);
    /// ```
    pub fn new(dimension: usize) -> Result<Self, BinaryStorageError> {
        if dimension == 0 || dimension % 8 != 0 {
            return Err(BinaryStorageError::InvalidDimension { dimension });
        }

        Ok(Self {
            data: Vec::new(),
            dimension,
            bytes_per_vector: dimension / 8,
            deleted: BitVec::new(),
            count: 0,
            next_id: 0,
        })
    }

    /// Returns the dimension of stored vectors.
    #[must_use]
    #[inline]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns the number of bytes per vector.
    #[must_use]
    #[inline]
    pub fn bytes_per_vector(&self) -> usize {
        self.bytes_per_vector
    }

    /// Returns the number of vectors (including deleted).
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if storage is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns total memory usage in bytes for vector data.
    ///
    /// Does not include metadata overhead (tombstones, etc.).
    #[must_use]
    #[inline]
    pub fn memory_bytes(&self) -> usize {
        self.data.len()
    }

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

    /// Inserts raw bytes as a binary vector and returns its ID.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw binary data (must have length = dimension / 8).
    ///
    /// # Errors
    ///
    /// Returns `BinaryStorageError::DimensionMismatch` if data length
    /// doesn't match bytes_per_vector.
    ///
    /// # Example
    ///
    /// ```
    /// use edgevec::storage::binary::BinaryVectorStorage;
    ///
    /// let mut storage = BinaryVectorStorage::new(128).unwrap();
    /// let data = vec![0xFF; 16]; // 128 bits
    ///
    /// let id = storage.insert_raw(&data).unwrap();
    /// assert_eq!(id, 0);
    /// ```
    pub fn insert_raw(&mut self, data: &[u8]) -> Result<u64, BinaryStorageError> {
        if data.len() != self.bytes_per_vector {
            return Err(BinaryStorageError::DimensionMismatch {
                expected: self.dimension,
                actual: data.len() * 8,
            });
        }

        let id = self.next_id;
        self.next_id += 1;

        self.data.extend_from_slice(data);
        self.deleted.push(false);
        self.count += 1;

        Ok(id)
    }

    /// Retrieves a vector by ID.
    ///
    /// Returns `None` if ID is out of bounds or vector is deleted.
    ///
    /// # Example
    ///
    /// ```
    /// use edgevec::storage::binary::BinaryVectorStorage;
    /// use edgevec::quantization::variable::BinaryVector;
    ///
    /// let mut storage = BinaryVectorStorage::new(128).unwrap();
    /// let v = vec![1.0f32; 128];
    /// let bv = BinaryVector::quantize(&v).unwrap();
    ///
    /// let id = storage.insert(&bv).unwrap();
    /// let retrieved = storage.get(id).unwrap();
    /// assert_eq!(retrieved.dimension(), 128);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
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
    ///
    /// # Performance
    ///
    /// This is the fastest way to access vector data for distance calculations.
    ///
    /// # Example
    ///
    /// ```
    /// use edgevec::storage::binary::BinaryVectorStorage;
    /// use edgevec::quantization::variable::BinaryVector;
    ///
    /// let mut storage = BinaryVectorStorage::new(128).unwrap();
    /// let v = vec![1.0f32; 128];
    /// let bv = BinaryVector::quantize(&v).unwrap();
    ///
    /// let id = storage.insert(&bv).unwrap();
    /// let raw = storage.get_raw(id).unwrap();
    /// assert_eq!(raw.len(), 16); // 128 / 8
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
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
    ///
    /// # Example
    ///
    /// ```
    /// use edgevec::storage::binary::BinaryVectorStorage;
    /// use edgevec::quantization::variable::BinaryVector;
    ///
    /// let mut storage = BinaryVectorStorage::new(128).unwrap();
    /// let v = vec![1.0f32; 128];
    /// let bv = BinaryVector::quantize(&v).unwrap();
    ///
    /// let id = storage.insert(&bv).unwrap();
    /// storage.delete(id).unwrap();
    /// assert!(storage.get(id).is_none());
    /// ```
    #[allow(clippy::cast_possible_truncation)]
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
    ///
    /// Returns false if ID is out of bounds.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn is_deleted(&self, id: u64) -> bool {
        let idx = id as usize;
        idx < self.count && self.deleted[idx]
    }

    /// Returns the number of live (non-deleted) vectors.
    #[must_use]
    pub fn live_count(&self) -> usize {
        self.count - self.deleted_count()
    }

    /// Returns the number of deleted vectors.
    #[must_use]
    pub fn deleted_count(&self) -> usize {
        self.deleted.count_ones()
    }

    /// Returns the ratio of live vectors to total vectors.
    ///
    /// Returns 1.0 if storage is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use edgevec::storage::binary::BinaryVectorStorage;
    /// use edgevec::quantization::variable::BinaryVector;
    ///
    /// let mut storage = BinaryVectorStorage::new(128).unwrap();
    ///
    /// // Insert 10 vectors
    /// for i in 0..10 {
    ///     let v = vec![(i as f32); 128];
    ///     let bv = BinaryVector::quantize(&v).unwrap();
    ///     storage.insert(&bv).unwrap();
    /// }
    ///
    /// // Delete 2
    /// storage.delete(0).unwrap();
    /// storage.delete(5).unwrap();
    ///
    /// // 8/10 = 0.8
    /// assert!((storage.compaction_ratio() - 0.8).abs() < 0.001);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
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
    ///
    /// # Example
    ///
    /// ```
    /// use edgevec::storage::binary::BinaryVectorStorage;
    /// use edgevec::quantization::variable::BinaryVector;
    ///
    /// let mut storage = BinaryVectorStorage::new(128).unwrap();
    ///
    /// for i in 0..5 {
    ///     let v = vec![(i as f32); 128];
    ///     let bv = BinaryVector::quantize(&v).unwrap();
    ///     storage.insert(&bv).unwrap();
    /// }
    ///
    /// storage.delete(1).unwrap();
    /// storage.delete(3).unwrap();
    ///
    /// let live_ids: Vec<_> = storage.iter_live().map(|(id, _)| id).collect();
    /// assert_eq!(live_ids, vec![0, 2, 4]);
    /// ```
    pub fn iter_live(&self) -> impl Iterator<Item = (u64, &[u8])> {
        let bytes_per_vector = self.bytes_per_vector;
        let data = &self.data;

        (0..self.count)
            .filter(|&idx| !self.deleted[idx])
            .map(move |idx| {
                let start = idx * bytes_per_vector;
                let end = start + bytes_per_vector;
                (idx as u64, &data[start..end])
            })
    }

    /// Returns an iterator over all vectors (including deleted).
    ///
    /// Yields `(id, raw_data, is_deleted)` triples.
    pub fn iter_all(&self) -> impl Iterator<Item = (u64, &[u8], bool)> {
        let bytes_per_vector = self.bytes_per_vector;
        let data = &self.data;
        let deleted = &self.deleted;

        (0..self.count).map(move |idx| {
            let start = idx * bytes_per_vector;
            let end = start + bytes_per_vector;
            (idx as u64, &data[start..end], deleted[idx])
        })
    }

    /// Compacts internal buffers to minimize memory usage.
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
        self.deleted.shrink_to_fit();
    }

    /// Reserves capacity for at least `additional` more vectors.
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional * self.bytes_per_vector);
        self.deleted.reserve(additional);
    }

    /// Returns the capacity (number of vectors) before reallocation.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.capacity() / self.bytes_per_vector
    }
}

impl fmt::Debug for BinaryVectorStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinaryVectorStorage")
            .field("dimension", &self.dimension)
            .field("bytes_per_vector", &self.bytes_per_vector)
            .field("count", &self.count)
            .field("live_count", &self.live_count())
            .field("memory_bytes", &self.memory_bytes())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[allow(clippy::cast_precision_loss)]
mod tests {
    use super::*;

    fn make_vector(dimension: usize, value: f32) -> BinaryVector {
        let v = vec![value; dimension];
        BinaryVector::quantize(&v).unwrap()
    }

    #[test]
    fn test_new_valid_dimensions() {
        assert!(BinaryVectorStorage::new(128).is_ok());
        assert!(BinaryVectorStorage::new(384).is_ok());
        assert!(BinaryVectorStorage::new(768).is_ok());
        assert!(BinaryVectorStorage::new(1024).is_ok());
        assert!(BinaryVectorStorage::new(1536).is_ok());
    }

    #[test]
    fn test_new_invalid_dimension_not_divisible() {
        let result = BinaryVectorStorage::new(100);
        assert!(matches!(
            result,
            Err(BinaryStorageError::InvalidDimension { dimension: 100 })
        ));
    }

    #[test]
    fn test_new_invalid_dimension_zero() {
        let result = BinaryVectorStorage::new(0);
        assert!(matches!(
            result,
            Err(BinaryStorageError::InvalidDimension { dimension: 0 })
        ));
    }

    #[test]
    fn test_insert_and_get() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let bv = make_vector(128, 1.0);

        let id = storage.insert(&bv).unwrap();
        assert_eq!(id, 0);
        assert_eq!(storage.len(), 1);

        let retrieved = storage.get(id).unwrap();
        assert_eq!(retrieved.dimension(), 128);
        assert_eq!(retrieved.data(), bv.data());
    }

    #[test]
    fn test_insert_multiple() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..5 {
            let bv = make_vector(128, i as f32);
            let id = storage.insert(&bv).unwrap();
            assert_eq!(id, i);
        }

        assert_eq!(storage.len(), 5);
    }

    #[test]
    fn test_insert_dimension_mismatch() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let bv = make_vector(256, 1.0);

        let result = storage.insert(&bv);
        assert!(matches!(
            result,
            Err(BinaryStorageError::DimensionMismatch {
                expected: 128,
                actual: 256
            })
        ));
    }

    #[test]
    fn test_insert_raw() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let data = vec![0xFF; 16];

        let id = storage.insert_raw(&data).unwrap();
        assert_eq!(id, 0);

        let raw = storage.get_raw(id).unwrap();
        assert_eq!(raw, &data[..]);
    }

    #[test]
    fn test_insert_raw_dimension_mismatch() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let data = vec![0xFF; 32]; // 256 bits instead of 128

        let result = storage.insert_raw(&data);
        assert!(matches!(
            result,
            Err(BinaryStorageError::DimensionMismatch { .. })
        ));
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = BinaryVectorStorage::new(128).unwrap();
        assert!(storage.get(0).is_none());
        assert!(storage.get(100).is_none());
    }

    #[test]
    fn test_get_raw() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let bv = make_vector(128, 1.0);

        let id = storage.insert(&bv).unwrap();
        let raw = storage.get_raw(id).unwrap();

        assert_eq!(raw.len(), 16); // 128 / 8
        assert_eq!(raw, bv.data());
    }

    #[test]
    fn test_get_raw_nonexistent() {
        let storage = BinaryVectorStorage::new(128).unwrap();
        assert!(storage.get_raw(0).is_none());
    }

    #[test]
    fn test_delete() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let bv = make_vector(128, 1.0);

        let id = storage.insert(&bv).unwrap();
        assert!(storage.get(id).is_some());

        storage.delete(id).unwrap();
        assert!(storage.get(id).is_none());
        assert!(storage.is_deleted(id));
    }

    #[test]
    fn test_delete_not_found() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let result = storage.delete(0);
        assert!(matches!(
            result,
            Err(BinaryStorageError::NotFound { id: 0 })
        ));
    }

    #[test]
    fn test_delete_already_deleted() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let bv = make_vector(128, 1.0);

        let id = storage.insert(&bv).unwrap();
        storage.delete(id).unwrap();

        let result = storage.delete(id);
        assert!(matches!(
            result,
            Err(BinaryStorageError::AlreadyDeleted { id: 0 })
        ));
    }

    #[test]
    fn test_is_deleted() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let bv = make_vector(128, 1.0);

        let id = storage.insert(&bv).unwrap();
        assert!(!storage.is_deleted(id));

        storage.delete(id).unwrap();
        assert!(storage.is_deleted(id));
    }

    #[test]
    fn test_is_deleted_out_of_bounds() {
        let storage = BinaryVectorStorage::new(128).unwrap();
        assert!(!storage.is_deleted(100));
    }

    #[test]
    fn test_live_count() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..5 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        assert_eq!(storage.len(), 5);
        assert_eq!(storage.live_count(), 5);
        assert_eq!(storage.deleted_count(), 0);

        storage.delete(1).unwrap();
        storage.delete(3).unwrap();

        assert_eq!(storage.len(), 5);
        assert_eq!(storage.live_count(), 3);
        assert_eq!(storage.deleted_count(), 2);
    }

    #[test]
    fn test_compaction_ratio() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        // Empty storage
        assert!((storage.compaction_ratio() - 1.0).abs() < 0.001);

        // Insert 10 vectors
        for i in 0..10 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        // No deletions: 10/10 = 1.0
        assert!((storage.compaction_ratio() - 1.0).abs() < 0.001);

        // Delete 2: 8/10 = 0.8
        storage.delete(0).unwrap();
        storage.delete(5).unwrap();
        assert!((storage.compaction_ratio() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_iter_live() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..5 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        storage.delete(1).unwrap();
        storage.delete(3).unwrap();

        let live_ids: Vec<_> = storage.iter_live().map(|(id, _)| id).collect();
        assert_eq!(live_ids, vec![0, 2, 4]);
    }

    #[test]
    fn test_iter_all() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..3 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        storage.delete(1).unwrap();

        let all: Vec<_> = storage.iter_all().collect();
        assert_eq!(all.len(), 3);
        assert!(!all[0].2); // not deleted
        assert!(all[1].2); // deleted
        assert!(!all[2].2); // not deleted
    }

    #[test]
    fn test_memory_bytes() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();

        assert_eq!(storage.memory_bytes(), 0);

        let bv = make_vector(768, 1.0);
        storage.insert(&bv).unwrap();

        assert_eq!(storage.memory_bytes(), 96); // 768 / 8

        storage.insert(&bv).unwrap();
        assert_eq!(storage.memory_bytes(), 192); // 2 * 96
    }

    #[test]
    fn test_bytes_per_vector() {
        let storage = BinaryVectorStorage::new(768).unwrap();
        assert_eq!(storage.bytes_per_vector(), 96);

        let storage = BinaryVectorStorage::new(128).unwrap();
        assert_eq!(storage.bytes_per_vector(), 16);

        let storage = BinaryVectorStorage::new(1536).unwrap();
        assert_eq!(storage.bytes_per_vector(), 192);
    }

    #[test]
    fn test_reserve_and_capacity() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        assert_eq!(storage.capacity(), 0);

        storage.reserve(100);
        assert!(storage.capacity() >= 100);
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        storage.reserve(1000);

        let bv = make_vector(128, 1.0);
        storage.insert(&bv).unwrap();

        let capacity_before = storage.capacity();
        storage.shrink_to_fit();
        let capacity_after = storage.capacity();

        assert!(capacity_after <= capacity_before);
    }

    #[test]
    fn test_debug_format() {
        let storage = BinaryVectorStorage::new(768).unwrap();
        let debug = format!("{storage:?}");
        assert!(debug.contains("BinaryVectorStorage"));
        assert!(debug.contains("dimension: 768"));
        assert!(debug.contains("bytes_per_vector: 96"));
        // Uses finish_non_exhaustive() so should contain ".."
        assert!(debug.contains(".."));
    }

    #[test]
    fn test_error_display() {
        let err = BinaryStorageError::InvalidDimension { dimension: 100 };
        let msg = err.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("divisible by 8"));

        let err = BinaryStorageError::DimensionMismatch {
            expected: 128,
            actual: 256,
        };
        let msg = err.to_string();
        assert!(msg.contains("128"));
        assert!(msg.contains("256"));

        let err = BinaryStorageError::NotFound { id: 42 };
        let msg = err.to_string();
        assert!(msg.contains("42"));

        let err = BinaryStorageError::AlreadyDeleted { id: 42 };
        let msg = err.to_string();
        assert!(msg.contains("42"));
    }

    #[test]
    fn test_clone() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let bv = make_vector(128, 1.0);
        storage.insert(&bv).unwrap();

        let cloned = storage.clone();
        assert_eq!(cloned.len(), storage.len());
        assert_eq!(cloned.dimension(), storage.dimension());
        assert_eq!(cloned.get_raw(0), storage.get_raw(0));
    }
}
