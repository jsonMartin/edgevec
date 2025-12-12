use edgevec::hnsw::neighbor::NeighborPool;
use proptest::prelude::*;

proptest! {
    /// Property: Roundtrip encoding/decoding preserves data integrity.
    ///
    /// For any vector of u32s, `decode(encode(v))` must equal `sorted(v)`.
    /// Duplicate values are preserved.
    #[test]
    fn prop_vbyte_roundtrip(input in proptest::collection::vec(any::<u32>(), 0..1000)) {
        // 1. Prepare expected result (sorted)
        let mut expected = input.clone();
        expected.sort_unstable();

        // 2. Encode
        let encoded = NeighborPool::encode_neighbors(&input);

        // 3. Decode
        let decoded = NeighborPool::decode_neighbors(&encoded);

        // 4. Assert
        prop_assert_eq!(decoded, expected);
    }

    /// Property: Compression efficiency for dense data.
    ///
    /// For sorted vectors with small gaps (deltas < 128), VByte should be efficient.
    /// Each element (except perhaps the first large one) contributes a small delta.
    ///
    /// We verify that for list of N items, size is roughly N bytes + overhead,
    /// definitely less than N * 4 (raw u32).
    #[test]
    fn prop_compression_small_deltas(
        // Generate sorted unique integers with small steps
        input in proptest::collection::vec(0u32..100u32, 0..100)
            .prop_map(|v| {
                let mut acc = 0;
                let mut res = Vec::with_capacity(v.len());
                for delta in v {
                    acc += delta;
                    res.push(acc);
                }
                res
            })
    ) {
        if input.is_empty() {
            return Ok(());
        }

        let encoded = NeighborPool::encode_neighbors(&input);

        // Count takes 1-5 bytes.
        // Each delta is < 100, so 1 byte each.
        // Total should be roughly 1 + N bytes.
        // We allow some slack for the count varint.
        let max_expected_size = 5 + input.len();

        prop_assert!(encoded.len() <= max_expected_size);

        // Definitely better than raw
        if input.len() > 2 {
            prop_assert!(encoded.len() < input.len() * 4);
        }
    }
}

#[test]
fn test_edge_case_large_gap() {
    // Test a gap of u32::MAX
    let input = vec![0, u32::MAX];
    let encoded = NeighborPool::encode_neighbors(&input);
    let decoded = NeighborPool::decode_neighbors(&encoded);
    assert_eq!(decoded, input);
}

#[test]
fn test_edge_case_empty() {
    let input: Vec<u32> = vec![];
    let encoded = NeighborPool::encode_neighbors(&input);
    let decoded = NeighborPool::decode_neighbors(&encoded);
    assert_eq!(decoded, input);
}
