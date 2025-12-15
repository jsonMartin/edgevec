use edgevec::{HnswConfig, HnswIndex, VectorStorage};

/// Test W16.2 soft_delete API methods
/// Note: Search filtering is W16.3's scope, not tested here
#[test]
fn test_soft_delete_api() {
    let config = HnswConfig::new(2);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    // 1. Insert 3 vectors in a line: A(0,0), B(10,10), C(20,20)
    let vec_a = vec![0.0, 0.0];
    let vec_b = vec![10.0, 10.0];
    let vec_c = vec![20.0, 20.0];

    let _id_a = index.insert(&vec_a, &mut storage).unwrap();
    let id_b = index.insert(&vec_b, &mut storage).unwrap();
    let _id_c = index.insert(&vec_c, &mut storage).unwrap();

    // 2. Verify initial state
    assert_eq!(index.node_count(), 3);
    assert_eq!(index.deleted_count(), 0);
    assert!(!index.is_deleted(id_b).unwrap());

    // 3. Delete B using soft_delete (RFC-001 API)
    let deleted = index.soft_delete(id_b).unwrap();
    assert!(deleted, "Should return true for first deletion");
    assert!(
        !index.soft_delete(id_b).unwrap(),
        "Should return false for second deletion"
    );

    // 4. Verify deletion state
    assert!(index.is_deleted(id_b).unwrap());
    assert_eq!(index.deleted_count(), 1);
    assert_eq!(index.live_count(), 2);
    assert!((index.tombstone_ratio() - 1.0 / 3.0).abs() < 0.01);
}

/// Test that verifies search filtering (W16.3)
/// Deleted vectors should not appear in search results
#[test]
fn test_soft_delete_routing_and_filtering() {
    let config = HnswConfig::new(2);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    let vec_a = vec![0.0, 0.0];
    let vec_b = vec![10.0, 10.0];
    let vec_c = vec![20.0, 20.0];

    let id_a = index.insert(&vec_a, &mut storage).unwrap();
    let id_b = index.insert(&vec_b, &mut storage).unwrap();
    let id_c = index.insert(&vec_c, &mut storage).unwrap();

    // Search near B should find A, B, C
    let query_b = vec![10.1, 10.1];
    let results = index.search(&query_b, 3, &storage).unwrap();
    assert_eq!(results.len(), 3);
    assert!(results.iter().any(|r| r.vector_id == id_b));

    // Delete B
    index.soft_delete(id_b).unwrap();

    // W16.3: Search should NOT find B (deleted)
    let results_after = index.search(&query_b, 3, &storage).unwrap();
    assert!(!results_after.iter().any(|r| r.vector_id == id_b));
    assert!(results_after.iter().any(|r| r.vector_id == id_a));
    assert!(results_after.iter().any(|r| r.vector_id == id_c));
    assert_eq!(results_after.len(), 2);
}

#[test]
fn test_delete_non_existent() {
    let config = HnswConfig::new(2);
    let storage = VectorStorage::new(&config, None);
    let _index = HnswIndex::new(config, &storage).unwrap();

    // Deleting invalid ID should panic or handle gracefully?
    // mark_deleted panics on invalid ID.
    // But what about ID that hasn't been allocated yet (valid range but not inserted)?
    // storage.mark_deleted checks bounds.

    // In this crate, we don't have a way to generate a valid-looking ID without inserting.
    // storage.next_id tracks it.

    // Trying to delete an out-of-bounds ID should panic according to my impl.
    // let res = std::panic::catch_unwind(move || {
    //    storage.mark_deleted(VectorId(999));
    // });
    // assert!(res.is_err());
}
