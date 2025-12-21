//! Integration tests for BinaryVectorStorage.
//!
//! These tests verify the storage layer for binary quantized vectors.

use edgevec::quantization::variable::BinaryVector;
use edgevec::storage::binary::{BinaryStorageError, BinaryVectorStorage};

// ============================================================================
// CONSTRUCTION TESTS
// ============================================================================

mod construction {
    use super::*;

    #[test]
    fn test_new_128d() {
        let storage = BinaryVectorStorage::new(128).unwrap();
        assert_eq!(storage.dimension(), 128);
        assert_eq!(storage.bytes_per_vector(), 16);
        assert!(storage.is_empty());
    }

    #[test]
    fn test_new_384d() {
        let storage = BinaryVectorStorage::new(384).unwrap();
        assert_eq!(storage.dimension(), 384);
        assert_eq!(storage.bytes_per_vector(), 48);
    }

    #[test]
    fn test_new_768d() {
        let storage = BinaryVectorStorage::new(768).unwrap();
        assert_eq!(storage.dimension(), 768);
        assert_eq!(storage.bytes_per_vector(), 96);
    }

    #[test]
    fn test_new_1024d() {
        let storage = BinaryVectorStorage::new(1024).unwrap();
        assert_eq!(storage.dimension(), 1024);
        assert_eq!(storage.bytes_per_vector(), 128);
    }

    #[test]
    fn test_new_1536d() {
        let storage = BinaryVectorStorage::new(1536).unwrap();
        assert_eq!(storage.dimension(), 1536);
        assert_eq!(storage.bytes_per_vector(), 192);
    }

    #[test]
    fn test_new_invalid_dimension() {
        let result = BinaryVectorStorage::new(100);
        assert!(matches!(
            result,
            Err(BinaryStorageError::InvalidDimension { dimension: 100 })
        ));
    }

    #[test]
    fn test_new_zero_dimension() {
        let result = BinaryVectorStorage::new(0);
        assert!(matches!(
            result,
            Err(BinaryStorageError::InvalidDimension { dimension: 0 })
        ));
    }
}

// ============================================================================
// INSERT TESTS
// ============================================================================

mod insert {
    use super::*;

    fn make_vector(dimension: usize, value: f32) -> BinaryVector {
        let v = vec![value; dimension];
        BinaryVector::quantize(&v).unwrap()
    }

    #[test]
    fn test_insert_single() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        let bv = make_vector(768, 1.0);

        let id = storage.insert(&bv).unwrap();
        assert_eq!(id, 0);
        assert_eq!(storage.len(), 1);
        assert_eq!(storage.memory_bytes(), 96);
    }

    #[test]
    fn test_insert_multiple() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();

        for i in 0..100 {
            let bv = make_vector(768, i as f32);
            let id = storage.insert(&bv).unwrap();
            assert_eq!(id, i);
        }

        assert_eq!(storage.len(), 100);
        assert_eq!(storage.memory_bytes(), 100 * 96);
    }

    #[test]
    fn test_insert_dimension_mismatch() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        let bv = make_vector(128, 1.0);

        let result = storage.insert(&bv);
        assert!(matches!(
            result,
            Err(BinaryStorageError::DimensionMismatch {
                expected: 768,
                actual: 128
            })
        ));
    }

    #[test]
    fn test_insert_raw() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let data = vec![0xAA; 16];

        let id = storage.insert_raw(&data).unwrap();
        assert_eq!(id, 0);

        let raw = storage.get_raw(id).unwrap();
        assert_eq!(raw, &data[..]);
    }

    #[test]
    fn test_insert_raw_dimension_mismatch() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let data = vec![0xAA; 32]; // 256 bits instead of 128

        let result = storage.insert_raw(&data);
        assert!(matches!(
            result,
            Err(BinaryStorageError::DimensionMismatch { .. })
        ));
    }

    #[test]
    fn test_insert_preserves_data() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        // Create a specific pattern
        let v: Vec<f32> = (0..128)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let bv = BinaryVector::quantize(&v).unwrap();

        let id = storage.insert(&bv).unwrap();
        let retrieved = storage.get(id).unwrap();

        assert_eq!(retrieved.data(), bv.data());
    }
}

// ============================================================================
// GET TESTS
// ============================================================================

mod get {
    use super::*;

    fn make_vector(dimension: usize, value: f32) -> BinaryVector {
        let v = vec![value; dimension];
        BinaryVector::quantize(&v).unwrap()
    }

    #[test]
    fn test_get_existing() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        let bv = make_vector(768, 1.0);

        let id = storage.insert(&bv).unwrap();
        let retrieved = storage.get(id).unwrap();

        assert_eq!(retrieved.dimension(), 768);
        assert_eq!(retrieved.data(), bv.data());
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = BinaryVectorStorage::new(768).unwrap();
        assert!(storage.get(0).is_none());
        assert!(storage.get(999).is_none());
    }

    #[test]
    fn test_get_deleted() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        let bv = make_vector(768, 1.0);

        let id = storage.insert(&bv).unwrap();
        storage.delete(id).unwrap();

        assert!(storage.get(id).is_none());
    }

    #[test]
    fn test_get_raw_existing() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        let bv = make_vector(768, 1.0);

        let id = storage.insert(&bv).unwrap();
        let raw = storage.get_raw(id).unwrap();

        assert_eq!(raw.len(), 96);
        assert_eq!(raw, bv.data());
    }

    #[test]
    fn test_get_raw_nonexistent() {
        let storage = BinaryVectorStorage::new(768).unwrap();
        assert!(storage.get_raw(0).is_none());
    }

    #[test]
    fn test_get_raw_deleted() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        let bv = make_vector(768, 1.0);

        let id = storage.insert(&bv).unwrap();
        storage.delete(id).unwrap();

        assert!(storage.get_raw(id).is_none());
    }

    #[test]
    fn test_get_multiple_vectors() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        let vectors: Vec<BinaryVector> = (0..10)
            .map(|i| {
                let v: Vec<f32> = (0..128).map(|j| ((i + j) % 2) as f32).collect();
                BinaryVector::quantize(&v).unwrap()
            })
            .collect();

        let ids: Vec<u64> = vectors
            .iter()
            .map(|bv| storage.insert(bv).unwrap())
            .collect();

        for (i, id) in ids.iter().enumerate() {
            let retrieved = storage.get(*id).unwrap();
            assert_eq!(retrieved.data(), vectors[i].data());
        }
    }
}

// ============================================================================
// DELETE TESTS
// ============================================================================

mod delete {
    use super::*;

    fn make_vector(dimension: usize, value: f32) -> BinaryVector {
        let v = vec![value; dimension];
        BinaryVector::quantize(&v).unwrap()
    }

    #[test]
    fn test_delete_existing() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        let bv = make_vector(768, 1.0);

        let id = storage.insert(&bv).unwrap();
        assert!(!storage.is_deleted(id));

        storage.delete(id).unwrap();
        assert!(storage.is_deleted(id));
    }

    #[test]
    fn test_delete_not_found() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();

        let result = storage.delete(0);
        assert!(matches!(
            result,
            Err(BinaryStorageError::NotFound { id: 0 })
        ));
    }

    #[test]
    fn test_delete_already_deleted() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        let bv = make_vector(768, 1.0);

        let id = storage.insert(&bv).unwrap();
        storage.delete(id).unwrap();

        let result = storage.delete(id);
        assert!(matches!(
            result,
            Err(BinaryStorageError::AlreadyDeleted { id: 0 })
        ));
    }

    #[test]
    fn test_delete_preserves_other_vectors() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        let bv1 = make_vector(128, 1.0);
        let bv2 = make_vector(128, -1.0);
        let bv3 = make_vector(128, 0.5);

        let id1 = storage.insert(&bv1).unwrap();
        let id2 = storage.insert(&bv2).unwrap();
        let id3 = storage.insert(&bv3).unwrap();

        storage.delete(id2).unwrap();

        assert!(storage.get(id1).is_some());
        assert!(storage.get(id2).is_none());
        assert!(storage.get(id3).is_some());
    }

    #[test]
    fn test_is_deleted_out_of_bounds() {
        let storage = BinaryVectorStorage::new(768).unwrap();
        assert!(!storage.is_deleted(999));
    }
}

// ============================================================================
// TOMBSTONE TESTS
// ============================================================================

mod tombstones {
    use super::*;

    fn make_vector(dimension: usize, value: f32) -> BinaryVector {
        let v = vec![value; dimension];
        BinaryVector::quantize(&v).unwrap()
    }

    #[test]
    fn test_live_count_after_delete() {
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
    fn test_iter_live_skips_deleted() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..5 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        storage.delete(1).unwrap();
        storage.delete(3).unwrap();

        let live_ids: Vec<u64> = storage.iter_live().map(|(id, _)| id).collect();
        assert_eq!(live_ids, vec![0, 2, 4]);
    }

    #[test]
    fn test_compaction_ratio() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..10 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        storage.delete(0).unwrap();
        storage.delete(5).unwrap();

        // 8/10 = 0.8
        assert!((storage.compaction_ratio() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_compaction_ratio_empty() {
        let storage = BinaryVectorStorage::new(128).unwrap();
        assert!((storage.compaction_ratio() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_compaction_ratio_all_deleted() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..5 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        for i in 0..5 {
            storage.delete(i).unwrap();
        }

        assert!((storage.compaction_ratio() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_iter_live_empty() {
        let storage = BinaryVectorStorage::new(128).unwrap();
        let count = storage.iter_live().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_iter_all() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..5 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        storage.delete(1).unwrap();
        storage.delete(3).unwrap();

        let all: Vec<(u64, bool)> = storage.iter_all().map(|(id, _, del)| (id, del)).collect();
        assert_eq!(
            all,
            vec![(0, false), (1, true), (2, false), (3, true), (4, false)]
        );
    }
}

// ============================================================================
// MEMORY TESTS
// ============================================================================

mod memory {
    use super::*;

    fn make_vector(dimension: usize, value: f32) -> BinaryVector {
        let v = vec![value; dimension];
        BinaryVector::quantize(&v).unwrap()
    }

    #[test]
    fn test_memory_bytes_empty() {
        let storage = BinaryVectorStorage::new(768).unwrap();
        assert_eq!(storage.memory_bytes(), 0);
    }

    #[test]
    fn test_memory_bytes_single() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        let bv = make_vector(768, 1.0);
        storage.insert(&bv).unwrap();

        assert_eq!(storage.memory_bytes(), 96); // 768 / 8
    }

    #[test]
    fn test_memory_bytes_multiple() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();

        for i in 0..100 {
            let bv = make_vector(768, i as f32);
            storage.insert(&bv).unwrap();
        }

        assert_eq!(storage.memory_bytes(), 100 * 96);
    }

    #[test]
    fn test_memory_bytes_after_delete() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();

        for i in 0..10 {
            let bv = make_vector(768, i as f32);
            storage.insert(&bv).unwrap();
        }

        storage.delete(5).unwrap();

        // Deletion doesn't reduce memory (soft delete)
        assert_eq!(storage.memory_bytes(), 10 * 96);
    }

    #[test]
    fn test_reserve_capacity() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        storage.reserve(100);
        assert!(storage.capacity() >= 100);
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut storage = BinaryVectorStorage::new(768).unwrap();
        storage.reserve(1000);

        let bv = make_vector(768, 1.0);
        storage.insert(&bv).unwrap();

        let before = storage.capacity();
        storage.shrink_to_fit();
        let after = storage.capacity();

        assert!(after <= before);
    }
}

// ============================================================================
// PROPERTY TESTS
// ============================================================================

mod properties {
    use super::*;

    fn make_vector(dimension: usize, value: f32) -> BinaryVector {
        let v = vec![value; dimension];
        BinaryVector::quantize(&v).unwrap()
    }

    #[test]
    fn test_insert_increases_len() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..10 {
            let bv = make_vector(128, i as f32);
            let before = storage.len();
            storage.insert(&bv).unwrap();
            assert_eq!(storage.len(), before + 1);
        }
    }

    #[test]
    fn test_delete_decreases_live_count() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..10 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        for i in 0..5 {
            let before = storage.live_count();
            storage.delete(i).unwrap();
            assert_eq!(storage.live_count(), before - 1);
        }
    }

    #[test]
    fn test_ids_are_sequential() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..100 {
            let bv = make_vector(128, 1.0);
            let id = storage.insert(&bv).unwrap();
            assert_eq!(id, i);
        }
    }

    #[test]
    fn test_live_plus_deleted_equals_len() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..20 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        // Delete some
        storage.delete(3).unwrap();
        storage.delete(7).unwrap();
        storage.delete(15).unwrap();

        assert_eq!(
            storage.live_count() + storage.deleted_count(),
            storage.len()
        );
    }

    #[test]
    fn test_get_roundtrip() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        // Create vectors with different patterns
        for i in 0..10 {
            let v: Vec<f32> = (0..128)
                .map(|j| {
                    let val = (i * 13 + j * 7) % 17;
                    if val < 8 {
                        1.0
                    } else {
                        -1.0
                    }
                })
                .collect();
            let bv = BinaryVector::quantize(&v).unwrap();
            let id = storage.insert(&bv).unwrap();
            let retrieved = storage.get(id).unwrap();
            assert_eq!(retrieved.data(), bv.data());
        }
    }
}

// ============================================================================
// CLONE TESTS
// ============================================================================

mod clone {
    use super::*;

    fn make_vector(dimension: usize, value: f32) -> BinaryVector {
        let v = vec![value; dimension];
        BinaryVector::quantize(&v).unwrap()
    }

    #[test]
    fn test_clone_empty() {
        let storage = BinaryVectorStorage::new(768).unwrap();
        let cloned = storage.clone();

        assert_eq!(cloned.dimension(), storage.dimension());
        assert_eq!(cloned.len(), storage.len());
        assert!(cloned.is_empty());
    }

    #[test]
    fn test_clone_with_data() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();

        for i in 0..5 {
            let bv = make_vector(128, i as f32);
            storage.insert(&bv).unwrap();
        }

        storage.delete(2).unwrap();

        let cloned = storage.clone();

        assert_eq!(cloned.len(), storage.len());
        assert_eq!(cloned.live_count(), storage.live_count());
        assert_eq!(cloned.deleted_count(), storage.deleted_count());

        for i in 0..5 {
            assert_eq!(cloned.get_raw(i), storage.get_raw(i));
        }
    }

    #[test]
    fn test_clone_independence() {
        let mut storage = BinaryVectorStorage::new(128).unwrap();
        let bv = make_vector(128, 1.0);
        storage.insert(&bv).unwrap();

        let mut cloned = storage.clone();
        cloned.delete(0).unwrap();

        // Original should be unaffected
        assert!(storage.get(0).is_some());
        assert!(cloned.get(0).is_none());
    }
}
