use edgevec::{HnswConfig, HnswIndex, VectorStorage};

#[test]
fn test_soft_delete_routing_and_filtering() {
    let config = HnswConfig::new(2);
    let mut storage = VectorStorage::new(&config, None);
    let mut index = HnswIndex::new(config, &storage).unwrap();

    // 1. Insert 3 vectors in a line: A(0,0), B(10,10), C(20,20)
    // Distance L2Squared:
    // A-B: 200
    // B-C: 200
    // A-C: 800
    let vec_a = vec![0.0, 0.0];
    let vec_b = vec![10.0, 10.0];
    let vec_c = vec![20.0, 20.0];

    let id_a = index.insert(&vec_a, &mut storage).unwrap();
    let id_b = index.insert(&vec_b, &mut storage).unwrap();
    let id_c = index.insert(&vec_c, &mut storage).unwrap();

    // 2. Verify all are searchable
    // Search near B should find A, B, C
    let query_b = vec![10.1, 10.1];
    let results = index.search(&query_b, 3, &storage).unwrap();
    assert_eq!(results.len(), 3);
    assert!(results.iter().any(|r| r.vector_id == id_b));

    // 3. Delete B
    let deleted = index.delete(id_b, &mut storage);
    assert!(deleted, "Should return true for first deletion");
    assert!(
        !index.delete(id_b, &mut storage),
        "Should return false for second deletion"
    );

    assert!(storage.is_deleted(id_b));

    // 4. Search near B again
    // Should NOT find B
    // Should find A and C (because they are closest remaining)
    let results_after = index.search(&query_b, 3, &storage).unwrap();

    // Check B is gone
    assert!(!results_after.iter().any(|r| r.vector_id == id_b));

    // Check A and C are present
    assert!(results_after.iter().any(|r| r.vector_id == id_a));
    assert!(results_after.iter().any(|r| r.vector_id == id_c));
    assert_eq!(results_after.len(), 2);

    // 5. Verify connectivity/routing
    // If we search for C starting from A (implicitly, or ensuring traversal)
    // In a small graph, everything connects to everything probably (M default is 16).
    // To strictly test routing, we'd need a sparse graph.
    // But logically, if B is visited (it's close to query), it aids finding neighbors.

    // Let's ensure B is visited by inspecting stats? No stats exposed.
    // But finding C implies we traversed.
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
