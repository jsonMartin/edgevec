use edgevec::hnsw::{HnswConfig, HnswIndex, NodeId};
use edgevec::storage::VectorStorage;
use proptest::prelude::*;
use std::collections::HashSet;

// Helper to create index and storage
fn create_env(dim: u32, m: u32, ef: u32) -> (HnswIndex, VectorStorage) {
    let mut config = HnswConfig::new(dim);
    config.m = m;
    config.m0 = m * 2;
    config.ef_construction = ef;
    let storage = VectorStorage::new(&config, None);
    let index = HnswIndex::new(config, &storage).unwrap();
    (index, storage)
}

proptest! {
    // Only run a moderate number of cases as graph building is expensive
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_graph_connectivity_and_integrity(
        // Generate a list of vectors.
        // Dim = 4 for speed. N = 10..100.
        vectors in prop::collection::vec(prop::collection::vec(-10.0f32..10.0, 4), 10..100),
        m in 4u32..16,
        ef in 30u32..100, // Higher ef to ensure high recall
    ) {
        let (mut index, mut storage) = create_env(4, m, ef);
        let mut inserted_ids = Vec::new();

        // 1. Insert N vectors
        for vec in &vectors {
            let id = index.insert(vec, &mut storage).unwrap();
            inserted_ids.push(id);
        }

        // Check 1: Node Count
        prop_assert_eq!(index.node_count(), vectors.len());
        prop_assert!(index.entry_point().is_some());

        // Check 2: Max Neighbors Constraint
        for i in 0..index.node_count() {
            let node_id = NodeId(i as u32);
            let node = index.get_node(node_id).unwrap();

            // Verify max layer constraint
            prop_assert!(node.max_layer <= index.max_layer());

            // Check neighbors for every layer the node is in
            for layer in 0..=node.max_layer {
                let neighbors = index.get_neighbors_layer(node, layer).unwrap();
                let m_max = if layer == 0 { index.config.m0 } else { index.config.m } as usize;

                // Assert we never exceed M_max
                prop_assert!(neighbors.len() <= m_max,
                    "Node {:?} at layer {} has {} neighbors, max {}",
                    node_id, layer, neighbors.len(), m_max);

                // Assert no self-loops
                prop_assert!(!neighbors.contains(&node_id), "Node {:?} has self-loop at layer {}", node_id, layer);

                // Assert neighbors are unique
                let unique: HashSet<_> = neighbors.iter().collect();
                prop_assert_eq!(unique.len(), neighbors.len(), "Node {:?} has duplicate neighbors at layer {}", node_id, layer);
            }
        }

        // Check 3: Reachability (Connectivity)
        // We pick a random target and try to search for it using the graph.
        // With ef high enough, we should find it with Recall ~100% for this scale.
        if let Some(&target_id) = inserted_ids.first() {
            // We need the raw vector to search
            let target_vec = storage.get_vector(target_id);

            // Perform search
            // Use ef_search same as construction or higher

            // We can't easily call `index.search` because we haven't implemented public search yet?
            // Wait, `search_layer` is in `search.rs` but `search` API is likely not exposed on `HnswIndex` directly yet.
            // The mandate said "Implement W3.4 Insertion Logic", search was W3.3 but `HnswIndex::search` might not be wired up.
            // Let's check `lib.rs` exports. `Searcher` is exported.

            use edgevec::hnsw::{Searcher, SearchContext};
            let mut ctx = SearchContext::new();

            // Full search from top
            let entry_point = index.entry_point().unwrap();
            let mut curr_ep = entry_point;
            let max_layer = index.max_layer();

            let searcher = Searcher::<edgevec::metric::L2Squared, VectorStorage>::new(&index, &storage);

            // Top layers greedy
            for lc in (1..=max_layer).rev() {
                 searcher.search_layer(
                     &mut ctx,
                     [curr_ep],
                     &target_vec,
                     1,
                     lc
                 ).unwrap();
                 if let Some(best) = ctx.scratch.first() {
                     curr_ep = best.node_id;
                 }
            }

            // Bottom layer beam
            searcher.search_layer(
                &mut ctx,
                [curr_ep],
                &target_vec,
                ef as usize,
                0
            ).unwrap();
            let results = &ctx.scratch;

            // Check if we found the target node
            // The target node corresponds to the inserted vector.
            // VectorId starts at 1. NodeId starts at 0.
            // We assume 1:1 mapping in insertion order: VectorId(1) -> NodeId(0).
            // But we can check via `get_node(res.node_id).vector_id`.

            let found = results.iter().any(|c| {
                let n = index.get_node(c.node_id).unwrap();
                n.vector_id == target_id
            });

            prop_assert!(found, "Could not reach target VectorId {:?} via graph search", target_id);
        }
    }

    #[test]
    fn prop_insert_idempotency(
        vec in prop::collection::vec(-10.0f32..10.0, 4)
    ) {
        let (mut index, mut storage) = create_env(4, 16, 50);

        let id1 = index.insert(&vec, &mut storage).unwrap();
        let id2 = index.insert(&vec, &mut storage).unwrap();

        // They should be distinct IDs because storage appends
        prop_assert_ne!(id1, id2);

        // Both should be in graph
        prop_assert_eq!(index.node_count(), 2);
    }
}
