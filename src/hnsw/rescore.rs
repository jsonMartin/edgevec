#![allow(clippy::cast_precision_loss)]

//! F32 rescoring for BQ search results (v0.7.0 - RFC-002 Phase 2).
//!
//! This module provides rescoring functions that recompute exact F32
//! distances for BQ search candidates, recovering recall lost during
//! binary quantization.
//!
//! # Algorithm
//!
//! 1. Take BQ search results (approximate similarity)
//! 2. Load F32 vectors for each candidate
//! 3. Compute exact L2 squared distance
//! 4. Sort by exact distance and return top-k
//!
//! # Performance
//!
//! Rescoring is O(n × d) where n = candidates, d = dimensions.
//! With rescore_factor=3, typical recall improves from ~0.85 to >0.95.

use crate::hnsw::VectorId;
use crate::metric::{L2Squared, Metric};
use crate::storage::VectorStorage;

/// Rescores BQ candidates using exact F32 L2 squared distance.
///
/// Takes approximate BQ results and recomputes the exact distance
/// for each candidate, returning them sorted by F32 distance.
///
/// # Arguments
///
/// * `candidates` - BQ search results (VectorId, approximate_similarity).
/// * `query` - The original query vector.
/// * `storage` - F32 vector storage.
///
/// # Returns
///
/// Candidates sorted by exact L2 squared distance (ascending = most similar first).
///
/// # Performance
///
/// Time: O(n × d) where n = candidates.len(), d = dimension.
/// Space: O(n) for storing rescored results.
///
/// # Example
///
/// ```ignore
/// let bq_results = index.search_bq(&query, k * 3, &storage)?;
/// let rescored = rescore(&bq_results, &query, &storage);
/// let final_results: Vec<_> = rescored.into_iter().take(k).collect();
/// ```
#[must_use]
pub fn rescore(
    candidates: &[(VectorId, f32)],
    query: &[f32],
    storage: &VectorStorage,
) -> Vec<(VectorId, f32)> {
    let mut rescored: Vec<(VectorId, f32)> = candidates
        .iter()
        .filter_map(|(id, _approx_score)| {
            // Skip invalid IDs
            if *id == VectorId::INVALID {
                return None;
            }

            // Skip deleted vectors
            if storage.is_deleted(*id) {
                return None;
            }

            // Load F32 vector from storage
            let vector = storage.get_vector(*id);

            // Compute exact L2 squared distance using SIMD
            let distance = L2Squared::distance(query, &vector);

            Some((*id, distance))
        })
        .collect();

    // Sort by distance (ascending = most similar first)
    rescored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    rescored
}

/// Rescores candidates and returns top-k.
///
/// Convenience function combining rescore + truncate.
///
/// # Arguments
///
/// * `candidates` - BQ search results.
/// * `query` - The query vector.
/// * `storage` - F32 vector storage.
/// * `k` - Number of results to return.
///
/// # Returns
///
/// Top-k candidates sorted by exact F32 distance.
#[must_use]
pub fn rescore_top_k(
    candidates: &[(VectorId, f32)],
    query: &[f32],
    storage: &VectorStorage,
    k: usize,
) -> Vec<(VectorId, f32)> {
    let mut rescored = rescore(candidates, query, storage);
    rescored.truncate(k);
    rescored
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hnsw::HnswConfig;

    #[test]
    fn test_rescore_empty() {
        let config = HnswConfig::new(4);
        let storage = VectorStorage::new(&config, None);
        let query = vec![1.0, 2.0, 3.0, 4.0];

        let rescored = rescore(&[], &query, &storage);
        assert!(rescored.is_empty());
    }

    #[test]
    fn test_rescore_single_vector() {
        let config = HnswConfig::new(4);
        let mut storage = VectorStorage::new(&config, None);

        let v = vec![1.0, 2.0, 3.0, 4.0];
        let id = storage.insert(&v).unwrap();

        let candidates = vec![(id, 0.9)]; // Approximate score (ignored)
        let query = vec![1.0, 2.0, 3.0, 4.0];

        let rescored = rescore(&candidates, &query, &storage);

        assert_eq!(rescored.len(), 1);
        assert_eq!(rescored[0].0, id);
        assert!((rescored[0].1 - 0.0).abs() < 1e-6); // Exact match = distance 0
    }

    #[test]
    fn test_rescore_sorts_correctly() {
        let config = HnswConfig::new(4);
        let mut storage = VectorStorage::new(&config, None);

        // Insert vectors at different distances from query
        let v1 = vec![10.0, 10.0, 10.0, 10.0]; // Far from query
        let v2 = vec![1.0, 2.0, 3.0, 4.0]; // Close to query
        let v3 = vec![2.0, 3.0, 4.0, 5.0]; // Medium distance

        let id1 = storage.insert(&v1).unwrap();
        let id2 = storage.insert(&v2).unwrap();
        let id3 = storage.insert(&v3).unwrap();

        // Candidates in wrong order (simulating BQ approximation error)
        let candidates = vec![(id1, 0.9), (id3, 0.85), (id2, 0.8)];
        let query = vec![1.0, 2.0, 3.0, 4.0];

        let rescored = rescore(&candidates, &query, &storage);

        // Should be sorted by distance: v2 (0.0), v3 (4.0), v1 (324.0)
        assert_eq!(rescored.len(), 3);
        assert_eq!(rescored[0].0, id2); // Closest
        assert_eq!(rescored[1].0, id3); // Medium
        assert_eq!(rescored[2].0, id1); // Farthest
    }

    #[test]
    fn test_rescore_top_k() {
        let config = HnswConfig::new(4);
        let mut storage = VectorStorage::new(&config, None);

        // Insert 5 vectors
        for i in 0..5 {
            let v = vec![i as f32; 4];
            storage.insert(&v).unwrap();
        }

        let candidates: Vec<_> = (1..=5).map(|i| (VectorId(i), 0.5)).collect();
        let query = vec![0.0, 0.0, 0.0, 0.0];

        let rescored = rescore_top_k(&candidates, &query, &storage, 3);

        assert_eq!(rescored.len(), 3);
        // First 3 closest vectors
        assert_eq!(rescored[0].0 .0, 1); // [0,0,0,0] - distance 0
        assert_eq!(rescored[1].0 .0, 2); // [1,1,1,1] - distance 4
        assert_eq!(rescored[2].0 .0, 3); // [2,2,2,2] - distance 16
    }

    #[test]
    fn test_rescore_skips_invalid_ids() {
        let config = HnswConfig::new(4);
        let mut storage = VectorStorage::new(&config, None);

        let v = vec![1.0, 2.0, 3.0, 4.0];
        let id = storage.insert(&v).unwrap();

        // Include invalid ID
        let candidates = vec![(VectorId::INVALID, 0.9), (id, 0.8)];
        let query = vec![1.0, 2.0, 3.0, 4.0];

        let rescored = rescore(&candidates, &query, &storage);

        // Invalid ID should be filtered out
        assert_eq!(rescored.len(), 1);
        assert_eq!(rescored[0].0, id);
    }

    #[test]
    fn test_rescore_skips_deleted_vectors() {
        let config = HnswConfig::new(4);
        let mut storage = VectorStorage::new(&config, None);

        let v1 = vec![1.0, 2.0, 3.0, 4.0];
        let v2 = vec![2.0, 3.0, 4.0, 5.0];

        let id1 = storage.insert(&v1).unwrap();
        let id2 = storage.insert(&v2).unwrap();

        // Delete v1
        storage.mark_deleted(id1);

        let candidates = vec![(id1, 0.9), (id2, 0.8)];
        let query = vec![1.0, 2.0, 3.0, 4.0];

        let rescored = rescore(&candidates, &query, &storage);

        // Deleted vector should be filtered out
        assert_eq!(rescored.len(), 1);
        assert_eq!(rescored[0].0, id2);
    }
}
