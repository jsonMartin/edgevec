//! Integration tests for insert_with_metadata (W26.1.2)
//!
//! Tests atomic vector + metadata insertion with fail-fast validation
//! per RFC-002 §3.1.

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
/// In tests, this is safe as we won't exceed u32::MAX vectors.
#[allow(clippy::cast_possible_truncation)]
fn meta_id(vid: VectorId) -> u32 {
    vid.0 as u32
}

/// Helper to create a valid metadata map.
fn valid_metadata() -> HashMap<String, MetadataValue> {
    let mut metadata = HashMap::new();
    metadata.insert(
        "category".to_string(),
        MetadataValue::String("books".into()),
    );
    metadata.insert("price".to_string(), MetadataValue::Float(29.99));
    metadata.insert("in_stock".to_string(), MetadataValue::Boolean(true));
    metadata
}

// =============================================================================
// Success Path Tests
// =============================================================================

mod success_path {
    use super::*;

    /// Test successful insert with valid metadata.
    #[test]
    fn test_insert_with_valid_metadata() {
        let (mut index, mut storage) = create_test_index(4);
        let metadata = valid_metadata();

        let result =
            index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata.clone());

        assert!(result.is_ok());
        let vector_id = result.unwrap();

        // Verify vector was inserted
        assert_eq!(index.node_count(), 1);

        // Verify metadata was inserted
        assert!(index.metadata().has_key(meta_id(vector_id), "category"));
        assert!(index.metadata().has_key(meta_id(vector_id), "price"));
        assert!(index.metadata().has_key(meta_id(vector_id), "in_stock"));

        // Verify metadata values
        assert_eq!(
            index
                .metadata()
                .get(meta_id(vector_id), "category")
                .unwrap()
                .as_string(),
            Some("books")
        );
        assert_eq!(
            index
                .metadata()
                .get(meta_id(vector_id), "price")
                .unwrap()
                .as_float(),
            Some(29.99)
        );
        assert_eq!(
            index
                .metadata()
                .get(meta_id(vector_id), "in_stock")
                .unwrap()
                .as_boolean(),
            Some(true)
        );
    }

    /// Test insert with empty metadata (should succeed).
    #[test]
    fn test_insert_with_empty_metadata() {
        let (mut index, mut storage) = create_test_index(4);
        let metadata = HashMap::new();

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        assert!(result.is_ok());
        let vector_id = result.unwrap();

        // Vector should be inserted
        assert_eq!(index.node_count(), 1);

        // Metadata should be empty for this vector
        assert!(index.metadata().get_all(meta_id(vector_id)).is_none());
    }

    /// Test insert with all metadata value types.
    #[test]
    fn test_insert_with_all_value_types() {
        let (mut index, mut storage) = create_test_index(4);

        let mut metadata = HashMap::new();
        metadata.insert(
            "string_val".to_string(),
            MetadataValue::String("test".into()),
        );
        metadata.insert("int_val".to_string(), MetadataValue::Integer(42));
        metadata.insert("float_val".to_string(), MetadataValue::Float(123.456));
        metadata.insert("bool_val".to_string(), MetadataValue::Boolean(false));
        metadata.insert(
            "array_val".to_string(),
            MetadataValue::StringArray(vec!["a".into(), "b".into()]),
        );

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        assert!(result.is_ok());
        let vector_id = result.unwrap();

        // Verify all 5 types stored correctly
        assert_eq!(
            index
                .metadata()
                .get(meta_id(vector_id), "string_val")
                .unwrap()
                .as_string(),
            Some("test")
        );
        assert_eq!(
            index
                .metadata()
                .get(meta_id(vector_id), "int_val")
                .unwrap()
                .as_integer(),
            Some(42)
        );
        assert_eq!(
            index
                .metadata()
                .get(meta_id(vector_id), "float_val")
                .unwrap()
                .as_float(),
            Some(123.456)
        );
        assert_eq!(
            index
                .metadata()
                .get(meta_id(vector_id), "bool_val")
                .unwrap()
                .as_boolean(),
            Some(false)
        );
        assert_eq!(
            index
                .metadata()
                .get(meta_id(vector_id), "array_val")
                .unwrap()
                .as_string_array(),
            Some(&["a".to_string(), "b".to_string()][..])
        );
    }

    /// Test multiple inserts with metadata.
    #[test]
    fn test_multiple_inserts_with_metadata() {
        let (mut index, mut storage) = create_test_index(4);

        // Insert first vector
        let mut meta1 = HashMap::new();
        meta1.insert("name".to_string(), MetadataValue::String("first".into()));
        let id1 = index
            .insert_with_metadata(&mut storage, &[1.0, 0.0, 0.0, 0.0], meta1)
            .unwrap();

        // Insert second vector
        let mut meta2 = HashMap::new();
        meta2.insert("name".to_string(), MetadataValue::String("second".into()));
        let id2 = index
            .insert_with_metadata(&mut storage, &[0.0, 1.0, 0.0, 0.0], meta2)
            .unwrap();

        // Verify both stored correctly
        assert_eq!(index.node_count(), 2);
        assert_eq!(index.metadata().vector_count(), 2);

        assert_eq!(
            index
                .metadata()
                .get(meta_id(id1), "name")
                .unwrap()
                .as_string(),
            Some("first")
        );
        assert_eq!(
            index
                .metadata()
                .get(meta_id(id2), "name")
                .unwrap()
                .as_string(),
            Some("second")
        );
    }
}

// =============================================================================
// Failure Path Tests
// =============================================================================

mod failure_path {
    use super::*;

    /// Test failure: too many keys (>64).
    #[test]
    fn test_fails_with_too_many_keys() {
        let (mut index, mut storage) = create_test_index(4);

        // Create metadata with 65 keys (exceeds 64 limit)
        let mut metadata = HashMap::new();
        for i in 0..65 {
            metadata.insert(
                format!("key_{i}"),
                MetadataValue::String(format!("value_{i}")),
            );
        }

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        assert!(result.is_err());

        // Verify no partial state: no vector inserted
        assert_eq!(index.node_count(), 0);
        assert!(index.metadata().is_empty());
    }

    /// Test failure: key name too long (>256 bytes).
    #[test]
    fn test_fails_with_key_too_long() {
        let (mut index, mut storage) = create_test_index(4);

        // Create key with 257 characters (exceeds 256 byte limit)
        let long_key = "a".repeat(257);
        let mut metadata = HashMap::new();
        metadata.insert(long_key, MetadataValue::String("value".into()));

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        assert!(result.is_err());

        // Verify no partial state
        assert_eq!(index.node_count(), 0);
        assert!(index.metadata().is_empty());
    }

    /// Test failure: string value too large (>64KB).
    #[test]
    fn test_fails_with_value_too_large() {
        let (mut index, mut storage) = create_test_index(4);

        // Create string value with 65KB (exceeds 64KB limit)
        let large_value = "x".repeat(65 * 1024 + 1);
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), MetadataValue::String(large_value));

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        assert!(result.is_err());

        // Verify no partial state
        assert_eq!(index.node_count(), 0);
        assert!(index.metadata().is_empty());
    }

    /// Test failure: string array too long (>1024 elements).
    #[test]
    fn test_fails_with_array_too_long() {
        let (mut index, mut storage) = create_test_index(4);

        // Create array with 1025 elements (exceeds 1024 limit)
        let large_array: Vec<String> = (0..1025).map(|i| format!("item_{i}")).collect();
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), MetadataValue::StringArray(large_array));

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        assert!(result.is_err());

        // Verify no partial state
        assert_eq!(index.node_count(), 0);
        assert!(index.metadata().is_empty());
    }

    /// Test failure: invalid key format.
    #[test]
    fn test_fails_with_invalid_key_format() {
        let (mut index, mut storage) = create_test_index(4);

        // Key with invalid characters (contains hyphen)
        let mut metadata = HashMap::new();
        metadata.insert(
            "invalid-key".to_string(),
            MetadataValue::String("value".into()),
        );

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        assert!(result.is_err());

        // Verify no partial state
        assert_eq!(index.node_count(), 0);
        assert!(index.metadata().is_empty());
    }

    /// Test failure: NaN float value.
    #[test]
    fn test_fails_with_nan_float() {
        let (mut index, mut storage) = create_test_index(4);

        let mut metadata = HashMap::new();
        metadata.insert("score".to_string(), MetadataValue::Float(f64::NAN));

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        assert!(result.is_err());

        // Verify no partial state
        assert_eq!(index.node_count(), 0);
        assert!(index.metadata().is_empty());
    }

    /// Test failure: Infinity float value.
    #[test]
    fn test_fails_with_infinity_float() {
        let (mut index, mut storage) = create_test_index(4);

        let mut metadata = HashMap::new();
        metadata.insert("score".to_string(), MetadataValue::Float(f64::INFINITY));

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        assert!(result.is_err());

        // Verify no partial state
        assert_eq!(index.node_count(), 0);
        assert!(index.metadata().is_empty());
    }

    /// Test failure: empty key.
    #[test]
    fn test_fails_with_empty_key() {
        let (mut index, mut storage) = create_test_index(4);

        let mut metadata = HashMap::new();
        metadata.insert(String::new(), MetadataValue::String("value".into()));

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        assert!(result.is_err());

        // Verify no partial state
        assert_eq!(index.node_count(), 0);
        assert!(index.metadata().is_empty());
    }
}

// =============================================================================
// Rollback Verification Tests
// =============================================================================

mod rollback_verification {
    use super::*;

    /// Verify that after a failed insert, the index is in the same state as before.
    #[test]
    fn test_rollback_preserves_existing_state() {
        let (mut index, mut storage) = create_test_index(4);

        // First, insert a valid vector
        let mut meta1 = HashMap::new();
        meta1.insert("name".to_string(), MetadataValue::String("original".into()));
        let id1 = index
            .insert_with_metadata(&mut storage, &[1.0, 0.0, 0.0, 0.0], meta1)
            .unwrap();

        // Capture state
        let node_count_before = index.node_count();
        let metadata_count_before = index.metadata().vector_count();

        // Now try to insert invalid metadata
        let mut invalid_meta = HashMap::new();
        invalid_meta.insert(
            "a".repeat(300), // Key too long
            MetadataValue::String("value".into()),
        );

        let result = index.insert_with_metadata(&mut storage, &[0.0, 1.0, 0.0, 0.0], invalid_meta);

        assert!(result.is_err());

        // Verify state is unchanged
        assert_eq!(index.node_count(), node_count_before);
        assert_eq!(index.metadata().vector_count(), metadata_count_before);

        // Verify original data still intact
        assert_eq!(
            index
                .metadata()
                .get(meta_id(id1), "name")
                .unwrap()
                .as_string(),
            Some("original")
        );
    }

    /// Verify that multiple failed inserts don't corrupt state.
    #[test]
    fn test_multiple_failures_dont_corrupt() {
        let (mut index, mut storage) = create_test_index(4);

        // Try multiple invalid inserts
        for _ in 0..5 {
            let mut bad_meta = HashMap::new();
            bad_meta.insert("".to_string(), MetadataValue::String("value".into())); // Empty key

            let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], bad_meta);

            assert!(result.is_err());
        }

        // Index should still be empty and valid
        assert_eq!(index.node_count(), 0);
        assert!(index.metadata().is_empty());

        // Should still be able to insert valid data
        let valid_meta = valid_metadata();
        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], valid_meta);

        assert!(result.is_ok());
        assert_eq!(index.node_count(), 1);
    }
}

// =============================================================================
// Error Type Tests
// =============================================================================

mod error_types {
    use super::*;
    use edgevec::hnsw::GraphError;

    /// Verify the error is GraphError::MetadataValidation.
    #[test]
    fn test_error_is_metadata_validation() {
        let (mut index, mut storage) = create_test_index(4);

        let mut metadata = HashMap::new();
        metadata.insert("".to_string(), MetadataValue::String("value".into())); // Empty key

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        match result {
            Err(GraphError::MetadataValidation(_)) => {} // Expected
            _ => panic!("Expected GraphError::MetadataValidation"),
        }
    }

    /// Verify error message contains useful information.
    #[test]
    fn test_error_message_is_descriptive() {
        let (mut index, mut storage) = create_test_index(4);

        let mut metadata = HashMap::new();
        metadata.insert("a".repeat(300), MetadataValue::String("value".into()));

        let result = index.insert_with_metadata(&mut storage, &[1.0, 2.0, 3.0, 4.0], metadata);

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("300") || error_msg.contains("too long"));
    }
}
