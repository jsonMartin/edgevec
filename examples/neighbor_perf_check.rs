use edgevec::hnsw::neighbor::NeighborPool;
use std::time::Instant;

fn main() {
    println!("Running Neighbor Performance Check...");

    // 1. Encode
    let neighbors: Vec<u32> = (0..32).map(|i| i * 100).collect();
    let start = Instant::now();
    let iters = 1_000_000;
    for _ in 0..iters {
        let _ = NeighborPool::encode_neighbors(&neighbors);
    }
    let elapsed = start.elapsed();
    println!(
        "Encode 32 neighbors: {:.2} ns/op",
        elapsed.as_nanos() as f64 / iters as f64
    );

    // 2. Decode
    let encoded = NeighborPool::encode_neighbors(&neighbors);
    let start = Instant::now();
    for _ in 0..iters {
        let _ = NeighborPool::decode_neighbors(&encoded);
    }
    let elapsed = start.elapsed();
    println!(
        "Decode 32 neighbors: {:.2} ns/op",
        elapsed.as_nanos() as f64 / iters as f64
    );

    // 3. Alloc/Free
    let mut pool = NeighborPool::new();
    let start = Instant::now();
    for _ in 0..iters {
        let (off, cap) = pool.alloc(64).unwrap();
        pool.free(off, cap);
    }
    let elapsed = start.elapsed();
    println!(
        "Alloc/Free 64B: {:.2} ns/op",
        elapsed.as_nanos() as f64 / iters as f64
    );
}
