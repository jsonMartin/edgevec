use edgevec::hnsw::{HnswConfig, HnswIndex};
use edgevec::storage::VectorStorage;
use proptest::prelude::*;
use std::collections::HashSet;

// Helper to create index and storage
fn create_env(dim: u32) -> (HnswIndex, VectorStorage) {
    let config = HnswConfig::new(dim);
    let storage = VectorStorage::new(&config, None);
    let index = HnswIndex::new(config, &storage).unwrap();
    (index, storage)
}

#[test]
fn test_ghost_routing_manual_construction() {
    // Scenario 2: A -> B -> C
    // Delete B. Search for C from A.
    // Coordinates: A(0,0), B(10,0), C(20,0)

    let dim = 2;
    let (mut index, mut storage) = create_env(dim);

    let vec_a = vec![0.0, 0.0];
    let vec_b = vec![10.0, 0.0];
    let vec_c = vec![20.0, 0.0];

    // Insert into storage manually to get IDs
    let id_a = storage.insert(&vec_a).unwrap(); // 1
    let id_b = storage.insert(&vec_b).unwrap(); // 2
    let id_c = storage.insert(&vec_c).unwrap(); // 3

    // Manually add nodes to graph at layer 0
    let node_a = index.add_node(id_a, 0).unwrap(); // NodeId(0)
    let node_b = index.add_node(id_b, 0).unwrap(); // NodeId(1)
    let node_c = index.add_node(id_c, 0).unwrap(); // NodeId(2)

    // Force linear connections: A <-> B <-> C
    // A connects to B
    index.set_neighbors(node_a, &[node_b]).unwrap();
    // B connects to A and C
    index.set_neighbors(node_b, &[node_a, node_c]).unwrap();
    // C connects to B
    index.set_neighbors(node_c, &[node_b]).unwrap();

    // Set entry point to A
    index.set_entry_point(node_a);

    // Verify reachability before delete
    let query = vec![20.0, 0.0]; // Exact match C
                                 // search should find C
    let results = index.search(&query, 5, &storage).unwrap();
    assert!(
        results.iter().any(|r| r.vector_id == id_c),
        "Should find C before delete"
    );

    // Delete B
    index.delete(id_b, &mut storage);
    assert!(storage.is_deleted(id_b));

    // Search for C again
    let results_after = index.search(&query, 5, &storage).unwrap();

    // Assertions
    // 1. C should still be found (routing via B ghost worked)
    assert!(
        results_after.iter().any(|r| r.vector_id == id_c),
        "Should still find C via ghost B"
    );

    // 2. B should NOT be found (filtering worked)
    assert!(
        !results_after.iter().any(|r| r.vector_id == id_b),
        "Should NOT find B (filtered)"
    );

    // 3. A should be found (it's reachable and not deleted)
    assert!(
        results_after.iter().any(|r| r.vector_id == id_a),
        "Should find A"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_soft_delete_recall(
        vectors in prop::collection::vec(prop::collection::vec(-10.0f32..10.0, 4), 10..50),
        delete_indices in prop::collection::vec(any::<usize>(), 1..5)
    ) {
        let dim = 4;
        let (mut index, mut storage) = create_env(dim);

        // Insert all
        let mut ids = Vec::new();
        for vec in &vectors {
             let id = index.insert(vec, &mut storage).unwrap();
             ids.push(id);
        }

        // Determine which to delete
        let mut deleted_ids = HashSet::new();
        for &idx in &delete_indices {
            if ids.is_empty() { break; }
            let i = idx % ids.len();
            let id = ids[i];
            if index.delete(id, &mut storage) {
                deleted_ids.insert(id);
            }
        }

        // Verification
        for (i, vec) in vectors.iter().enumerate() {
            let id = ids[i];

            // Search for exact vector
            let results = index.search(vec, 10, &storage).unwrap();

            if deleted_ids.contains(&id) {
                // Should not be in results
                assert!(!results.iter().any(|r| r.vector_id == id), "Deleted ID {} found in results", id.0);
            } else {
                // Should be in results (top 1 usually, since exact match)
                assert!(results.iter().any(|r| r.vector_id == id), "Active ID {} not found in results", id.0);
            }
        }
    }
}

#[test]
fn test_pathological_delete() {
    // Scenario: High delete ratio (99%).
    // Ensure search doesn't hang and returns quickly.
    let dim = 4;
    let (mut index, mut storage) = create_env(dim);

    // 1. Insert 100 vectors
    let count = 100;
    let mut ids = Vec::new();
    for i in 0..count {
        let val = i as f32;
        let vec = vec![val, 0.0, 0.0, 0.0];
        let id = index.insert(&vec, &mut storage).unwrap();
        ids.push(id);
    }

    // 2. Delete 99 (keep the last one)
    for i in 0..count - 1 {
        index.delete(ids[i], &mut storage);
    }

    // 3. Search for the survivor (last one)
    let survivor_vec = vec![(count - 1) as f32, 0.0, 0.0, 0.0];

    // Use a timer to ensure "quickly" (though standard test timeout handles hangs)
    let start = std::time::Instant::now();
    let results = index.search(&survivor_vec, 5, &storage).unwrap();
    let duration = start.elapsed();

    // 4. Assertions
    // It should find the survivor (since it's reachable and valid)
    // OR it might miss it if the traversal limit cuts it off (but with 100 nodes and ef=20*10=200, it should be fine).
    // The requirement is "returns ... quickly".

    // 500ms is generous for 100 nodes, but ensures no infinite loop
    assert!(
        duration.as_millis() < 500,
        "Search took too long: {:?}",
        duration
    );

    // Ideally it finds the result
    assert!(!results.is_empty(), "Should find the survivor");
    assert_eq!(results[0].vector_id, ids[count - 1]);
}
