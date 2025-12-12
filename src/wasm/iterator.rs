use crate::persistence::chunking::ChunkIter;
use js_sys::Uint8Array;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use wasm_bindgen::prelude::*;

/// Iterator for saving the database in chunks.
///
/// This avoids large allocations by serializing the database incrementally.
///
/// # Safety Warning
///
/// **WARNING:** This iterator holds a reference to the `EdgeVec` instance (via `unsafe` transmutation).
///
/// - You **MUST NOT** call `free()` on the `EdgeVec` instance while this iterator is in use.
/// - If `EdgeVec` is garbage collected or explicitly freed, usage of this iterator will panic
///   to prevent Use-After-Free (UAF) vulnerabilities.
/// - Ensure the `EdgeVec` instance remains in scope in JavaScript for the duration of the iteration.
#[wasm_bindgen]
pub struct PersistenceIterator {
    // We can't store a reference to EdgeVec here because self-referential structs are hard in Rust.
    // However, ChunkIter holds references to Storage and Index.
    // The safest way for WASM is to require the user to keep EdgeVec alive while iterating.
    /// The underlying iterator, with lifetime erased via unsafe.
    pub(crate) iter: ChunkIter<'static>,

    /// Safety guard to detect if the parent EdgeVec has been dropped.
    pub(crate) liveness: Arc<AtomicBool>,
}

#[wasm_bindgen]
impl PersistenceIterator {
    /// Returns the next chunk of data.
    ///
    /// # Returns
    ///
    /// * `Some(Uint8Array)` - The next chunk of data.
    /// * `None` - If iteration is complete.
    ///
    /// # Panics
    ///
    /// Panics if the parent `EdgeVec` instance has been freed.
    #[wasm_bindgen]
    pub fn next_chunk(&mut self) -> Option<Uint8Array> {
        // [m1] Safety Check: Ensure parent EdgeVec is still alive
        assert!(
            self.liveness.load(Ordering::Acquire),
            "EdgeVec Use-After-Free: EdgeVec instance was freed while PersistenceIterator is still active. Do not call .free() on EdgeVec while iterating."
        );

        self.iter.next().map(|chunk| {
            // Convert Vec<u8> to Uint8Array
            Uint8Array::from(&chunk[..])
        })
    }
}
