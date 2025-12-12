#![no_main]
use libfuzzer_sys::fuzz_target;
use edgevec::hnsw::{HnswGraph, Searcher, VectorId, VectorProvider, NodeId};
use edgevec::metric::L2Squared;

// Mock storage for fuzzer
struct FuzzStorage {
    vectors: Vec<Vec<f32>>,
}

impl VectorProvider for FuzzStorage {
    fn get_vector(&self, id: VectorId) -> &[f32] {
        let idx = (id.0 as usize).saturating_sub(1);
        if idx < self.vectors.len() {
            &self.vectors[idx]
        } else {
            // Fallback for invalid IDs if they somehow pass graph checks
            &self.vectors[0]
        }
    }
}

fuzz_target!(|data: &[u8]| {
    // Min size: 4 bytes for NodeId + 1 float (4 bytes)
    if data.len() < 8 {
        return;
    }

    // 1. Parse Entry Point (first 4 bytes)
    let entry_point_raw = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let entry_point = NodeId(entry_point_raw);

    // 2. Parse Query Vector (remaining bytes as f32s)
    let query_bytes = &data[4..];
    let chunks: Vec<&[u8]> = query_bytes.chunks_exact(4).collect();
    if chunks.is_empty() {
        return;
    }
    
    let query: Vec<f32> = chunks.iter()
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();
        
    // Sanitize query (Metric panics on NaN)
    if query.iter().any(|x| x.is_nan()) {
        return;
    }

    // 3. Setup Static Graph (Small Diamond)
    // Vectors: [0,0], [1,0], [0,1], [1,1]
    // Dimension must match query length. 
    // To support variable query length, we adjust the stored vectors or pad.
    // Easier: Use query length as dimension.
    let dim = query.len();
    let mut vectors = vec![
        vec![0.0; dim],
        vec![1.0; dim], // actually just fill with 1s
        vec![0.5; dim],
        vec![0.1; dim],
    ];
    
    let storage = FuzzStorage { vectors };
    let mut graph = HnswGraph::new();
    
    // Create 4 nodes
    let n1 = graph.add_node(VectorId(1), 0).unwrap();
    let n2 = graph.add_node(VectorId(2), 0).unwrap();
    let n3 = graph.add_node(VectorId(3), 0).unwrap();
    let n4 = graph.add_node(VectorId(4), 0).unwrap();

    // Connect them
    let _ = graph.set_neighbors(n1, &[n2, n3]);
    let _ = graph.set_neighbors(n2, &[n4]);
    let _ = graph.set_neighbors(n3, &[n4]);
    let _ = graph.set_neighbors(n4, &[]); // Sink

    // 4. Run Search
    let mut searcher = Searcher::<L2Squared, _>::new(&graph, &storage);
    
    // We specifically want to fuzz the `search_layer` robustness against random NodeIds
    // passed as entry_point.
    let _result = searcher.search_layer(entry_point, &query, 10, 0);
    
    // We don't assert result correctness here (that's for proptest),
    // just that it DOES NOT PANIC (unless Metric panics on NaN, which we handled).
    // search_layer returns Result, so it should handle invalid entry points gracefully.
});

