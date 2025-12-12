#![cfg(target_arch = "wasm32")]

use edgevec::error::EdgeVecError;
use edgevec::hnsw::GraphError;
use edgevec::wasm::{EdgeVec, EdgeVecConfig};
use js_sys::{Float32Array, Reflect, Uint8Array};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// Helper to setup corrupted backend to trigger deserialization error
fn setup_corrupted_backend() {
    let setup_code = r#"
        if (typeof globalThis.IndexedDbBackend === 'undefined' || globalThis.IndexedDbBackend.isMock) {
            class IndexedDbBackend {
                static isMock = true;
                static async write(name, data) {}
                static async read(name) {
                    // Return garbage bytes (not a valid postcard archive)
                    return new Uint8Array([0, 1, 2, 3, 255, 255]);
                }
            }
            globalThis.IndexedDbBackend = IndexedDbBackend;
        }
    "#;
    js_sys::eval(setup_code).expect("failed to evaluate mock JS");
}

#[wasm_bindgen_test]
fn verify_error_mapping_structure() {
    // 1. Verify IO Error mapping
    let io_err = EdgeVecError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Simulated failure",
    ));
    let js_val: JsValue = io_err.into();

    let code = Reflect::get(&js_val, &"code".into()).unwrap();
    assert_eq!(code.as_string().unwrap(), "ERR_IO");

    let msg = Reflect::get(&js_val, &"message".into()).unwrap();
    assert_eq!(msg.as_string().unwrap(), "IO error: Simulated failure");

    // 2. Verify Graph Error mapping (Dimension)
    let graph_err = EdgeVecError::Graph(GraphError::DimensionMismatch {
        expected: 128,
        actual: 64,
    });
    let js_val: JsValue = graph_err.into();

    let code = Reflect::get(&js_val, &"code".into()).unwrap();
    assert_eq!(code.as_string().unwrap(), "ERR_DIMENSION");
}

#[wasm_bindgen_test]
async fn test_insert_dimension_mismatch() {
    let config = EdgeVecConfig::new(128);
    let mut db = EdgeVec::new(&config).unwrap();

    // Create vector with wrong dimensions (64 instead of 128)
    let wrong_dim_vec = Float32Array::new_with_length(64);

    let result = db.insert(wrong_dim_vec);

    assert!(result.is_err(), "Should fail with dimension mismatch");
    let err = result.unwrap_err();

    // Verify it's an object with code=ERR_DIMENSION
    let code = Reflect::get(&err, &"code".into()).unwrap();
    assert_eq!(code.as_string().unwrap(), "ERR_DIMENSION");

    let msg = Reflect::get(&err, &"message".into()).unwrap();
    assert!(msg.as_string().unwrap().contains("Dimension mismatch"));
}

#[wasm_bindgen_test]
async fn test_batch_dimension_mismatch() {
    let config = EdgeVecConfig::new(128);
    let mut db = EdgeVec::new(&config).unwrap();

    let count = 10;
    // Expected length: 10 * 128 = 1280
    // Provided length: 1000 (mismatch)
    let vectors = Float32Array::new_with_length(1000);

    let result = db.insert_batch(vectors, count);

    assert!(result.is_err(), "Should fail with batch dimension mismatch");
    let err = result.unwrap_err();

    // Verify error structure
    let code = Reflect::get(&err, &"code".into()).expect("Error should have code property");
    let code_str = code.as_string().expect("code should be a string");

    // In insert_batch, we return EdgeVecError::Validation
    assert_eq!(code_str, "ERR_VALIDATION");

    let msg = Reflect::get(&err, &"message".into()).expect("Error should have message property");
    assert!(msg
        .as_string()
        .unwrap()
        .contains("Batch dimension mismatch"));
}

#[wasm_bindgen_test]
async fn test_persistence_error() {
    setup_corrupted_backend();

    // Attempt to load from corrupted backend
    let result = EdgeVec::load("corrupted_db".to_string()).await;

    assert!(result.is_err(), "Should fail to load corrupted data");
    let err = result.unwrap_err();

    // The load function wraps postcard errors in EdgeVecError::Persistence(PersistenceError::Corrupted)
    // which maps to ERR_CORRUPTION

    let code = Reflect::get(&err, &"code".into()).expect("Error should have code property");
    let code_str = code.as_string().expect("code should be a string");

    assert_eq!(code_str, "ERR_CORRUPTION");

    let msg = Reflect::get(&err, &"message".into()).expect("Error should have message property");
    assert!(msg.as_string().unwrap().contains("Deserialization failed"));
}
