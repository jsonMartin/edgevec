//! WASM SIMD Benchmark Harness
//!
//! Run with: wasm-pack test --headless --chrome --features simd

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use web_sys::console;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn bench_wasm_simd_vs_scalar_performance() {
    // Setup vectors
    let dim = 4096; // Larger vector for SIMD to shine
    let iter = 5000;
    let a = vec![0.5f32; dim];
    let b = vec![0.4f32; dim];

    let window = web_sys::window().expect("should have a window");
    let performance = window.performance().expect("should have performance");

    // Warmup
    let mut _warmup = 0.0;
    for _ in 0..100 {
        _warmup += scalar_l2(&a, &b);
    }

    // Scalar Run
    let t0 = performance.now();
    let mut scalar_sum = 0.0;
    for _ in 0..iter {
        scalar_sum += scalar_l2(&a, &b);
    }
    let t1 = performance.now();
    let scalar_time = t1 - t0;

    // SIMD Run
    #[cfg(all(target_feature = "simd128", feature = "simd"))]
    {
        use edgevec::metric::simd::wasm::l2_squared;

        // Warmup SIMD
        let mut _warmup_simd = 0.0;
        for _ in 0..100 {
            _warmup_simd += l2_squared(&a, &b);
        }

        let t2 = performance.now();
        let mut simd_sum = 0.0;
        for _ in 0..iter {
            simd_sum += l2_squared(&a, &b);
        }
        let t3 = performance.now();
        let simd_time = t3 - t2;

        // Logging
        console::log_1(&format!("BENCHMARK REPORT [WASM SIMD]").into());
        console::log_1(&format!("Dimensions: {}", dim).into());
        console::log_1(&format!("Iterations: {}", iter).into());
        console::log_1(&format!("Scalar Time: {:.2}ms", scalar_time).into());
        console::log_1(&format!("SIMD Time:   {:.2}ms", simd_time).into());
        let speedup = scalar_time / simd_time;
        console::log_1(&format!("Speedup:     {:.2}x", speedup).into());

        // Assertions
        assert!(
            (scalar_sum - simd_sum).abs() < 1e-4,
            "Scalar and SIMD results should match"
        );

        // We expect some speedup, but CI environments can be flaky.
        // We log it clearly for the user to see.
        if speedup > 1.0 {
            console::log_1(&"✅ SIMD is faster".into());
        } else {
            console::warn_1(&"⚠️ SIMD is NOT faster (might be environment issue)".into());
        }
    }

    #[cfg(not(all(target_feature = "simd128", feature = "simd")))]
    {
        // Use variables to suppress warnings
        console::log_1(&format!("Scalar Result (verify): {}", scalar_sum).into());
        console::log_1(&format!("Scalar Time: {}ms", scalar_time).into());
        console::warn_1(&"Skipping SIMD benchmark: simd128 not enabled".into());
    }
}

// Manual scalar implementation for baseline comparison
fn scalar_l2(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        let diff = x - y;
        sum += diff * diff;
    }
    sum
}
