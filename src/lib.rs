#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub)]

//! rs-ucan

pub mod builder;
pub mod capability;
pub mod crypto;
pub mod did_verifier;
pub mod error;
pub mod plugins;
pub mod semantics;
pub mod time;
pub mod ucan;

/// A decentralized identifier.
pub type Did = String;

/// Test utilities.
#[cfg(any(test, feature = "test_utils"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test_utils")))]
pub mod test_utils;
