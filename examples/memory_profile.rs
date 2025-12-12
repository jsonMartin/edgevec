//! Memory usage profiling for EdgeVec.
//!
//! Run with: `cargo run --release --example memory_profile`

use edgevec::hnsw::{HnswConfig, HnswIndex};
use edgevec::storage::VectorStorage;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tracking allocator to measure memory usage.
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

fn main() {
    println!("Memory Profile: EdgeVec");
    println!("========================\n");

    let dims = 128;
    // Only run up to 10k to be fast
    let counts = [1_000, 10_000];

    for count in counts {
        let before = ALLOCATED.load(Ordering::SeqCst);

        let config = HnswConfig::new(dims as u32);
        let mut storage = VectorStorage::new(&config, None);
        let mut index = HnswIndex::new(config, &storage).unwrap();

        // Insert dummy data
        for i in 0..count {
            let v: Vec<f32> = (0..dims).map(|_| (i % 10) as f32).collect();
            index.insert(&v, &mut storage).unwrap();
        }

        // Force compaction to exclude Vec doubling overhead from the metric
        storage.compact();

        let after = ALLOCATED.load(Ordering::SeqCst);

        // This is rough because Vec reallocations might not be perfectly tracked if freed immediately,
        // but since we keep `index` and `storage` alive, `after - before` should capture their retained size.
        let used = after.saturating_sub(before);
        let vector_data_size = count * dims * 4;
        let overhead = used.saturating_sub(vector_data_size);
        let per_vector_overhead = overhead / count;

        index.log_stats();

        println!(
            "{:>10} vectors: Total {:>10} bytes, Data {:>10} bytes, Overhead {:>10} bytes ({:>4} bytes/vector)",
            count, used, vector_data_size, overhead, per_vector_overhead
        );
    }
}
