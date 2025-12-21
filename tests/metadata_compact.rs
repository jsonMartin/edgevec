//! Integration tests for compact metadata behavior (W26.2.2)
//!
//! Tests that compact() handles metadata correctly per RFC-002 §2.3.
//!
//! NOTE: The current compact() implementation creates a NEW index and storage.
//! Metadata is NOT preserved during compaction (this is expected behavior).
//! The new index starts with an empty MetadataStore.

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
// compact() metadata behavior tests
// =============================================================================

mod compact_metadata {
    use super::*;

    /// Test that new index after compact has empty metadata.
    ///
    /// NOTE: Metadata is NOT preserved during compact. This is by design
    /// because VectorIds are remapped during compaction.
    #[test]
    fn test_compact_creates_empty_metadata_store() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vectors with metadata
        let mut meta1 = HashMap::new();
        meta1.insert("name".to_string(), MetadataValue::String("first".into()));
        let id1 = index
            .insert_with_metadata(&mut storage, &[1.0, 0.0, 0.0, 0.0], meta1)
            .unwrap();

        let mut meta2 = HashMap::new();
        meta2.insert("name".to_string(), MetadataValue::String("second".into()));
        let _id2 = index
            .insert_with_metadata(&mut storage, &[0.0, 1.0, 0.0, 0.0], meta2)
            .unwrap();

        // Verify both have metadata before compact
        assert_eq!(index.metadata().vector_count(), 2);

        // Delete one vector (this removes its metadata via W26.2.1)
        index.soft_delete(id1).unwrap();
        assert_eq!(index.metadata().vector_count(), 1);

        // Compact
        let (new_index, _new_storage, result) = index.compact(&storage).unwrap();

        assert_eq!(result.tombstones_removed, 1);
        assert_eq!(new_index.node_count(), 1);

        // New index has empty metadata (NOT preserved during compact)
        assert!(new_index.metadata().is_empty());
        assert_eq!(new_index.metadata().vector_count(), 0);
    }

    /// Test that compact with no deletions still has empty metadata in new index.
    #[test]
    fn test_compact_no_deletions_empty_metadata() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vectors with metadata (no deletions)
        let metadata = sample_metadata();
        let _id = index
            .insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata)
            .unwrap();

        assert_eq!(index.metadata().vector_count(), 1);

        // Compact without any deletions
        let (new_index, _new_storage, result) = index.compact(&storage).unwrap();

        assert_eq!(result.tombstones_removed, 0);
        assert_eq!(new_index.node_count(), 1);

        // New index still has empty metadata (NOT preserved)
        assert!(new_index.metadata().is_empty());
    }

    /// Test that original index metadata is unchanged after compact.
    #[test]
    fn test_compact_does_not_modify_original_metadata() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert vector with metadata
        let metadata = sample_metadata();
        let vector_id = index
            .insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata)
            .unwrap();

        // Capture original state
        let original_count = index.metadata().vector_count();
        let has_category = index.metadata().has_key(meta_id(vector_id), "category");

        // Compact
        let (_new_index, _new_storage, _result) = index.compact(&storage).unwrap();

        // Original index metadata is unchanged
        assert_eq!(index.metadata().vector_count(), original_count);
        assert_eq!(
            index.metadata().has_key(meta_id(vector_id), "category"),
            has_category
        );
    }

    /// Test that soft_delete before compact removes metadata (W26.2.1 integration).
    #[test]
    fn test_soft_delete_then_compact_metadata_flow() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert 3 vectors with metadata
        let mut ids = Vec::new();
        for i in 0..3 {
            let mut meta = HashMap::new();
            meta.insert("index".to_string(), MetadataValue::Integer(i64::from(i)));
            let id = index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], meta)
                .unwrap();
            ids.push(id);
        }

        assert_eq!(index.metadata().vector_count(), 3);

        // Delete first two vectors (metadata removed immediately by soft_delete)
        index.soft_delete(ids[0]).unwrap();
        index.soft_delete(ids[1]).unwrap();

        // After soft_delete, metadata count should be 1 (only third vector)
        assert_eq!(index.metadata().vector_count(), 1);
        assert!(!index.metadata().has_key(meta_id(ids[0]), "index"));
        assert!(!index.metadata().has_key(meta_id(ids[1]), "index"));
        assert!(index.metadata().has_key(meta_id(ids[2]), "index"));

        // Compact
        let (new_index, _new_storage, result) = index.compact(&storage).unwrap();

        assert_eq!(result.tombstones_removed, 2);
        assert_eq!(new_index.node_count(), 1);

        // New index has empty metadata
        assert!(new_index.metadata().is_empty());
    }

    /// Test that metadata count always matches non-deleted vector count (W26.3.2).
    #[test]
    fn test_metadata_count_matches_vector_count() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert 5 vectors with metadata
        let mut ids = Vec::new();
        for i in 0..5 {
            let mut meta = HashMap::new();
            meta.insert(
                "name".to_string(),
                MetadataValue::String(format!("vec_{}", i)),
            );
            let id = index
                .insert_with_metadata(&mut storage, &[i as f32, 0.0, 0.0, 0.0], meta)
                .unwrap();
            ids.push(id);
        }

        // Initially, counts should match
        assert_eq!(index.metadata().vector_count(), 5);
        assert_eq!(index.node_count(), 5);

        // Delete 2 vectors
        index.soft_delete(ids[1]).unwrap();
        index.soft_delete(ids[3]).unwrap();

        // Metadata count should now be 3 (5 - 2 deleted)
        assert_eq!(index.metadata().vector_count(), 3);

        // Note: node_count() includes tombstones, but active count is 3
        // Metadata only tracks non-deleted vectors, so it should be 3

        // Verify exactly 3 vectors have metadata
        let mut with_metadata = 0;
        for id in &ids {
            if index.metadata().has_key(meta_id(*id), "name") {
                with_metadata += 1;
            }
        }
        assert_eq!(with_metadata, 3);
    }
}
