use edgevec::hnsw::{HnswConfig, HnswIndex};
use edgevec::storage::VectorStorage;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))] // High iteration count to simulate fuzzing

    #[test]
    fn fuzz_hnsw_config_init(
        m in any::<u32>(),
        m0 in any::<u32>(),
        ef_construction in any::<u32>(),
        ef_search in any::<u32>(),
        dimensions in any::<u32>()
    ) {
        // Construct config with completely random values (no constraints)
        let mut config = HnswConfig::new(dimensions);
        config.m = m;
        config.m0 = m0;
        config.ef_construction = ef_construction;
        config.ef_search = ef_search;

        // We need to create a storage that respects the dimensions if possible,
        // but if dimensions is 0 or huge, we need to handle that.
        // VectorStorage::new assumes valid config usually.
        // But here we are testing HnswIndex::new's resilience.

        let storage = VectorStorage::new(&config, None);

        // The Invariant: HnswIndex::new must NOT panic.
        // It can return Ok or Err, but no panics.
        let _ = HnswIndex::new(config, &storage);
    }
}
