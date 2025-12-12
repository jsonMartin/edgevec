//! WASM SIMD Verification Tests
//!
//! Run with: wasm-pack test --headless --chrome --features simd

#![cfg(target_arch = "wasm32")]

#[cfg(all(target_feature = "simd128", feature = "simd"))]
use edgevec::metric::simd;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_simd_l2_squared() {
    // Expected:
    // Sum = 92.0

    #[cfg(all(target_feature = "simd128", feature = "simd"))]
    {
        // 1. Arrange
        let a = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![2.0f32, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0];

        let result = simd::wasm::l2_squared(&a, &b);
        assert_eq!(result, 92.0);
    }

    #[cfg(not(all(target_feature = "simd128", feature = "simd")))]
    {
        // Skip or warn if SIMD not enabled
        web_sys::console::log_1(&"Skipping SIMD test: simd128 not enabled".into());
    }
}

#[wasm_bindgen_test]
fn test_wasm_simd_dot_product() {
    #[cfg(all(target_feature = "simd128", feature = "simd"))]
    {
        let a = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = vec![1.0f32, 0.0, 1.0, 0.0];
        // 1*1 + 2*0 + 3*1 + 4*0 = 4.0
        let result = simd::wasm::dot_product(&a, &b);
        assert_eq!(result, 4.0);
    }
}
