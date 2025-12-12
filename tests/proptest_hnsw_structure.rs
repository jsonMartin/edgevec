use edgevec::hnsw::{HnswConfig, HnswIndex};
use edgevec::storage::VectorStorage;
use proptest::prelude::*;

// Strategy for generating valid HnswConfig
fn hnsw_config_strategy() -> impl Strategy<Value = HnswConfig> {
    (
        2u32..64u32,   // m: [2, 64)
        1u32..128u32,  // m0 offset: [1, 128) to ensure m0 >= m
        10u32..500u32, // ef_construction
        10u32..200u32, // ef_search
        1u32..1024u32, // dimensions
    )
        .prop_map(
            |(m, m0_offset, ef_construction, ef_search, dimensions)| HnswConfig {
                m,
                m0: m + m0_offset, // Ensure m0 >= m
                ef_construction,
                ef_search,
                dimensions,
                metric: 0, // L2
                _reserved: [0; 2],
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property 1 (Initialization):
    /// Assert that HnswIndex::new(config) always produces a valid state for valid configs.
    #[test]
    fn test_hnsw_initialization_valid_config(config in hnsw_config_strategy()) {
        // Create a storage that matches the config dimensions
        let storage = VectorStorage::new(&config, None);

        // Attempt to create the index
        let index_res = HnswIndex::new(config.clone(), &storage);

        // Check result
        prop_assert!(index_res.is_ok(), "Failed to initialize with valid config: {:?}", config);
        let index = index_res.unwrap();

        // Verify state invariants
        prop_assert_eq!(index.entry_point(), None, "New index must have no entry point");
        prop_assert_eq!(index.node_count(), 0, "New index must have 0 nodes");
        prop_assert_eq!(index.max_layer(), 0, "New index must start at layer 0");
    }

    /// Property 2 (Config Validation):
    /// Assert that invalid configs always return Err.
    /// Focus on M <= 1 edge case.
    #[test]
    fn test_hnsw_initialization_invalid_m(
        m in 0u32..=1u32,
        dimensions in 1u32..100u32
    ) {
        let mut config = HnswConfig::new(dimensions);
        config.m = m;

        // Create storage matching config
        let storage = VectorStorage::new(&config, None);

        // Attempt creation
        let result = HnswIndex::new(config, &storage);

        // Must fail
        prop_assert!(result.is_err(), "Should reject M <= 1");
    }

    /// Property 2b (Config Validation):
    /// Assert that m0 < m always returns Err.
    #[test]
    fn test_hnsw_initialization_invalid_m0(
        m in 10u32..100u32,
        dimensions in 1u32..100u32
    ) {
        let mut config = HnswConfig::new(dimensions);
        config.m = m;
        config.m0 = m - 1; // Invalid: m0 must be >= m

        // Create storage matching config
        let storage = VectorStorage::new(&config, None);

        // Attempt creation
        let result = HnswIndex::new(config, &storage);

        // Must fail
        prop_assert!(result.is_err(), "Should reject m0 < m");
    }

    /// Property 3 (Send/Sync):
    /// Static assertion test that HnswIndex implements Send and Sync.
    /// Note: This is technically a compile-time check, but we include it in the suite.
    #[test]
    fn test_hnsw_send_sync(_ in 0..1) {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HnswIndex>();
    }
}
