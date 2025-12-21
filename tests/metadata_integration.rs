//! Integration tests for HnswIndex metadata field integration (W26.1.1)
//!
//! Tests the integration of MetadataStore into HnswIndex per RFC-002 §2.1.

use edgevec::hnsw::{HnswConfig, HnswIndex};
use edgevec::metadata::{MetadataStore, MetadataValue};
use edgevec::storage::VectorStorage;

/// Helper to create a test index and storage.
fn create_test_index(dim: u32) -> (HnswIndex, VectorStorage) {
    let config = HnswConfig::new(dim);
    let storage = VectorStorage::new(&config, None);
    let index = HnswIndex::new(config, &storage).unwrap();
    (index, storage)
}

// =============================================================================
// W26.1.1: Metadata Field Integration Tests
// =============================================================================

mod metadata_field_integration {
    use super::*;

    /// Test that HnswIndex::new() initializes with empty MetadataStore.
    #[test]
    fn test_new_initializes_empty_metadata() {
        let (index, _storage) = create_test_index(4);

        // Verify metadata store exists and is empty
        assert!(index.metadata().is_empty());
        assert_eq!(index.metadata().vector_count(), 0);
    }

    /// Test that HnswIndex::with_config() (via new()) initializes empty MetadataStore.
    #[test]
    fn test_with_config_initializes_empty_metadata() {
        let config = HnswConfig::new(4);
        let storage = VectorStorage::new(&config, None);
        let index = HnswIndex::new(config, &storage).unwrap();

        assert!(index.metadata().is_empty());
    }

    /// Test that HnswIndex::with_metadata() constructor works.
    #[test]
    fn test_with_metadata_constructor() {
        let config = HnswConfig::new(4);
        let storage = VectorStorage::new(&config, None);

        // Create a pre-populated MetadataStore
        let mut metadata = MetadataStore::new();
        metadata
            .insert(1, "category", MetadataValue::String("books".to_string()))
            .unwrap();
        metadata
            .insert(2, "price", MetadataValue::Float(29.99))
            .unwrap();

        // Create index with pre-populated metadata
        let index = HnswIndex::with_metadata(config, &storage, metadata).unwrap();

        // Verify metadata is preserved
        assert!(!index.metadata().is_empty());
        assert_eq!(index.metadata().vector_count(), 2);
        assert!(index.metadata().has_key(1, "category"));
        assert!(index.metadata().has_key(2, "price"));
    }

    /// Test that metadata is accessible via immutable reference.
    #[test]
    fn test_metadata_getter_returns_immutable_ref() {
        let (index, _storage) = create_test_index(4);

        let metadata_ref: &MetadataStore = index.metadata();
        assert!(metadata_ref.is_empty());
    }

    /// Test that metadata is accessible via mutable reference.
    #[test]
    fn test_metadata_mut_getter_returns_mutable_ref() {
        let (mut index, _storage) = create_test_index(4);

        // Get mutable reference and modify
        index
            .metadata_mut()
            .insert(1, "key", MetadataValue::String("value".to_string()))
            .unwrap();

        // Verify modification persisted
        assert!(!index.metadata().is_empty());
        assert!(index.metadata().has_key(1, "key"));
    }

    /// Test that existing tests still pass (no breaking changes).
    #[test]
    fn test_existing_insert_still_works() {
        let (mut index, mut storage) = create_test_index(4);

        // Standard insert should still work
        let id = index.insert(&[1.0, 2.0, 3.0, 4.0], &mut storage).unwrap();

        assert_eq!(index.node_count(), 1);
        assert_eq!(id.0, 1);

        // Metadata should still be empty (insert doesn't add metadata)
        assert!(index.metadata().is_empty());
    }

    /// Test that search still works with metadata field present.
    #[test]
    fn test_search_still_works_with_metadata() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert some vectors
        for i in 0..10 {
            index
                .insert(&[i as f32, 0.0, 0.0, 0.0], &mut storage)
                .unwrap();
        }

        // Search should still work
        let results = index.search(&[5.0, 0.0, 0.0, 0.0], 3, &storage).unwrap();

        assert!(!results.is_empty());
        assert!(results.len() <= 3);
    }

    /// Test that soft_delete still works with metadata field present.
    #[test]
    fn test_soft_delete_still_works_with_metadata() {
        let (mut index, mut storage) = create_test_index(4);

        let id = index.insert(&[1.0, 2.0, 3.0, 4.0], &mut storage).unwrap();

        // Soft delete should still work
        assert!(index.soft_delete(id).unwrap());
        assert!(index.is_deleted(id).unwrap());
    }

    /// Test that compact still works with metadata field present.
    #[test]
    fn test_compact_still_works_with_metadata() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert and delete some vectors
        let id1 = index.insert(&[1.0, 0.0, 0.0, 0.0], &mut storage).unwrap();
        let _id2 = index.insert(&[0.0, 1.0, 0.0, 0.0], &mut storage).unwrap();

        index.soft_delete(id1).unwrap();

        // Compact should still work
        let (new_index, new_storage, result) = index.compact(&storage).unwrap();

        assert_eq!(result.tombstones_removed, 1);
        assert_eq!(new_index.node_count(), 1);
        assert!(new_index.metadata().is_empty()); // Metadata is NOT preserved during compact (expected)

        // Ensure we can use the new index
        let search_results = new_index
            .search(&[0.0, 1.0, 0.0, 0.0], 1, &new_storage)
            .unwrap();
        assert!(!search_results.is_empty());
    }
}

// =============================================================================
// Thread Safety Tests
// =============================================================================

mod thread_safety {
    use super::*;

    /// Test that HnswIndex with metadata is Send.
    #[test]
    fn test_hnsw_with_metadata_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<HnswIndex>();
    }

    /// Test that HnswIndex with metadata is Sync.
    #[test]
    fn test_hnsw_with_metadata_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<HnswIndex>();
    }
}

// =============================================================================
// Serialization Tests
// =============================================================================

mod serialization {
    use super::*;

    /// Test that HnswIndex with metadata can be serialized/deserialized.
    #[test]
    fn test_serialization_roundtrip() {
        let (mut index, _storage) = create_test_index(4);

        // Add some metadata
        index
            .metadata_mut()
            .insert(1, "key", MetadataValue::String("value".to_string()))
            .unwrap();

        // Serialize
        let serialized = serde_json::to_string(&index).unwrap();

        // Deserialize
        let deserialized: HnswIndex = serde_json::from_str(&serialized).unwrap();

        // Verify metadata preserved
        assert!(!deserialized.metadata().is_empty());
        assert!(deserialized.metadata().has_key(1, "key"));
        assert_eq!(
            deserialized.metadata().get(1, "key").unwrap().as_string(),
            Some("value")
        );
    }

    /// Test that empty metadata serializes correctly.
    #[test]
    fn test_empty_metadata_serialization() {
        let (index, _storage) = create_test_index(4);

        // Serialize
        let serialized = serde_json::to_string(&index).unwrap();

        // Deserialize
        let deserialized: HnswIndex = serde_json::from_str(&serialized).unwrap();

        // Verify still empty
        assert!(deserialized.metadata().is_empty());
    }
}
