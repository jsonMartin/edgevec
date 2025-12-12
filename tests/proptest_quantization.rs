use edgevec::quantization::ScalarQuantizer;
use proptest::prelude::*;

// Helper to calculate L2 distance between two f32 vectors
fn l2_sq(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
}

// Helper to calculate L2 distance between two u8 vectors
// This mimics the logic in metric/l2.rs but we implement it here for the test
// to ensure independence from the implementation under test.
fn l2_sq_u8(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff = (*x as i32) - (*y as i32);
            (diff * diff) as u32
        })
        .sum()
}

// Helper to calculate Dot Product between two u8 vectors
fn dot_u8(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (*x as u32) * (*y as u32))
        .sum()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // -----------------------------------------------------------------------
    // Property 1: Quantization Roundtrip Error Bound
    // -----------------------------------------------------------------------
    // Verify that the error introduced by quantization is within expected bounds.
    // For SQ8, the max error per dimension is roughly (max-min)/255 / 2.
    // We check that the L2 distance between original and reconstructed is small.
    // -----------------------------------------------------------------------
    #[test]
    fn prop_quantization_roundtrip_error(
        // Generate a vector of f32s with values in a reasonable range
        // Dimensions: 768 (standard embedding size)
        vector in prop::collection::vec(-10.0f32..10.0f32, 768)
    ) {
        // 1. Train Quantizer (Oracle: we know the exact range for this vector)
        // In reality, we train on a batch, but training on the vector itself
        // ensures the range covers the vector perfectly.
        let batch = vec![vector.as_slice()];
        let quantizer = ScalarQuantizer::train(&batch);

        // 2. Quantize
        let quantized = quantizer.quantize(&vector);

        // 3. Dequantize
        let reconstructed = quantizer.dequantize(&quantized);

        // 4. Assert Error Bounds
        // Max range is 20.0 (-10 to 10).
        // Step size = 20.0 / 255 ≈ 0.078
        // Max quantization error per element = step_size / 2 ≈ 0.039
        // Max squared error per element ≈ 0.039^2 ≈ 0.0015
        // Max total L2 squared error ≈ 768 * 0.0015 ≈ 1.15
        // We use a slightly looser bound to account for floating point noise.
        let distance = l2_sq(&vector, &reconstructed);

        // Normalized error check: RMSE (Root Mean Square Error)
        // RMSE = sqrt(distance / 768)
        // Expected RMSE <= 0.04
        let rmse = (distance / 768.0).sqrt();

        prop_assert!(rmse < 0.05,
            "RMSE too high: {}, distance: {}, range: {:?}",
            rmse, distance, quantizer.config());
    }

    // -----------------------------------------------------------------------
    // Property 2: Order Preservation (Ranking)
    // -----------------------------------------------------------------------
    // Verify that quantization preserves the relative ranking of vectors.
    // If A is much closer to Q than B is, then quantized(A) should be closer
    // to quantized(Q) than quantized(B) is.
    // -----------------------------------------------------------------------
    #[test]
    fn prop_order_preservation(
        // Query vector
        query in prop::collection::vec(-1.0f32..1.0f32, 128),
        // Base vector (near)
        base in prop::collection::vec(-1.0f32..1.0f32, 128),
        // Noise to create "far" vector
        noise in prop::collection::vec(2.0f32..3.0f32, 128)
    ) {
        let far: Vec<f32> = base.iter().zip(noise.iter())
            .map(|(b, n)| b + n)
            .collect();

        // Ensure "far" is actually far in f32 space
        let dist_near_f32 = l2_sq(&query, &base);
        let dist_far_f32 = l2_sq(&query, &far);

        // We only care about cases where the separation is significant enough
        // to survive quantization.
        if dist_far_f32 > dist_near_f32 * 1.5 {
            // Train quantizer on all data to establish a common coordinate system
            let batch = vec![query.as_slice(), base.as_slice(), far.as_slice()];
            let quantizer = ScalarQuantizer::train(&batch);

            let q_u8 = quantizer.quantize(&query);
            let near_u8 = quantizer.quantize(&base);
            let far_u8 = quantizer.quantize(&far);

            // Calculate u8 distances (simulating search)
            let dist_near_u8 = l2_sq_u8(&q_u8, &near_u8);
            let dist_far_u8 = l2_sq_u8(&q_u8, &far_u8);

            prop_assert!(dist_far_u8 > dist_near_u8,
                "Order flipped! F32: near={}, far={}; U8: near={}, far={}",
                dist_near_f32, dist_far_f32, dist_near_u8, dist_far_u8
            );
        }
    }

    // -----------------------------------------------------------------------
    // Property 3: u8 SIMD Equivalence
    // -----------------------------------------------------------------------
    // Verify that the SIMD implementations match the scalar implementation.
    // -----------------------------------------------------------------------
    #[test]
    fn prop_u8_simd_equivalence(
        a in prop::collection::vec(0u8..255u8, 1..1024),
        b in prop::collection::vec(0u8..255u8, 1..1024)
    ) {
        // We need same length vectors for distance
        let len = std::cmp::min(a.len(), b.len());
        let a = &a[..len];
        let b = &b[..len];

        // 1. Oracle (Local Simple Implementation)
        let expected_l2 = l2_sq_u8(a, b);
        let expected_dot = dot_u8(a, b);

        // 2. Scalar Implementation from Library
        let scalar_l2 = edgevec::metric::scalar::l2_squared_u8(a, b);
        let scalar_dot = edgevec::metric::scalar::dot_product_u8(a, b);

        prop_assert_eq!(scalar_l2, expected_l2, "Scalar L2 mismatch");
        prop_assert_eq!(scalar_dot, expected_dot, "Scalar Dot mismatch");

        // 3. AVX2 Implementation (if available)
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        {
            unsafe {
                let avx_l2 = edgevec::metric::simd::x86::l2_squared_u8(a, b);
                let avx_dot = edgevec::metric::simd::x86::dot_product_u8(a, b);
                prop_assert_eq!(avx_l2, expected_l2, "AVX2 L2 mismatch");
                prop_assert_eq!(avx_dot, expected_dot, "AVX2 Dot mismatch");
            }
        }

        // 4. WASM SIMD128 Implementation (if available)
        #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
        {
            unsafe {
                let wasm_l2 = edgevec::metric::simd::wasm::l2_squared_u8(a, b);
                let wasm_dot = edgevec::metric::simd::wasm::dot_product_u8(a, b);
                prop_assert_eq!(wasm_l2, expected_l2, "WASM L2 mismatch");
                prop_assert_eq!(wasm_dot, expected_dot, "WASM Dot mismatch");
            }
        }
    }
}
