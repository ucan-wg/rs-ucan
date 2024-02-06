//! Bindings for the JavaScript via Wasm
//!
//! Note that these are all [`wasm_bindgen`]-specific,
//! and are not recommended elsewhere due to limited
//! type safety, poorer performance, and restrictions
//! on the API placed by [`wasm_bindgen`].
//!
//! The overall pattern is roughly: "JS code hands the
//! Rust code a config object with handlers at runtime".
//! The Rust takes those handlers, and dispatches them
//! as part of the normal flow.
//!
//! When compiled for Wasm, the other abilities in this
//! crate export JS bindings. This allows them to be
//! plugged into e.g. ability hierarchies from the JS
//! side as an extension mechanism.

pub mod parentful;
pub mod parentless;
