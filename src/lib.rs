#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    // FIXME missing_debug_implementations,
    future_incompatible,
    let_underscore,
    // FIXME missing_docs,
    rust_2021_compatibility,
    nonstandard_style
)]
#![deny(unreachable_pub)]

//! ucan

#[cfg(target_arch = "wasm32")]
extern crate alloc;

pub mod ability;
pub mod capsule;
pub mod crypto;
pub mod delegation;
pub mod did;
pub mod invocation;
pub mod ipld;
pub mod reader;
pub mod receipt;
pub mod task;
pub mod time;
pub mod url;

#[cfg(feature = "test_utils")]
pub mod test_utils;

pub use delegation::Delegation;
pub use invocation::Invocation;
pub use receipt::Receipt;
