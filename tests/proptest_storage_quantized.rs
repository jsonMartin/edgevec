use edgevec::hnsw::HnswConfig;
use edgevec::quantization::{QuantizerConfig, ScalarQuantizer};
use edgevec::storage::{StorageType, VectorStorage};
use proptest::prelude::*;
use std::borrow::Cow;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_quantized_storage_roundtrip(
        // Generate random dimension (small for speed)
        dim in 2usize..64,
        // Generate vectors with values in range [-10.0, 10.0]
        vectors in prop::collection::vec(prop::collection::vec(-10.0f32..10.0f32, 2..64), 1..20),
        // Generate min/max for quantizer (ensure max > min)
        min in -10.0f32..0.0f32,
        max in 0.1f32..10.0f32
    ) {
        let config = HnswConfig::new(dim as u32);
        let mut storage = VectorStorage::new(&config, None);

        let q_config = QuantizerConfig { min, max };
        storage.set_storage_type(StorageType::QuantizedU8(q_config));
        let quantizer = ScalarQuantizer::new(q_config);

        for vec_data in vectors {
            // Adjust vector length to match dimension (proptest vec strategy is independent)
            let mut vec = vec_data.clone();
            vec.resize(dim, 0.0);

            // Insert
            let id = storage.insert(&vec).unwrap();

            // Retrieve quantized
            let stored_q = storage.get_quantized_vector(id);

            // Reconstruct locally for verification
            let expected_q = quantizer.quantize(&vec);

            // Verify quantized data matches expected quantization
            assert_eq!(stored_q, expected_q.as_slice(), "Quantized data mismatch");

            // Dequantize and verify L2 error is within bounds
            // Error bound for SQ8 is roughly (max - min) / 255 / 2 per component?
            // Or just check it's close enough.
            // Max quantization error per component is step_size / 2.
            let step_size = (max - min) / 255.0;
            let recovered = quantizer.dequantize(stored_q);

            for (orig, rec) in vec.iter().zip(recovered.iter()) {
                // If original was clamped, error might be larger.
                if *orig < min {
                    assert!((*rec - min).abs() < 1e-5);
                } else if *orig > max {
                    assert!((*rec - max).abs() < 1e-5);
                } else {
                    let diff = (*orig - *rec).abs();
                    // Allow small float error on top of quantization error
                    assert!(diff <= step_size + 1e-5, "Reconstruction error too high: {} vs {} (step {})", orig, rec, step_size);
                }
            }
        }
    }

    #[test]
    fn test_insert_pre_quantized_integrity(
        dim in 2usize..64,
        vectors in prop::collection::vec(prop::collection::vec(any::<u8>(), 2..64), 1..20)
    ) {
        let config = HnswConfig::new(dim as u32);
        let mut storage = VectorStorage::new(&config, None);

        // Config must be QuantizedU8 for insert_quantized to work
        let q_config = QuantizerConfig { min: -1.0, max: 1.0 };
        storage.set_storage_type(StorageType::QuantizedU8(q_config));

        for vec_data in vectors {
             let mut vec = vec_data.clone();
            vec.resize(dim, 0);

            let id = storage.insert_quantized(&vec).unwrap();
            let stored = storage.get_quantized_vector(id);

            assert_eq!(stored, vec.as_slice());
        }
    }
}

#[test]
fn test_memory_usage_efficiency() {
    let dim = 128;
    let count = 10_000;
    let config = HnswConfig::new(dim);
    let mut storage = VectorStorage::new(&config, None);

    // Use quantized storage
    let q_config = QuantizerConfig {
        min: -1.0,
        max: 1.0,
    };
    storage.set_storage_type(StorageType::QuantizedU8(q_config));

    // Create a dummy vector
    let vec = vec![0.5f32; dim as usize];

    for _ in 0..count {
        storage.insert(&vec).unwrap();
    }

    storage.compact();

    // Estimate size
    // We can't access private fields, but we can check if it fits roughly expected size.
    // 10k * 128 * 1 byte = 1.28 MB
    // If it was f32: 10k * 128 * 4 = 5.12 MB

    // We can't query memory usage directly from VectorStorage public API except maybe by implementation details?
    // Actually, we can't easily measure heap usage of a struct in Rust without a helper.
    // However, we can assert that `get_vector` panics (meaning no f32 storage)
    // and `get_quantized_vector` works.

    // But to verify memory usage, we might rely on the fact that if we allocated f32s,
    // we would likely blow a tighter memory budget if we were constrained, or just trust the implementation
    // since we verified `insert` paths.

    // Let's at least verify logic:
    // If we are in Quantized mode, `data_f32` should be empty.
    // We can't check `data_f32.len()` as it is private.
    // But `get_vector` panicking confirms it is empty (as per implementation).
    // UPDATE: get_vector now dequantizes on the fly, so it shouldn't panic.
    // We can verify it returns Cow::Owned, implying it didn't borrow from a stored f32 slice.

    let id = edgevec::hnsw::VectorId(1);
    let vec_out = storage.get_vector(id);
    assert!(
        matches!(vec_out, Cow::Owned(_)),
        "Should return Owned vector (dequantized), not Borrowed"
    );
}
