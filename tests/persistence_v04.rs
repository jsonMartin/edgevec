//! Integration tests for v0.4 persistence format (W26.5).
//!
//! Tests the metadata section in snapshot files per RFC-002 Persistence Format.

use edgevec::hnsw::{HnswConfig, HnswIndex};
use edgevec::metadata::MetadataValue;
use edgevec::persistence::{read_snapshot, write_snapshot, Flags, MemoryBackend, StorageBackend};
use edgevec::storage::VectorStorage;
use std::collections::HashMap;

// =============================================================================
// Helper functions
// =============================================================================

/// Creates a test index with vectors but no metadata.
fn create_test_index_no_metadata(dim: u32, count: usize) -> (HnswIndex, VectorStorage) {
    let config = HnswConfig::new(dim);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).expect("Failed to create index");

    for i in 0..count {
        #[allow(clippy::cast_precision_loss)]
        let vec: Vec<f32> = (0..dim)
            .map(|d| (i * dim as usize + d as usize) as f32)
            .collect();
        index.insert(&vec, &mut storage).expect("Failed to insert");
    }

    (index, storage)
}

/// Creates a test index with vectors and metadata.
fn create_test_index_with_metadata(dim: u32, count: usize) -> (HnswIndex, VectorStorage) {
    let config = HnswConfig::new(dim);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).expect("Failed to create index");

    for i in 0..count {
        #[allow(clippy::cast_precision_loss)]
        let vec: Vec<f32> = (0..dim)
            .map(|d| (i * dim as usize + d as usize) as f32)
            .collect();

        let mut metadata = HashMap::new();
        metadata.insert(
            "name".to_string(),
            MetadataValue::String(format!("vector_{i}")),
        );
        metadata.insert("index".to_string(), MetadataValue::Integer(i as i64));
        metadata.insert("score".to_string(), MetadataValue::Float(i as f64 * 0.1));

        index
            .insert_with_metadata(&mut storage, &vec, metadata)
            .expect("Failed to insert with metadata");
    }

    (index, storage)
}

// =============================================================================
// v0.4 format write tests
// =============================================================================

mod write_v04 {
    use super::*;

    #[test]
    fn test_write_v04_without_metadata_no_flag() {
        let (index, storage) = create_test_index_no_metadata(4, 10);

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        // Read data to verify
        let data = backend.read().expect("Failed to read backend");
        assert!(data.len() >= 64, "Snapshot too small");

        // Check version is 0.4
        assert_eq!(data[4], 0, "version_major should be 0");
        assert_eq!(data[5], 4, "version_minor should be 4");

        // Check HAS_METADATA flag is NOT set
        let flags = u16::from_le_bytes([data[6], data[7]]);
        assert_eq!(
            flags & Flags::HAS_METADATA,
            0,
            "HAS_METADATA should not be set"
        );
    }

    #[test]
    fn test_write_v04_with_metadata_sets_flag() {
        let (index, storage) = create_test_index_with_metadata(4, 5);

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        // Read data to verify
        let data = backend.read().expect("Failed to read backend");
        assert!(data.len() >= 64, "Snapshot too small");

        // Check version is 0.4
        assert_eq!(data[4], 0, "version_major should be 0");
        assert_eq!(data[5], 4, "version_minor should be 4");

        // Check HAS_METADATA flag IS set
        let flags = u16::from_le_bytes([data[6], data[7]]);
        assert_ne!(flags & Flags::HAS_METADATA, 0, "HAS_METADATA should be set");
    }

    #[test]
    fn test_write_v04_metadata_section_exists() {
        let (index, storage) = create_test_index_with_metadata(4, 3);

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        let data = backend.read().expect("Failed to read backend");

        // Find the metadata section (after tombstones)
        // The metadata section should contain "META" magic bytes
        let meta_magic = b"META";
        let found = data.windows(4).any(|w| w == meta_magic);

        assert!(found, "Metadata section magic 'META' not found in snapshot");
    }
}

// =============================================================================
// v0.4 format read tests
// =============================================================================

mod read_v04 {
    use super::*;

    #[test]
    fn test_read_v04_without_metadata() {
        let (index, storage) = create_test_index_no_metadata(4, 10);

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        // Read back
        let (loaded_index, loaded_storage) =
            read_snapshot(&backend).expect("Failed to read snapshot");

        assert_eq!(loaded_storage.len(), 10);
        assert_eq!(loaded_index.len(), 10);
        assert!(
            loaded_index.metadata().is_empty(),
            "Metadata should be empty"
        );
    }

    #[test]
    fn test_read_v04_with_metadata() {
        let (index, storage) = create_test_index_with_metadata(4, 5);

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        // Read back
        let (loaded_index, loaded_storage) =
            read_snapshot(&backend).expect("Failed to read snapshot");

        assert_eq!(loaded_storage.len(), 5);
        assert_eq!(loaded_index.len(), 5);
        assert!(
            !loaded_index.metadata().is_empty(),
            "Metadata should not be empty"
        );
        assert_eq!(loaded_index.metadata().vector_count(), 5);
    }

    #[test]
    fn test_read_v04_metadata_values_preserved() {
        let (index, storage) = create_test_index_with_metadata(4, 3);

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        // Read back
        let (loaded_index, _) = read_snapshot(&backend).expect("Failed to read snapshot");

        // Check specific metadata values
        // Note: VectorId starts at 1, not 0. First insert (i=0) -> VectorId(1).
        // So metadata ID 1 has name="vector_0", index=0, score=0.0
        let meta1 = loaded_index
            .metadata()
            .get_all(1)
            .expect("Metadata for vector_id=1 (first vector)");
        assert_eq!(
            meta1.get("name"),
            Some(&MetadataValue::String("vector_0".into()))
        );
        assert_eq!(meta1.get("index"), Some(&MetadataValue::Integer(0)));
        assert_eq!(meta1.get("score"), Some(&MetadataValue::Float(0.0)));

        // VectorId(3) has name="vector_2", index=2, score=0.2
        let meta3 = loaded_index
            .metadata()
            .get_all(3)
            .expect("Metadata for vector_id=3 (third vector)");
        assert_eq!(
            meta3.get("name"),
            Some(&MetadataValue::String("vector_2".into()))
        );
        assert_eq!(meta3.get("index"), Some(&MetadataValue::Integer(2)));
    }
}

// =============================================================================
// Round-trip tests
// =============================================================================

mod roundtrip {
    use super::*;

    #[test]
    fn test_roundtrip_empty_index() {
        let config = HnswConfig::new(128);
        let storage = VectorStorage::new(&config, None);
        let index = HnswIndex::new(config, &storage).expect("Failed to create index");

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        let (loaded_index, loaded_storage) =
            read_snapshot(&backend).expect("Failed to read snapshot");

        assert_eq!(loaded_storage.len(), 0);
        assert_eq!(loaded_index.len(), 0);
        assert!(loaded_index.metadata().is_empty());
    }

    #[test]
    fn test_roundtrip_large_with_metadata() {
        let (index, storage) = create_test_index_with_metadata(32, 100);

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        let (loaded_index, loaded_storage) =
            read_snapshot(&backend).expect("Failed to read snapshot");

        assert_eq!(loaded_storage.len(), 100);
        assert_eq!(loaded_index.len(), 100);
        assert_eq!(loaded_index.metadata().vector_count(), 100);
        assert_eq!(loaded_index.metadata().total_key_count(), 300); // 3 keys per vector
    }

    #[test]
    fn test_roundtrip_mixed_metadata() {
        // Create index with some vectors having metadata, some not
        let config = HnswConfig::new(8);
        let mut storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::new(config, &storage).expect("Failed to create index");

        // Insert 5 vectors with metadata
        for i in 0..5 {
            let vec: Vec<f32> = (0..8).map(|d| (i * 8 + d) as f32).collect();
            let mut metadata = HashMap::new();
            metadata.insert("has_meta".to_string(), MetadataValue::Boolean(true));
            index
                .insert_with_metadata(&mut storage, &vec, metadata)
                .unwrap();
        }

        // Insert 5 vectors without metadata
        for i in 5..10 {
            let vec: Vec<f32> = (0..8).map(|d| (i * 8 + d) as f32).collect();
            index.insert(&vec, &mut storage).unwrap();
        }

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        let (loaded_index, loaded_storage) =
            read_snapshot(&backend).expect("Failed to read snapshot");

        assert_eq!(loaded_storage.len(), 10);
        assert_eq!(loaded_index.len(), 10);
        assert_eq!(loaded_index.metadata().vector_count(), 5); // Only 5 have metadata

        // Verify metadata preserved for first 5 (VectorIds 1-5)
        // Note: VectorId starts at 1, not 0
        for id in 1..=5 {
            assert!(
                loaded_index.metadata().get(id, "has_meta").is_some(),
                "VectorId {id} should have metadata"
            );
        }

        // Verify no metadata for last 5 (VectorIds 6-10)
        for id in 6..=10 {
            assert!(
                loaded_index.metadata().get(id, "has_meta").is_none(),
                "VectorId {id} should not have metadata"
            );
        }
    }
}

// =============================================================================
// Error handling tests
// =============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn test_corrupted_metadata_crc_detected() {
        let (index, storage) = create_test_index_with_metadata(4, 3);

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        // Get data, corrupt it, and create new backend
        let mut data = backend.read().expect("Failed to read backend");
        if !data.is_empty() {
            let last = data.len() - 1;
            data[last] ^= 0xFF;
        }

        // Create new backend with corrupted data
        let corrupted_backend = MemoryBackend::new();
        corrupted_backend
            .atomic_write("", &data)
            .expect("Failed to write corrupted data");

        // Attempt to read - should fail with CRC error
        let result = read_snapshot(&corrupted_backend);
        assert!(result.is_err(), "Should fail to read corrupted snapshot");

        let err = result.err().unwrap();
        let err_str = err.to_string();
        assert!(
            err_str.contains("CRC")
                || err_str.contains("crc")
                || err_str.contains("Corrupted")
                || err_str.contains("checksum"),
            "Error should mention CRC, checksum, or corruption: {err_str}"
        );
    }
}

// =============================================================================
// Version compatibility tests
// =============================================================================

mod version_compatibility {
    use super::*;

    #[test]
    fn test_v04_header_version() {
        let (index, storage) = create_test_index_no_metadata(4, 5);

        let mut backend = MemoryBackend::new();
        write_snapshot(&index, &storage, &mut backend).expect("Failed to write snapshot");

        let data = backend.read().expect("Failed to read backend");

        // Check magic: "EVEC"
        assert_eq!(&data[0..4], b"EVEC", "Magic should be EVEC");

        // Check version
        assert_eq!(data[4], 0, "version_major should be 0");
        assert_eq!(data[5], 4, "version_minor should be 4");
    }
}
