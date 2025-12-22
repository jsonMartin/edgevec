//! HNSW module containing graph logic, configuration, and search.

/// Configuration types.
pub mod config;
/// Graph data structures.
pub mod graph;
/// Insertion algorithms.
pub mod insert;
/// Neighbor management.
pub mod neighbor;
/// F32 rescoring for BQ search results (v0.7.0 - RFC-002 Phase 2).
pub mod rescore;
/// Search algorithms.
pub mod search;
/// Binary quantization search algorithms (v0.7.0 - RFC-002 Phase 2).
pub mod search_bq;

pub use config::HnswConfig;
pub use graph::{
    BatchDeleteError, BatchDeleteResult, CompactionResult, GraphError, HnswIndex, HnswNode, NodeId,
    VectorId, VectorProvider,
};
pub use neighbor::NeighborPool;
pub use search::{Candidate, SearchContext, SearchResult, Searcher};

/// Alias for `HnswIndex` to support legacy tests.
pub type HnswGraph = HnswIndex;
