//! Unified error hierarchy for EdgeVec.
//!
//! This module defines the top-level `EdgeVecError` enum which wraps all
//! component-specific errors (Persistence, Graph, IO) and provides
//! consistent mapping to WASM/JavaScript exceptions.

use crate::hnsw::GraphError;
use crate::persistence::PersistenceError;
use thiserror::Error;

/// The Unified EdgeVec Error type.
#[derive(Debug, Error)]
pub enum EdgeVecError {
    /// Input/Output errors (filesystem, network, etc).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Persistence and storage errors.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),

    /// Graph algorithm and index errors.
    #[error(transparent)]
    Graph(#[from] GraphError),

    /// Validation errors (invalid arguments, dimensions, etc).
    #[error("Validation error: {0}")]
    Validation(String),
}

use wasm_bindgen::prelude::*;

impl From<EdgeVecError> for JsValue {
    fn from(err: EdgeVecError) -> Self {
        let (code, msg) = match &err {
            EdgeVecError::Io(e) => ("ERR_IO", e.to_string()),

            EdgeVecError::Persistence(pe) => match pe {
                PersistenceError::Io(e) => ("ERR_IO", e.to_string()),
                PersistenceError::ChecksumMismatch { .. }
                | PersistenceError::Corrupted(_)
                | PersistenceError::InvalidMagic { .. } => ("ERR_CORRUPTION", pe.to_string()),
                _ => ("ERR_PERSISTENCE", pe.to_string()),
            },

            EdgeVecError::Graph(ge) => match ge {
                GraphError::ConfigMismatch { .. } | GraphError::DimensionMismatch { .. } => {
                    ("ERR_DIMENSION", ge.to_string())
                }
                GraphError::CapacityExceeded => ("ERR_CAPACITY", ge.to_string()),
                _ => ("ERR_GRAPH", ge.to_string()),
            },

            EdgeVecError::Validation(msg) => ("ERR_VALIDATION", msg.clone()),
        };

        // Create JS Error object with code property
        // Note: In a real implementation we might use a helper to create
        // a custom object with code/message, or just an Error with prefix.
        // Here we return a plain Error object, but we could attach `code`.
        // Ideally: new Error(msg); err.code = code;

        // Simple string for now as fallback, but ideally Object:
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &"code".into(), &code.into());
        let _ = js_sys::Reflect::set(&obj, &"message".into(), &msg.into());
        obj.into()
    }
}
