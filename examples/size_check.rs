//! Struct size analysis for RFC-001 (Soft Delete)
//!
//! This example verifies that adding a `deleted` field to HnswNode
//! does NOT increase struct size by reusing the existing padding byte.
//!
//! Run: cargo run --example size_check

use std::mem::{align_of, size_of};

/// Current HnswNode structure (from src/hnsw/graph.rs)
/// This represents the v0.2.x layout
#[repr(C)]
struct HnswNodeCurrent {
    /// Vector ID (8 bytes)
    vector_id: u64,
    /// Offset into neighbor pool (4 bytes)
    neighbor_offset: u32,
    /// Length of neighbor data (2 bytes)
    neighbor_len: u16,
    /// Maximum layer (1 byte)
    max_layer: u8,
    /// Explicit padding (1 byte)
    pad: u8,
}

/// Proposed HnswNode structure for v0.3.0
/// The `deleted` field replaces the padding byte
#[repr(C)]
struct HnswNodeWithDelete {
    /// Vector ID (8 bytes)
    vector_id: u64,
    /// Offset into neighbor pool (4 bytes)
    neighbor_offset: u32,
    /// Length of neighbor data (2 bytes)
    neighbor_len: u16,
    /// Maximum layer (1 byte)
    max_layer: u8,
    /// Deleted flag: 0 = live, 1 = deleted (1 byte, was padding)
    deleted: u8,
}

/// Alternative: Separate tombstone entry (Option B in RFC)
#[allow(dead_code)]
struct TombstoneEntry {
    vector_id: u64,
    deleted_at: u64,
}

fn main() {
    println!("=== RFC-001 Struct Size Analysis ===\n");

    println!("HnswNode Comparison:");
    println!(
        "  Current (v0.2.x):     {} bytes, align {}",
        size_of::<HnswNodeCurrent>(),
        align_of::<HnswNodeCurrent>()
    );
    println!(
        "  With deleted (v0.3.0): {} bytes, align {}",
        size_of::<HnswNodeWithDelete>(),
        align_of::<HnswNodeWithDelete>()
    );
    println!(
        "  Overhead: {} bytes",
        size_of::<HnswNodeWithDelete>() as isize - size_of::<HnswNodeCurrent>() as isize
    );
    println!();

    println!("Field Breakdown (HnswNodeWithDelete):");
    println!("  vector_id:      {} bytes", size_of::<u64>());
    println!("  neighbor_offset: {} bytes", size_of::<u32>());
    println!("  neighbor_len:    {} bytes", size_of::<u16>());
    println!("  max_layer:       {} byte", size_of::<u8>());
    println!("  deleted:         {} byte (was padding)", size_of::<u8>());
    println!(
        "  Total:          {} bytes",
        size_of::<u64>() + size_of::<u32>() + size_of::<u16>() + size_of::<u8>() + size_of::<u8>()
    );
    println!();

    println!("Alternative: Separate Tombstone Set (Option B):");
    println!(
        "  TombstoneEntry: {} bytes per deleted vector",
        size_of::<TombstoneEntry>()
    );
    println!("  HashSet overhead: ~24 bytes per entry (conservative estimate)");
    println!();

    println!("Memory Impact at Scale:");
    let scales = [100_000u64, 500_000, 1_000_000];
    println!("  | Vectors   | Option A (inline) | Option B (HashSet @ 10% deleted) |");
    println!("  |-----------|-------------------|----------------------------------|");
    for &count in &scales {
        let option_a_overhead = 0u64; // Reuses padding
        let option_b_overhead = (count / 10) * 24; // 10% deleted, 24 bytes each
        println!(
            "  | {:>9} | {:>15} B | {:>30} B |",
            format_with_commas(count),
            format_with_commas(option_a_overhead),
            format_with_commas(option_b_overhead)
        );
    }
    println!();

    // Verify key properties
    println!("=== Verification ===");
    let size_unchanged = size_of::<HnswNodeCurrent>() == size_of::<HnswNodeWithDelete>();
    println!(
        "  Size unchanged: {} ({})",
        if size_unchanged { "PASS" } else { "FAIL" },
        if size_unchanged {
            "zero overhead confirmed"
        } else {
            "unexpected size change!"
        }
    );

    let alignment_ok = align_of::<HnswNodeWithDelete>() == 8;
    println!(
        "  Alignment OK:   {} (align = {})",
        if alignment_ok { "PASS" } else { "FAIL" },
        align_of::<HnswNodeWithDelete>()
    );

    let size_is_16 = size_of::<HnswNodeWithDelete>() == 16;
    println!(
        "  Size is 16B:    {} (size = {})",
        if size_is_16 { "PASS" } else { "FAIL" },
        size_of::<HnswNodeWithDelete>()
    );

    println!();
    if size_unchanged && alignment_ok && size_is_16 {
        println!("All checks PASSED. RFC-001 memory claims are valid.");
    } else {
        println!("Some checks FAILED. Review struct layout.");
        std::process::exit(1);
    }
}

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
