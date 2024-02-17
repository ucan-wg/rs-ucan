#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_debug_implementations,
    future_incompatible,
    let_underscore,
    // FIXME missing_docs,
    rust_2021_compatibility,
    nonstandard_style,
)]
// FIXME consider removing for Prove
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
pub mod nonce;
pub mod number;
pub mod proof;
pub mod reader;
pub mod receipt;
pub mod signature;
pub mod task;
pub mod time;
pub mod url;

pub use delegation::Delegation;
pub use invocation::Invocation;
pub use receipt::Receipt;
