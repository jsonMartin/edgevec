//! Integration tests for search_filtered (W26.2.3)
//!
//! Tests post-filtering with adaptive overfetch per RFC-002 §3.2.

use std::collections::HashMap;

use edgevec::hnsw::{HnswConfig, HnswIndex};
use edgevec::metadata::MetadataValue;
use edgevec::storage::VectorStorage;

/// Helper to create a test index and storage.
fn create_test_index(dim: u32) -> (HnswIndex, VectorStorage) {
    let config = HnswConfig::new(dim);
    let storage = VectorStorage::new(&config, None);
    let index = HnswIndex::new(config, &storage).unwrap();
    (index, storage)
}

/// Helper to create metadata with category and price.
fn create_metadata(category: &str, price: i64) -> HashMap<String, MetadataValue> {
    let mut metadata = HashMap::new();
    metadata.insert(
        "category".to_string(),
        MetadataValue::String(category.to_string()),
    );
    metadata.insert("price".to_string(), MetadataValue::Integer(price));
    metadata
}

// =============================================================================
// search_filtered basic tests
// =============================================================================

mod search_filtered_basic {
    use super::*;

    /// Test search_filtered with simple equality filter.
    #[test]
    fn test_search_filtered_category_eq() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert 10 vectors with different categories
        for i in 0..10 {
            let category = if i % 2 == 0 { "books" } else { "electronics" };
            let metadata = create_metadata(category, i * 10);
            index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], metadata)
                .unwrap();
        }

        // Search for books only
        let query = [5.0, 0.0, 0.0, 0.0];
        let results = index
            .search_filtered(&storage, &query, "category = \"books\"", 5)
            .unwrap();

        // Should return up to 5 books
        assert!(!results.is_empty());
        assert!(results.len() <= 5);

        // All results should be books (even indices: 0, 2, 4, 6, 8)
        for (vid, _distance) in &results {
            // VectorId starts at 1, even IDs: 1, 3, 5, 7, 9 → original indices 0, 2, 4, 6, 8
            let original_idx = vid.0 - 1;
            assert!(
                original_idx % 2 == 0,
                "Expected even index (books), got {}",
                original_idx
            );
        }
    }

    /// Test search_filtered with numeric comparison.
    #[test]
    fn test_search_filtered_price_lt() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert 10 vectors with increasing prices
        for i in 0..10 {
            let metadata = create_metadata("item", i * 100);
            index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], metadata)
                .unwrap();
        }

        // Search for items with price < 500
        let query = [5.0, 0.0, 0.0, 0.0];
        let results = index
            .search_filtered(&storage, &query, "price < 500", 10)
            .unwrap();

        // Should return items with price 0, 100, 200, 300, 400 (indices 0-4)
        assert!(!results.is_empty());
        assert!(results.len() <= 5); // Only 5 items have price < 500
    }

    /// Test search_filtered with compound AND filter.
    #[test]
    fn test_search_filtered_compound_and() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert 10 vectors
        for i in 0..10 {
            let category = if i % 2 == 0 { "books" } else { "electronics" };
            let metadata = create_metadata(category, i * 100);
            index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], metadata)
                .unwrap();
        }

        // Search for books with price < 500
        let query = [0.0, 0.0, 0.0, 0.0];
        let results = index
            .search_filtered(&storage, &query, "category = \"books\" AND price < 500", 5)
            .unwrap();

        // Books at indices 0, 2, 4 have prices 0, 200, 400 (all < 500)
        assert!(!results.is_empty());
        assert!(results.len() <= 3); // Max 3 books with price < 500
    }

    /// Test search_filtered returns empty when no matches.
    #[test]
    fn test_search_filtered_no_matches() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert 10 vectors with category "books"
        for i in 0..10 {
            let metadata = create_metadata("books", i * 100);
            index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], metadata)
                .unwrap();
        }

        // Search for non-existent category
        let query = [5.0, 0.0, 0.0, 0.0];
        let results = index
            .search_filtered(&storage, &query, "category = \"electronics\"", 5)
            .unwrap();

        // Should return empty (not an error)
        assert!(results.is_empty());
    }

    /// Test search_filtered on empty index.
    #[test]
    fn test_search_filtered_empty_index() {
        let (index, storage) = create_test_index(4);

        let query = [1.0, 2.0, 3.0, 4.0];
        let results = index
            .search_filtered(&storage, &query, "category = \"books\"", 5)
            .unwrap();

        assert!(results.is_empty());
    }

    /// Test search_filtered returns sorted by distance.
    #[test]
    fn test_search_filtered_sorted_by_distance() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vectors at different distances from query
        for i in 0..10 {
            let metadata = create_metadata("books", 100);
            index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], metadata)
                .unwrap();
        }

        // Query at position 5
        let query = [5.0, 0.0, 0.0, 0.0];
        let results = index
            .search_filtered(&storage, &query, "category = \"books\"", 5)
            .unwrap();

        // Results should be sorted by distance (ascending)
        for i in 1..results.len() {
            assert!(
                results[i].1 >= results[i - 1].1,
                "Results not sorted: {} < {}",
                results[i].1,
                results[i - 1].1
            );
        }
    }

    /// Test search_filtered with OR compound filter (W26.3.2).
    #[test]
    fn test_search_filtered_compound_or() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vectors with 3 different categories
        for i in 0..15 {
            let category = match i % 3 {
                0 => "books",
                1 => "movies",
                _ => "music",
            };
            let metadata = create_metadata(category, i * 10);
            index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], metadata)
                .unwrap();
        }

        // Search for books OR movies
        let query = [7.0, 0.0, 0.0, 0.0];
        let results = index
            .search_filtered(
                &storage,
                &query,
                "category = \"books\" OR category = \"movies\"",
                10,
            )
            .unwrap();

        // Should return 10 results (books: 5 + movies: 5 = 10)
        assert_eq!(results.len(), 10);
    }

    /// Test search_filtered when all vectors match (W26.3.2).
    #[test]
    fn test_search_filtered_all_match() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert 10 vectors, all with same category
        for i in 0..10 {
            let metadata = create_metadata("books", i * 100);
            index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], metadata)
                .unwrap();
        }

        // Search with filter that matches all
        let query = [5.0, 0.0, 0.0, 0.0];
        let results = index
            .search_filtered(&storage, &query, "category = \"books\"", 5)
            .unwrap();

        // Should return exactly k results
        assert_eq!(results.len(), 5);
    }

    /// Test search_filtered on non-existent key (W26.3.2).
    #[test]
    fn test_search_filtered_nonexistent_key() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vectors with "category" key
        for i in 0..10 {
            let metadata = create_metadata("books", i * 100);
            index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], metadata)
                .unwrap();
        }

        // Search with filter on non-existent key
        let query = [5.0, 0.0, 0.0, 0.0];
        let results = index
            .search_filtered(&storage, &query, "nonexistent = \"value\"", 5)
            .unwrap();

        // Should return empty (no vectors have "nonexistent" key)
        assert!(results.is_empty());
    }
}

// =============================================================================
// Error handling tests
// =============================================================================

mod search_filtered_errors {
    use super::*;
    use edgevec::hnsw::GraphError;

    /// Test search_filtered with invalid filter syntax.
    #[test]
    fn test_search_filtered_invalid_syntax() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert a vector
        let metadata = create_metadata("books", 100);
        index
            .insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata)
            .unwrap();

        // Invalid filter syntax
        let query = [1.0, 2.0, 3.0, 4.0];
        let result = index.search_filtered(&storage, &query, "invalid @@@ filter", 5);

        assert!(result.is_err());
        match result {
            Err(GraphError::FilterParse(_)) => {} // Expected
            other => panic!("Expected GraphError::FilterParse, got {:?}", other),
        }
    }
}

// =============================================================================
// Overfetch behavior tests
// =============================================================================

mod search_filtered_overfetch {
    use super::*;

    /// Test that search_filtered returns k results when possible.
    #[test]
    fn test_search_filtered_returns_k() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert 20 vectors, all with matching metadata
        for i in 0..20 {
            let metadata = create_metadata("books", 100);
            index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], metadata)
                .unwrap();
        }

        // Request k=10, all match filter
        let query = [10.0, 0.0, 0.0, 0.0];
        let results = index
            .search_filtered(&storage, &query, "category = \"books\"", 10)
            .unwrap();

        // Should return exactly 10
        assert_eq!(results.len(), 10);
    }

    /// Test that search_filtered handles restrictive filters.
    #[test]
    fn test_search_filtered_restrictive_filter() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert 100 vectors, only 5 match filter
        for i in 0..100 {
            let category = if i < 5 { "rare" } else { "common" };
            let metadata = create_metadata(category, 100);
            index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], metadata)
                .unwrap();
        }

        // Request k=10 but only 5 match
        let query = [2.0, 0.0, 0.0, 0.0];
        let results = index
            .search_filtered(&storage, &query, "category = \"rare\"", 10)
            .unwrap();

        // Should return all 5 matching vectors (less than k)
        assert_eq!(results.len(), 5);
    }
}
