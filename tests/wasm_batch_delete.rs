//! WASM Batch Delete Tests (W18.5)
//!
//! Tests for batch delete WASM bindings with browser compatibility.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_batch_delete_basic() {
    use edgevec::wasm::{EdgeVec, EdgeVecConfig};
    use js_sys::Uint32Array;

    // Create index with some vectors
    let config = EdgeVecConfig::new(4);
    let mut index = EdgeVec::new(&config).unwrap();

    // Insert 20 vectors
    for i in 0..20 {
        let vec = js_sys::Float32Array::new_with_length(4);
        vec.set_index(0, i as f32);
        vec.set_index(1, i as f32);
        vec.set_index(2, i as f32);
        vec.set_index(3, i as f32);
        index.insert(vec).unwrap();
    }

    // Batch delete
    let ids = Uint32Array::new_with_length(5);
    ids.set_index(0, 1);
    ids.set_index(1, 3);
    ids.set_index(2, 5);
    ids.set_index(3, 7);
    ids.set_index(4, 9);

    let result = index.soft_delete_batch(ids).unwrap();

    assert_eq!(result.deleted(), 5);
    assert_eq!(result.total(), 5);
    assert_eq!(result.unique_count(), 5);
    assert!(result.all_valid());
    assert!(result.any_deleted());
}

#[wasm_bindgen_test]
fn test_wasm_batch_delete_with_invalid_ids() {
    use edgevec::wasm::{EdgeVec, EdgeVecConfig};
    use js_sys::Uint32Array;

    let config = EdgeVecConfig::new(4);
    let mut index = EdgeVec::new(&config).unwrap();

    // Insert 10 vectors
    for i in 0..10 {
        let vec = js_sys::Float32Array::new_with_length(4);
        vec.fill(i as f32, 0, 4);
        index.insert(vec).unwrap();
    }

    // Include invalid IDs
    let ids = Uint32Array::new_with_length(3);
    ids.set_index(0, 1); // Valid
    ids.set_index(1, 999); // Invalid
    ids.set_index(2, 2); // Valid

    let result = index.soft_delete_batch(ids).unwrap();

    assert_eq!(result.deleted(), 2);
    assert_eq!(result.invalid_ids(), 1);
    assert_eq!(result.total(), 3);
    assert!(!result.all_valid());
    assert!(result.any_deleted());
}

#[wasm_bindgen_test]
fn test_wasm_batch_delete_idempotent() {
    use edgevec::wasm::{EdgeVec, EdgeVecConfig};
    use js_sys::Uint32Array;

    let config = EdgeVecConfig::new(4);
    let mut index = EdgeVec::new(&config).unwrap();

    // Insert 10 vectors
    for i in 0..10 {
        let vec = js_sys::Float32Array::new_with_length(4);
        vec.fill(i as f32, 0, 4);
        index.insert(vec).unwrap();
    }

    // First batch delete
    let ids = Uint32Array::new_with_length(3);
    ids.set_index(0, 1);
    ids.set_index(1, 2);
    ids.set_index(2, 3);

    let result1 = index.soft_delete_batch(ids.clone()).unwrap();
    assert_eq!(result1.deleted(), 3);
    assert_eq!(result1.already_deleted(), 0);

    // Second batch delete (same IDs)
    let result2 = index.soft_delete_batch(ids).unwrap();
    assert_eq!(result2.deleted(), 0);
    assert_eq!(result2.already_deleted(), 3);
    assert!(result2.all_valid());
    assert!(!result2.any_deleted());
}

#[wasm_bindgen_test]
fn test_wasm_batch_delete_compat() {
    use edgevec::wasm::{EdgeVec, EdgeVecConfig};
    use js_sys::Float64Array;

    let config = EdgeVecConfig::new(4);
    let mut index = EdgeVec::new(&config).unwrap();

    // Insert 15 vectors
    for i in 0..15 {
        let vec = js_sys::Float32Array::new_with_length(4);
        vec.fill(i as f32, 0, 4);
        index.insert(vec).unwrap();
    }

    // Use compat method with Float64Array
    let ids = Float64Array::new_with_length(4);
    ids.set_index(0, 1.0);
    ids.set_index(1, 5.0);
    ids.set_index(2, 10.0);
    ids.set_index(3, 15.0);

    let result = index.soft_delete_batch_compat(ids).unwrap();

    assert_eq!(result.deleted(), 4);
    assert_eq!(result.total(), 4);
    assert_eq!(result.unique_count(), 4);
    assert!(result.all_valid());
}

#[wasm_bindgen_test]
fn test_wasm_batch_delete_empty() {
    use edgevec::wasm::{EdgeVec, EdgeVecConfig};
    use js_sys::Uint32Array;

    let config = EdgeVecConfig::new(4);
    let mut index = EdgeVec::new(&config).unwrap();

    // Insert some vectors
    for i in 0..5 {
        let vec = js_sys::Float32Array::new_with_length(4);
        vec.fill(i as f32, 0, 4);
        index.insert(vec).unwrap();
    }

    // Empty batch
    let ids = Uint32Array::new_with_length(0);
    let result = index.soft_delete_batch(ids).unwrap();

    assert_eq!(result.deleted(), 0);
    assert_eq!(result.total(), 0);
    assert_eq!(result.unique_count(), 0);
    assert!(result.all_valid());
    assert!(!result.any_deleted());
}

#[wasm_bindgen_test]
fn test_wasm_batch_delete_updates_counts() {
    use edgevec::wasm::{EdgeVec, EdgeVecConfig};
    use js_sys::Uint32Array;

    let config = EdgeVecConfig::new(4);
    let mut index = EdgeVec::new(&config).unwrap();

    // Insert 50 vectors
    for i in 0..50 {
        let vec = js_sys::Float32Array::new_with_length(4);
        vec.fill(i as f32, 0, 4);
        index.insert(vec).unwrap();
    }

    assert_eq!(index.live_count(), 50);
    assert_eq!(index.deleted_count(), 0);

    // Delete 20 vectors
    let ids = Uint32Array::new_with_length(20);
    for i in 0..20 {
        ids.set_index(i, (i + 1) as u32);
    }

    index.soft_delete_batch(ids).unwrap();

    assert_eq!(index.live_count(), 30);
    assert_eq!(index.deleted_count(), 20);
    assert!((index.tombstone_ratio() - 0.4).abs() < 0.01); // ~40% deleted
}

#[wasm_bindgen_test]
fn test_wasm_batch_delete_with_duplicates() {
    use edgevec::wasm::{EdgeVec, EdgeVecConfig};
    use js_sys::Uint32Array;

    let config = EdgeVecConfig::new(4);
    let mut index = EdgeVec::new(&config).unwrap();

    // Insert 10 vectors
    for i in 0..10 {
        let vec = js_sys::Float32Array::new_with_length(4);
        vec.fill(i as f32, 0, 4);
        index.insert(vec).unwrap();
    }

    // Batch with duplicates
    let ids = Uint32Array::new_with_length(7);
    ids.set_index(0, 1);
    ids.set_index(1, 2);
    ids.set_index(2, 1); // Duplicate
    ids.set_index(3, 3);
    ids.set_index(4, 1); // Duplicate
    ids.set_index(5, 2); // Duplicate
    ids.set_index(6, 4);

    let result = index.soft_delete_batch(ids).unwrap();

    assert_eq!(result.total(), 7); // Total input IDs
    assert_eq!(result.unique_count(), 4); // Only 4 unique IDs
    assert_eq!(result.deleted(), 4); // All 4 deleted
    assert!(result.all_valid());
}
