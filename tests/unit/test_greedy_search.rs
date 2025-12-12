use edgevec::hnsw::{HnswGraph, Searcher, VectorId, VectorProvider, NodeId, GraphError};
use edgevec::metric::L2Squared;

struct TestVectorStorage {
    vectors: Vec<Vec<f32>>,
}

impl VectorProvider for TestVectorStorage {
    fn get_vector(&self, id: VectorId) -> &[f32] {
        &self.vectors[id.0 as usize - 1] // VectorId starts at 1
    }
}

#[test]
fn test_greedy_search_finds_target() {
    // Setup
    let mut graph = HnswGraph::new();
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
    let mut searcher = Searcher::<L2Squared, _>::new(&graph, &storage);
    let query = vec![1.1, 0.0];
    
    // We want to find top 1
    let results = searcher.search_layer(id3, &query, 1, 0).unwrap();
    let top_candidates = results.into_vec();
    
    // Should find A (id1) as best match
    assert!(!top_candidates.is_empty());
    assert_eq!(top_candidates[0].node_id, id1);
    // (1.0-1.1)^2 = 0.01
    assert!((top_candidates[0].distance - 0.01).abs() < 1e-5); 
}

#[test]
fn test_greedy_search_local_optimum() {
    // Test where greedy search might get stuck if not enough ef
    // But with ef=1 it behaves strictly greedy.
    // Setup:
    // Entry P (0,0).
    // N1 (10, 0). N2 (0, 10).
    // Target T (12, 0).
    // P connected to N1 and N2.
    // N1 connected to T.
    // Query (12, 0.1).
    // P dist to Q: sqrt(144 + 0.01) = 12.0004
    // N1 dist to Q: sqrt(4 + 0.01) = 2.002
    // N2 dist to Q: sqrt(144 + 98) = 15.5
    // Greedy from P should choose N1.
    // N1 neighbors T. T dist to Q: 0.1.
    // Result T.
    
    let mut graph = HnswGraph::new();
    let mut storage = TestVectorStorage { vectors: vec![] };

    storage.vectors.push(vec![0.0, 0.0]);   // P: Vid 1
    storage.vectors.push(vec![10.0, 0.0]);  // N1: Vid 2
    storage.vectors.push(vec![0.0, 10.0]);  // N2: Vid 3
    storage.vectors.push(vec![12.0, 0.0]);  // T: Vid 4

    let p = graph.add_node(VectorId(1), 0).unwrap();
    let n1 = graph.add_node(VectorId(2), 0).unwrap();
    let n2 = graph.add_node(VectorId(3), 0).unwrap();
    let t = graph.add_node(VectorId(4), 0).unwrap();

    graph.set_neighbors(p, &[n1, n2]).unwrap();
    graph.set_neighbors(n1, &[t]).unwrap();
    graph.set_neighbors(n2, &[]).unwrap();
    graph.set_neighbors(t, &[]).unwrap();

    let mut searcher = Searcher::<L2Squared, _>::new(&graph, &storage);
    let query = vec![12.0, 0.1];

    let results = searcher.search_layer(p, &query, 1, 0).unwrap();
    let best = results.into_vec()[0];

    assert_eq!(best.node_id, t);
}

#[test]
fn test_greedy_search_out_of_bounds_neighbor() {
    // Test that search handles invalid neighbor IDs gracefully (by returning error)
    // instead of panicking or accessing OOB memory.
    let mut graph = HnswGraph::new();
    let mut storage = TestVectorStorage { vectors: vec![] };
    
    let v1 = vec![1.0, 0.0];
    storage.vectors.push(v1);
    
    let id1 = graph.add_node(VectorId(1), 0).unwrap();
    
    // Set a neighbor that is out of bounds (NodeId(999))
    // HnswGraph::set_neighbors currently doesn't validate target IDs exist
    graph.set_neighbors(id1, &[NodeId(999)]).unwrap();
    
    let mut searcher = Searcher::<L2Squared, _>::new(&graph, &storage);
    let query = vec![1.0, 0.0];
    
    let result = searcher.search_layer(id1, &query, 1, 0);
    
    assert!(matches!(result, Err(GraphError::NodeIdOutOfBounds)));
}

