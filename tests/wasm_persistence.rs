#![cfg(target_arch = "wasm32")]

use edgevec::wasm::{EdgeVec, EdgeVecConfig};
use js_sys::{Float32Array, Reflect};
use wasm_bindgen_test::*;

// Use Node.js for testing as it's more stable in this environment
// wasm_bindgen_test_configure!(run_in_node);

#[wasm_bindgen_test]
async fn test_save_load_roundtrip() {
    // 1. Setup Mock Backend
    // Use globalThis to be environment agnostic (works in Node and Browser)
    let setup_code = r#"
        class IndexedDbBackend {
            static storage = new Map();
            
            static async write(name, data) {
                // data comes as Uint8Array from Rust
                // console.log(`MockDB: Writing ${name}, size ${data.length}`);
                IndexedDbBackend.storage.set(name, new Uint8Array(data));
            }
            
            static async read(name) {
                // console.log(`MockDB: Reading ${name}`);
                if (!IndexedDbBackend.storage.has(name)) {
                    throw new Error(`File not found: ${name}`);
                }
                return IndexedDbBackend.storage.get(name);
            }
        }
        globalThis.IndexedDbBackend = IndexedDbBackend;
    "#;

    js_sys::eval(setup_code).expect("failed to evaluate mock JS");

    // 2. Initialize EdgeVec
    let config = EdgeVecConfig::new(4); // 4 dimensions
    let mut edge_vec = EdgeVec::new(&config).expect("failed to create EdgeVec");

    // 3. Insert Data
    let count = 50;
    for i in 0..count {
        // Create distinct vectors
        let vec = vec![i as f32, i as f32 + 0.1, i as f32 + 0.2, i as f32 + 0.3];
        let arr = Float32Array::from(&vec[..]);
        let id = edge_vec.insert(arr).expect("failed to insert");
        assert_eq!(id, (i + 1) as u32);
    }

    // 4. Save
    // This calls IndexedDbBackend.write via the WASM binding
    edge_vec
        .save("test-db".to_string())
        .await
        .expect("failed to save");

    // 5. Load
    // This calls IndexedDbBackend.read via the WASM binding
    let restored_vec = EdgeVec::load("test-db".to_string())
        .await
        .expect("failed to load");

    // 6. Verify Roundtrip
    // Search for the 10th vector
    let query_idx = 9;
    let query_vec = vec![
        query_idx as f32,
        query_idx as f32 + 0.1,
        query_idx as f32 + 0.2,
        query_idx as f32 + 0.3,
    ];
    let query_arr = Float32Array::from(&query_vec[..]);

    let results = restored_vec.search(query_arr, 5).expect("failed to search");
    let results_arr = js_sys::Array::from(&results);

    assert!(results_arr.length() > 0, "should find results");

    // Top result should be ID 10
    let top_hit = results_arr.get(0);
    let id_val = Reflect::get(&top_hit, &"id".into()).unwrap();
    let score_val = Reflect::get(&top_hit, &"score".into()).unwrap();

    let id = id_val.as_f64().unwrap() as u32;
    let score = score_val.as_f64().unwrap() as f32;

    assert_eq!(id, 10);
    assert!(score < 0.0001, "score should be near 0 (L2 distance)");

    // Ensure we can't load non-existent file
    let err = EdgeVec::load("non-existent".to_string()).await;
    assert!(err.is_err());
}
