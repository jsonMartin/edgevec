//! Integration tests for soft_delete metadata cleanup (W26.2.1)
//!
//! Tests that soft_delete() removes metadata per RFC-002 §2.3.

use std::collections::HashMap;

use edgevec::hnsw::{HnswConfig, HnswIndex, VectorId};
use edgevec::metadata::MetadataValue;
use edgevec::storage::VectorStorage;

/// Helper to create a test index and storage.
fn create_test_index(dim: u32) -> (HnswIndex, VectorStorage) {
    let config = HnswConfig::new(dim);
    let storage = VectorStorage::new(&config, None);
    let index = HnswIndex::new(config, &storage).unwrap();
    (index, storage)
}

/// Helper to convert VectorId (u64) to metadata ID (u32).
#[allow(clippy::cast_possible_truncation)]
fn meta_id(vid: VectorId) -> u32 {
    vid.0 as u32
}

/// Helper to create sample metadata.
fn sample_metadata() -> HashMap<String, MetadataValue> {
    let mut metadata = HashMap::new();
    metadata.insert(
        "category".to_string(),
        MetadataValue::String("books".into()),
    );
    metadata.insert("price".to_string(), MetadataValue::Float(29.99));
    metadata
}

// =============================================================================
// soft_delete metadata cleanup tests
// =============================================================================

mod soft_delete_cleanup {
    use super::*;

    /// Test that soft_delete removes metadata for the vector.
    #[test]
    fn test_soft_delete_removes_metadata() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vector with metadata
        let metadata = sample_metadata();
        let vector_id = index
            .insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata)
            .unwrap();

        // Verify metadata exists before delete
        assert!(index.metadata().has_key(meta_id(vector_id), "category"));
        assert!(index.metadata().has_key(meta_id(vector_id), "price"));
        assert_eq!(index.metadata().vector_count(), 1);

        // Soft delete the vector
        let deleted = index.soft_delete(vector_id).unwrap();
        assert!(deleted);

        // Verify metadata is removed
        assert!(!index.metadata().has_key(meta_id(vector_id), "category"));
        assert!(!index.metadata().has_key(meta_id(vector_id), "price"));
        assert_eq!(index.metadata().vector_count(), 0);
    }

    /// Test soft_delete on vector without metadata (should not error).
    #[test]
    fn test_soft_delete_without_metadata() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vector WITHOUT metadata (using regular insert)
        let vector_id = index.insert(&[1.0, 2.0, 3.0, 4.0], &mut storage).unwrap();

        // Verify no metadata exists
        assert!(index.metadata().is_empty());

        // Soft delete should succeed (no metadata to remove)
        let deleted = index.soft_delete(vector_id).unwrap();
        assert!(deleted);

        // Index should still be valid
        assert!(index.metadata().is_empty());
    }

    /// Test soft_delete of already deleted vector (idempotent, no double-remove).
    #[test]
    fn test_soft_delete_already_deleted_idempotent() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vector with metadata
        let metadata = sample_metadata();
        let vector_id = index
            .insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata)
            .unwrap();

        // First delete
        let deleted1 = index.soft_delete(vector_id).unwrap();
        assert!(deleted1);
        assert_eq!(index.metadata().vector_count(), 0);

        // Second delete should return false (already deleted)
        let deleted2 = index.soft_delete(vector_id).unwrap();
        assert!(!deleted2);

        // Metadata should still be empty (no crash from double-remove)
        assert_eq!(index.metadata().vector_count(), 0);
    }

    /// Test that deleting one vector doesn't affect other vectors' metadata.
    #[test]
    fn test_soft_delete_preserves_other_metadata() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert two vectors with metadata
        let mut meta1 = HashMap::new();
        meta1.insert("name".to_string(), MetadataValue::String("first".into()));
        let id1 = index
            .insert_with_metadata(&mut storage, &[1.0, 0.0, 0.0, 0.0], meta1)
            .unwrap();

        let mut meta2 = HashMap::new();
        meta2.insert("name".to_string(), MetadataValue::String("second".into()));
        let id2 = index
            .insert_with_metadata(&mut storage, &[0.0, 1.0, 0.0, 0.0], meta2)
            .unwrap();

        // Verify both have metadata
        assert_eq!(index.metadata().vector_count(), 2);

        // Delete first vector
        index.soft_delete(id1).unwrap();

        // First vector's metadata should be gone
        assert!(!index.metadata().has_key(meta_id(id1), "name"));

        // Second vector's metadata should be intact
        assert!(index.metadata().has_key(meta_id(id2), "name"));
        assert_eq!(
            index
                .metadata()
                .get(meta_id(id2), "name")
                .unwrap()
                .as_string(),
            Some("second")
        );
        assert_eq!(index.metadata().vector_count(), 1);
    }

    /// Test soft_delete of invalid vector ID returns error (not metadata-related).
    #[test]
    fn test_soft_delete_invalid_id_error() {
        let (mut index, _storage) = create_test_index(4);

        // Try to delete non-existent vector
        let result = index.soft_delete(VectorId(999));

        // Should return error (vector not found)
        assert!(result.is_err());
    }

    /// Test that re-inserting after delete gets fresh metadata (W26.3.2).
    #[test]
    fn test_fresh_metadata_after_reinsert() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vector with metadata
        let mut meta1 = HashMap::new();
        meta1.insert("name".to_string(), MetadataValue::String("original".into()));
        let id1 = index
            .insert_with_metadata(&mut storage, &[1.0, 0.0, 0.0, 0.0], meta1)
            .unwrap();

        // Delete the vector (removes metadata)
        index.soft_delete(id1).unwrap();

        // Insert new vector with different metadata
        let mut meta2 = HashMap::new();
        meta2.insert(
            "name".to_string(),
            MetadataValue::String("new_vector".into()),
        );
        meta2.insert("version".to_string(), MetadataValue::Integer(2));
        let id2 = index
            .insert_with_metadata(&mut storage, &[2.0, 0.0, 0.0, 0.0], meta2)
            .unwrap();

        // Verify new vector has fresh metadata
        assert_eq!(
            index
                .metadata()
                .get(meta_id(id2), "name")
                .unwrap()
                .as_string(),
            Some("new_vector")
        );
        assert!(index.metadata().has_key(meta_id(id2), "version"));

        // Verify old vector's metadata is still gone
        assert!(!index.metadata().has_key(meta_id(id1), "name"));

        // Verify metadata count
        assert_eq!(index.metadata().vector_count(), 1);
    }
}

// =============================================================================
// soft_delete_batch metadata cleanup tests
// =============================================================================

mod soft_delete_batch_cleanup {
    use super::*;

    /// Test that soft_delete_batch removes metadata for all deleted vectors.
    #[test]
    fn test_soft_delete_batch_removes_all_metadata() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert 5 vectors with metadata
        let mut ids = Vec::new();
        for i in 0..5 {
            let mut meta = HashMap::new();
            meta.insert("index".to_string(), MetadataValue::Integer(i64::from(i)));
            let id = index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], meta)
                .unwrap();
            ids.push(id);
        }

        // Verify all have metadata
        assert_eq!(index.metadata().vector_count(), 5);

        // Batch delete first 3 vectors
        let to_delete = vec![ids[0], ids[1], ids[2]];
        let result = index.soft_delete_batch(&to_delete);

        assert_eq!(result.deleted, 3);

        // First 3 should have no metadata
        assert!(!index.metadata().has_key(meta_id(ids[0]), "index"));
        assert!(!index.metadata().has_key(meta_id(ids[1]), "index"));
        assert!(!index.metadata().has_key(meta_id(ids[2]), "index"));

        // Last 2 should still have metadata
        assert!(index.metadata().has_key(meta_id(ids[3]), "index"));
        assert!(index.metadata().has_key(meta_id(ids[4]), "index"));

        assert_eq!(index.metadata().vector_count(), 2);
    }

    /// Test that soft_delete_batch with duplicates only removes metadata once.
    #[test]
    fn test_soft_delete_batch_duplicates_handled() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vector with metadata
        let metadata = sample_metadata();
        let vector_id = index
            .insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata)
            .unwrap();

        // Batch delete with duplicates
        let to_delete = vec![vector_id, vector_id, vector_id];
        let result = index.soft_delete_batch(&to_delete);

        // Should delete once, handle duplicates
        assert_eq!(result.deleted, 1);
        assert_eq!(result.unique_count, 1);

        // Metadata should be gone
        assert_eq!(index.metadata().vector_count(), 0);
    }
}
