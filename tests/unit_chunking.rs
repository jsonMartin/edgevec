use edgevec::hnsw::{HnswNode, NeighborPool, NodeId, VectorId};
use edgevec::persistence::read_file_header;
use edgevec::{ChunkedWriter, HnswConfig, HnswIndex, VectorStorage};
use proptest::prelude::*;
use std::mem::size_of;

fn deserialize_and_verify(
    bytes: &[u8],
    original_storage: &VectorStorage,
    original_index: &HnswIndex,
) {
    // 1. Header
    let header = read_file_header(&bytes[0..64]).expect("Invalid header");
    assert_eq!(header.dimensions, original_storage.dimensions());
    assert_eq!(header.vector_count, original_storage.len() as u64);
    assert_eq!(header.hnsw_m, original_index.config.m);
    assert_eq!(header.hnsw_m0, original_index.config.m0);

    // 2. Vectors
    let dim = header.dimensions as usize;
    let count = header.vector_count as usize;
    let vector_section_size = count * dim * 4;
    let vector_data = &bytes[64..64 + vector_section_size];

    for i in 0..count {
        let start = i * dim * 4;
        let end = start + dim * 4;
        let vec_bytes = &vector_data[start..end];

        let mut vec = Vec::with_capacity(dim);
        for chunk in vec_bytes.chunks_exact(4) {
            let val = f32::from_le_bytes(chunk.try_into().unwrap());
            vec.push(val);
        }

        let id = VectorId::FIRST.0 + i as u64;
        let original_vec = original_storage.get_vector(VectorId(id));
        assert_eq!(
            &vec[..],
            &original_vec[..],
            "Vector mismatch at index {}",
            i
        );
    }

    // 3. Index Nodes
    let index_offset = header.index_offset as usize;
    assert_eq!(index_offset, 64 + vector_section_size);

    let node_count = original_index.node_count();
    let nodes_size = node_count * size_of::<HnswNode>();
    let nodes_data = &bytes[index_offset..index_offset + nodes_size];

    for i in 0..node_count {
        let start = i * 16;
        let node_bytes = &nodes_data[start..start + 16];

        let vid_bytes: [u8; 8] = node_bytes[0..8].try_into().unwrap();
        let vid = u64::from_le_bytes(vid_bytes);

        let noff_bytes: [u8; 4] = node_bytes[8..12].try_into().unwrap();
        let n_offset = u32::from_le_bytes(noff_bytes);

        let nlen_bytes: [u8; 2] = node_bytes[12..14].try_into().unwrap();
        let n_len = u16::from_le_bytes(nlen_bytes);

        let max_layer = node_bytes[14];

        let original_node = original_index.get_node(NodeId(i as u32)).unwrap();

        assert_eq!(
            vid, original_node.vector_id.0,
            "Node {} vector_id mismatch",
            i
        );
        assert_eq!(
            n_offset, original_node.neighbor_offset,
            "Node {} neighbor_offset mismatch",
            i
        );
        assert_eq!(
            n_len, original_node.neighbor_len,
            "Node {} neighbor_len mismatch",
            i
        );
        assert_eq!(
            max_layer, original_node.max_layer,
            "Node {} max_layer mismatch",
            i
        );
    }

    // 4. Neighbors
    let neighbor_start = index_offset + nodes_size;
    let neighbor_data = &bytes[neighbor_start..];

    for i in 0..node_count {
        let start = i * 16;
        let node_bytes = &nodes_data[start..start + 16];
        let noff_bytes: [u8; 4] = node_bytes[8..12].try_into().unwrap();
        let n_offset = u32::from_le_bytes(noff_bytes) as usize;
        let nlen_bytes: [u8; 2] = node_bytes[12..14].try_into().unwrap();
        let n_len = u16::from_le_bytes(nlen_bytes) as usize;

        if n_len > 0 {
            if n_offset + n_len > neighbor_data.len() {
                panic!(
                    "Neighbor offset out of bounds: {} + {} > {}",
                    n_offset,
                    n_len,
                    neighbor_data.len()
                );
            }
            let slice = &neighbor_data[n_offset..n_offset + n_len];
            // Decode to verify content
            // NOTE: We can't verify EXACT neighbor lists trivially because
            // decode_neighbors decodes ALL layers flattened.
            // HnswIndex::get_neighbors only returns one layer or all?
            // HnswIndex::get_neighbors calls NeighborPool::decode_neighbors which returns flattened list?
            // Yes, graph.rs: `get_neighbors` calls `NeighborPool::decode_neighbors(slice)`.
            // So we can compare directly.

            let decoded = NeighborPool::decode_neighbors(slice);

            let original_node = original_index.get_node(NodeId(i as u32)).unwrap();
            let original_neighbors = original_index.get_neighbors(original_node).unwrap();
            let decoded_ids: Vec<u32> = original_neighbors.iter().map(|n| n.0).collect();

            assert_eq!(decoded, decoded_ids, "Neighbors mismatch for node {}", i);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_chunked_export_roundtrip(
        vectors in prop::collection::vec(prop::collection::vec(0.0f32..1.0, 4), 10..50)
    ) {
        let config = HnswConfig::new(4);
        let mut storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::new(config, &storage).unwrap();

        for vec in &vectors {
            index.insert(vec, &mut storage).unwrap();
        }

        // chunk_size = 100 bytes (small enough to force many chunks)
        // 64 header + 16 vector + 16 node + neighbors...
        // 100 bytes ensures header is separate or header+partial vector.
        let chunk_size = 100;
        let writer = (&storage, &index);
        let iter = writer.export_chunked(chunk_size);

        let mut buffer = Vec::new();
        let mut chunk_count = 0;
        for chunk in iter {
            assert!(chunk.len() <= chunk_size);
            assert!(!chunk.is_empty());
            buffer.extend_from_slice(&chunk);
            chunk_count += 1;
        }

        assert!(chunk_count > 1, "Should produce multiple chunks");
        deserialize_and_verify(&buffer, &storage, &index);
    }
}
