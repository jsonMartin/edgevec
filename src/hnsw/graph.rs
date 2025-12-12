#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use super::config::HnswConfig;
use super::neighbor::NeighborPool;
use crate::storage::VectorStorage;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::vec::Vec;
use thiserror::Error;

/// Unique identifier for a vector in the database.
///
/// # Size
/// 8 bytes, aligned to 8
///
/// # Invariants
/// - IDs are never reused (monotonically increasing)
/// - ID 0 is reserved (invalid sentinel)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct VectorId(pub u64);

impl VectorId {
    /// Sentinel value indicating "no vector"
    pub const INVALID: Self = VectorId(0);

    /// First valid ID
    pub const FIRST: Self = VectorId(1);
}

/// Internal node identifier within HNSW graph.
///
/// # Size
/// 4 bytes, aligned to 4
///
/// # Invariants
/// - `NodeId` corresponds 1:1 with `VectorId` (lower 32 bits)
/// - `NodeId` 0xFFFFFFFF is reserved (invalid sentinel)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct NodeId(pub u32);

impl NodeId {
    /// Sentinel value indicating invalid node
    pub const INVALID: Self = NodeId(u32::MAX);
}

/// Represents a layer level in the HNSW graph.
///
/// Layer 0 is the base layer containing all nodes.
/// Higher layers contain a subset of nodes for faster navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[allow(dead_code)]
pub struct Layer(pub u8);

/// Errors that can occur during graph operations.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    /// The graph has reached its maximum node capacity (`u32::MAX`).
    #[error("node capacity exceeded")]
    CapacityExceeded,

    /// The provided `VectorId` is invalid (e.g., sentinel value).
    #[error("invalid vector id")]
    InvalidVectorId,

    /// Neighbor data is corrupted or offset is out of bounds.
    #[error("neighbor data corrupted")]
    NeighborError,

    /// Node ID is out of bounds.
    #[error("node id out of bounds")]
    NodeIdOutOfBounds,

    /// Configuration mismatch with storage.
    #[error("config dimension mismatch: expected {expected}, got {actual}")]
    ConfigMismatch {
        /// Expected dimensions.
        expected: u32,
        /// Actual dimensions in config.
        actual: u32,
    },

    /// Query vector has wrong dimensions.
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch {
        /// Expected dimensions.
        expected: usize,
        /// Actual dimensions.
        actual: usize,
    },

    /// Storage operation failed.
    #[error("storage error: {0}")]
    Storage(String),

    /// Invalid configuration parameter.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

/// A node in the HNSW graph with its adjacency information.
///
/// # Layout
///
/// Total size: 16 bytes
/// Alignment: 8 bytes
///
/// # Fields
///
/// - `vector_id`: 8 bytes
/// - `neighbor_offset`: 4 bytes
/// - `neighbor_len`: 2 bytes
/// - `max_layer`: 1 byte
/// - `pad`: 1 byte
#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct HnswNode {
    /// The vector ID this node represents
    pub vector_id: VectorId,

    /// Offset into COMPRESSED neighbor pool
    pub neighbor_offset: u32,

    /// Length of neighbor data in bytes (Allocated Capacity)
    pub neighbor_len: u16,

    /// The maximum layer this node appears in
    pub max_layer: u8,

    /// Padding for alignment
    pub pad: u8,
}

/// The HNSW Graph structure managing layers and nodes.
///
/// # Memory
///
/// Uses a flattened representation for cache efficiency.
/// Nodes are stored in a contiguous vector.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HnswIndex {
    /// Algorithm configuration
    pub config: HnswConfig,

    /// Node metadata (fixed-size per node)
    pub(crate) nodes: Vec<HnswNode>,

    /// Compressed neighbor lists
    pub(crate) neighbors: NeighborPool,

    /// Entry point (highest layer node)
    pub(crate) entry_point: Option<NodeId>,

    /// Maximum layer in the graph
    pub(crate) max_layer: u8,

    /// Level probability multiplier (1/ln(M))
    pub(crate) level_mult: f32,

    /// Deterministic RNG state
    rng: ChaCha8Rng,
}

impl HnswIndex {
    /// Creates a new empty HNSW graph.
    ///
    /// # Arguments
    ///
    /// * `config` - HNSW configuration parameters.
    /// * `storage` - Vector storage to validate against.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::ConfigMismatch` if storage dimensions differ from config.
    /// Returns `GraphError::InvalidConfig` if configuration parameters are invalid (e.g., M <= 1).
    pub fn new(config: HnswConfig, storage: &VectorStorage) -> Result<Self, GraphError> {
        if config.dimensions != storage.dimensions() {
            return Err(GraphError::ConfigMismatch {
                expected: storage.dimensions(),
                actual: config.dimensions,
            });
        }

        if config.m <= 1 {
            return Err(GraphError::InvalidConfig(format!(
                "m must be > 1, got {}",
                config.m
            )));
        }
        if config.m0 < config.m {
            return Err(GraphError::InvalidConfig(format!(
                "m0 must be >= m, got {} < {}",
                config.m0, config.m
            )));
        }

        // Calculate level multiplier: m_l = 1 / ln(M)
        let m_float = config.m as f32;
        let level_mult = if m_float > 1.0 {
            1.0 / m_float.ln()
        } else {
            0.0
        };

        // Initialize RNG with a default seed for determinism.
        let rng = ChaCha8Rng::seed_from_u64(42);

        Ok(Self {
            config,
            nodes: Vec::new(),
            neighbors: NeighborPool::new(),
            entry_point: None,
            max_layer: 0,
            level_mult,
            rng,
        })
    }

    /// Generates a random level for a new node.
    ///
    /// Formula: `floor(-ln(uniform(0,1)) * m_l)`
    /// Clamped to `max_level` (e.g. 16) to prevent memory explosion.
    #[must_use]
    pub fn get_random_level(&mut self) -> u8 {
        // Generate uniform(0, 1)
        let r: f32 = self.rng.gen_range(f32::EPSILON..=1.0);
        let level = (-r.ln() * self.level_mult).floor();

        // Safety cap (e.g. 16)
        if level > 16.0 {
            16
        } else {
            level as u8
        }
    }

    /// Adds a node to the graph.
    ///
    /// # Arguments
    ///
    /// * `vector_id` - The external vector identifier
    /// * `max_layer` - The maximum layer for this node
    ///
    /// # Returns
    ///
    /// The new `NodeId` assigned to this node, or a `GraphError`.
    pub fn add_node(&mut self, vector_id: VectorId, max_layer: u8) -> Result<NodeId, GraphError> {
        if vector_id == VectorId::INVALID {
            return Err(GraphError::InvalidVectorId);
        }

        // Safety limit for NodeId
        if self.nodes.len() >= u32::MAX as usize {
            return Err(GraphError::CapacityExceeded);
        }

        let node = HnswNode {
            vector_id,
            neighbor_offset: 0,
            neighbor_len: 0,
            max_layer,
            pad: 0,
        };

        #[allow(clippy::cast_possible_truncation)]
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);

        // Update max layer if needed
        if max_layer > self.max_layer {
            self.max_layer = max_layer;
        }

        Ok(id)
    }

    /// Sets the neighbors for a node.
    ///
    /// # Arguments
    /// * `node_id` - The node to update.
    /// * `neighbors` - The list of neighbor IDs.
    pub fn set_neighbors(
        &mut self,
        node_id: NodeId,
        neighbors: &[NodeId],
    ) -> Result<(), GraphError> {
        if node_id.0 as usize >= self.nodes.len() {
            return Err(GraphError::InvalidVectorId);
        }

        // Convert NodeId to u32 for encoding
        let neighbor_u32s: Vec<u32> = neighbors.iter().map(|n| n.0).collect();
        let encoded = NeighborPool::encode_neighbors(&neighbor_u32s);

        // Alloc new space
        let (offset, capacity) = self.neighbors.alloc(encoded.len())?;

        // Write data
        let start = offset as usize;
        let end = start + encoded.len();
        self.neighbors.buffer[start..end].copy_from_slice(&encoded);

        // Update node and free old
        let node = &mut self.nodes[node_id.0 as usize];

        // Free old slot if it existed
        if node.neighbor_len > 0 {
            self.neighbors.free(node.neighbor_offset, node.neighbor_len);
        }

        node.neighbor_offset = offset;
        node.neighbor_len = capacity; // Store allocated capacity

        Ok(())
    }

    /// Retrieves a node by its ID.
    #[must_use]
    pub fn get_node(&self, id: NodeId) -> Option<&HnswNode> {
        if id == NodeId::INVALID {
            return None;
        }
        self.nodes.get(id.0 as usize)
    }

    /// Retrieves the neighbors for a node.
    pub fn get_neighbors(&self, node: &HnswNode) -> Result<Vec<NodeId>, GraphError> {
        let start = node.neighbor_offset as usize;
        // Read up to allocated capacity
        let end = start + node.neighbor_len as usize;

        if end > self.neighbors.buffer.len() {
            return Err(GraphError::NeighborError);
        }

        let slice = &self.neighbors.buffer[start..end];
        let raw_neighbors = NeighborPool::decode_neighbors(slice);

        // Convert back to NodeId
        let neighbors = raw_neighbors.into_iter().map(NodeId).collect();
        Ok(neighbors)
    }

    /// Retrieves the neighbors for a specific layer.
    pub fn get_neighbors_layer(
        &self,
        node: &HnswNode,
        layer: u8,
    ) -> Result<Vec<NodeId>, GraphError> {
        let start = node.neighbor_offset as usize;
        let end = start + node.neighbor_len as usize;

        if end > self.neighbors.buffer.len() {
            return Err(GraphError::NeighborError);
        }

        let slice = &self.neighbors.buffer[start..end];
        let raw_neighbors = NeighborPool::decode_layer(slice, layer);

        // Convert back to NodeId
        let neighbors = raw_neighbors.into_iter().map(NodeId).collect();
        Ok(neighbors)
    }

    /// Returns the number of nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the entry point node ID, if any.
    #[must_use]
    pub fn entry_point(&self) -> Option<NodeId> {
        self.entry_point
    }

    /// Sets the entry point node ID.
    pub fn set_entry_point(&mut self, id: NodeId) {
        self.entry_point = Some(id);
    }

    /// Returns the current maximum layer in the graph.
    #[must_use]
    pub fn max_layer(&self) -> u8 {
        self.max_layer
    }

    /// Marks a vector as deleted in the storage.
    ///
    /// The node remains in the graph for routing, but will be filtered from search results.
    ///
    /// # Arguments
    ///
    /// * `id` - The vector ID to delete.
    /// * `storage` - The vector storage to update.
    ///
    /// # Returns
    ///
    /// `true` if the vector was active and is now deleted.
    /// `false` if it was already deleted.
    pub fn delete(&self, id: VectorId, storage: &mut VectorStorage) -> bool {
        storage.mark_deleted(id)
    }

    /// DEBUG: Print memory stats
    pub fn log_stats(&self) {
        println!("Index Stats:");
        println!("  Node Count: {}", self.nodes.len());
        println!("  Neighbor Buffer Len: {}", self.neighbors.buffer.len());
        println!(
            "  Neighbor Buffer Cap: {}",
            self.neighbors.buffer.capacity()
        );
        println!("  Total Memory Usage: {} bytes", self.memory_usage());
        // bucket stats are internal to NeighborPool
    }

    /// Returns the approximate memory usage in bytes.
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        let nodes_size = self.nodes.capacity() * std::mem::size_of::<HnswNode>();
        let neighbors_size = self.neighbors.memory_usage();

        std::mem::size_of::<Self>() + nodes_size + neighbors_size
    }
}

/// Trait for providing vector data by ID.
pub trait VectorProvider {
    /// Returns the vector data for a given ID.
    fn get_vector(&self, id: VectorId) -> Cow<'_, [f32]>;

    /// Returns true if the vector is marked as deleted.
    fn is_deleted(&self, id: VectorId) -> bool {
        let _ = id;
        false
    }

    /// Returns the quantized vector data for a given ID, if available.
    ///
    /// # Returns
    ///
    /// * `Some(&[u8])` - If the provider supports direct quantized access.
    /// * `None` - If not supported or data is not quantized.
    fn get_quantized_vector(&self, id: VectorId) -> Option<&[u8]> {
        let _ = id;
        None
    }

    /// Quantizes a query vector into the provided output buffer.
    ///
    /// # Arguments
    ///
    /// * `query` - The query vector in f32.
    /// * `output` - buffer to write quantized data into.
    ///
    /// # Returns
    ///
    /// * `Some(&[u8])` - The quantized slice (borrowed from output).
    /// * `None` - If quantization is not supported.
    fn quantize_query<'a>(&self, query: &[f32], output: &'a mut Vec<u8>) -> Option<&'a [u8]> {
        let _ = query;
        let _ = output;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HnswIndex>();
    }

    #[test]
    fn test_initialization() {
        let config = HnswConfig::new(128);
        // Create storage with matching dimensions
        let storage = VectorStorage::new(&config, None);

        let index = HnswIndex::new(config.clone(), &storage).unwrap();

        assert_eq!(index.node_count(), 0);
        assert_eq!(index.entry_point(), None);
        assert_eq!(index.max_layer(), 0);
    }

    #[test]
    fn test_dimension_mismatch() {
        let config_idx = HnswConfig::new(128);
        let config_store = HnswConfig::new(64);
        let storage = VectorStorage::new(&config_store, None);

        let result = HnswIndex::new(config_idx, &storage);
        assert!(matches!(
            result,
            Err(GraphError::ConfigMismatch {
                expected: 64,
                actual: 128
            })
        ));
    }

    #[test]
    fn test_layer_distribution() {
        // Geometric distribution test
        // m=16 => m_l = 1/ln(16) ≈ 0.36
        // Prob(level > 0) = e^(-1/m_l) = 1/M = 1/16
        // We can't strictly test randomness without huge samples, but we can sanity check.
        let config = HnswConfig::new(128);
        let storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::new(config, &storage).unwrap();

        let mut levels = vec![0u8; 1000];
        for l in levels.iter_mut() {
            *l = index.get_random_level();
        }

        // Level 0 should be most common
        let l0_count = levels.iter().filter(|&&l| l == 0).count();
        assert!(
            l0_count > 800,
            "Level 0 should be dominant (expected ~93% for M=16)"
        );

        // Max level shouldn't be crazy
        let max = *levels.iter().max().unwrap();
        assert!(max < 16, "Level should be reasonable");
    }

    #[test]
    fn test_neighbor_roundtrip() {
        let config = HnswConfig::new(128);
        let storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::new(config, &storage).unwrap();

        let id1 = index.add_node(VectorId(1), 0).unwrap();
        let id2 = index.add_node(VectorId(2), 0).unwrap();
        let id3 = index.add_node(VectorId(3), 0).unwrap();

        // Neighbors: [2, 3]
        let neighbors = vec![id2, id3];
        index.set_neighbors(id1, &neighbors).unwrap();

        {
            let node1 = index.get_node(id1).unwrap();
            let retrieved = index.get_neighbors(node1).unwrap();
            assert_eq!(retrieved, neighbors);
        } // Drop node1 borrow

        // Update neighbors: [3] (shrink)
        let neighbors2 = vec![id3];
        index.set_neighbors(id1, &neighbors2).unwrap();

        {
            let node1 = index.get_node(id1).unwrap();
            let retrieved2 = index.get_neighbors(node1).unwrap();
            assert_eq!(retrieved2, neighbors2);
        }

        // Check if free list got populated (cannot check directly as NeighborPool is private,
        // but we can trust neighbor.rs tests for internal logic)
    }
}
