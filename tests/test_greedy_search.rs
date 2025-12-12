use edgevec::hnsw::{
    GraphError, HnswConfig, HnswGraph, NodeId, SearchContext, Searcher, VectorId, VectorProvider,
};
use edgevec::metric::L2Squared;
use edgevec::storage::VectorStorage;

struct TestVectorStorage {
    vectors: Vec<Vec<f32>>,
}

impl VectorProvider for TestVectorStorage {
    fn get_vector(&self, id: VectorId) -> std::borrow::Cow<'_, [f32]> {
        std::borrow::Cow::Borrowed(&self.vectors[id.0 as usize - 1]) // VectorId starts at 1
    }
}

#[test]
fn test_greedy_search_finds_target() {
    // Setup
    let config = HnswConfig::new(2);
    let dummy_storage = VectorStorage::new(&config, None); // Real storage for config check
    let mut graph = HnswGraph::new(config, &dummy_storage).unwrap();

    let mut storage = TestVectorStorage { vectors: vec![] };

    // Create 3 vectors: A (target), B (neighbor of A), C (entry point, neighbor of B)
    // A = [1.0, 0.0]
    // B = [2.0, 0.0]
    // C = [3.0, 0.0]
    // Query = [1.1, 0.0] -> Closest is A.

    let v1 = vec![1.0, 0.0];
    let v2 = vec![2.0, 0.0];
    let v3 = vec![3.0, 0.0];

    storage.vectors.push(v1.clone());
    storage.vectors.push(v2.clone());
    storage.vectors.push(v3.clone());

    // Add nodes. VectorId(1) -> A, VectorId(2) -> B, VectorId(3) -> C
    let id1 = graph.add_node(VectorId(1), 0).unwrap();
    let id2 = graph.add_node(VectorId(2), 0).unwrap();
    let id3 = graph.add_node(VectorId(3), 0).unwrap();

    // Edges: C -> B -> A
    // id3 (C) connects to id2 (B)
    // id2 (B) connects to id1 (A)
    graph.set_neighbors(id3, &[id2]).unwrap();
    graph.set_neighbors(id2, &[id1]).unwrap();
    graph.set_neighbors(id1, &[]).unwrap();

    // Search from C (id3)
    let searcher = Searcher::<L2Squared, _>::new(&graph, &storage);
    let mut ctx = SearchContext::new();
    let query = vec![1.1, 0.0];

    // We want to find top 1
    // Note: entry_points is usually a slice/iterator. search_layer expects iterable.
    searcher
        .search_layer(&mut ctx, vec![id3], &query, 1, 0)
        .unwrap();
    let top_candidates = &ctx.scratch;

    // Should find A (id1) as best match
    assert!(!top_candidates.is_empty());
    assert_eq!(top_candidates[0].node_id, id1);
    // (1.0-1.1)^2 = 0.01
    assert!((top_candidates[0].distance - 0.01).abs() < 1e-5);
}

#[test]
fn test_greedy_search_local_optimum() {
    // Setup:
    let config = HnswConfig::new(2);
    let dummy_storage = VectorStorage::new(&config, None);
    let mut graph = HnswGraph::new(config, &dummy_storage).unwrap();

    let mut storage = TestVectorStorage { vectors: vec![] };

    storage.vectors.push(vec![0.0, 0.0]); // P: Vid 1
    storage.vectors.push(vec![10.0, 0.0]); // N1: Vid 2
    storage.vectors.push(vec![0.0, 10.0]); // N2: Vid 3
    storage.vectors.push(vec![12.0, 0.0]); // T: Vid 4

    let p = graph.add_node(VectorId(1), 0).unwrap();
    let n1 = graph.add_node(VectorId(2), 0).unwrap();
    let n2 = graph.add_node(VectorId(3), 0).unwrap();
    let t = graph.add_node(VectorId(4), 0).unwrap();

    graph.set_neighbors(p, &[n1, n2]).unwrap();
    graph.set_neighbors(n1, &[t]).unwrap();
    graph.set_neighbors(n2, &[]).unwrap();
    graph.set_neighbors(t, &[]).unwrap();

    let searcher = Searcher::<L2Squared, _>::new(&graph, &storage);
    let mut ctx = SearchContext::new();
    let query = vec![12.0, 0.1];

    searcher
        .search_layer(&mut ctx, vec![p], &query, 1, 0)
        .unwrap();
    let best = ctx.scratch[0];

    assert_eq!(best.node_id, t);
}

#[test]
fn test_greedy_search_out_of_bounds_neighbor() {
    let config = HnswConfig::new(2);
    let dummy_storage = VectorStorage::new(&config, None);
    let mut graph = HnswGraph::new(config, &dummy_storage).unwrap();

    let mut storage = TestVectorStorage { vectors: vec![] };

    let v1 = vec![1.0, 0.0];
    storage.vectors.push(v1);

    let id1 = graph.add_node(VectorId(1), 0).unwrap();

    // Set a neighbor that is out of bounds (NodeId(999))
    graph.set_neighbors(id1, &[NodeId(999)]).unwrap();

    let searcher = Searcher::<L2Squared, _>::new(&graph, &storage);
    let mut ctx = SearchContext::new();
    let query = vec![1.0, 0.0];

    let result = searcher.search_layer(&mut ctx, vec![id1], &query, 1, 0);

    assert!(matches!(result, Err(GraphError::NodeIdOutOfBounds)));
}
