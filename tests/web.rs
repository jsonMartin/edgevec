#![cfg(target_arch = "wasm32")]

use edgevec::wasm::{EdgeVec, EdgeVecConfig};
use js_sys::{Float32Array, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// Helper to setup mock backend
fn setup_mock_backend() {
    let setup_code = r#"
        if (typeof globalThis.IndexedDbBackend === 'undefined') {
            class IndexedDbBackend {
                static storage = new Map();
                
                static async write(name, data) {
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

                static clear() {
                    IndexedDbBackend.storage.clear();
                }
            }
            globalThis.IndexedDbBackend = IndexedDbBackend;
        } else {
             // Reset storage between tests if it persists
             globalThis.IndexedDbBackend.clear();
        }
    "#;
    js_sys::eval(setup_code).expect("failed to evaluate mock JS");
}

#[wasm_bindgen_test]
async fn test_core_flow() {
    setup_mock_backend();

    // 1. Init
    let dims = 4;
    let config = EdgeVecConfig::new(dims);
    let mut db = EdgeVec::new(&config).expect("Failed to init EdgeVec");

    // 2. Insert Batch
    let start = js_sys::Date::now();
    let count = 10;
    let mut data = Vec::with_capacity(count * dims as usize);
    for i in 0..count {
        // Create vectors like [0,0,0,0], [1,1,1,1], etc.
        for _ in 0..dims {
            data.push(i as f32);
        }
    }
    let data_array = Float32Array::from(&data[..]);
    let ids = db
        .insert_batch(data_array, count)
        .expect("Failed to insert batch");
    let end = js_sys::Date::now();
    // console_log::init() is called inside EdgeVec::new, so we can use log::info!
    // But we need to make sure log crate is available in dev-dependencies or use web_sys::console
    web_sys::console::log_1(&JsValue::from_str(&format!(
        "Insert batch took {}ms",
        end - start
    )));

    assert_eq!(ids.length() as usize, count);

    // 3. Search
    // Query for [5.1, 5.1, 5.1, 5.1], should match index 5 best
    let query_vec = vec![5.1f32; dims as usize];
    let query = Float32Array::from(&query_vec[..]);

    let results_js = db.search(query, 3).expect("Search failed");
    let results = js_sys::Array::from(&results_js);

    assert!(results.length() > 0, "Should find results");

    // Check top result
    let top = results.get(0);
    let id_js = Reflect::get(&top, &JsValue::from_str("id")).unwrap();
    let score_js = Reflect::get(&top, &JsValue::from_str("score")).unwrap();

    let id = id_js.as_f64().unwrap() as u32; // JS numbers are f64
    let score = score_js.as_f64().unwrap() as f32;

    // Expected ID is 6 (since IDs start at 1 usually, or 0? insert returns u32. Let's assume 1-based from previous tests seen or 0-based.
    // wait, ids.push(id.0 as u32) in insert_batch.
    // Let's check insert implementation: `self.inner.insert`. VectorId typically 0-based or 1-based?
    // In `edgevec/src/hnsw.rs`, usually 0-based. But `wasm_persistence.rs` had `assert_eq!(id, (i + 1) as u32);`.
    // Let's verify if IDs are 1-based.
    // In `tests/wasm_persistence.rs`: `assert_eq!(id, (i + 1) as u32);`
    // So IDs seem to start at 1.
    // If I inserted 0..9, the vector with value 5 is at index 5. ID should be 6.

    assert_eq!(id, 6, "Expected vector with value 5 (ID 6) to be top match");
    assert!(score < 1.0, "Score should be low for close match");

    // 4. Assert Result (Done above)
}

#[wasm_bindgen_test]
async fn test_persistence_flow() {
    setup_mock_backend();
    let db_name = "test_persistence_flow.db";

    // 1. Init & Insert
    {
        let dims = 2;
        let config = EdgeVecConfig::new(dims);
        let mut db = EdgeVec::new(&config).expect("Init failed");

        let vec1 = Float32Array::from(&[1.0, 0.0][..]);
        let id1 = db.insert(vec1).expect("Insert failed");
        assert_eq!(id1, 1);

        let vec2 = Float32Array::from(&[0.0, 1.0][..]);
        let id2 = db.insert(vec2).expect("Insert failed");
        assert_eq!(id2, 2);

        // 2. Save
        db.save(db_name.to_string()).await.expect("Save failed");
    }

    // 3. Reload Page (Simulated by new scope)
    // 4. Load
    let db2 = EdgeVec::load(db_name.to_string())
        .await
        .expect("Load failed");

    // 5. Search
    let query = Float32Array::from(&[0.9, 0.1][..]);
    let results_js = db2.search(query, 1).expect("Search failed");
    let results = js_sys::Array::from(&results_js);

    assert_eq!(results.length(), 1);

    let top = results.get(0);
    let id_js = Reflect::get(&top, &JsValue::from_str("id")).unwrap();
    let id = id_js.as_f64().unwrap() as u32;

    assert_eq!(id, 1, "Should find vector 1");
}

#[wasm_bindgen_test]
async fn test_error_handling() {
    setup_mock_backend();

    // 1. Init with invalid config (if possible?)
    // EdgeVecConfig doesn't have many invalid states exposed via setters that crash Init,
    // but maybe passing "unknown" metric
    let mut config = EdgeVecConfig::new(4);
    config.set_metric("invalid_metric".to_string());

    let res = EdgeVec::new(&config);
    assert!(res.is_err(), "Should fail with invalid metric");

    // 2. Search with wrong dimension
    let config_ok = EdgeVecConfig::new(4);
    let mut db = EdgeVec::new(&config_ok).unwrap();
    let v = Float32Array::from(&[1.0, 2.0, 3.0, 4.0][..]);
    db.insert(v).unwrap();

    let wrong_query = Float32Array::from(&[1.0, 2.0][..]); // Only 2 dims
    let search_res = db.search(wrong_query, 5);

    assert!(
        search_res.is_err(),
        "Should fail with wrong dimension query"
    );

    // Also check insert dimension mismatch
    let wrong_insert = Float32Array::from(&[1.0][..]);
    let insert_res = db.insert(wrong_insert);
    assert!(
        insert_res.is_err(),
        "Should fail insert with wrong dimension"
    );
}
