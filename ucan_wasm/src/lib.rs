//! Wasm bindings for `rs-ucan`.

#![allow(clippy::multiple_crate_versions)] // syn

use wasm_bindgen::prelude::*;

/// Returns the library version.
#[must_use]
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
