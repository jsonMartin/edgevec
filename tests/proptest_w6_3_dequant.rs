use edgevec::hnsw::HnswConfig;
use edgevec::quantization::QuantizerConfig;
use edgevec::storage::{StorageType, VectorStorage};
use proptest::prelude::*;
use std::borrow::Cow;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_verify_trait_return_types(
        dim in 2usize..64,
        vectors in prop::collection::vec(prop::collection::vec(-10.0f32..10.0f32, 2..64), 1..10),
        min in -10.0f32..0.0f32,
        max in 0.1f32..10.0f32
    ) {
        let config = HnswConfig::new(dim as u32);

        // 1. Verify Float32 Mode (Cow::Borrowed)
        {
            let mut storage = VectorStorage::new(&config, None);
            for vec_data in &vectors {
                let mut vec = vec_data.clone();
                vec.resize(dim, 0.0);

                let id = storage.insert(&vec).unwrap();
                let cow = storage.get_vector(id);

                // Must be Borrowed
                prop_assert!(matches!(cow, Cow::Borrowed(_)), "Float32 storage must return Cow::Borrowed");

                // Must be exact match
                prop_assert_eq!(&cow[..], vec.as_slice());
            }
        }

        // 2. Verify QuantizedU8 Mode (Cow::Owned)
        {
            let mut storage = VectorStorage::new(&config, None);
            let q_config = QuantizerConfig { min, max };
            storage.set_storage_type(StorageType::QuantizedU8(q_config));

            for vec_data in &vectors {
                let mut vec = vec_data.clone();
                vec.resize(dim, 0.0);

                let id = storage.insert(&vec).unwrap();
                let cow = storage.get_vector(id);

                // Must be Owned (dequantized)
                prop_assert!(matches!(cow, Cow::Owned(_)), "QuantizedU8 storage must return Cow::Owned");

                // Must be close to original (within quantization error)
                let step = (max - min) / 255.0;
                let recovered = &cow[..];

                for (orig, rec) in vec.iter().zip(recovered.iter()) {
                    if *orig >= min && *orig <= max {
                         let diff = (*orig - *rec).abs();
                         prop_assert!(diff <= step + 1e-4, "Reconstruction error too high");
                    }
                }
            }
        }
    }
}
