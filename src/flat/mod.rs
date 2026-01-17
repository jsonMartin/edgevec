//! Flat (brute-force) index for binary vectors.
//!
//! This module provides a simple flat index optimized for binary vectors.
//! Unlike HNSW, it uses O(1) insert and O(n) search, which is faster for
//! small-to-medium datasets (< 100K vectors) due to the extremely fast
//! SIMD Hamming distance calculation.
//!
//! # Performance Characteristics
//!
//! | Operation | Complexity | Time (10K vectors) |
//! |-----------|------------|-------------------|
//! | Insert    | O(1)       | ~1 μs             |
//! | Search    | O(n)       | ~1ms (SIMD)       |
//!
//! # When to Use
//!
//! - Insert-heavy workloads (semantic caching)
//! - Datasets < 100K vectors
//! - When 100% recall (exact search) is required
//! - When insert latency is critical

use crate::hnsw::VectorId;
use crate::metric::{Hamming, Metric};
use serde::{Deserialize, Serialize};

/// Search result from flat index.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FlatSearchResult {
    /// Vector ID.
    pub id: VectorId,
    /// Hamming distance (lower is more similar).
    pub distance: f32,
}

/// A flat (brute-force) index for binary vectors.
///
/// Stores vectors in a contiguous array for cache-friendly linear scan.
/// Insert is O(1), search is O(n) with SIMD-accelerated Hamming distance.
#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryFlatIndex {
    /// Contiguous storage for all vectors.
    vectors: Vec<u8>,
    /// Number of bits per vector.
    dimensions: usize,
    /// Number of bytes per vector (dimensions / 8).
    bytes_per_vector: usize,
    /// Number of vectors stored.
    count: usize,
}

impl BinaryFlatIndex {
    /// Create a new binary flat index.
    ///
    /// # Arguments
    ///
    /// * `dimensions` - Number of bits per vector (must be divisible by 8)
    ///
    /// # Panics
    ///
    /// Panics if dimensions is not divisible by 8.
    #[must_use]
    pub fn new(dimensions: usize) -> Self {
        assert!(
            dimensions % 8 == 0,
            "dimensions must be divisible by 8, got {}",
            dimensions
        );
        Self {
            vectors: Vec::new(),
            dimensions,
            bytes_per_vector: dimensions / 8,
            count: 0,
        }
    }

    /// Create a new binary flat index with pre-allocated capacity.
    ///
    /// # Arguments
    ///
    /// * `dimensions` - Number of bits per vector (must be divisible by 8)
    /// * `capacity` - Number of vectors to pre-allocate space for
    #[must_use]
    pub fn with_capacity(dimensions: usize, capacity: usize) -> Self {
        assert!(
            dimensions % 8 == 0,
            "dimensions must be divisible by 8, got {}",
            dimensions
        );
        let bytes_per_vector = dimensions / 8;
        Self {
            vectors: Vec::with_capacity(capacity * bytes_per_vector),
            dimensions,
            bytes_per_vector,
            count: 0,
        }
    }

    /// Insert a binary vector into the index.
    ///
    /// # Arguments
    ///
    /// * `vector` - Binary vector as packed bytes
    ///
    /// # Returns
    ///
    /// The ID of the inserted vector.
    ///
    /// # Panics
    ///
    /// Panics if vector length doesn't match bytes_per_vector.
    #[inline]
    pub fn insert(&mut self, vector: &[u8]) -> VectorId {
        assert_eq!(
            vector.len(),
            self.bytes_per_vector,
            "vector length {} doesn't match expected {}",
            vector.len(),
            self.bytes_per_vector
        );

        self.vectors.extend_from_slice(vector);
        self.count += 1;
        // Start IDs at 1 to match EdgeVec/VectorStorage convention (0 is reserved sentinel)
        VectorId(self.count as u64)
    }

    /// Search for the k nearest neighbors to a query vector.
    ///
    /// Uses SIMD-accelerated Hamming distance for fast linear scan.
    ///
    /// # Arguments
    ///
    /// * `query` - Query vector as packed bytes
    /// * `k` - Number of nearest neighbors to return
    ///
    /// # Returns
    ///
    /// Vector of search results sorted by distance (ascending).
    ///
    /// # Panics
    ///
    /// Panics if query length doesn't match `bytes_per_vector()`.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn search(&self, query: &[u8], k: usize) -> Vec<FlatSearchResult> {
        assert_eq!(
            query.len(),
            self.bytes_per_vector,
            "query length {} doesn't match expected {}",
            query.len(),
            self.bytes_per_vector
        );

        if self.count == 0 || k == 0 {
            return Vec::new();
        }

        let k = k.min(self.count);

        // For small k, use a simple approach: compute all distances, partial sort
        // For larger datasets, could use a heap for O(n log k) instead of O(n log n)
        let mut results: Vec<(VectorId, f32)> = Vec::with_capacity(self.count);

        for i in 0..self.count {
            let start = i * self.bytes_per_vector;
            let end = start + self.bytes_per_vector;
            let stored = &self.vectors[start..end];
            let dist = Hamming::distance(query, stored);
            // IDs are 1-based (i+1) to match EdgeVec convention
            results.push((VectorId((i + 1) as u64), dist));
        }

        // Partial sort to get top k (more efficient than full sort for small k)
        if k < self.count / 10 {
            // Use partial sort for small k
            results.select_nth_unstable_by(k - 1, |a, b| {
                a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            results.truncate(k);
            results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        } else {
            // Full sort for large k
            results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            results.truncate(k);
        }

        results
            .into_iter()
            .map(|(id, distance)| FlatSearchResult { id, distance })
            .collect()
    }

    /// Get a vector by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Vector ID
    ///
    /// # Returns
    ///
    /// The vector bytes, or None if ID is out of bounds.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get(&self, id: VectorId) -> Option<&[u8]> {
        // IDs are 1-based, so subtract 1 to get 0-based index
        // SAFETY: VectorId.0 is u64 but in practice never exceeds usize::MAX
        // on any supported platform (WASM32 or x86_64).
        let idx = (id.0 as usize).checked_sub(1)?;
        if idx >= self.count {
            return None;
        }
        let start = idx * self.bytes_per_vector;
        let end = start + self.bytes_per_vector;
        Some(&self.vectors[start..end])
    }

    /// Get the number of vectors in the index.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if the index is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get the dimensions (bits) per vector.
    #[must_use]
    #[inline]
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Get the bytes per vector.
    #[must_use]
    #[inline]
    pub fn bytes_per_vector(&self) -> usize {
        self.bytes_per_vector
    }

    /// Get approximate memory usage in bytes.
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + self.vectors.capacity()
    }

    /// Get the length of the internal vectors buffer.
    #[inline]
    #[must_use]
    pub fn vectors_len(&self) -> usize {
        self.vectors.len()
    }

    /// Estimate the serialized size in bytes.
    ///
    /// Format: header (8 bytes: dimensions u32 + count u32) + vector data.
    #[must_use]
    pub fn serialized_size(&self) -> usize {
        8 + self.vectors.len()
    }

    /// Clear all vectors from the index.
    pub fn clear(&mut self) {
        self.vectors.clear();
        self.count = 0;
    }

    /// Shrink the internal storage to fit the current number of vectors.
    pub fn shrink_to_fit(&mut self) {
        self.vectors.shrink_to_fit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let index = BinaryFlatIndex::new(1024);
        assert_eq!(index.dimensions(), 1024);
        assert_eq!(index.bytes_per_vector(), 128);
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_insert_and_get() {
        let mut index = BinaryFlatIndex::new(64); // 8 bytes per vector
        let v1 = vec![0xFF; 8];
        let v2 = vec![0x00; 8];

        let id1 = index.insert(&v1);
        let id2 = index.insert(&v2);

        assert_eq!(id1, VectorId(1)); // IDs are 1-based
        assert_eq!(id2, VectorId(2));
        assert_eq!(index.len(), 2);
        assert_eq!(index.get(id1), Some(v1.as_slice()));
        assert_eq!(index.get(id2), Some(v2.as_slice()));
        assert_eq!(index.get(VectorId(99)), None);
    }

    #[test]
    fn test_search_exact_match() {
        let mut index = BinaryFlatIndex::new(64);

        // Insert some vectors
        let v1 = vec![0xFF; 8];
        let v2 = vec![0x00; 8];
        let v3 = vec![0xAA; 8];

        index.insert(&v1);
        index.insert(&v2);
        index.insert(&v3);

        // Search for exact match
        let results = index.search(&v2, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, VectorId(2)); // v2 is second insert, ID=2
        assert_eq!(results[0].distance, 0.0);
    }

    #[test]
    fn test_search_ordering() {
        let mut index = BinaryFlatIndex::new(64);

        // Insert vectors with known distances from query
        let query = vec![0x00; 8]; // All zeros
        let v1 = vec![0xFF; 8]; // 64 bits different
        let v2 = vec![0x0F; 8]; // 32 bits different
        let v3 = vec![0x01; 8]; // 8 bits different

        index.insert(&v1); // id=1, dist=64
        index.insert(&v2); // id=2, dist=32
        index.insert(&v3); // id=3, dist=8

        let results = index.search(&query, 3);

        // Should be ordered by distance (IDs are 1-based)
        assert_eq!(results[0].id, VectorId(3)); // Closest
        assert_eq!(results[0].distance, 8.0);
        assert_eq!(results[1].id, VectorId(2));
        assert_eq!(results[1].distance, 32.0);
        assert_eq!(results[2].id, VectorId(1)); // Farthest
        assert_eq!(results[2].distance, 64.0);
    }

    #[test]
    fn test_search_k_limit() {
        let mut index = BinaryFlatIndex::new(64);

        for i in 0..100 {
            let v: Vec<u8> = (0..8).map(|j| ((i + j) % 256) as u8).collect();
            index.insert(&v);
        }

        let query = vec![0x00; 8];
        let results = index.search(&query, 5);

        assert_eq!(results.len(), 5);
        // Results should be sorted by distance
        for i in 1..results.len() {
            assert!(results[i - 1].distance <= results[i].distance);
        }
    }

    #[test]
    fn test_empty_search() {
        let index = BinaryFlatIndex::new(64);
        let query = vec![0x00; 8];
        let results = index.search(&query, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_k_zero() {
        let mut index = BinaryFlatIndex::new(64);
        // Insert enough vectors to trigger the partial sort path (count >= 10)
        for _ in 0..20 {
            index.insert(&vec![0xFF; 8]);
        }
        // k=0 should return empty, not panic
        let results = index.search(&vec![0x00; 8], 0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut index = BinaryFlatIndex::new(64);
        index.insert(&vec![0xFF; 8]);
        index.insert(&vec![0x00; 8]);

        assert_eq!(index.len(), 2);

        index.clear();

        assert_eq!(index.len(), 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_memory_usage() {
        let mut index = BinaryFlatIndex::with_capacity(1024, 1000);
        assert!(index.memory_usage() > 0);

        for _ in 0..100 {
            index.insert(&vec![0xAA; 128]);
        }

        let usage = index.memory_usage();
        // Should be at least 100 * 128 = 12,800 bytes
        assert!(usage >= 12_800);
    }

    #[test]
    #[should_panic(expected = "dimensions must be divisible by 8")]
    fn test_invalid_dimensions() {
        let _ = BinaryFlatIndex::new(100); // Not divisible by 8
    }

    #[test]
    #[should_panic(expected = "vector length")]
    fn test_invalid_vector_length() {
        let mut index = BinaryFlatIndex::new(64);
        index.insert(&vec![0xFF; 16]); // Wrong size
    }
}
