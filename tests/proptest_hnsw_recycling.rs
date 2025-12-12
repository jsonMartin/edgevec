use edgevec::hnsw::neighbor::NeighborPool;
use proptest::prelude::*;

#[test]
fn test_recycling_reuse_exact_fit() {
    let mut pool = NeighborPool::new();

    // 1. Alloc 10 bytes -> Get Offset A
    let (off_a, cap_a) = pool.alloc(10).unwrap();
    assert_eq!(cap_a, 16); // Rounded up to 16

    // 2. Free Offset A
    pool.free(off_a, cap_a);

    // 3. Alloc 10 bytes -> Get Offset B
    let (off_b, cap_b) = pool.alloc(10).unwrap();

    // 4. Assert Reuse
    assert_eq!(off_a, off_b, "Should reuse exact fit slot");
    assert_eq!(cap_b, 16);
}

#[test]
fn test_recycling_fragmentation_no_reuse() {
    let mut pool = NeighborPool::new();

    // 1. Alloc 10 bytes
    let (off_a, cap_a) = pool.alloc(10).unwrap();

    // 2. Free it
    pool.free(off_a, cap_a);

    // 3. Alloc 20 bytes (Too big for the hole)
    let (off_b, _cap_b) = pool.alloc(20).unwrap();

    // 4. Assert No Reuse (should append)
    assert_ne!(off_a, off_b, "Should not reuse too small slot");
}

#[derive(Debug, Clone)]
enum Op {
    Alloc(usize),
    Free(usize), // Index into active allocations
}

fn op_strategy() -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec(
        prop_oneof![
            // Alloc sizes 1..100
            (1usize..100).prop_map(Op::Alloc),
            // Free random index (logic to modulo this later)
            (0usize..1000).prop_map(Op::Free),
        ],
        1..500, // 500 operations
    )
}

proptest! {
    /// Property: Memory Integrity
    ///
    /// Ensures that no two active allocations ever overlap in memory.
    #[test]
    fn prop_recycling_integrity(ops in op_strategy()) {
        let mut pool = NeighborPool::new();

        // Track active allocations: (offset, capacity)
        // We use a Vec to random access for freeing.
        let mut active: Vec<(u32, u16)> = Vec::new();

        for op in ops {
            match op {
                Op::Alloc(size) => {
                    let (off, cap) = pool.alloc(size).unwrap();

                    // Verify no overlap with ANY active allocation
                    for (exist_off, exist_cap) in &active {
                        let start_a = off;
                        let end_a = off + u32::from(cap);

                        let start_b = *exist_off;
                        let end_b = *exist_off + u32::from(*exist_cap);

                        // Check intersection: start_a < end_b && start_b < end_a
                        let overlaps = start_a < end_b && start_b < end_a;

                        prop_assert!(
                            !overlaps,
                            "New alloc {:?} overlaps with existing {:?} (Req size: {})",
                            (off, cap), (exist_off, exist_cap), size
                        );
                    }

                    active.push((off, cap));
                }
                Op::Free(idx) => {
                    if active.is_empty() {
                        continue;
                    }
                    // Pick one to free
                    let actual_idx = idx % active.len();
                    let (off, cap) = active.swap_remove(actual_idx);

                    pool.free(off, cap);
                }
            }
        }
    }
}
