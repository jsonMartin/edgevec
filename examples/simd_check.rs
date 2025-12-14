//! SIMD capability check example
//!
//! Run with: cargo run --example simd_check
//!
//! This example demonstrates how to check SIMD capabilities at runtime
//! and warn users about potential performance issues.

use edgevec::{capabilities, warn_if_suboptimal};

fn main() {
    println!("EdgeVec SIMD Capability Check");
    println!("==============================\n");

    let caps = capabilities();

    println!("Detected capabilities:");
    println!("  AVX2:   {}", if caps.avx2 { "YES" } else { "NO" });
    println!("  FMA:    {}", if caps.fma { "YES" } else { "NO" });
    println!("  SSE4.2: {}", if caps.sse42 { "YES" } else { "NO" });
    println!("  NEON:   {}", if caps.neon { "YES" } else { "NO" });
    println!();

    println!(
        "Optimal configuration: {}",
        if caps.is_optimal() { "YES" } else { "NO" }
    );
    println!();

    // This will print a warning to stderr if configuration is suboptimal
    warn_if_suboptimal();

    if caps.is_optimal() {
        println!("Your system is configured for optimal EdgeVec performance!");
    } else {
        println!("\nTo enable optimal performance, add the following to .cargo/config.toml:");
        println!();
        println!("[build]");
        println!("rustflags = [\"-C\", \"target-cpu=native\"]");
    }
}
